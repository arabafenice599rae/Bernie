//! Ambiente Mollusk: programma Bernie compilato in SBF, Token-2022 reale, store persistente.
#![allow(dead_code)]

use mollusk_svm::{program::loader_keys::LOADER_V3, Mollusk, MolluskContext};
use mollusk_svm_programs_token_2022 as token2022;
use solana_account::Account;
use solana_instruction::{error::InstructionError, AccountMeta, Instruction};
use solana_pubkey::Pubkey;
use solana_transaction_error::TransactionError;
use std::collections::HashMap;

pub const PROGRAM_ID: Pubkey = Pubkey::new_from_array(bernie_program::ID.to_bytes());
pub const TOKEN_2022: Pubkey = token2022::ID;
pub const SYSTEM: Pubkey = solana_sdk_ids::system_program::ID;
pub const VAULT_LEN: usize = 128;
pub const SCALE: u128 = 1_000_000_000;

pub fn treasury(i: usize) -> Pubkey {
    Pubkey::new_from_array(bernie_program::TREASURIES[i].to_bytes())
}

pub fn rent(len: usize) -> u64 {
    solana_rent::Rent::default().minimum_balance(len)
}

/// Esito normalizzato di un'istruzione o di una transazione.
pub struct Outcome {
    pub raw: Result<(), InstructionError>,
    pub cu: u64,
}

impl Outcome {
    pub fn is_ok(&self) -> bool {
        self.raw.is_ok()
    }
}

pub struct Env {
    pub ctx: MolluskContext<HashMap<Pubkey, Account>>,
    pub creator: Pubkey,
    pub mint: Pubkey,
    pub vault: Pubkey,
}

/// Stato del vault letto dall'account (§3).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct VaultState {
    pub k: u128,
    pub reserve: u128,
    pub residual: u128,
    pub supply: u64,
}

