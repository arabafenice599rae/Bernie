//! Consumatore del flusso di `tools/redteam/stream.py` (pre-audit, fase 2).
//!
//! Applica ogni operazione a `Vault<SCALE>` e confronta con l'oracolo (Appendice A),
//! dopo ogni passo: esito (ok o nome dell'errore), importo pagato o ricevuto, fee al creator
//! e al protocollo, k, R, Q, S. Su un errore lo stato deve restare identico. Alla prima
//! divergenza stampa seed, passo e dettagli ed esce con codice 1.
//!
//!   python3 tools/redteam/stream.py --seqs 1000 --ops 1000 --scale 1000000000 \
//!     | cargo run --release -p bernie-state --example diff_stream

use bernie_state::{BernieError, Vault};
use std::io::{BufRead, Write};

enum Any {
    S10(Vault<10>),
    S9(Vault<1_000_000_000>),
}

macro_rules! with {
    ($v:expr, $x:ident => $body:expr) => {
        match $v {
            Any::S10($x) => $body,
            Any::S9($x) => $body,
        }
    };
}

fn main() {
    let stdin = std::io::stdin();
    let mut v: Option<Any> = None;
    let (mut seed, mut step, mut total, mut seqs) = (String::new(), 0u64, 0u64, 0u64);
    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let t: Vec<&str> = line.split_whitespace().collect();
        let n = |i: usize| -> u128 { t[i].parse().unwrap() };
        match t[0] {
            "#" => {
                seed = line.clone();
                step = 0;
                seqs += 1;
                continue;
            }
            "H" => {
                let (price, p, e) = (n(2) as u64, n(3) as u16, n(4) as u16);
                v = Some(match n(1) {
                    10 => Any::S10(Vault::create(price, p, e).unwrap()),
                    1_000_000_000 => Any::S9(Vault::create(price, p, e).unwrap()),
                    s => panic!("SCALE {s} non supportata"),
                });
                continue;
            }
            _ => {}
        }
        let vault = v.as_mut().unwrap();
        let before = with!(vault, x => (x.k, x.reserve, x.residual, x.supply));
        // (esito, [importo, fc, fp])
        let got: Result<Option<[u64; 3]>, BernieError> = with!(vault, x => match t[0] {
            "M" => x.mint(n(1) as u64, n(2) as u64).map(|m| Some([m.total_paid(), m.fees.creator, m.fees.protocol])),
            "R" => x.redeem(n(1) as u64, n(2) as u64).map(|r| Some([r.out, r.fees.creator, r.fees.protocol])),
            "D" => x.donate(n(1) as u64).map(|()| None),
            o => panic!("operazione {o}"),
        });
        let args = if t[0] == "D" { 2 } else { 3 };
        let (want_ok, rest) = (t[args] == "ok", &t[args + 1..]);
        let fail = |why: String| -> ! {
            eprintln!("DIVERGENZA {seed} passo {step}: {line}\n  {why}");
            std::process::exit(1);
        };
        let state_at = if want_ok && t[0] != "D" {
            3
        } else if want_ok {
            0
        } else {
            1
        };
        match (&got, want_ok) {
            (Ok(Some(vals)), true) => {
                for (i, name) in ["importo", "fc", "fp"].iter().enumerate() {
                    let w: u64 = rest[i].parse().unwrap();
                    if vals[i] != w {
                        fail(format!("{name}: atteso {w}, ottenuto {}", vals[i]));
                    }
                }
            }
            (Ok(None), true) => {}
            (Err(e), false) => {
                if !rest[0].split('|').any(|n| n == e.name()) {
                    fail(format!("errore {} invece di {}", e.name(), rest[0]));
                }
                let after = with!(vault, x => (x.k, x.reserve, x.residual, x.supply));
                if after != before {
                    fail("stato cambiato da un'operazione fallita".into());
                }
            }
            (g, _) => fail(format!("esito {g:?}, atteso {}", t[args..].join(" "))),
        }
        let s = &rest[state_at..];
        let want: (u128, u128, u128, u64) = (
            s[0].parse().unwrap(),
            s[1].parse().unwrap(),
            s[2].parse().unwrap(),
            s[3].parse().unwrap(),
        );
        let got_state = with!(vault, x => (x.k, x.reserve, x.residual, x.supply));
        if got_state != want {
            fail(format!("stato (k, R, Q, S) {got_state:?}, atteso {want:?}"));
        }
        step += 1;
        total += 1;
    }
    let _ = writeln!(
        std::io::stderr(),
        "diff_stream: {seqs} sequenze, {total} passi identici"
    );
}
