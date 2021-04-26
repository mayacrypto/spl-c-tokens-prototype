use solana_program::program_error::ProgramError;

use crate::{
    error::CTokenError::InvalidInstruction,
    state::{
        TXState,
        Transaction,
    }
};

pub enum ConfTXInstruction {

    /// Generates the initial state of the transactions
    // TODO: List associated accounts
    Initialize {
        st: TXState,
    },

    /// Aggregates a transaction to the state of the transactions
    // TODO: List associated accounts
    Transact {
        tx: Transaction,
    },
}

impl ConfTXInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => Self::Initialize {
                st: Self::unpack_state(rest)?,
            },
            1 => Self::Transact {
                tx: Self::unpack_transaction(rest)?,
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    pub fn unpack_state(_input: &[u8]) -> Result<TXState, ProgramError> {
        // TODO: Update
        Ok(TXState)
    }

    pub fn unpack_transaction(_input: &[u8]) -> Result<Transaction, ProgramError> {
        // TODO: Update
        Ok(Transaction)
    }
}


