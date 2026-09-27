//! Test di istruzione con Mollusk (§17): programma SBF reale contro Token-2022 reale.

mod common;

use common::*;
use serde_json::Value;
use solana_instruction::error::InstructionError;
use solana_pubkey::Pubkey;

const P: u64 = 1_000_000_000; // 1 SOL per token intero
const RICH: u64 = 15_000_000_000_000_000_000;

fn setup(p: u16, e: u16) -> Env {
    let mut env = Env::new();
    let r = env.create(
        P,
        p,
        e,
        b"Bernie Test",
        b"BRN",
        b"https://example.invalid/b.json",
    );
    assert!(r.is_ok(), "create: {:?}", r.raw);
    env
}

/// Scorre il TLV del mint fino al tipo richiesto (§11, lettura dei metadati).
fn tlv(data: &[u8], wanted: u16) -> Option<&[u8]> {
    let mut at = 166;
    while at + 4 <= data.len() {
        let ty = u16::from_le_bytes([data[at], data[at + 1]]);
        let len = u16::from_le_bytes([data[at + 2], data[at + 3]]) as usize;
        if ty == wanted {
            return Some(&data[at + 4..at + 4 + len]);
        }
        if ty == 0 {
            return None;
        }
        at += 4 + len;
    }
    None
}

#[test]
fn create_mint_metadata_and_vault() {
    let env = setup(200, 100);
    let mint = env.account(&env.mint);
    let d = &mint.data;
    assert_eq!(mint.owner, TOKEN_2022);
    // Base del mint: authority = vault PDA, supply 0, 9 decimali, freeze nulla.
    assert_eq!(&d[0..4], &1u32.to_le_bytes());
    assert_eq!(&d[4..36], env.vault.as_ref());
    assert_eq!(env.mint_supply(), 0);
    assert_eq!(d[44], 9);
    assert_eq!(&d[46..50], &0u32.to_le_bytes());
    assert_eq!(d[165], 1, "AccountType::Mint");
    // MetadataPointer (18): authority nulla, indirizzo = mint.
    let mp = tlv(d, 18).expect("MetadataPointer");
    assert_eq!(&mp[0..32], &[0u8; 32]);
    assert_eq!(&mp[32..64], env.mint.as_ref());
    // TokenMetadata (19): update authority nulla, nome, simbolo, URI.
    let md = tlv(d, 19).expect("TokenMetadata");
    assert_eq!(&md[0..32], &[0u8; 32], "metadati immutabili");
    assert_eq!(&md[32..64], env.mint.as_ref());
    assert_eq!(&md[64..68], &11u32.to_le_bytes());
    assert_eq!(&md[68..79], b"Bernie Test");
    // Vault v4.
    let v = env.account(&env.vault);
    assert_eq!(v.owner, PROGRAM_ID);
    assert_eq!(v.data.len(), VAULT_LEN);
    assert_eq!(&v.data[0..2], &[0x52, 4]);
    assert_eq!(&v.data[8..40], env.creator.as_ref());
    assert_eq!(&v.data[40..72], env.mint.as_ref());
    assert_eq!(v.lamports, rent(VAULT_LEN));
    assert_eq!(
        env.vault_state(),
        VaultState {
            k: P as u128,
            reserve: 0,
            residual: 0,
            supply: 0
        }
    );
}

#[test]
fn create_rejects_bad_parameters() {
    let mut env = Env::new();
    // (prezzo, p, e, nome, simbolo, URI, codice atteso)
    type Case<'a> = (u64, u16, u16, &'a [u8], &'a [u8], &'a [u8], u32);
    let cases: [Case; 6] = [
        (999_999, 200, 100, b"N", b"S", b"", 9),
        (1_000_000_000_000_001, 200, 100, b"N", b"S", b"", 9),
        (P, 99, 0, b"N", b"S", b"", 8),
        (P, 200, 201, b"N", b"S", b"", 8),
        (P, 200, 100, &[b'n'; 33], b"S", b"", 16),
        (P, 200, 100, b"N", b"SYMBOLTOOLONG", b"", 16),
    ];
    for (price, p, e, name, symbol, uri, code) in cases {
        let r = env.create(price, p, e, name, symbol, uri);
        assert_eq!(custom(&r), Some(code), "{price} {p} {e}");
    }
    let uri = [b'u'; 201];
    assert_eq!(custom(&env.create(P, 200, 100, b"N", b"S", &uri)), Some(16));
}

