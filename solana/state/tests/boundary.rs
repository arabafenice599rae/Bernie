//! Vettori di confine (pre-audit, fasi 5–9): `vectors/boundary_sol.txt`, generati da
//! `tools/redteam/boundary.py` con l'oracolo dell'Appendice A e i domini di Solana.
//!
//! Ogni caso parte da uno stato costruito direttamente (`Z k R Q S p e`: S = 0, 1, 2, grande;
//! Q = 0, 1, S − 1; k al prezzo minimo, massimo e oltre) e applica un'operazione con
//! argomenti ai bordi (0, 1, 2, S − 1, S, S + 1, u64::MAX − 1, u64::MAX; slippage esatto
//! − 1, esatto, esatto + 1). Si confrontano esito, importi, fee e stato finale; un errore
//! deve lasciare lo stato identico.

use bernie_state::{BernieError, Vault};

type V = Vault<1_000_000_000>;

#[test]
fn boundary_vectors_solana() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../vectors/boundary_sol.txt"
    );
    let text = std::fs::read_to_string(path).expect("eseguire tools/redteam/boundary.py");
    let mut lines = text.lines().filter(|l| !l.starts_with('#'));
    let mut cases = 0;
    while let Some(z) = lines.next() {
        let line = lines.next().expect("riga d'operazione dopo Z");
        let n = |s: &str| -> u128 { s.parse().unwrap() };
        let z: Vec<&str> = z.split_whitespace().collect();
        assert_eq!(z[0], "Z");
        let mut v = V {
            k: n(z[1]),
            reserve: n(z[2]),
            residual: n(z[3]),
            supply: n(z[4]) as u64,
            penalty_bps: n(z[5]) as u16,
            entry_bps: n(z[6]) as u16,
        };
        let start = v;
        let t: Vec<&str> = line.split_whitespace().collect();
        let got: Result<Option<[u64; 3]>, BernieError> = match t[0] {
            "M" => v
                .mint(n(t[1]) as u64, n(t[2]) as u64)
                .map(|m| Some([m.total_paid(), m.fees.creator, m.fees.protocol])),
            "R" => v
                .redeem(n(t[1]) as u64, n(t[2]) as u64)
                .map(|r| Some([r.out, r.fees.creator, r.fees.protocol])),
            "D" => v.donate(n(t[1]) as u64).map(|()| None),
            o => panic!("{o}"),
        };
        let at = if t[0] == "D" { 2 } else { 3 };
        let ctx = format!("stato {z:?}, operazione `{line}`");
        let rest = &t[at + 1..];
        let state_from = match (&got, t[at]) {
            (Ok(Some(vals)), "ok") => {
                for i in 0..3 {
                    assert_eq!(vals[i], n(rest[i]) as u64, "importo/fee {i}: {ctx}");
                }
                3
            }
            (Ok(None), "ok") => 0,
            (Err(e), "err") => {
                // un solo errore previsto: il primo nell'ordine di §7
                assert_eq!(e.name(), rest[0], "errore diverso: {ctx}");
                assert_eq!(v, start, "stato cambiato da un errore: {ctx}");
                1
            }
            (g, w) => panic!("esito {g:?}, atteso {w}: {ctx}"),
        };
        let s = &rest[state_from..];
        assert_eq!(
            (v.k, v.reserve, v.residual, v.supply as u128),
            (n(s[0]), n(s[1]), n(s[2]), n(s[3])),
            "stato finale: {ctx}"
        );
        cases += 1;
    }
    assert!(cases > 1000, "troppo pochi casi: {cases}");
}

/// absorb (fase 8): Q = 0, 1, S − 1, S, S + 1, 2S, molto grande; ripetuto è idempotente.
#[test]
fn absorb_boundaries_and_idempotence() {
    for s in [1u64, 2, 3, 7, 1_000_000_000, u64::MAX / 2] {
        let su = s as u128;
        for q in [0u128, 1, su - 1, su, su + 1, 2 * su, 2 * su + 1, 1 << 100] {
            let k = 1_000_000u128;
            let mut v = V {
                k,
                reserve: su * k,
                residual: q,
                supply: s,
                penalty_bps: 200,
                entry_bps: 100,
            };
            let total = v.reserve + v.residual;
            v.absorb().unwrap();
            // oracolo indipendente: δ = ⌊Q/S⌋
            let d = q / su;
            assert_eq!(v.k, k + d, "k: S {s}, Q {q}");
            assert_eq!(v.reserve, su * (k + d));
            assert_eq!(v.residual, q - d * su);
            assert!(v.residual < su, "Q' < S");
            assert_eq!(v.reserve + v.residual, total, "R + Q conservato");
            let once = v;
            v.absorb().unwrap();
            v.absorb().unwrap();
            assert_eq!(v, once, "absorb ripetuto non cambia nulla");
        }
    }
    let mut empty = V::create(1_000_000, 200, 100).unwrap();
    empty.residual = 0;
    let before = empty;
    empty.absorb().unwrap();
    assert_eq!(empty, before, "S = 0: nessun effetto");
}

/// Fee (fase 7): importi elencati nel piano, oracolo indipendente con divisione intera.
#[test]
fn fee_table() {
    for base in [
        0u64,
        1,
        2,
        3,
        10,
        99,
        100,
        101,
        249,
        250,
        251,
        499,
        500,
        501,
        999,
        1000,
        10_000,
        u64::MAX - 1,
        u64::MAX,
    ] {
        let f = bernie_state::split_fees(base);
        let b = base as u128;
        let total = b * 40 / 10_000; // un solo arrotondamento sulla fee totale (§5)
        let protocol = total * 20 / 40;
        assert_eq!(f.total as u128, total, "totale, base {base}");
        assert_eq!(f.protocol as u128, protocol, "protocollo, base {base}");
        assert_eq!(f.creator as u128, total - protocol, "creator, base {base}");
        assert_eq!(f.creator + f.protocol, f.total);
        assert!(
            f.creator >= f.protocol && f.creator - f.protocol <= 1,
            "ripartizione 50/50 ±1"
        );
    }
}
