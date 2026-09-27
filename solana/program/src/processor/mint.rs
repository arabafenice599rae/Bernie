//! Tag 1, mint(u, max_cost) (§6).
//!
//! Account: utente (s,w), ATA Token-2022 (w), mint (w), vault (w), tesoreria (w),
//! System, Token-2022. v1.6: il creator non compare; la sua fee resta nel vault.

use super::{signer_writable, system_program, treasury, writable};
use crate::{err, token, vault, Reader};
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::Transfer;
use pinocchio_token::instructions::mint_to::MintTo;
use pinocchio_token_2022::Token2022Program;

pub fn process(program_id: &Address, accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [user, ata, mint, vault_acc, treasury_acc, system, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let mut r = Reader::new(data);
    let u = r.u64()?;
    let max_cost = r.u64()?;
    r.finish()?;

    signer_writable(user)?;
    writable(ata)?;
    writable(mint)?;
    writable(vault_acc)?;
    treasury(treasury_acc)?;
    system_program(system)?;
    token::check_program(token_program)?;

    let (header, mut state) = vault::load(program_id, vault_acc, mint.address())?;
    token::check_supply(mint, state.supply)?;
    let quote = state.mint(u, max_cost).map_err(err)?;

    // utente → vault: c + fc; utente → tesoreria: fp.
    let to_vault = quote
        .cost
        .checked_add(quote.fees.creator)
        .ok_or(ProgramError::ArithmeticOverflow)?;
    Transfer {
        from: user,
        to: vault_acc,
        lamports: to_vault,
    }
    .invoke()?;
    if quote.fees.protocol > 0 {
        Transfer {
            from: user,
            to: treasury_acc,
            lamports: quote.fees.protocol,
        }
        .invoke()?;
    }

    let bump = [header.bump];
    let seeds = [
        Seed::from(vault::SEED),
        Seed::from(mint.address().as_ref()),
        Seed::from(&bump),
    ];
    MintTo::<'_, '_, &AccountView, Token2022Program>::new(mint, ata, vault_acc, u)
        .invoke_signed(&[Signer::from(&seeds)])?;

    token::check_supply(mint, state.supply)?;
    vault::check_solvency(vault_acc, &state)?;
    vault::store(vault_acc, &state)?;
    vault::log_state(&state);
    Ok(())
}
