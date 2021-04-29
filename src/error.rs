use thiserror::Error;

use solana_program::program_error::ProgramError;

/// Errors that may be returned by the CToken program.
#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub enum CTokenError {
    /// The account cannot be initialized because it is already being used.
    #[error("Already in use")]
    AlreadyInUse,
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,
    /// Lamport balance below rent-exempt threshold. (TODO: Update)
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,
    /// Operation overflowed
    #[error("Operation overflowed")]
    Overflow,
    /// Cryptographic proof is invalid
    #[error("Invalid proof")]
    InvalidProof,
}

impl From<CTokenError> for ProgramError {
    fn from(e: CTokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
