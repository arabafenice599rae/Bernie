//! Esaustivo su host con SCALE = 10 (§17): ogni stato canonico piccolo, ogni operazione.

use bernie_state::{split_fees, BernieError, Vault};

type V = Vault<10>;
const SC: u128 = 10;

fn canonical_states() -> Vec<V> {
    let mut out = Vec::new();
    for &(p, e) in &[
        (100u16, 0u16),
        (100, 100),
        (333, 150),
        (1000, 0),
        (1000, 1000),
    ] {
        for k in 1..=40u128 {
            for s in 0..=10u64 {
                let r = k * u128::from(s);
                let qs: Vec<u128> = if s == 0 {
                    vec![0]
                } else {
                    (0..u128::from(s)).collect()
                };
                for q in qs {
                    if (r + q) % SC == 0 && !(s == 0 && r != 0) {
                        out.push(V {
                            penalty_bps: p,
                            entry_bps: e,
                            k,
                            reserve: r,
                            residual: q,
                            supply: s,
                        });
                    }
                }
            }
        }
    }
    out
}

fn is_canonical(v: &V) -> bool {
    v.check(0).is_ok()
        && if v.supply == 0 {
            v.residual == 0 && v.reserve == 0
        } else {
            v.residual < u128::from(v.supply)
        }
}

#[test]
fn every_transition_is_canonical_or_rejected() {
    let states = canonical_states();
    assert!(states.len() > 1000);
    let mut ok = 0usize;
    for v in &states {
        let before = v.reserve + v.residual;
        for u in 1..=12u64 {
            let mut w = *v;
            match w.mint(u, u64::MAX) {
                Ok(m) => {
                    ok += 1;
                    assert!(is_canonical(&w) && w.k >= v.k, "{v:?} mint {u}");
                    // Entra solo c·SCALE nella contabilità; le fee restano fuori (I5).
                    assert_eq!(w.reserve + w.residual, before + u128::from(m.cost) * SC);
                    assert!(u128::from(m.cost) * SC >= m.full + m.epen);
                    assert!(m.full.div_ceil(SC) <= u128::from(m.cost));
                }
                Err(err) => {
                    assert_eq!(w, *v);
                    assert_eq!(err, BernieError::Slippage, "{v:?} mint {u}");
                }
            }
            let mut w = *v;
            match w.redeem(u, 0) {
                Ok(r) => {
                    ok += 1;
                    assert!(is_canonical(&w) && w.k >= v.k, "{v:?} redeem {u}");
                    assert!(r.out <= r.gross && u128::from(r.gross) * SC <= r.full - r.pen);
                    if w.supply == 0 {
                        assert_eq!(w.reserve + w.residual, 0);
                    } else {
                        assert_eq!(w.reserve + w.residual, before - u128::from(r.gross) * SC);
                    }
                }
                Err(err) => {
                    assert_eq!(w, *v);
                    assert!(matches!(
                        err,
                        BernieError::ExceedsSupply | BernieError::Dust | BernieError::ZeroPayout
                    ));
                    if u <= v.supply {
                        assert_ne!(err, BernieError::ExceedsSupply);
                    }
                }
            }
            let mut w = *v;
            match w.donate(u) {
                Ok(()) => {
                    assert!(is_canonical(&w) && w.k >= v.k);
                    assert_eq!(w.reserve + w.residual, before + u128::from(u) * SC);
                }
                Err(err) => {
                    assert_eq!(err, BernieError::NoHolders);
                    assert_eq!(v.supply, 0);
                }
            }
        }
    }
    assert!(ok > 10_000);
}

#[test]
fn fees_single_rounding() {
    let mut prev = split_fees(0);
    for base in 1..2_000_000u64 {
        let f = split_fees(base);
        assert!(f.total - prev.total <= 1);
        assert_eq!(f.creator + f.protocol, f.total);
        assert!(base - f.total >= (base - 1) - prev.total); // P7
        prev = f;
    }
    let f = split_fees(u64::MAX);
    assert_eq!(u128::from(f.total), u128::from(u64::MAX) * 40 / 10_000);
}
