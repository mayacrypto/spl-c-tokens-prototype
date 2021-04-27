use solana_program::{
    program_error::ProgramError,
};

use crate::{
    error::CTokenError::InvalidInstruction,
    state::{
        MintTX,
        TransferTX,
    }
};

pub enum ConfTXInstruction {

    /// Generates the initial state of the transactions
    // Associated accounts:
    // 0. `[signer]` The initializer of the tokens
    // 1. `[writable]` The world state
    // 2. `[]` The rent sysvar
    // 3. `[]` The token program
    Initialize,

    /// Aggregates a mint transaction to the world state
    // TODO: List associated accounts
    Mint {
        tx: MintTX,
    },

    /// Aggregates a transfer transaction to the world state
    // TODO: List associated accounts
    Transfer {
        tx: TransferTX,
    },
}

impl ConfTXInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::Initialize,
            1 => Self::Mint {
                tx: Self::unpack_mint(rest)?,
            },
            2 => Self::Transfer {
                tx: Self::unpack_transfer(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    pub fn unpack_mint(_input: &[u8]) -> Result<MintTX, ProgramError> {
        // TODO: Update
        Ok(Default::default())
    }

    pub fn unpack_transfer(_input: &[u8]) -> Result<TransferTX, ProgramError> {
        // TODO: Update
        Ok(Default::default())
    }
}


