//! Bernie v1.6: programma Solana (§11, §12).
//!
//! Il byte 0 dei dati è il tag dell'istruzione; interi little-endian; stringhe come
//! lunghezza u8 seguita dai byte UTF-8. L'aritmetica è tutta in `bernie-state`: qui
//! si validano gli account, si muovono lamport e token e si scrive il vault.
#![no_std]

pub mod processor;
pub mod token;
pub mod vault;

use bernie_state::BernieError;
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

pinocchio::address::declare_id!("GJLMDDuCe7LxSbvRnHukAonSwmQDRVd2wofRjrzUuZpP");

/// Tesorerie del protocollo (§11): lista costante di N indirizzi; il client ne sceglie
/// una a caso per operazione. SEGNAPOSTO deterministici, da sostituire prima del deploy
/// con indirizzi reali, sempre rent-exempt (§17, checklist).
pub const TREASURIES: [Address; 4] = [
    Address::from_str_const("G7EBp22JknV1CypYgiXjL37itb5DUgPUaf4q56E8ZxgZ"),
    Address::from_str_const("13hVFWngnyVLfXfWt9BWrJkQTmkDrk3EVRuvzN4SKPio"),
    Address::from_str_const("2pnMTJEN1qtSgTQRmxrK3sLD2aJQSFqnxSscynre1rTc"),
    Address::from_str_const("GaPJwGBApWFfYq4ciFAMYqZERrcjEaqxiFLi6EZFRfNt"),
];

pub const SYSTEM_PROGRAM_ID: Address = pinocchio_system::ID;
pub const TOKEN_2022_ID: Address = pinocchio_token_2022::ID;

/// Tag delle istruzioni (§11).
pub mod tag {
    pub const CREATE: u8 = 0;
    pub const MINT: u8 = 1;
    pub const REDEEM: u8 = 2;
    pub const DONATE: u8 = 3;
    pub const SWEEP: u8 = 4;
}

#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint {
    use super::*;

    pinocchio::program_entrypoint!(process_instruction);
    pinocchio::no_allocator!();
    pinocchio::nostd_panic_handler!();
}

pub fn process_instruction(
    program_id: &Address,
    accounts: &mut [AccountView],
    data: &[u8],
) -> ProgramResult {
    let (tag, rest) = data
        .split_first()
        .ok_or(ProgramError::InvalidInstructionData)?;
    match *tag {
        tag::CREATE => processor::create::process(program_id, accounts, rest),
        tag::MINT => processor::mint::process(program_id, accounts, rest),
        tag::REDEEM => processor::redeem::process(program_id, accounts, rest),
        tag::DONATE => processor::donate::process(program_id, accounts, rest),
        tag::SWEEP => processor::sweep::process(program_id, accounts, rest),
        _ => Err(ProgramError::InvalidInstructionData),
    }
}

/// Codici di §7 come errori custom del programma.
pub fn err(e: BernieError) -> ProgramError {
    ProgramError::Custom(e.code())
}

/// Lettore dei dati d'istruzione: interi LE e stringhe con lunghezza u8.
pub struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self(data)
    }

    fn take(&mut self, n: usize) -> Result<&'a [u8], ProgramError> {
        if self.0.len() < n {
            return Err(ProgramError::InvalidInstructionData);
        }
        let (head, tail) = self.0.split_at(n);
        self.0 = tail;
        Ok(head)
    }

    pub fn u16(&mut self) -> Result<u16, ProgramError> {
        Ok(u16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }

    pub fn u64(&mut self) -> Result<u64, ProgramError> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }

    pub fn string(&mut self) -> Result<&'a [u8], ProgramError> {
        let len = self.take(1)?[0] as usize;
        self.take(len)
    }

    /// I dati devono essere consumati per intero.
    pub fn finish(self) -> ProgramResult {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(ProgramError::InvalidInstructionData)
        }
    }
}
