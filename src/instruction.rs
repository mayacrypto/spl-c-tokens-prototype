use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};
use borsh::{BorshSerialize, BorshDeserialize};
use std::mem::size_of;

use crate::{
    error::CTokenError::InvalidInstruction,
    txdata::{MintData, TransferData, CloseAccountData},
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
    InitializeMint {
        mint_authority: Pubkey,
    },
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

    /// Transfers tokens.
    ///
    /// This is analogous to the `Transfer` instruction in the SPL token program. There is no
    /// signature check required for any accounts. The validity of the transaction is checked
    /// internally by the c-token program.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[]` The mint
    ///   1. `[writable]` The first source account.
    ///   2. `[writable]` The second source account
    ///   3. `[writable]` The first destination account.
    ///   4. `[writable]` The second destination account.
    ///   5. `[]` Rent sysvar
    ///
    Transfer {
        /// Data for the transfer
        transfer_data: TransferData,
    },

    /// Close an account by transferring all its ZOL to the destination in SOL.
    ///
    /// This instruction is for prototyping purposes only and may be more natural as part of a
    /// separate exchange program. For the prototype, 1 SOL equals 1 ZOL in value.
    ///
    /// There is no signature check required for any accounts. The validity of the transaction is
    /// checked internally by the c-token program.
    ///
    /// Accounts expected by this instruction:
    ///
    ///   0. `[writable]` The source account.
    ///   1. `[writable]` The destination account.
    ///
    CloseAccount {
        /// Data for close account
        close_account_data: CloseAccountData,
    },
}

impl CTokenInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (mint_authority, _) = Self::unpack_pubkey(rest)?;
                Self::InitializeMint { mint_authority }
            },
            1 => {
                let mint_data = MintData::try_from_slice(rest)?;
                Self::Mint { mint_data }
            },
            2 => {
                let transfer_data = TransferData::try_from_slice(rest)?;
                Self::Transfer { transfer_data }
            },
            3 => {
                let close_account_data = CloseAccountData::try_from_slice(rest)?;
                Self::CloseAccount { close_account_data }
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
            &Self::Mint {
                ref mint_data,
            } => {
                buf.push(1);
                buf.extend_from_slice(mint_data.try_to_vec().unwrap().as_ref());
            },
            &Self::Transfer {
                ref transfer_data,
            } => {
                buf.push(2);
                buf.extend_from_slice(transfer_data.try_to_vec().unwrap().as_ref());
            },
            &Self::CloseAccount {
                ref close_account_data,
            } => {
                buf.push(3);
                buf.extend_from_slice(close_account_data.try_to_vec().unwrap().as_ref());
            },
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

/// Creates a `Transfer` instruction.
pub fn transfer(
    c_token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    sender_source_pubkey: &Pubkey,
    receiver_source_pubkey: &Pubkey,
    sender_dest_pubkey: &Pubkey,
    receiver_dest_pubkey: &Pubkey,
    transfer_data: TransferData,
) -> Result<Instruction, ProgramError> {
    let data = CTokenInstruction::Transfer { transfer_data }.pack();

    let accounts = vec![
        AccountMeta::new(*sender_source_pubkey, false),
        AccountMeta::new(*receiver_source_pubkey, false),
        AccountMeta::new(*sender_dest_pubkey, false),
        AccountMeta::new(*receiver_dest_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];
    Ok(Instruction {
        program_id: *c_token_program_id,
        accounts,
        data,
    })
}

/// Creates a `CloseAccount` instruction.
pub fn close_account(
    c_token_program_id: &Pubkey,
    source_pubkey: &Pubkey,
    destination_pubkey: &Pubkey,
    close_account_data: CloseAccountData,
) -> Result<Instruction, ProgramError> {
    let data = CTokenInstruction::CloseAccount { close_account_data }.pack();

    let accounts = vec![
        AccountMeta::new(*source_pubkey, false),
        AccountMeta::new(*destination_pubkey, false),
    ];
    Ok(Instruction {
        program_id: *c_token_program_id,
        accounts,
        data,
    })
}

