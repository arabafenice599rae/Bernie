//! Un modulo per istruzione (§12). Ogni processor valida gli account, calcola il nuovo
//! stato con `bernie-state` (che fallisce senza effetti), esegue CPI e movimenti di
//! lamport, poi riscrive il vault e riverifica supply e solvenza.

pub mod create;
pub mod donate;
pub mod mint;
pub mod redeem;
pub mod sweep;

use crate::{SYSTEM_PROGRAM_ID, TREASURIES};
use pinocchio::{error::ProgramError, AccountView, ProgramResult};

pub fn signer_writable(acc: &AccountView) -> ProgramResult {
    if !acc.is_signer() {
        return Err(ProgramError::MissingRequiredSignature);
    }
    writable(acc)
}

pub fn writable(acc: &AccountView) -> ProgramResult {
    if !acc.is_writable() {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(())
}

pub fn system_program(acc: &AccountView) -> ProgramResult {
    if acc.address() != &SYSTEM_PROGRAM_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

pub fn treasury(acc: &AccountView) -> ProgramResult {
    if !TREASURIES.contains(acc.address()) {
        return Err(ProgramError::InvalidArgument);
    }
    writable(acc)
}

/// Sposta lamport da un account del programma (il vault) a un altro account.
pub fn move_lamports(from: &AccountView, to: &AccountView, amount: u64) -> ProgramResult {
    if amount == 0 {
        return Ok(());
    }
    let mut from = *from;
    let mut to = *to;
    let f = from
        .lamports()
        .checked_sub(amount)
        .ok_or(ProgramError::InsufficientFunds)?;
    let t = to
        .lamports()
        .checked_add(amount)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    from.set_lamports(f);
    to.set_lamports(t);
    Ok(())
}
