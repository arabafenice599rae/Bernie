//! Tag 3, donate(a) (§6). Account: donatore (s,w), mint, vault (w), System.

use super::{signer_writable, system_program, writable};
use crate::{err, token, vault, Reader};
use pinocchio::{error::ProgramError, AccountView, Address, ProgramResult};
use pinocchio_system::instructions::Transfer;

pub fn process(program_id: &Address, accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [donor, mint, vault_acc, system] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let mut r = Reader::new(data);
    let a = r.u64()?;
    r.finish()?;

    signer_writable(donor)?;
    writable(vault_acc)?;
    system_program(system)?;

    let (_, mut state) = vault::load(program_id, vault_acc, mint.address())?;
    token::check_supply(mint, state.supply)?;
    state.donate(a).map_err(err)?;

    Transfer {
        from: donor,
        to: vault_acc,
        lamports: a,
    }
    .invoke()?;

    vault::check_solvency(vault_acc, &state)?;
    vault::store(vault_acc, &state)?;
    vault::log_state(&state);
    Ok(())
}