#[test]
fn lifecycle_with_sweep_and_reopen() {
    let env = setup(200, 100);
    let (alice, bob) = (Pubkey::new_unique(), Pubkey::new_unique());
    env.fund(&alice, 100 * P);
    env.fund(&bob, 100 * P);
    let (ata_a, ata_b) = (
        env.token_account(&alice, None),
        env.token_account(&bob, None),
    );
    let t0 = env.lamports(&treasury(0));

    // Mint: utente → vault c + fc, utente → tesoreria fp, token all'utente.
    let before = env.lamports(&alice);
    let r = env.run(&[env.mint_ix(&alice, &ata_a, 0, 3_000_000_000, u64::MAX)]);
    assert!(r.is_ok(), "{:?}", r.raw);
    let paid = before - env.lamports(&alice);
    // Primo mint: 3 token a 1 SOL, fee 0,4% del backing.
    assert_eq!(paid, 3 * P + 3 * P * 40 / 10_000);
    assert_eq!(env.lamports(&treasury(0)) - t0, 3 * P * 20 / 10_000);
    assert_eq!(env.token_amount(&ata_a), 3_000_000_000);
    assert_eq!(env.mint_supply(), 3_000_000_000);

    let r = env.run(&[env.mint_ix(&bob, &ata_b, 1, 1_000_000_000, u64::MAX)]);
    assert!(r.is_ok());
    let k_after_bob = env.vault_state().k;
    assert!(k_after_bob > P as u128, "l'epen di bob alza k per alice");

    // Il creator non firma e non compare: sweep lo paga dall'excess.
    let c0 = env.lamports(&env.creator);
    let r = env.run(&[env.sweep_ix(&env.creator)]);
    assert!(r.is_ok(), "{:?}", r.raw);
    let fc = env.lamports(&env.creator) - c0;
    assert!(fc > 0);
    assert_eq!(
        custom(&env.run(&[env.sweep_ix(&env.creator)])),
        Some(12),
        "NothingToClaim"
    );
    let intruder = Pubkey::new_unique();
    env.fund(&intruder, P);
    assert_eq!(
        env.run(&[env.sweep_ix(&intruder)]).raw,
        Err(InstructionError::InvalidArgument)
    );

    // Donate e redeem parziale.
    assert!(env.run(&[env.donate_ix(&bob, 10_000_000)]).is_ok());
    let r = env.run(&[
        env.approve_ix(&alice, &ata_a, 1_000_000_000),
        env.redeem_ix(&alice, &ata_a, 2, 1_000_000_000, 1),
    ]);
    assert!(r.is_ok(), "{:?}", r.raw);
    assert_eq!(env.token_amount(&ata_a), 2_000_000_000);

    // Tutti escono: ultimo uscente, stato vuoto, excess al creator, rientro al k raggiunto.
    for (who, ata, t) in [(alice, ata_a, 0), (bob, ata_b, 3)] {
        let amt = env.token_amount(&ata);
        let r = env.run(&[
            env.approve_ix(&who, &ata, amt),
            env.redeem_ix(&who, &ata, t, amt, 1),
        ]);
        assert!(r.is_ok(), "{:?}", r.raw);
    }
    let s = env.vault_state();
    assert_eq!((s.supply, s.reserve, s.residual), (0, 0, 0));
    let k_end = s.k;
    assert!(env.lamports(&env.vault) > rent(VAULT_LEN));
    assert!(env.run(&[env.sweep_ix(&env.creator)]).is_ok());
    assert_eq!(
        env.lamports(&env.vault),
        rent(VAULT_LEN),
        "vault aperto, solo rent"
    );
    assert_eq!(
        custom(&env.run(&[env.donate_ix(&bob, 1)])),
        Some(6),
        "NoHolders"
    );
    assert!(env
        .run(&[env.mint_ix(&bob, &ata_b, 0, 1_000, u64::MAX)])
        .is_ok());
    // Rientro al k raggiunto: il resto di arrotondamento del mint (< SCALE sotto-unità)
    // va alle sole 1.000 unità del nuovo entrante.
    let k = env.vault_state().k;
    assert!(k >= k_end && k - k_end <= SCALE / 1_000, "{k} {k_end}");
}

