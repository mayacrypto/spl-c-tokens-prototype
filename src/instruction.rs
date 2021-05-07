use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvar,
};
use std::ops::Deref;
use borsh::{BorshSerialize, BorshDeserialize};
use std::io::{Write, Error};
use std::io;
use std::mem::size_of;
use std::convert::TryInto;

use crate::{
    error::CTokenError::InvalidInstruction,
    proof::{MintData, TransferData},
};

pub enum CTokenInstruction {

    /// Initializes a new mint.
    ///
    /// This is analogous to the `InitializeMint` instruction in the SPL token program with
    /// decimals and freeze_authority removed for prototyping purposes.
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
    ///   0. `[writable]` The source account.
    ///   1. `[writable]` The destination account.
    ///   2. `[]` Rent sysvar
    ///
    Transfer {
        /// Data for the transfer
        tx_data: TransferData,
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
    CloseAccount,
}

impl CTokenInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        let (tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (mint_authority, _) = Self::unpack_pubkey(rest)?;
                Self::InitializeMint { mint_authority }
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

/// Creates a `MintTo` instruction.
pub fn mint_to(
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
    ];
    Ok(Instruction {
        program_id: *c_token_program_id,
        accounts,
        data,
    })
}

/// Type wrapper of Pubkey: to implement the Borsh Serialize/Deserialize traits
/// using the New Type Pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorshPubkey(Pubkey);
impl Deref for BorshPubkey {
    type Target = Pubkey;
    
    fn deref(&self) -> &Pubkey {
        let Self(pubkey) = self;
        pubkey
    }
}
impl BorshSerialize for BorshPubkey {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        Ok(())
    }
}
impl BorshDeserialize for BorshPubkey {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        if buf.len() == 32 {
            Ok(BorshPubkey(
                    Pubkey::new(buf)
            ))
        } else {
            Err(io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Bytes does not match Pubkey size"
            ))
        }
    }
}


