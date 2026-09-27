//! create, metadati, codici d'errore, sweep (§6, §7, §20).

use bernie_state::*;

#[test]
fn error_codes_match_section_7() {
    use BernieError::*;
    let table = [
        (ZeroAmount, 1),
        (ExceedsSupply, 2),
        (Dust, 3),
        (ZeroPayout, 4),
        (Slippage, 5),
        (NoHolders, 6),
        (PenaltyOutOfRange, 8),
        (PriceOutOfRange, 9),
        (Overflow, 10),
        (InvariantViolated, 11),
        (NothingToClaim, 12),
        (TransferToSelf, 13),
        (SupplyMismatch, 14),
        (MissingDelegation, 15),
        (MetadataTooLong, 16),
        (NotOwner, 17),
    ];
    for (e, code) in table {
        assert_eq!(e.code(), code, "{}", e.name());
        assert_eq!(BernieError::from_code(code), Some(e));
    }
    assert_eq!(BernieError::from_code(7), None, "7 riservato");
    assert_eq!(BernieError::from_code(18), None);
}

#[test]
fn create_params() {
    assert!(validate_params(MIN_PRICE, 100, 0).is_ok());
    assert!(validate_params(MAX_PRICE, 1000, 1000).is_ok());
    assert_eq!(
        validate_params(MIN_PRICE - 1, 200, 100),
        Err(BernieError::PriceOutOfRange)
    );
    assert_eq!(
        validate_params(MAX_PRICE + 1, 200, 100),
        Err(BernieError::PriceOutOfRange)
    );
    assert_eq!(
        validate_params(MIN_PRICE, 99, 0),
        Err(BernieError::PenaltyOutOfRange)
    );
    assert_eq!(
        validate_params(MIN_PRICE, 1001, 0),
        Err(BernieError::PenaltyOutOfRange)
    );
    assert_eq!(
        validate_params(MIN_PRICE, 200, 201),
        Err(BernieError::PenaltyOutOfRange)
    );
    let v = Vault::<SOLANA_SCALE>::create(MIN_PRICE, 200, 100).unwrap();
    assert_eq!(
        (v.k, v.reserve, v.residual, v.supply),
        (u128::from(MIN_PRICE), 0, 0, 0)
    );
}

#[test]
fn metadata_limits() {
    let a = |n: usize| vec![b'a'; n];
    assert!(validate_metadata(&a(1), &a(1), &a(0)).is_ok());
    assert!(validate_metadata(&a(32), &a(10), &a(200)).is_ok());
    for (n, s, u) in [(0, 1, 0), (33, 1, 0), (1, 0, 0), (1, 11, 0), (1, 1, 201)] {
        assert_eq!(
            validate_metadata(&a(n), &a(s), &a(u)),
            Err(BernieError::MetadataTooLong)
        );
    }
    // Limiti in byte UTF-8, non in caratteri: 11 × "è" = 22 byte > 10.
    assert!(validate_metadata("è".repeat(16).as_bytes(), b"B", b"").is_ok());
    assert!(validate_metadata(b"Bernie", "è".repeat(6).as_bytes(), b"").is_err());
}

#[test]
fn sweep_on_solana() {
    // v1.6: la fee del creator resta nel vault come excess ed esce solo con sweep.
    let mut v = Vault::<SOLANA_SCALE>::create(1_000_000_000, 200, 100).unwrap();
    let mut lamports = 0u64; // lamport del vault oltre il rent
    let m = v.mint(5_000_000_000, u64::MAX).unwrap();
    lamports += m.cost + m.fees.creator; // v1.6: c + fc al vault, fp alla tesoreria
    let excess = v.sweep_amount(lamports).unwrap();
    assert_eq!(excess, m.fees.creator);
    let before = v;
    lamports -= excess;
    assert_eq!(v, before, "sweep non tocca lo stato");
    assert_eq!(v.sweep_amount(lamports), Err(BernieError::NothingToClaim));
    let r = v.redeem(5_000_000_000, 0).unwrap();
    lamports += r.fees.creator;
    lamports -= r.out + r.fees.protocol;
    // Ultimo uscente: vault vuoto, penalità + resti + fc sono excess.
    assert_eq!((v.supply, v.reserve, v.residual), (0, 0, 0));
    assert_eq!(v.sweep_amount(lamports).unwrap(), lamports);
    assert!(v.check_solvency(0).is_ok());
}
