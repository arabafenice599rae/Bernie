//! Pre-audit, fasi 10, 12, 13, 18, 21 (Solana): account falsi, alterati, di un altro
//! mercato o in ordine sbagliato. Ogni tentativo deve fallire con l'errore previsto e non
//! cambiare nessun account (si confronta l'intero store di Mollusk prima e dopo).
//!
//! Alcuni scenari iniettano stati che on-chain nessuno può produrre (un mint Token-2022 con
//! supply diversa da quella del vault, un vault senza lamport sufficienti): servono a
//! verificare che le guardie difensive del programma intervengano davvero. Aggiunti dopo il
//! mutation testing, che le mostrava non esercitate (docs/testing/PreAuditReport.md).

mod common;

use common::*;
use solana_account::Account;
use solana_instruction::{error::InstructionError, AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::collections::HashMap;

type Store = HashMap<Pubkey, Account>;

struct Market {
    env: Env,
    alice: Pubkey,
    alice_ata: Pubkey,
    bob: Pubkey,
    bob_ata: Pubkey,
}

const SOL: u64 = 1_000_000_000;

fn market() -> Market {
    let mut env = Env::new();
    assert!(env
        .create(SOL / 100, 200, 100, b"Rosso", b"RED", b"")
        .is_ok());
    let alice = Pubkey::new_unique();
    let bob = Pubkey::new_unique();
    env.fund(&alice, 1_000 * SOL);
    env.fund(&bob, 1_000 * SOL);
    let alice_ata = env.token_account(&alice, None);
    let bob_ata = env.token_account(&bob, None);
    for (u, a) in [(&alice, &alice_ata), (&bob, &bob_ata)] {
        let r = env.run(&[env.mint_ix(u, a, 0, 50 * SOL, u64::MAX)]);
        assert!(r.is_ok(), "mint iniziale: {:?}", r.raw);
    }
    // delega preventiva di Alice al vault: i test di redeem usano la sola istruzione del
    // programma, così un account corrotto non viene rifiutato prima da ApproveChecked
    assert!(env
        .run(&[env.approve_ix(&alice, &alice_ata, 10 * SOL)])
        .is_ok());
    Market {
        env,
        alice,
        alice_ata,
        bob,
        bob_ata,
    }
}

impl Market {
    fn store(&self) -> Store {
        self.env.ctx.account_store.borrow().clone()
    }

    fn set(&self, key: &Pubkey, f: impl FnOnce(&mut Account)) {
        let mut st = self.env.ctx.account_store.borrow_mut();
        f(st.get_mut(key).expect("account presente"));
    }

    fn put(&self, key: Pubkey, acc: Account) {
        self.env.ctx.account_store.borrow_mut().insert(key, acc);
    }

    /// L'operazione deve fallire con `want` e lasciare lo store identico.
    fn reject(&self, ixs: &[Instruction], want: InstructionError, what: &str) {
        let before = self.store();
        let r = self.env.run(ixs);
        assert_eq!(r.raw, Err(want), "{what}");
        assert!(
            before == self.store(),
            "{what}: stato cambiato da un'operazione fallita"
        );
    }

    fn mint_ix(&self) -> Instruction {
        self.env
            .mint_ix(&self.alice, &self.alice_ata, 0, SOL, u64::MAX)
    }

    fn redeem_ixs(&self, u: u64) -> Vec<Instruction> {
        vec![
            self.env.approve_ix(&self.alice, &self.alice_ata, u),
            self.env.redeem_ix(&self.alice, &self.alice_ata, 0, u, 0),
        ]
    }

    fn donate_ix(&self) -> Instruction {
        self.env.donate_ix(&self.alice, SOL)
    }

    fn sweep_ix(&self) -> Instruction {
        self.env.sweep_ix(&self.env.creator)
    }

    /// Le quattro istruzioni che leggono il vault, con un'etichetta.
    fn all_ops(&self) -> Vec<(&'static str, Vec<Instruction>)> {
        vec![
            ("mint", vec![self.mint_ix()]),
            (
                "redeem",
                vec![self.env.redeem_ix(&self.alice, &self.alice_ata, 0, SOL, 0)],
            ),
            ("donate", vec![self.donate_ix()]),
            ("sweep", vec![self.sweep_ix()]),
        ]
    }
}

fn swap_account(ix: &Instruction, index: usize, key: Pubkey) -> Instruction {
    let mut ix = ix.clone();
    ix.accounts[index].pubkey = key;
    ix
}

fn with_last(ixs: &[Instruction], f: impl FnOnce(&Instruction) -> Instruction) -> Vec<Instruction> {
    let mut v = ixs.to_vec();
    let last = v.pop().unwrap();
    v.push(f(&last));
    v
}

/// Indice dell'account vault in ogni istruzione (mint 3, redeem 3, donate 2, sweep 2).
fn vault_index(op: &str) -> usize {
    match op {
        "mint" | "redeem" => 3,
        _ => 2,
    }
}

fn mint_index(op: &str) -> usize {
    match op {
        "mint" | "redeem" => 2,
        _ => 1,
    }
}

// ── fase 12: vault falsi e di altri mercati ──

#[test]
fn forged_vault_owned_by_another_program() {
    let m = market();
    let mut fake = m.env.account(&m.env.vault);
    fake.owner = Pubkey::new_unique(); // programma dell'attaccante
    fake.data[72..88].copy_from_slice(&(u128::MAX / 4).to_le_bytes()); // k gonfiato
    let fake_key = Pubkey::new_unique();
    m.put(fake_key, fake);
    for (op, ixs) in m.all_ops() {
        let ixs = with_last(&ixs, |ix| swap_account(ix, vault_index(op), fake_key));
        m.reject(&ixs, InstructionError::IncorrectProgramId, op);
    }
}

#[test]
fn program_owned_account_with_bad_header_is_rejected() {
    let m = market();
    for (label, edit) in [("discriminatore", 0usize), ("versione", 1usize)] {
        let mut fake = m.env.account(&m.env.vault);
        fake.data[edit] ^= 0xff;
        let key = Pubkey::new_unique();
        m.put(key, fake);
        for (op, ixs) in m.all_ops() {
            let ixs = with_last(&ixs, |ix| swap_account(ix, vault_index(op), key));
            m.reject(
                &ixs,
                InstructionError::InvalidAccountData,
                &format!("{op} {label}"),
            );
        }
    }
    let mut short = m.env.account(&m.env.vault);
    short.data.truncate(127);
    let key = Pubkey::new_unique();
    m.put(key, short);
    for (op, ixs) in m.all_ops() {
        let ixs = with_last(&ixs, |ix| swap_account(ix, vault_index(op), key));
        m.reject(
            &ixs,
            InstructionError::InvalidAccountData,
            &format!("{op} lunghezza"),
        );
    }
}

#[test]
fn cross_market_vault_and_mint_are_rejected() {
    let m = market();
    // secondo mercato nello stesso ambiente
    let (a_mint, a_vault) = (m.env.mint, m.env.vault);
    let mut env = m.env;
    assert!(env.create(SOL, 300, 0, b"Blu", b"BLU", b"").is_ok());
    let (b_mint, b_vault) = (env.mint, env.vault);
    env.mint = a_mint;
    env.vault = a_vault;
    let m = Market { env, ..m };
    for (op, ixs) in m.all_ops() {
        // vault di A con mint di B, e vault di B con mint di A
        let x = with_last(&ixs, |ix| swap_account(ix, mint_index(op), b_mint));
        m.reject(
            &x,
            InstructionError::InvalidAccountData,
            &format!("{op}: vault A, mint B"),
        );
        let y = with_last(&ixs, |ix| swap_account(ix, vault_index(op), b_vault));
        m.reject(
            &y,
            InstructionError::InvalidAccountData,
            &format!("{op}: vault B, mint A"),
        );
    }
    // redeem con token account di A verso il mercato B (mint e vault di B)
    let ixs = m.redeem_ixs(SOL);
    let ixs = with_last(&ixs, |ix| {
        let ix = swap_account(ix, 2, b_mint);
        swap_account(&ix, 3, b_vault)
    });
    let r = m.env.run(&ixs);
    assert!(r.raw.is_err(), "token di A riscattati nel mercato B");
}

// ── fase 13: Token-2022 incoerente ──

#[test]
fn mint_supply_must_match_vault_supply() {
    let m = market();
    for delta in [1i64, -1] {
        let before = m.store();
        m.set(&m.env.mint, |a| {
            let s = u64::from_le_bytes(a.data[36..44].try_into().unwrap());
            a.data[36..44].copy_from_slice(&((s as i64 + delta) as u64).to_le_bytes());
        });
        for (op, ixs) in m.all_ops() {
            m.reject(
                &ixs,
                InstructionError::Custom(14),
                &format!("{op} supply {delta:+}"),
            );
        }
        *m.env.ctx.account_store.borrow_mut() = before;
    }
}

#[test]
fn mint_must_be_token2022_and_initialized() {
    let m = market();
    let orig = m.env.account(&m.env.mint);
    m.set(&m.env.mint, |a| a.owner = SYSTEM);
    for (op, ixs) in m.all_ops() {
        m.reject(
            &ixs,
            InstructionError::IncorrectProgramId,
            &format!("{op} mint non Token-2022"),
        );
    }
    m.put(m.env.mint, orig.clone());
    m.set(&m.env.mint, |a| a.data[45] = 0);
    for (op, ixs) in m.all_ops() {
        m.reject(
            &ixs,
            InstructionError::InvalidAccountData,
            &format!("{op} mint non inizializzato"),
        );
    }
    m.put(m.env.mint, orig);
}

#[test]
fn redeem_token_account_attacks() {
    let m = market();
    let redeem = m.env.redeem_ix(&m.alice, &m.alice_ata, 0, SOL, 0);
    let approve = m.env.approve_ix(&m.alice, &m.alice_ata, SOL);
    let orig = m.env.account(&m.alice_ata);

    // token account non Token-2022
    m.set(&m.alice_ata, |a| a.owner = SYSTEM);
    m.reject(
        std::slice::from_ref(&redeem),
        InstructionError::IncorrectProgramId,
        "ata non Token-2022",
    );
    m.put(m.alice_ata, orig.clone());
    // non inizializzato
    m.set(&m.alice_ata, |a| a.data[108] = 0);
    m.reject(
        std::slice::from_ref(&redeem),
        InstructionError::InvalidAccountData,
        "ata non inizializzato",
    );
    m.put(m.alice_ata, orig.clone());
    // mint diverso
    m.set(&m.alice_ata, |a| {
        a.data[0..32].copy_from_slice(Pubkey::new_unique().as_ref())
    });
    m.reject(
        std::slice::from_ref(&redeem),
        InstructionError::InvalidAccountData,
        "ata di un altro mint",
    );
    m.put(m.alice_ata, orig.clone());
    // conto di Bob, firmato da Alice (con delega di Bob al vault)
    let r = m.env.run(&[m.env.approve_ix(&m.bob, &m.bob_ata, SOL)]);
    assert!(r.is_ok());
    let steal = swap_account(&redeem, 1, m.bob_ata);
    m.reject(
        &[steal],
        InstructionError::Custom(17),
        "conto di un altro utente",
    );
    // delegato diverso dal vault
    let other = Pubkey::new_unique();
    let mut approve_other = approve.clone();
    approve_other.accounts[2].pubkey = other;
    let r = m.env.run(&[approve_other]);
    assert!(r.is_ok());
    m.reject(
        std::slice::from_ref(&redeem),
        InstructionError::Custom(15),
        "delegato diverso",
    );
    // delega insufficiente, poi revocata
    m.reject(
        &[
            m.env.approve_ix(&m.alice, &m.alice_ata, SOL - 1),
            redeem.clone(),
        ],
        InstructionError::Custom(15),
        "delega insufficiente",
    );
    let revoke = Instruction::new_with_bytes(
        TOKEN_2022,
        &[5u8],
        vec![
            AccountMeta::new(m.alice_ata, false),
            AccountMeta::new_readonly(m.alice, true),
        ],
    );
    assert!(m.env.run(std::slice::from_ref(&approve)).is_ok());
    assert!(m.env.run(&[revoke]).is_ok());
    m.reject(
        std::slice::from_ref(&redeem),
        InstructionError::Custom(15),
        "delega revocata",
    );
    // oltre il saldo con delega sufficiente: il burn fallisce
    let bal = m.env.token_amount(&m.alice_ata);
    let before = m.store();
    let r = m.env.run(&[
        m.env.approve_ix(&m.alice, &m.alice_ata, bal + 1),
        m.env.redeem_ix(&m.alice, &m.alice_ata, 0, bal + 1, 0),
    ]);
    assert!(r.raw.is_err(), "riscatto oltre il saldo");
    assert!(before == m.store());
    // esattamente il saldo: passa
    let r = m.env.run(&[
        m.env.approve_ix(&m.alice, &m.alice_ata, bal),
        m.env.redeem_ix(&m.alice, &m.alice_ata, 0, bal, 0),
    ]);
    assert!(r.is_ok(), "riscatto dell'intero saldo: {:?}", r.raw);
    assert_eq!(m.env.token_amount(&m.alice_ata), 0);
}

// ── fase 12: firme, programmi, tesoreria ──

#[test]
fn signer_writable_program_and_treasury_checks() {
    let m = market();
    let fake_prog = Pubkey::new_unique();
    let mut cases: Vec<(String, Vec<Instruction>, InstructionError)> = vec![];
    for (op, ixs) in m.all_ops() {
        let last = ixs.last().unwrap().clone();
        let n = last.accounts.len();
        if op != "sweep" {
            let mut x = last.clone();
            x.accounts[0].is_signer = false;
            cases.push((
                format!("{op}: utente non firmatario"),
                vec![x],
                InstructionError::MissingRequiredSignature,
            ));
        }
        let mut x = last.clone();
        x.accounts[vault_index(op)].is_writable = false;
        cases.push((
            format!("{op}: vault non scrivibile"),
            vec![x],
            InstructionError::InvalidAccountData,
        ));
        if op == "mint" || op == "redeem" {
            let x = swap_account(&last, 4, Pubkey::new_unique());
            cases.push((
                format!("{op}: tesoreria fuori lista"),
                vec![x],
                InstructionError::InvalidArgument,
            ));
            let mut x = last.clone();
            x.accounts[4].is_writable = false;
            cases.push((
                format!("{op}: tesoreria non scrivibile"),
                vec![x],
                InstructionError::InvalidAccountData,
            ));
            let x = swap_account(&last, n - 1, fake_prog);
            cases.push((
                format!("{op}: programma token falso"),
                vec![x],
                InstructionError::IncorrectProgramId,
            ));
        }
        if op == "mint" || op == "donate" {
            let x = swap_account(&last, n - if op == "mint" { 2 } else { 1 }, fake_prog);
            cases.push((
                format!("{op}: System falso"),
                vec![x],
                InstructionError::IncorrectProgramId,
            ));
        }
    }
    for (what, ixs, want) in cases {
        m.reject(&ixs, want, &what);
    }
}

// ── fase 11: sweep ──

#[test]
fn sweep_beneficiary_is_always_the_registered_creator() {
    let m = market();
    let gift = 777_777;
    m.set(&m.env.vault, |a| a.lamports += gift); // lamport inviati al vault: excess
    let fee_excess = {
        let vs = m.env.vault_state();
        m.env.lamports(&m.env.vault) - rent(VAULT_LEN) - ((vs.reserve + vs.residual) / SCALE) as u64
    };
    assert!(fee_excess >= gift);
    // beneficiario diverso dal creator registrato
    let thief = swap_account(&m.sweep_ix(), 0, m.alice);
    m.reject(
        &[thief],
        InstructionError::InvalidArgument,
        "sweep verso un altro account",
    );
    // chiunque può chiamarlo (nessuna firma), e paga solo il creator
    let c0 = m.env.lamports(&m.env.creator);
    let a0 = m.env.lamports(&m.alice);
    assert!(m.env.run(&[m.sweep_ix()]).is_ok());
    assert_eq!(m.env.lamports(&m.env.creator) - c0, fee_excess);
    assert_eq!(m.env.lamports(&m.alice), a0);
    m.reject(
        &[m.sweep_ix()],
        InstructionError::Custom(12),
        "secondo sweep",
    );
}

// ── fase 3/21: vault insolvente, ultima difesa ──

#[test]
fn insolvent_vault_refuses_every_operation() {
    let m = market();
    let vs = m.env.vault_state();
    let liab = ((vs.reserve + vs.residual) / SCALE) as u64;
    // Metà delle passività: nessuna fee dell'operazione può coprire l'ammanco (un ammanco
    // di pochi lamport verrebbe coperto dalla fee del creator, che resta nel vault).
    m.set(&m.env.vault, |a| a.lamports = rent(VAULT_LEN) + liab / 2);
    for (op, ixs) in m.all_ops() {
        m.reject(
            &ixs,
            InstructionError::Custom(11),
            &format!("{op} su vault insolvente"),
        );
    }
}

// ── create ──

#[test]
fn create_account_attacks() {
    let mut env = Env::new();
    env.new_mint();
    let good_vault = env.vault;
    // vault che non è il PDA [vault, mint]
    env.vault = Pubkey::new_unique();
    let before = env.ctx.account_store.borrow().clone();
    let r = env.create_current(SOL, 200, 100, b"N", b"S", b"");
    assert_eq!(r.raw, Err(InstructionError::InvalidSeeds));
    assert!(before == *env.ctx.account_store.borrow());
    env.vault = good_vault;
    // mint o creator non firmatari: si costruisce l'istruzione a mano
    let mut data = vec![0u8];
    data.extend_from_slice(&SOL.to_le_bytes());
    data.extend_from_slice(&200u16.to_le_bytes());
    data.extend_from_slice(&100u16.to_le_bytes());
    data.extend_from_slice(&[1, b'N', 1, b'S', 0]);
    for unsigned in [0usize, 1] {
        let mut metas = vec![
            AccountMeta::new(env.creator, true),
            AccountMeta::new(env.mint, true),
            AccountMeta::new(env.vault, false),
            AccountMeta::new_readonly(SYSTEM, false),
            AccountMeta::new_readonly(TOKEN_2022, false),
        ];
        metas[unsigned].is_signer = false;
        let ix = Instruction::new_with_bytes(PROGRAM_ID, &data, metas);
        let before = env.ctx.account_store.borrow().clone();
        let r = env.run(&[ix]);
        assert_eq!(
            r.raw,
            Err(InstructionError::MissingRequiredSignature),
            "account {unsigned} non firmatario"
        );
        assert!(before == *env.ctx.account_store.borrow());
    }
    // una seconda create sullo stesso mint fallisce
    assert!(env.create_current(SOL, 200, 100, b"N", b"S", b"").is_ok());
    let before = env.ctx.account_store.borrow().clone();
    assert!(env
        .create_current(SOL, 200, 100, b"N", b"S", b"")
        .raw
        .is_err());
    assert!(before == *env.ctx.account_store.borrow());
}

// ── fase 12: permutazioni e duplicati degli account ──

#[test]
fn account_permutations_and_duplicates_are_rejected() {
    let m = market();
    let mut tried = 0;
    for (op, ixs) in m.all_ops() {
        let base = ixs.last().unwrap().clone();
        let prefix = &ixs[..ixs.len() - 1];
        let n = base.accounts.len();
        // tutte le permutazioni (mint: 7! = 5040)
        let mut idx: Vec<usize> = (0..n).collect();
        let mut perms = vec![];
        permute(&mut idx, 0, &mut perms);
        for p in perms {
            if p.iter().enumerate().all(|(i, &j)| i == j) {
                continue;
            }
            let mut ix = base.clone();
            ix.accounts = p.iter().map(|&j| base.accounts[j].clone()).collect();
            let mut all = prefix.to_vec();
            all.push(ix);
            let before = m.store();
            let r = m.env.run(&all);
            assert!(r.raw.is_err(), "{op}: permutazione {p:?} accettata");
            assert!(before == m.store(), "{op}: permutazione {p:?} con effetti");
            tried += 1;
        }
        // duplicati: ogni slot sostituito con ogni altro account dell'istruzione
        for i in 0..n {
            for j in 0..n {
                if base.accounts[i].pubkey == base.accounts[j].pubkey {
                    continue;
                }
                let mut ix = base.clone();
                ix.accounts[i] = base.accounts[j].clone();
                let mut all = prefix.to_vec();
                all.push(ix);
                let before = m.store();
                let r = m.env.run(&all);
                assert!(r.raw.is_err(), "{op}: slot {i} = account {j} accettato");
                assert!(before == m.store(), "{op}: duplicato {i}/{j} con effetti");
                tried += 1;
            }
        }
    }
    eprintln!("{tried} combinazioni di account rifiutate");
}

fn permute(v: &mut Vec<usize>, k: usize, out: &mut Vec<Vec<usize>>) {
    if k == v.len() {
        out.push(v.clone());
        return;
    }
    for i in k..v.len() {
        v.swap(k, i);
        permute(v, k + 1, out);
        v.swap(k, i);
    }
}

// ── create: rent esatto per la dimensione finale (§12) ──

#[test]
fn create_funds_exact_rent_for_final_sizes() {
    for (name, symbol, uri) in [
        (&b"N"[..], &b"S"[..], &b""[..]),
        (&[b'n'; 32][..], &[b's'; 10][..], &[b'u'; 200][..]),
        (
            &b"Bernie Test"[..],
            &b"BRN"[..],
            &b"https://example.invalid/b.json"[..],
        ),
    ] {
        let mut env = Env::new();
        assert!(env.create(SOL, 200, 100, name, symbol, uri).is_ok());
        let mint = env.account(&env.mint);
        assert_eq!(
            mint.lamports,
            rent(mint.data.len()),
            "mint: rent esatto per {} byte",
            mint.data.len()
        );
        let vault = env.account(&env.vault);
        assert_eq!(vault.lamports, rent(VAULT_LEN), "vault: rent esatto");
        assert_eq!(vault.data.len(), VAULT_LEN);
    }
}

// ── §11: log `State k S R Q` dopo ogni operazione, con i valori scritti nel vault ──

#[test]
fn state_log_matches_vault_after_every_operation() {
    use solana_svm_log_collector::LogCollector;
    use std::{cell::RefCell, rc::Rc};
    let mut m = market();
    let logger = Rc::new(RefCell::new(LogCollector::default()));
    // il contesto espone Mollusk: il collettore vale per le istruzioni successive
    m.env.ctx.mollusk.logger = Some(logger.clone());
    m.set(&m.env.vault, |a| a.lamports += 5_000); // excess per lo sweep
    for (op, ixs) in m.all_ops() {
        logger.replace(LogCollector::default());
        let r = m.env.run(&ixs);
        assert!(r.is_ok(), "{op}: {:?}", r.raw);
        let vs = m.env.vault_state();
        let want = format!(
            "Program log: State {} {} {} {}",
            vs.k, vs.supply, vs.reserve, vs.residual
        );
        let logs = logger.borrow().get_recorded_content().to_vec();
        assert!(logs.contains(&want), "{op}: manca `{want}` in {logs:?}");
    }
}
