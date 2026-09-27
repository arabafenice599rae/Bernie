//! Account vault v4 (§3): PDA `[b"vault", mint]`, 128 byte, little-endian.

use crate::err;
use bernie_state::{BernieError, Vault, SOLANA_SCALE};
use pinocchio::{
    error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    AccountView, Address, ProgramResult,
};

pub type State = Vault<SOLANA_SCALE>;

pub const LEN: usize = 128;
pub const DISCRIMINATOR: u8 = 0x52;
pub const VERSION: u8 = 4;
pub const SEED: &[u8] = b"vault";

const O_DISC: usize = 0;
const O_VERSION: usize = 1;
const O_BUMP: usize = 2;
const O_PENALTY: usize = 4;
const O_ENTRY: usize = 6;
const O_CREATOR: usize = 8;
const O_MINT: usize = 40;
const O_K: usize = 72;
const O_RESERVE: usize = 88;
const O_RESIDUAL: usize = 104;
const O_SUPPLY: usize = 120;

/// Campi immutabili più lo stato mutabile.
pub struct Header {
    pub bump: u8,
    pub creator: Address,
    pub mint: Address,
}

fn u128_at(d: &[u8], o: usize) -> u128 {
    u128::from_le_bytes(d[o..o + 16].try_into().unwrap())
}

/// Legge e valida il vault: owner, dimensione, discriminatore, versione, mint.
pub fn load(
    program_id: &Address,
    vault: &AccountView,
    mint: &Address,
) -> Result<(Header, State), ProgramError> {
    if !vault.owned_by(program_id) {
        return Err(ProgramError::IncorrectProgramId);
    }
    let d = vault.try_borrow()?;
    if d.len() != LEN || d[O_DISC] != DISCRIMINATOR || d[O_VERSION] != VERSION {
        return Err(ProgramError::InvalidAccountData);
    }
    let header = Header {
        bump: d[O_BUMP],
        creator: Address::new_from_array(d[O_CREATOR..O_CREATOR + 32].try_into().unwrap()),
        mint: Address::new_from_array(d[O_MINT..O_MINT + 32].try_into().unwrap()),
    };
    if &header.mint != mint {
        return Err(ProgramError::InvalidAccountData);
    }
    let state = State {
        penalty_bps: u16::from_le_bytes([d[O_PENALTY], d[O_PENALTY + 1]]),
        entry_bps: u16::from_le_bytes([d[O_ENTRY], d[O_ENTRY + 1]]),
        k: u128_at(&d, O_K),
        reserve: u128_at(&d, O_RESERVE),
        residual: u128_at(&d, O_RESIDUAL),
        supply: u64::from_le_bytes(d[O_SUPPLY..O_SUPPLY + 8].try_into().unwrap()),
    };
    Ok((header, state))
}

/// Scrive un vault nuovo (create).
pub fn init(
    vault: &mut AccountView,
    bump: u8,
    creator: &Address,
    mint: &Address,
    s: &State,
) -> ProgramResult {
    let mut d = vault.try_borrow_mut()?;
    if d.len() != LEN {
        return Err(ProgramError::InvalidAccountData);
    }
    d.fill(0);
    d[O_DISC] = DISCRIMINATOR;
    d[O_VERSION] = VERSION;
    d[O_BUMP] = bump;
    d[O_PENALTY..O_PENALTY + 2].copy_from_slice(&s.penalty_bps.to_le_bytes());
    d[O_ENTRY..O_ENTRY + 2].copy_from_slice(&s.entry_bps.to_le_bytes());
    d[O_CREATOR..O_CREATOR + 32].copy_from_slice(creator.as_ref());
    d[O_MINT..O_MINT + 32].copy_from_slice(mint.as_ref());
    drop(d);
    store(vault, s)
}

/// Scrive i quattro valori mutabili: k, R, Q, S.
pub fn store(vault: &mut AccountView, s: &State) -> ProgramResult {
    let mut d = vault.try_borrow_mut()?;
    d[O_K..O_K + 16].copy_from_slice(&s.k.to_le_bytes());
    d[O_RESERVE..O_RESERVE + 16].copy_from_slice(&s.reserve.to_le_bytes());
    d[O_RESIDUAL..O_RESIDUAL + 16].copy_from_slice(&s.residual.to_le_bytes());
    d[O_SUPPLY..O_SUPPLY + 8].copy_from_slice(&s.supply.to_le_bytes());
    Ok(())
}

/// Lamport del vault oltre il minimo rent-exempt: la base di I2, excess e sweep.
pub fn available(vault: &AccountView) -> Result<u64, ProgramError> {
    let rent = Rent::get()?.try_minimum_balance(LEN)?;
    vault
        .lamports()
        .checked_sub(rent)
        .ok_or_else(|| err(BernieError::InvariantViolated))
}

/// I2 con i lamport reali, dopo i movimenti dell'istruzione.
pub fn check_solvency(vault: &AccountView, s: &State) -> ProgramResult {
    s.check_solvency(available(vault)?).map_err(err)
}

/// Log per gli indexer (§11): `State k S R Q`.
pub fn log_state(s: &State) {
    pinocchio_log::log!("State {} {} {} {}", s.k, s.supply, s.reserve, s.residual);
}
