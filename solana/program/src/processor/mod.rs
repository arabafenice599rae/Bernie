//! Un modulo per istruzione (§12). Ogni processor valida gli account, calcola il nuovo
//! stato con `bernie-state` (che fallisce senza effetti), esegue CPI e movimenti di
//! lamport, poi riscrive il vault e riverifica supply e solvenza.

pub mod create;
pub mod donate;
pub mod mint;
pub mod redeem;
pub mod sweep;

use crate::{SYSTEM_PROGRAM_ID, TREASURIES};
use pinocchio::{cpi::Signer, error::ProgramError, AccountView, Address, ProgramResult};
use pinocchio_system::instructions::{Allocate, Assign, CreateAccount, Transfer};

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

/// Crea `target` con `space` byte, owner `owner` e almeno `lamports`, anche se qualcuno
/// vi ha già inviato lamport: `CreateAccount` fallirebbe e chiunque veda la transazione
/// potrebbe bloccare `create` (griefing). In quel caso si versa solo la differenza,
/// poi `Allocate` e `Assign`. `signers` firma per `target` quando è un PDA.
pub fn create_account(
    payer: &AccountView,
    target: &AccountView,
    lamports: u64,
    space: u64,
    owner: &Address,
    signers: &[Signer],
) -> ProgramResult {
    let current = target.lamports();
    if current == 0 {
        return CreateAccount {
            from: payer,
            to: target,
            lamports,
            space,
            owner,
        }
        .invoke_signed(signers);
    }
    let missing = lamports.saturating_sub(current);
    if missing > 0 {
        Transfer {
            from: payer,
            to: target,
            lamports: missing,
        }
        .invoke()?;
    }
    Allocate {
        account: target,
        space,
    }
    .invoke_signed(signers)?;
    Assign {
        account: target,
        owner,
    }
    .invoke_signed(signers)
}
