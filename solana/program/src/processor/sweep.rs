//! Tag 4, sweep (§6, v1.6): chiunque può inviare l'excess al creator registrato.
//! Non modifica k, R, Q, S e non chiude il vault.
//!
//! Account: creator (w, deve coincidere con `vault.creator`), mint, vault (w).

use super::{move_lamports, writable};
use crate::{err, token, vault, Reader};
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};

pub fn process(program_id: &Address, accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [creator, mint, vault_acc] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    Reader::new(data).finish()?;

    writable(creator)?;
    writable(vault_acc)?;
    let (header, state) = vault::load(program_id, vault_acc, mint.address())?;
    if creator.address() != &header.creator {
        return Err(ProgramError::InvalidArgument);
    }
    token::check_supply(mint, state.supply)?;

    let amount = state
        .sweep_amount(vault::available(vault_acc)?)
        .map_err(err)?;
    move_lamports(vault_acc, creator, amount)?;

    vault::check_solvency(vault_acc, &state)?;
    vault::log_state(&state);
    Ok(())
}
