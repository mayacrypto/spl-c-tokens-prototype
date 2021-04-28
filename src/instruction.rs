use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};
use std::mem::size_of;

use crate::{
    error::CTokenError::InvalidInstruction,
    state::{
        MintTX,
        TransferTX,
    }
};

pub enum CTokenInstruction {

    /// Initializes a new mint.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The mint to initialize.
    ///   1. `[]` Rent sysvar
    ///
    InitializeMint {
        mint_authority: Pubkey,
    },

    MintTo {
        tx: MintTX,
    },

    InitializeAccount,

    CloseAccount,

    Transfer {
        tx: TransferTX,
    },


}

impl CTokenInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (mint_authority, _) = Self::unpack_pubkey(rest)?;
                Self::InitializeMint {
                    mint_authority,
                }
            },
            _ => return Err(InvalidInstruction.into()),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::InitializeMint {
                ref mint_authority,
            } => {
                buf.push(0);
                buf.extend_from_slice(mint_authority.as_ref());
            },
            _ => {}
        };
        buf
    }

    pub fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        if input.len() >= 32 {
            let (key, rest) = input.split_at(32);
            let pk = Pubkey::new(key);
            Ok((pk, rest))
        } else {
            Err(InvalidInstruction.into())
        }
    }
}

pub fn initialize_mint(
    c_token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
) -> Result<Instruction, ProgramError> {
    let data = CTokenInstruction::InitializeMint {
        mint_authority: *mint_authority_pubkey,
    }
    .pack();

    let accounts = vec![
        AccountMeta::new(*mint_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction {
        program_id: *c_token_program_id,
        accounts,
        data,
    })
}
