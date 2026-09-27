//! Guardie difensive di §4 e soglie di slippage di §6, verificate direttamente.
//!
//! Aggiunti dal pre-audit (docs/testing): senza questi test i mutanti che sostituiscono
//! `check` o `check_solvency` con `Ok(())`, o che spostano `<` in `<=` sulle soglie,
//! sopravvivevano. L'oracolo è la definizione delle invarianti in §4, non il codice.

use bernie_state::*;

type V = Vault<10>;

/// Stato canonico: k = 7, S = 4, R = 28, Q = 2 (R + Q = 30, multiplo di SCALE).
fn canonical() -> V {
    V {
        penalty_bps: 200,
        entry_bps: 100,
        k: 7,
        reserve: 28,
        residual: 2,
        supply: 4,
    }
}

#[test]
fn check_accepts_canonical_state() {
    assert_eq!(canonical().check(7), Ok(()));
    assert_eq!(canonical().check(6), Ok(()), "k cresciuto");
    let empty = V {
        supply: 0,
        reserve: 0,
        residual: 0,
        ..canonical()
    };
    assert_eq!(empty.check(7), Ok(()), "vault vuoto");
}

#[test]
fn check_rejects_each_invariant_in_isolation() {
    let bad = Err(BernieError::InvariantViolated);
    // I1: R ≠ S·k (R + Q resta multiplo di SCALE, Q < S).
    let i1 = V {
        reserve: 18,
        residual: 2,
        ..canonical()
    };
    assert_eq!(i1.check(7), bad, "I1");
    // I3: k diminuito.
    assert_eq!(canonical().check(8), bad, "I3");
    // I4: S = 0 con Q > 0 (Q multiplo di SCALE, quindi I5 regge; I6 non si applica).
    let i4 = V {
        supply: 0,
        reserve: 0,
        residual: 10,
        ..canonical()
    };
    assert_eq!(i4.check(7), bad, "I4");
    // I5: R + Q non multiplo di SCALE.
    let i5 = V {
        residual: 3,
        ..canonical()
    };
    assert_eq!(i5.check(7), bad, "I5");
    // I6: Q = S e Q > S, con R + Q multiplo di SCALE.
    let i6_eq = V {
        k: 4,
        reserve: 16,
        residual: 4,
        supply: 4,
        ..canonical()
    };
    assert_eq!(i6_eq.check(4), bad, "I6, Q = S");
    let i6_gt = V {
        k: 4,
        reserve: 16,
        residual: 14,
        supply: 4,
        ..canonical()
    };
    assert_eq!(i6_gt.check(4), bad, "I6, Q > S");
    // R + Q oltre u128: I5 non valutabile, rifiutato.
    // S = 5 divide 2¹²⁸ − 1, quindi R = S·k = u128::MAX esatto e R + Q trabocca.
    let ovf = V {
        k: u128::MAX / 5,
        reserve: u128::MAX,
        residual: 1,
        supply: 5,
        ..canonical()
    };
    assert_eq!(ovf.reserve, 5 * ovf.k, "I1 regge: fallisce solo R + Q");
    assert!(ovf.reserve.checked_add(ovf.residual).is_none());
    assert_eq!(ovf.check(0), bad, "R + Q in overflow");
}

#[test]
fn solvency_excess_and_sweep_thresholds() {
    let v = canonical(); // passività (R + Q) / SCALE = 3
    assert_eq!(v.liabilities(), Ok(3));
    assert_eq!(v.check_solvency(3), Ok(()));
    assert_eq!(v.check_solvency(2), Err(BernieError::InvariantViolated));
    assert_eq!(v.check_solvency(u64::MAX), Ok(()));
    assert_eq!(v.excess(2), Err(BernieError::InvariantViolated));
    assert_eq!(v.excess(3), Ok(0));
    assert_eq!(v.excess(4), Ok(1));
    assert_eq!(v.sweep_amount(3), Err(BernieError::NothingToClaim));
    assert_eq!(v.sweep_amount(4), Ok(1));
    assert_eq!(v.sweep_amount(2), Err(BernieError::InvariantViolated));
}

#[test]
fn redeem_min_out_is_inclusive() {
    let mut v = V::create(100, 200, 100).unwrap();
    v.mint(50, u64::MAX).unwrap();
    let out = {
        let mut w = v;
        w.redeem(10, 0).unwrap().out
    };
    let mut w = v;
    assert_eq!(
        w.redeem(10, out).map(|r| r.out),
        Ok(out),
        "min_out = out passa"
    );
    let mut w2 = v;
    assert_eq!(w2.redeem(10, out + 1), Err(BernieError::Slippage));
    assert_eq!(w2, v, "slippage senza effetti");
}

#[test]
fn mint_max_cost_is_inclusive() {
    let mut v = V::create(100, 200, 100).unwrap();
    v.mint(50, u64::MAX).unwrap();
    let paid = {
        let mut w = v;
        w.mint(7, u64::MAX).unwrap().total_paid()
    };
    let mut w = v;
    assert!(w.mint(7, paid).is_ok(), "max_cost = pagato passa");
    let mut w2 = v;
    assert_eq!(w2.mint(7, paid - 1), Err(BernieError::Slippage));
    assert_eq!(w2, v, "slippage senza effetti");
}
