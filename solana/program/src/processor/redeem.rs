//! Tag 2, redeem(u, min_out) (§6).
//!
//! Account: utente (s,w), ATA (w), mint (w), vault (w), tesoreria (w), Token-2022.
//! Il client premette `ApproveChecked` verso il vault PDA; il programma brucia come
//! delegato, quindi funziona con o senza CPI Guard (§7).

use super::{move_lamports, signer_writable, treasury, writable};
use crate::{err, token, vault, Reader};
use bernie_state::BernieError;
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    AccountView, Address, ProgramResult,
};
use pinocchio_token::instructions::burn_checked::BurnChecked;
use pinocchio_token_2022::Token2022Program;

pub fn process(program_id: &Address, accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [user, ata, mint, vault_acc, treasury_acc, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let mut r = Reader::new(data);
    let u = r.u64()?;
    let min_out = r.u64()?;
    r.finish()?;

    signer_writable(user)?;
    writable(ata)?;
    writable(mint)?;
    writable(vault_acc)?;
    treasury(treasury_acc)?;
    token::check_program(token_program)?;

    let (header, mut state) = vault::load(program_id, vault_acc, mint.address())?;
    token::check_supply(mint, state.supply)?;

    // Il token account deve essere dell'utente firmatario: la delega al vault non
    // basta, altrimenti chiunque potrebbe riscattare dai conti di altri che l'hanno concessa.
    let acct = token::token_account(ata)?;
    if &acct.mint != mint.address() {
        return Err(ProgramError::InvalidAccountData);
    }
    if &acct.owner != user.address() {
        return Err(err(BernieError::NotOwner));
    }
    if acct.delegate.as_ref() != Some(vault_acc.address()) || acct.delegated_amount < u {
        return Err(err(BernieError::MissingDelegation));
    }

    let quote = state.redeem(u, min_out).map_err(err)?;

    let bump = [header.bump];
    let seeds = [
        Seed::from(vault::SEED),
        Seed::from(mint.address().as_ref()),
        Seed::from(&bump),
    ];
    BurnChecked::<'_, '_, &AccountView, Token2022Program>::new(
        ata,
        mint,
        vault_acc,
        u,
        token::DECIMALS,
    )
    .invoke_signed(&[Signer::from(&seeds)])?;

    // Dopo le CPI: vault → utente out, vault → tesoreria fp; fc resta nel vault.
    move_lamports(vault_acc, user, quote.out)?;
    move_lamports(vault_acc, treasury_acc, quote.fees.protocol)?;

    token::check_supply(mint, state.supply)?;
    vault::check_solvency(vault_acc, &state)?;
    vault::store(vault_acc, &state)?;
    vault::log_state(&state);
    Ok(())
}
