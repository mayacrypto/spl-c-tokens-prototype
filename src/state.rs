use std::collections::HashSet;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};
use borsh::{BorshSerialize, BorshDeserialize};

use solana_program::{
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    program_error::ProgramError,
};

use crate::{
    proof::PedersenComm,
    instruction::BorshPubkey,
};

/// Mint data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Mint {
    /// Mint authority.
    pub mint_authority: BorshPubkey, // 32 bytes
    /// Total supply of tokens.
    pub supply: u64, // 8 bytes
    /// Is `true` if this structure has been initialized
    pub is_initialized: bool, // 1 byte
}
impl Sealed for Mint {}
impl IsInitialized for Mint {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for Mint {
    const LEN: usize = 41;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if let Ok(mint) = Mint::try_from_slice(src) {
            Ok(mint)
        } else {
            Err(ProgramError::InvalidAccountData)
        }
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst.copy_from_slice(
            self
            .try_to_vec()
            .unwrap()
            .as_ref()
        );
    }
}

/// Account data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Account {
    /// The mint associated with this account
    pub mint: BorshPubkey, // 32 bytes
    /// Is `true` if this account has been initialized
    pub is_initialized: bool, // 1 byte
    /// The commitment associated with this account
    pub comm: PedersenComm, // 32 bytes
}
impl Sealed for Account {}
impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for Account {
    const LEN: usize = 65;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        if let Ok(account) = Account::try_from_slice(src) {
            Ok(account)
        } else {
            Err(ProgramError::InvalidAccountData)
        }
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst.copy_from_slice(
            self
            .try_to_vec()
            .unwrap()
            .as_ref()
        );
    }
}
