//! Differential e state-machine fuzzing (pre-audit, fasi 2–4, 19–21).
//!
//! Riesegue sul programma reale (SBF su Mollusk, Token-2022 reale) le sequenze prodotte da
//! `tools/redteam/gen.py sol`, il cui oracolo è il modello dell'Appendice A. Dopo *ogni*
//! passo confronta: esito (ok o uno degli errori ammessi), k, R, Q, S del vault, supply del
//! mint, lamport e token di ogni utente, delega al vault, lamport disponibili del vault,
//! lamport ricevuti dalle tesorerie. Inoltre, in modo indipendente dall'oracolo:
//! - un passo fallito non cambia nessun account (atomicità, fase 21);
//! - la somma dei lamport di tutti gli account resta costante (conservazione, fase 19);
//! - somma dei saldi token = supply = S.
//!
//! Le sequenze si leggono da `BERNIE_DIFF_DIR` (tutte le `sol_*.json`). Senza la variabile
//! il test genera un piccolo lotto con python3, così gira anche in CI.

mod common;

use common::*;
use serde_json::Value;
use solana_account::Account;
use solana_instruction::{error::InstructionError, AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::collections::BTreeMap;
use std::path::PathBuf;

fn num(v: &Value) -> u128 {
    match v {
        Value::String(s) => s.parse().expect("intero decimale"),
        Value::Number(n) => n.as_u64().expect("u64") as u128,
        other => panic!("numero atteso, trovato {other}"),
    }
}

/// Importo di un'istruzione: deve stare in u64, mai troncato.
fn u64v(v: &Value) -> u64 {
    u64::try_from(num(v)).expect("importo oltre u64 nella sequenza")
}

fn nums(v: &Value) -> Vec<u128> {
    v.as_array().unwrap().iter().map(num).collect()
}

/// Nome dell'errore osservato, confrontato con l'insieme ammesso dall'oracolo.
/// Il codice custom 1 è ambiguo tra ZeroAmount (Bernie), InsufficientFunds (Token-2022)
/// e ResultWithNegativeLamports (System): vale per chiunque dei tre sia ammesso.
fn matches(actual: &InstructionError, allowed: &[String]) -> bool {
    allowed.iter().any(|name| match (name.as_str(), actual) {
        ("InsufficientTokens", InstructionError::Custom(1)) => true,
        ("InsufficientNative", InstructionError::Custom(1)) => true,
        ("InsufficientNative", InstructionError::InsufficientFunds) => true,
        (n, InstructionError::Custom(c)) => bernie_state::BernieError::from_code(*c)
            .map(|e| e.name() == n)
            .unwrap_or(false),
        _ => false,
    })
}

struct Harness {
    env: Env,
    users: Vec<Pubkey>,
    atas: Vec<Pubkey>,
    treasury0: u128,
    rent_vault: u64,
}

impl Harness {
    fn new(seq: &Value) -> Self {
        let mut env = Env::new();
        let wallets = nums(&seq["wallets"]);
        let users: Vec<Pubkey> = (0..wallets.len())
            .map(|i| {
                if i == 0 {
                    env.creator
                } else {
                    Pubkey::new_unique()
                }
            })
            .collect();
        let price = num(&seq["P"]) as u64;
        let p = num(&seq["p"]) as u16;
        let e = num(&seq["e"]) as u16;
        let r = env.create(price, p, e, b"Diff", b"DIF", b"");
        assert!(r.is_ok(), "create: {:?}", r.raw);
        // Saldi iniziali dell'oracolo, dopo create (che il creator paga a parte).
        for (u, w) in users.iter().zip(&wallets) {
            env.fund(u, *w as u64);
        }
        let atas = users.iter().map(|u| env.token_account(u, None)).collect();
        let treasury0 = (0..4).map(|t| env.lamports(&treasury(t)) as u128).sum();
        Self {
            env,
            users,
            atas,
            treasury0,
            rent_vault: rent(VAULT_LEN),
        }
    }

    fn keys(&self) -> Vec<Pubkey> {
        let mut k = vec![self.env.mint, self.env.vault];
        k.extend(&self.users);
        k.extend(&self.atas);
        k.extend((0..4).map(treasury));
        k
    }

    fn snapshot(&self) -> BTreeMap<Pubkey, Account> {
        self.keys()
            .into_iter()
            .map(|k| (k, self.env.account(&k)))
            .collect()
    }

    fn delegated(&self, i: usize) -> u128 {
        let d = self.env.account(&self.atas[i]).data;
        let has = u32::from_le_bytes(d[72..76].try_into().unwrap()) != 0;
        let who = Pubkey::new_from_array(d[76..108].try_into().unwrap());
        let amt = u64::from_le_bytes(d[121..129].try_into().unwrap()) as u128;
        if has && who == self.env.vault {
            amt
        } else {
            0
        }
    }

    fn transfer_ix(&self, a: usize, b: usize, x: u64) -> Instruction {
        let mut data = vec![12u8];
        data.extend_from_slice(&x.to_le_bytes());
        data.push(9);
        Instruction::new_with_bytes(
            TOKEN_2022,
            &data,
            vec![
                AccountMeta::new(self.atas[a], false),
                AccountMeta::new_readonly(self.env.mint, false),
                AccountMeta::new(self.atas[b], false),
                AccountMeta::new_readonly(self.users[a], true),
            ],
        )
    }

    fn revoke_ix(&self, a: usize) -> Instruction {
        Instruction::new_with_bytes(
            TOKEN_2022,
            &[5u8],
            vec![
                AccountMeta::new(self.atas[a], false),
                AccountMeta::new_readonly(self.users[a], true),
            ],
        )
    }

    fn ixs(&self, s: &Value) -> Vec<Instruction> {
        let a = num(&s["a"]) as usize;
        let t = |s: &Value| num(&s["t"]) as usize;
        match s["op"].as_str().unwrap() {
            "mint" => vec![self.env.mint_ix(
                &self.users[a],
                &self.atas[a],
                t(s),
                u64v(&s["u"]),
                u64v(&s["pay"]),
            )],
            "redeem" => {
                let mut v = vec![];
                if !s["approve"].is_null() {
                    v.push(
                        self.env
                            .approve_ix(&self.users[a], &self.atas[a], u64v(&s["approve"])),
                    );
                }
                v.push(self.env.redeem_ix(
                    &self.users[a],
                    &self.atas[a],
                    t(s),
                    u64v(&s["u"]),
                    u64v(&s["min_out"]),
                ));
                v
            }
            "donate" => vec![self.env.donate_ix(&self.users[a], u64v(&s["x"]))],
            "sweep" => vec![self.env.sweep_ix(&self.users[0])],
            "transfer" => vec![self.transfer_ix(a, num(&s["b"]) as usize, u64v(&s["x"]))],
            "revoke" => vec![self.revoke_ix(a)],
            other => panic!("operazione sconosciuta {other}"),
        }
    }

    /// Stato osservato, nello stesso formato dello stato atteso.
    fn observed(&self) -> BTreeMap<&'static str, Vec<u128>> {
        let vs = self.env.vault_state();
        let n = self.users.len();
        let mut m = BTreeMap::new();
        m.insert("k", vec![vs.k]);
        m.insert("R", vec![vs.reserve]);
        m.insert("Q", vec![vs.residual]);
        m.insert("S", vec![vs.supply as u128]);
        m.insert(
            "native",
            self.users
                .iter()
                .map(|u| self.env.lamports(u) as u128)
                .collect(),
        );
        m.insert(
            "tokens",
            self.atas
                .iter()
                .map(|a| self.env.token_amount(a) as u128)
                .collect(),
        );
        m.insert("delegated", (0..n).map(|i| self.delegated(i)).collect());
        m.insert(
            "available",
            vec![(self.env.lamports(&self.env.vault) - self.rent_vault) as u128],
        );
        let tr: u128 = (0..4)
            .map(|t| self.env.lamports(&treasury(t)) as u128)
            .sum();
        m.insert("treasury_in", vec![tr - self.treasury0]);
        m
    }
}

