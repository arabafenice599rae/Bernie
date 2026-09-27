//! Tag 0, create(P, p, e, nome, simbolo, URI) (§6, sequenza di §12).
//!
//! Account: creator (s,w), mint (s,w, keypair nuova), vault PDA (w), System, Token-2022.

use super::{signer_writable, system_program, writable};
use crate::{err, token, vault, Reader, TOKEN_2022_ID};
use bernie_state::{validate_metadata, validate_params, Vault};
use pinocchio::{
    cpi::{Seed, Signer},
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};
use pinocchio_system::instructions::CreateAccount;
use pinocchio_token::instructions::initialize_mint2::InitializeMint2;
use pinocchio_token_2022::{instructions::metadata_pointer, Token2022Program};

pub fn process(program_id: &Address, accounts: &mut [AccountView], data: &[u8]) -> ProgramResult {
    let [creator, mint, vault_acc, system, token_program] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };
    let mut r = Reader::new(data);
    let price = r.u64()?;
    let penalty_bps = r.u16()?;
    let entry_bps = r.u16()?;
    let name = r.string()?;
    let symbol = r.string()?;
    let uri = r.string()?;
    r.finish()?;

    signer_writable(creator)?;
    signer_writable(mint)?;
    writable(vault_acc)?;
    system_program(system)?;
    token::check_program(token_program)?;
    validate_params(price, penalty_bps, entry_bps).map_err(err)?;
    validate_metadata(name, symbol, uri).map_err(err)?;
    let state = Vault::create(price, penalty_bps, entry_bps).map_err(err)?;

    let (expected, bump) =
        Address::find_program_address(&[vault::SEED, mint.address().as_ref()], program_id);
    if vault_acc.address() != &expected {
        return Err(ProgramError::InvalidSeeds);
    }
    let bump_seed = [bump];
    let seeds = [
        Seed::from(vault::SEED),
        Seed::from(mint.address().as_ref()),
        Seed::from(&bump_seed),
    ];

    // 1. Mint con spazio per MetadataPointer e rent per la dimensione finale:
    //    TokenMetadata Initialize estende l'account da sé.
    let rent = Rent::get()?;
    let final_len =
        token::MINT_WITH_POINTER_LEN + token::metadata_tlv_len(name.len(), symbol.len(), uri.len());
    CreateAccount {
        from: creator,
        to: mint,
        lamports: rent.try_minimum_balance(final_len)?,
        space: token::MINT_WITH_POINTER_LEN as u64,
        owner: &TOKEN_2022_ID,
    }
    .invoke()?;

    // 2. MetadataPointer verso il mint stesso, authority nulla.
    metadata_pointer::Initialize {
        mint,
        authority: None,
        metadata_address: Some(mint.address()),
        token_program: &TOKEN_2022_ID,
    }
    .invoke()?;

    // 3. InitializeMint2: 9 decimali, authority = vault PDA, freeze nulla.
    InitializeMint2::<Token2022Program>::new(mint, token::DECIMALS, vault_acc.address(), None)
        .invoke()?;

    // 4–5. Metadati firmati dal PDA, poi update authority → nulla.
    token::initialize_metadata(
        mint,
        vault_acc,
        token_program,
        name,
        symbol,
        uri,
        Signer::from(&seeds),
    )?;
    token::revoke_metadata_authority(mint, vault_acc, token_program, Signer::from(&seeds))?;

    // 6. Vault PDA.
    CreateAccount {
        from: creator,
        to: vault_acc,
        lamports: rent.try_minimum_balance(vault::LEN)?,
        space: vault::LEN as u64,
        owner: program_id,
    }
    .invoke_signed(&[Signer::from(&seeds)])?;

    vault::init(vault_acc, bump, creator.address(), mint.address(), &state)?;
    vault::log_state(&state);
    Ok(())
}
