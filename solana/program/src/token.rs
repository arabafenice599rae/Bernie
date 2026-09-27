//! Token-2022: lettura di mint e token account, CPI dei metadati scritte a mano (§12).

use crate::{err, TOKEN_2022_ID};
use bernie_state::{BernieError, NAME_MAX, SYMBOL_MAX, URI_MAX};
use pinocchio::{
    cpi::{invoke_signed, Signer},
    error::ProgramError,
    instruction::{InstructionAccount, InstructionView},
    AccountView, Address, ProgramResult,
};

pub const DECIMALS: u8 = 9;

/// Mint base 82 byte, padding fino a 165, AccountType, TLV di MetadataPointer (2 + 2 + 64).
pub const MINT_WITH_POINTER_LEN: usize = 165 + 1 + 4 + 64;

/// Voce TLV di TokenMetadata (§11): intestazione, update authority, mint, tre stringhe
/// con lunghezza u32, `additional_metadata` vuoto (u32 = 0).
pub fn metadata_tlv_len(name: usize, symbol: usize, uri: usize) -> usize {
    4 + 32 + 32 + (4 + name) + (4 + symbol) + (4 + uri) + 4
}

const MD_INITIALIZE: [u8; 8] = [210, 225, 30, 162, 88, 184, 77, 141];
const MD_UPDATE_AUTHORITY: [u8; 8] = [215, 228, 166, 228, 84, 100, 86, 123];

pub fn check_program(token_program: &AccountView) -> ProgramResult {
    if token_program.address() != &TOKEN_2022_ID {
        return Err(ProgramError::IncorrectProgramId);
    }
    Ok(())
}

/// Supply di un mint Token-2022 inizializzato.
pub fn mint_supply(mint: &AccountView) -> Result<u64, ProgramError> {
    if !mint.owned_by(&TOKEN_2022_ID) {
        return Err(ProgramError::IncorrectProgramId);
    }
    let d = mint.try_borrow()?;
    if d.len() < 82 || d[45] != 1 {
        return Err(ProgramError::InvalidAccountData);
    }
    Ok(u64::from_le_bytes(d[36..44].try_into().unwrap()))
}

/// §3 e §4: `vault.supply` deve coincidere con la supply del mint.
pub fn check_supply(mint: &AccountView, supply: u64) -> ProgramResult {
    if mint_supply(mint)? != supply {
        return Err(err(BernieError::SupplyMismatch));
    }
    Ok(())
}

/// Campi del token account usati dal redeem.
pub struct TokenAccount {
    pub mint: Address,
    pub owner: Address,
    pub delegate: Option<Address>,
    pub delegated_amount: u64,
}

pub fn token_account(acc: &AccountView) -> Result<TokenAccount, ProgramError> {
    if !acc.owned_by(&TOKEN_2022_ID) {
        return Err(ProgramError::IncorrectProgramId);
    }
    let d = acc.try_borrow()?;
    // Layout base: mint 0, owner 32, amount 64, delegate COption 72 (tag u32 + 32),
    // state 108, is_native 109, delegated_amount 121, close_authority 129.
    if d.len() < 165 || d[108] == 0 {
        return Err(ProgramError::InvalidAccountData);
    }
    let addr = |o: usize| Address::new_from_array(d[o..o + 32].try_into().unwrap());
    let delegate = match u32::from_le_bytes(d[72..76].try_into().unwrap()) {
        0 => None,
        _ => Some(addr(76)),
    };
    Ok(TokenAccount {
        mint: addr(0),
        owner: addr(32),
        delegate,
        delegated_amount: u64::from_le_bytes(d[121..129].try_into().unwrap()),
    })
}

/// TokenMetadata Initialize: metadati nel mint, update authority = vault PDA,
/// firmata dal PDA come mint authority.
pub fn initialize_metadata(
    mint: &AccountView,
    vault: &AccountView,
    token_program: &AccountView,
    name: &[u8],
    symbol: &[u8],
    uri: &[u8],
    signer: Signer,
) -> ProgramResult {
    const MAX: usize = 8 + 3 * 4 + NAME_MAX + SYMBOL_MAX + URI_MAX;
    let mut data = [0u8; MAX];
    data[..8].copy_from_slice(&MD_INITIALIZE);
    let mut at = 8;
    for s in [name, symbol, uri] {
        data[at..at + 4].copy_from_slice(&(s.len() as u32).to_le_bytes());
        at += 4;
        data[at..at + s.len()].copy_from_slice(s);
        at += s.len();
    }
    let ix = InstructionView {
        program_id: token_program.address(),
        accounts: &[
            InstructionAccount::writable(mint.address()),
            InstructionAccount::readonly(vault.address()),
            InstructionAccount::readonly(mint.address()),
            InstructionAccount::readonly_signer(vault.address()),
        ],
        data: &data[..at],
    };
    invoke_signed(&ix, &[mint, vault, mint, vault], &[signer])
}

/// TokenMetadata UpdateAuthority → nessuna: metadati immutabili (§1).
pub fn revoke_metadata_authority(
    mint: &AccountView,
    vault: &AccountView,
    token_program: &AccountView,
    signer: Signer,
) -> ProgramResult {
    let mut data = [0u8; 8 + 32];
    data[..8].copy_from_slice(&MD_UPDATE_AUTHORITY);
    let ix = InstructionView {
        program_id: token_program.address(),
        accounts: &[
            InstructionAccount::writable(mint.address()),
            InstructionAccount::readonly_signer(vault.address()),
        ],
        data: &data,
    };
    invoke_signed(&ix, &[mint, vault], &[signer])
}
