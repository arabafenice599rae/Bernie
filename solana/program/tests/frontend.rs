//! Il frontend contro il programma: le transazioni costruite da `frontend/index.html`
//! (ixCreate, ixAta, ixMint, ixApprove, ixRedeem, ixDonate, ixSweep, withBudget),
//! generate da `frontend/test/emit.mjs`, eseguite su Mollusk senza modifiche.

mod common;

use bernie_state::BernieError;
use common::*;
use serde_json::Value;
use solana_instruction::{AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use std::str::FromStr;

const COMPUTE_BUDGET: &str = "ComputeBudget111111111111111111111111111111";

fn pk(v: &Value) -> Pubkey {
    Pubkey::from_str(v.as_str().unwrap()).unwrap()
}

fn fixtures() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../frontend/test/fixtures.json"
    );
    serde_json::from_str(&std::fs::read_to_string(path).expect("node frontend/test/emit.mjs"))
        .unwrap()
}

/// Istruzioni della transazione e limite di CU chiesto con SetComputeUnitLimit.
fn decode(tx: &Value) -> (Vec<Instruction>, Option<u32>) {
    let (mut ixs, mut limit) = (Vec::new(), None);
    for ix in tx.as_array().unwrap() {
        let program = ix["program"].as_str().unwrap();
        let data: Vec<u8> = (0..ix["data"].as_str().unwrap().len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&ix["data"].as_str().unwrap()[i..i + 2], 16).unwrap())
            .collect();
        if program == COMPUTE_BUDGET {
            if data[0] == 2 {
                limit = Some(u32::from_le_bytes(data[1..5].try_into().unwrap()));
            }
            continue; // il budget lo applica il runtime, non fa parte del test di istruzione
        }
        let metas = ix["keys"]
            .as_array()
            .unwrap()
            .iter()
            .map(|k| {
                let key = pk(&k["pubkey"]);
                match (
                    k["signer"].as_bool().unwrap(),
                    k["writable"].as_bool().unwrap(),
                ) {
                    (s, true) => AccountMeta::new(key, s),
                    (s, false) => AccountMeta::new_readonly(key, s),
                }
            })
            .collect();
        ixs.push(Instruction::new_with_bytes(
            Pubkey::from_str(program).unwrap(),
            &data,
            metas,
        ));
    }
    (ixs, limit)
}

#[test]
fn frontend_transactions_run_on_the_program() {
    let fx = fixtures();
    assert_eq!(pk(&fx["program"]), PROGRAM_ID, "Program ID del frontend");
    let keys = &fx["keys"];
    let (creator, user, mint) = (pk(&keys["creator"]), pk(&keys["user"]), pk(&keys["mint"]));
    assert_eq!(pk(&keys["treasury"]), treasury(0));

    let mut env = Env::new();
    env.creator = creator;
    env.mint = mint;
    env.vault = Pubkey::find_program_address(&[b"vault", mint.as_ref()], &PROGRAM_ID).0;
    assert_eq!(
        pk(&fx["vault"]),
        env.vault,
        "vault PDA calcolato dal frontend"
    );
    env.fund(&creator, 100 * 1_000_000_000);
    env.fund(&user, 100 * 1_000_000_000);
    env.fund(&pk(&keys["whale"]), 15_000_000_000_000_000_000);
    let ata = pk(&fx["ata"]);

    for name in fx["order"].as_array().unwrap() {
        let name = name.as_str().unwrap();
        let (ixs, limit) = decode(&fx["txs"][name]);
        let before_creator = env.lamports(&creator);
        let r = env.run(&ixs);
        assert!(r.is_ok(), "{name}: {:?}", r.raw);
        let limit = limit.expect("withBudget imposta sempre il limite") as u64;
        println!("{name}: CU {} su limite {limit}", r.cu);
        assert!(
            r.cu <= limit,
            "{name}: {} CU oltre il limite del frontend {limit}",
            r.cu
        );
        match name {
            "create" => assert_eq!(env.account(&env.vault).owner, PROGRAM_ID),
            "mint" => assert_eq!(env.token_amount(&ata), 2_000_000_000),
            "mint_again" => assert_eq!(env.token_amount(&ata), 3_000_000_000),
            "mint_big" => assert_eq!(env.vault_state().supply, 400_000_000_000_000_000),
            "redeem" => assert_eq!(env.token_amount(&ata), 2_000_000_000),
            "sweep" => assert!(
                env.lamports(&creator) > before_creator,
                "fee del creator ritirate"
            ),
            _ => {}
        }
    }
    // user 2 token (dopo il redeem) + whale 4·10⁸ token.
    assert_eq!(env.vault_state().supply, 400_000_002_000_000_000);
}

#[test]
fn frontend_error_map_matches_program() {
    let codes = fixtures()["err_codes"].as_array().unwrap().clone();
    assert_eq!(codes.len(), 18, "codici 1–17 più lo zero");
    for (code, name) in codes.iter().enumerate().skip(1) {
        if let Some(e) = BernieError::from_code(code as u32) {
            assert_eq!(name.as_str().unwrap(), e.name(), "codice {code}");
        }
    }
}
