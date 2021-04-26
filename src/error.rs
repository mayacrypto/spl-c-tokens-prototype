use thiserror::Error;

use solana_program::program_error::ProgramError;

#[derive(Error, Debug, Copy, Clone)]
pub enum CTokenError {
    /// Invalid instruction
    #[error("Invalid Instruction")]
    InvalidInstruction,
}

impl From<CTokenError> for ProgramError {
    fn from(e: CTokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