impl Env {
    pub fn new() -> Self {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../target/deploy/bernie_program.so"
        );
        let elf = std::fs::read(path)
            .expect("eseguire `cargo build-sbf` in solana/program prima dei test");
        let mut mollusk = Mollusk::default();
        mollusk.add_program_with_loader_and_elf(&PROGRAM_ID, &LOADER_V3, &elf);
        token2022::add_program(&mut mollusk);
        mollusk_svm_programs_token::associated_token::add_program(&mut mollusk);
        let ctx = mollusk.with_context(HashMap::new());
        let env = Self {
            ctx,
            creator: Pubkey::new_unique(),
            mint: Pubkey::new_unique(),
            vault: Pubkey::default(),
        };
        env.fund(&env.creator, 100_000_000_000);
        for i in 0..bernie_program::TREASURIES.len() {
            env.fund(&treasury(i), rent(0));
        }
        env
    }

    pub fn fund(&self, key: &Pubkey, lamports: u64) {
        self.ctx.account_store.borrow_mut().insert(
            *key,
            Account {
                lamports,
                owner: SYSTEM,
                ..Account::default()
            },
        );
    }

    pub fn account(&self, key: &Pubkey) -> Account {
        self.ctx
            .account_store
            .borrow()
            .get(key)
            .cloned()
            .unwrap_or_default()
    }

    pub fn lamports(&self, key: &Pubkey) -> u64 {
        self.account(key).lamports
    }

    /// Un'istruzione, o più istruzioni in una transazione atomica (approve + redeem).
    pub fn run(&self, ixs: &[Instruction]) -> Outcome {
        if let [ix] = ixs {
            let r = self.ctx.process_instruction(ix);
            return Outcome {
                raw: r.raw_result,
                cu: r.compute_units_consumed,
            };
        }
        let r = self.ctx.process_transaction_instructions(ixs, None);
        let raw = match r.raw_result {
            Ok(()) => Ok(()),
            Err(TransactionError::InstructionError(_, e)) => Err(e),
            Err(other) => panic!("errore di transazione: {other:?}"),
        };
        Outcome {
            raw,
            cu: r.compute_units_consumed,
        }
    }

    pub fn create(
        &mut self,
        price: u64,
        p: u16,
        e: u16,
        name: &[u8],
        symbol: &[u8],
        uri: &[u8],
    ) -> Outcome {
        self.new_mint();
        self.create_current(price, p, e, name, symbol, uri)
    }

    /// Sceglie un mint nuovo e il suo vault PDA, senza inviare create.
    pub fn new_mint(&mut self) {
        self.mint = Pubkey::new_unique();
        self.vault = Pubkey::find_program_address(&[b"vault", self.mint.as_ref()], &PROGRAM_ID).0;
    }

    /// create sul mint già scelto (per i test che preparano gli indirizzi).
    pub fn create_current(
        &mut self,
        price: u64,
        p: u16,
        e: u16,
        name: &[u8],
        symbol: &[u8],
        uri: &[u8],
    ) -> Outcome {
        let mut data = vec![0u8];
        data.extend_from_slice(&price.to_le_bytes());
        data.extend_from_slice(&p.to_le_bytes());
        data.extend_from_slice(&e.to_le_bytes());
        for s in [name, symbol, uri] {
            data.push(s.len() as u8);
            data.extend_from_slice(s);
        }
        let ix = Instruction::new_with_bytes(
            PROGRAM_ID,
            &data,
            vec![
                AccountMeta::new(self.creator, true),
                AccountMeta::new(self.mint, true),
                AccountMeta::new(self.vault, false),
                AccountMeta::new_readonly(SYSTEM, false),
                AccountMeta::new_readonly(TOKEN_2022, false),
            ],
        );
        self.run(&[ix])
    }

    /// Token account Token-2022 dell'utente, opzionalmente con CPI Guard.
    pub fn token_account(&self, owner: &Pubkey, cpi_guard: Option<bool>) -> Pubkey {
        let key = Pubkey::new_unique();
        let mut data = vec![0u8; 165];
        data[0..32].copy_from_slice(self.mint.as_ref());
        data[32..64].copy_from_slice(owner.as_ref());
        data[108] = 1; // Initialized
        if let Some(lock) = cpi_guard {
            data.push(2); // AccountType::Account
            data.extend_from_slice(&11u16.to_le_bytes()); // ExtensionType::CpiGuard
            data.extend_from_slice(&1u16.to_le_bytes());
            data.push(lock as u8);
        }
        let lamports = rent(data.len());
        self.ctx.account_store.borrow_mut().insert(
            key,
            Account {
                lamports,
                data,
                owner: TOKEN_2022,
                ..Account::default()
            },
        );
        key
    }

    pub fn token_amount(&self, acc: &Pubkey) -> u64 {
        u64::from_le_bytes(self.account(acc).data[64..72].try_into().unwrap())
    }

    pub fn mint_supply(&self) -> u64 {
        u64::from_le_bytes(self.account(&self.mint).data[36..44].try_into().unwrap())
    }

    pub fn vault_state(&self) -> VaultState {
        let d = self.account(&self.vault).data;
        let u = |o: usize| u128::from_le_bytes(d[o..o + 16].try_into().unwrap());
        VaultState {
            k: u(72),
            reserve: u(88),
            residual: u(104),
            supply: u64::from_le_bytes(d[120..128].try_into().unwrap()),
        }
    }

    pub fn mint_ix(
        &self,
        user: &Pubkey,
        ata: &Pubkey,
        t: usize,
        u: u64,
        max_cost: u64,
    ) -> Instruction {
        let mut data = vec![1u8];
        data.extend_from_slice(&u.to_le_bytes());
        data.extend_from_slice(&max_cost.to_le_bytes());
        Instruction::new_with_bytes(
            PROGRAM_ID,
            &data,
            vec![
                AccountMeta::new(*user, true),
                AccountMeta::new(*ata, false),
                AccountMeta::new(self.mint, false),
                AccountMeta::new(self.vault, false),
                AccountMeta::new(treasury(t), false),
                AccountMeta::new_readonly(SYSTEM, false),
                AccountMeta::new_readonly(TOKEN_2022, false),
            ],
        )
    }

    /// ApproveChecked Token-2022 verso il vault PDA (§11, composizione del client).
    pub fn approve_ix(&self, owner: &Pubkey, ata: &Pubkey, amount: u64) -> Instruction {
        let mut data = vec![13u8];
        data.extend_from_slice(&amount.to_le_bytes());
        data.push(9);
        Instruction::new_with_bytes(
            TOKEN_2022,
            &data,
            vec![
                AccountMeta::new(*ata, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new_readonly(self.vault, false),
                AccountMeta::new_readonly(*owner, true),
            ],
        )
    }

    pub fn redeem_ix(
        &self,
        user: &Pubkey,
        ata: &Pubkey,
        t: usize,
        u: u64,
        min_out: u64,
    ) -> Instruction {
        let mut data = vec![2u8];
        data.extend_from_slice(&u.to_le_bytes());
        data.extend_from_slice(&min_out.to_le_bytes());
        Instruction::new_with_bytes(
            PROGRAM_ID,
            &data,
            vec![
                AccountMeta::new(*user, true),
                AccountMeta::new(*ata, false),
                AccountMeta::new(self.mint, false),
                AccountMeta::new(self.vault, false),
                AccountMeta::new(treasury(t), false),
                AccountMeta::new_readonly(TOKEN_2022, false),
            ],
        )
    }

    pub fn donate_ix(&self, donor: &Pubkey, a: u64) -> Instruction {
        let mut data = vec![3u8];
        data.extend_from_slice(&a.to_le_bytes());
        Instruction::new_with_bytes(
            PROGRAM_ID,
            &data,
            vec![
                AccountMeta::new(*donor, true),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new(self.vault, false),
                AccountMeta::new_readonly(SYSTEM, false),
            ],
        )
    }

    pub fn sweep_ix(&self, creator: &Pubkey) -> Instruction {
        Instruction::new_with_bytes(
            PROGRAM_ID,
            &[4u8],
            vec![
                AccountMeta::new(*creator, false),
                AccountMeta::new_readonly(self.mint, false),
                AccountMeta::new(self.vault, false),
            ],
        )
    }
}

/// Codice custom di un errore Bernie (§7), come lo vede il runtime.
pub fn custom(result: &Outcome) -> Option<u32> {
    match &result.raw {
        Err(InstructionError::Custom(c)) => Some(*c),
        _ => None,
    }
}

pub fn code_of(name: &str) -> u32 {
    match name {
        "ZeroAmount" => 1,
        "ExceedsSupply" => 2,
        "Dust" => 3,
        "ZeroPayout" => 4,
        "Slippage" => 5,
        "NoHolders" => 6,
        "NothingToClaim" => 12,
        other => panic!("errore inatteso nei vettori: {other}"),
    }
}