fn run_sequence(path: &PathBuf) -> usize {
    let seq: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let h = Harness::new(&seq);
    let lamports_total =
        |h: &Harness| -> u128 { h.snapshot().values().map(|a| a.lamports as u128).sum() };
    let total0 = lamports_total(&h);
    let steps = seq["steps"].as_array().unwrap();
    for (i, s) in steps.iter().enumerate() {
        let before = h.snapshot();
        let out = h.env.run(&h.ixs(s));
        let ctx = || {
            format!(
                "\nseed {} passo {} ({}): {}\natteso {}\nesito {:?}",
                seq["seed"],
                i,
                path.display(),
                s,
                s["expect"],
                out.raw
            )
        };
        match (&out.raw, s["expect"]["ok"].as_bool() == Some(true)) {
            (Ok(()), true) => {}
            (Err(e), false) => {
                let allowed: Vec<String> = s["expect"]["err"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x.as_str().unwrap().to_string())
                    .collect();
                assert!(matches(e, &allowed), "errore diverso{}", ctx());
                assert!(before == h.snapshot(), "passo fallito con effetti{}", ctx());
            }
            _ => panic!("esito diverso{}", ctx()),
        }
        let obs = h.observed();
        let st = &s["state"];
        for (key, got) in &obs {
            if let Some(sparse) = st[*key].as_object() {
                // marathon: solo gli utenti toccati dal passo
                for (idx, v) in sparse {
                    let i: usize = idx.parse().unwrap();
                    assert_eq!(num(v), got[i], "stato `{key}[{i}]` diverso{}", ctx());
                }
                continue;
            }
            let want: Vec<u128> = if st[*key].is_array() {
                nums(&st[*key])
            } else {
                vec![num(&st[*key])]
            };
            assert_eq!(&want, got, "stato `{key}` diverso{}", ctx());
        }
        assert_eq!(
            h.env.mint_supply() as u128,
            obs["S"][0],
            "supply ≠ S{}",
            ctx()
        );
        assert_eq!(
            obs["tokens"].iter().sum::<u128>(),
            obs["S"][0],
            "somma dei saldi ≠ S{}",
            ctx()
        );
        assert_eq!(
            lamports_total(&h),
            total0,
            "lamport creati o distrutti{}",
            ctx()
        );
    }
    steps.len()
}

#[test]
fn differential_sequences() {
    let dir = match std::env::var("BERNIE_DIFF_DIR") {
        Ok(d) => PathBuf::from(d),
        Err(_) => {
            // Lotto piccolo e deterministico, generato dall'oracolo.
            let d = std::env::temp_dir().join("bernie-diff-sol-ci");
            let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
            let st = std::process::Command::new("python3")
                .args([
                    "tools/redteam/gen.py",
                    "sol",
                    "--seed",
                    "1",
                    "--seqs",
                    "20",
                    "--ops",
                    "150",
                ])
                .arg("--out")
                .arg(&d)
                .current_dir(root)
                .status()
                .expect("python3 per generare le sequenze");
            assert!(st.success());
            d
        }
    };
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.file_name().unwrap().to_string_lossy().starts_with("sol_"))
        .collect();
    files.sort();
    assert!(!files.is_empty(), "nessuna sequenza in {}", dir.display());
    let t = std::time::Instant::now();
    let steps: usize = files.iter().map(run_sequence).sum();
    eprintln!(
        "differential Solana: {} sequenze, {} passi in {:.1}s",
        files.len(),
        steps,
        t.elapsed().as_secs_f64()
    );
}
