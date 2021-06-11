use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};
use std::mem::size_of;

use crate::{
    error::CTokenError::InvalidInstruction,
    txdata::MintData,
};

pub enum CTokenInstruction {
    /// Initializes a new mint.
    ///
    /// This is analogous to the `InitializeMint` instruction in the SPL token program with
    /// decimals and freeze_authority removed for prototyping purposes. As in the regular
    /// SPL program, the instruction requires no signers and must be included within the
    /// same transaction as the system program's `CreateAccount` instruction.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The mint to initialize.
    ///   1. `[]` Rent sysvar
    ///
    InitializeMint { mint_authority: Pubkey },
    /// Mints new tokens.
    ///
    /// This is analogous to the combination of the `InitializeAccount` and `MintTo` instructions
    /// in the SPL token program.
    ///
    /// Account expected by this instruction:
    ///
    ///   0. `[writable]` The mint.
    ///   1. `[writable]` The account to mint tokens to.
    ///   2. `[signer]` The mint's minting authority.
    ///   3. `[]` Rent sysvar
    ///
    Mint {
        /// Data for the new tokens to mint.
        mint_data: MintData,
    },
}

impl CTokenInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (mint_authority, _) = Self::unpack_pubkey(rest)?;
                Self::InitializeMint { mint_authority }
            }
            1 => {
                let mint_data = MintData::try_from_slice(rest)?;
                Self::Mint { mint_data }
            }
            _ => return Err(InvalidInstruction.into()),
        })
    }

    pub fn pack(&self) -> Vec<u8> {
        let mut buf = Vec::with_capacity(size_of::<Self>());
        match self {
            &Self::InitializeMint { ref mint_authority } => {
                buf.push(0);
                buf.extend_from_slice(mint_authority.as_ref());
            }
            &Self::Mint { ref mint_data } => {
                buf.push(1);
                buf.extend_from_slice(mint_data.try_to_vec().unwrap().as_ref());
            }
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

/// Creates a `InitializeMint` instruction.
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

/// Creates a `Mint` instruction.
pub fn mint(
    c_token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    account_pubkey: &Pubkey,
    signer_pubkey: &Pubkey,
    mint_data: MintData,
) -> Result<Instruction, ProgramError> {
    let data = CTokenInstruction::Mint { mint_data }.pack();

    let accounts = vec![
        AccountMeta::new(*mint_pubkey, false),
        AccountMeta::new(*account_pubkey, false),
        AccountMeta::new_readonly(*signer_pubkey, true),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];
    Ok(Instruction {
        program_id: *c_token_program_id,
        accounts,
        data,
    })
}
