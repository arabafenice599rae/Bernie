//! Bernie v1.6: nucleo puro del programma Solana (§12, `state.rs` ed `error.rs`).
//!
//! Nessuna dipendenza e nessun accesso agli account: solo aritmetica esatta su
//! `k`, `R`, `Q`, `S`. Il processor Pinocchio la chiama e poi muove lamport e token.
#![no_std]

pub mod error;
pub mod state;

pub use error::BernieError;
pub use state::*;