#[test]
fn redeem_with_cpi_guard_on_and_off() {
    for guard in [None, Some(false), Some(true)] {
        let env = setup(500, 250);
        let user = Pubkey::new_unique();
        env.fund(&user, 100 * P);
        let ata = env.token_account(&user, guard);
        assert!(env
            .run(&[env.mint_ix(&user, &ata, 0, 2_000_000_000, u64::MAX)])
            .is_ok());
        let r = env.run(&[
            env.approve_ix(&user, &ata, 2_000_000_000),
            env.redeem_ix(&user, &ata, 0, 2_000_000_000, 1),
        ]);
        assert!(r.is_ok(), "CPI Guard {guard:?}: {:?}", r.raw);
        assert_eq!(env.token_amount(&ata), 0);
        if let Some(lock) = guard {
            assert_eq!(
                *env.account(&ata).data.last().unwrap(),
                lock as u8,
                "CPI Guard invariato"
            );
        }
    }
}

#[test]
fn redeem_needs_own_account_and_delegation() {
    let env = setup(200, 100);
    let (alice, eve) = (Pubkey::new_unique(), Pubkey::new_unique());
    env.fund(&alice, 100 * P);
    env.fund(&eve, 100 * P);
    let ata_a = env.token_account(&alice, None);
    assert!(env
        .run(&[env.mint_ix(&alice, &ata_a, 0, 1_000_000_000, u64::MAX)])
        .is_ok());

    // Senza delega: MissingDelegation (15).
    assert_eq!(
        custom(&env.run(&[env.redeem_ix(&alice, &ata_a, 0, 1_000, 0)])),
        Some(15)
    );
    // Delega inferiore a u.
    let r = env.run(&[
        env.approve_ix(&alice, &ata_a, 999),
        env.redeem_ix(&alice, &ata_a, 0, 1_000, 0),
    ]);
    assert_eq!(custom(&r), Some(15));
    // Alice ha delegato il vault: Eve non può riscattare dal conto di Alice.
    assert!(env
        .run(&[env.approve_ix(&alice, &ata_a, 1_000_000_000)])
        .is_ok());
    let r = env.run(&[env.redeem_ix(&eve, &ata_a, 0, 1_000_000_000, 0)]);
    assert_eq!(r.raw, Err(InstructionError::InvalidAccountData));
    assert_eq!(env.token_amount(&ata_a), 1_000_000_000);
}

#[test]
fn rejects_unknown_treasury() {
    let env = setup(200, 100);
    let user = Pubkey::new_unique();
    env.fund(&user, 100 * P);
    let ata = env.token_account(&user, None);
    let mut ix = env.mint_ix(&user, &ata, 0, 1_000, u64::MAX);
    let fake = Pubkey::new_unique();
    env.fund(&fake, rent(0));
    ix.accounts[4].pubkey = fake;
    assert_eq!(env.run(&[ix]).raw, Err(InstructionError::InvalidArgument));
}

