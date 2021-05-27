use thiserror::Error;
// use bulletproofs::ProofError;

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
    /// Commitments in instructions and source account does not match.
    #[error("Commitment mismatch")]
    CommitmentMismatch,
    /// Commitment opening is invalid
    #[error("Opening invalid")]
    OpeningInvalid,
    /// Account belongs to a different mint
    #[error("Mint mismatch")]
    MintMismatch,
    /// Mint owner does not match
    #[error("OwnerMismatch")]
    OwnerMismatch,
}

impl From<CTokenError> for ProgramError {
    fn from(e: CTokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

// impl From<ProofError> for CTokenError {
//     fn from(_: ProofError) -> Self {
//         CTokenError::InvalidProof
//     }
// }