/// Differenziale (§17): i vettori SCALE 10⁹ del modello, eseguiti dal programma reale.
#[test]
fn vectors_scale_1e9() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../vectors/scale_1e9.json");
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let int = |v: &Value| -> u128 { v.as_str().unwrap().parse().unwrap() };
    let arg = |v: &Value, d: u64| -> u64 {
        if v.is_null() {
            d
        } else {
            int(v) as u64
        }
    };
    let (mut steps, mut max_cu) = (0usize, [0u64; 3]);
    for case in doc["cases"].as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        let prm = &case["params"];
        let mut env = Env::new();
        let r = env.create(
            int(&prm["price"]) as u64,
            prm["penalty_bps"].as_u64().unwrap() as u16,
            prm["entry_bps"].as_u64().unwrap() as u16,
            b"V",
            b"V",
            b"",
        );
        assert!(r.is_ok(), "{name}: create {:?}", r.raw);
        let user = Pubkey::new_unique();
        env.fund(&user, RICH);
        let ata = env.token_account(&user, None);
        let (vault0, treas0) = (env.lamports(&env.vault), env.lamports(&treasury(0)));
        for (i, st) in case["steps"].as_array().unwrap().iter().enumerate() {
            let op = st["op"].as_str().unwrap();
            let (r, slot) = match op {
                "mint" => (
                    env.run(&[env.mint_ix(
                        &user,
                        &ata,
                        0,
                        arg(&st["u"], 0),
                        arg(&st["max_cost"], u64::MAX),
                    )]),
                    0,
                ),
                "redeem" => {
                    let u = arg(&st["u"], 0);
                    (
                        env.run(&[
                            env.approve_ix(&user, &ata, u),
                            env.redeem_ix(&user, &ata, 0, u, arg(&st["min_out"], 0)),
                        ]),
                        1,
                    )
                }
                "donate" => (env.run(&[env.donate_ix(&user, arg(&st["a"], 0))]), 2),
                other => panic!("{other}"),
            };
            let ctx = format!("{name} passo {i} ({op})");
            let exp = &st["expect"];
            if exp["ok"].as_bool().unwrap() {
                assert!(r.is_ok(), "{ctx}: {:?}", r.raw);
                max_cu[slot] = max_cu[slot].max(r.cu);
            } else {
                assert_eq!(
                    custom(&r),
                    Some(code_of(exp["error"].as_str().unwrap())),
                    "{ctx}"
                );
            }
            // Fee del creator cumulative del modello: su Solana restano nel vault (v1.6).
            let fc_cum = int(&st["state"]["fc"]);
            let s = &st["state"];
            let v = env.vault_state();
            assert_eq!(v.k, int(&s["k"]), "{ctx}: k");
            assert_eq!(v.reserve, int(&s["R"]), "{ctx}: R");
            assert_eq!(v.residual, int(&s["Q"]), "{ctx}: Q");
            assert_eq!(u128::from(v.supply), int(&s["S"]), "{ctx}: S");
            assert_eq!(
                u128::from(env.mint_supply()),
                int(&s["S"]),
                "{ctx}: supply del mint"
            );
            assert_eq!(
                u128::from(env.token_amount(&ata)),
                int(&s["S"]),
                "{ctx}: saldo token"
            );
            // v1.6: nel vault restano backing, resti ed excess più le fee del creator.
            assert_eq!(
                u128::from(env.lamports(&env.vault) - vault0),
                int(&s["bal"]) + fc_cum,
                "{ctx}: lamport del vault"
            );
            assert_eq!(
                u128::from(env.lamports(&treasury(0)) - treas0),
                int(&s["fp"]),
                "{ctx}: tesoreria"
            );
            steps += 1;
        }
    }
    println!(
        "passi {steps}; CU massime mint {} redeem {} donate {}",
        max_cu[0], max_cu[1], max_cu[2]
    );
    assert!(steps > 1000);
}

/// Budget di calcolo di §11, misurato: il test fallisce se un'istruzione supera la stima.
#[test]
fn compute_units() {
    let mut env = Env::new();
    let create = env.create(P, 200, 100, &[b'n'; 32], &[b's'; 10], &[b'u'; 200]);
    assert!(create.is_ok());
    let user = Pubkey::new_unique();
    env.fund(&user, 100 * P);
    let ata = env.token_account(&user, None);
    let mint = env.run(&[env.mint_ix(&user, &ata, 0, 5_000_000_000, u64::MAX)]);
    let other = Pubkey::new_unique();
    env.fund(&other, 100 * P);
    let ata_o = env.token_account(&other, None);
    let mint2 = env.run(&[env.mint_ix(&other, &ata_o, 1, 1_000_000_000, u64::MAX)]);
    let approve = env.run(&[env.approve_ix(&user, &ata, 1_000_000_000)]);
    let redeem = env.run(&[env.redeem_ix(&user, &ata, 0, 1_000_000_000, 1)]);
    let donate = env.run(&[env.donate_ix(&user, 1_000_000)]);
    let sweep = env.run(&[env.sweep_ix(&env.creator)]);
    for r in [&create, &mint, &mint2, &approve, &redeem, &donate, &sweep] {
        assert!(r.is_ok(), "{:?}", r.raw);
    }
    println!(
        "CU create {} mint {} mint(S>0) {} approve {} redeem {} donate {} sweep {}",
        create.cu, mint.cu, mint2.cu, approve.cu, redeem.cu, donate.cu, sweep.cu
    );
    assert!(create.cu <= 90_000 && mint2.cu.max(mint.cu) <= 60_000 && redeem.cu <= 45_000);
}
