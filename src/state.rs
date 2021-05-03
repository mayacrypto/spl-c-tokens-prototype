use std::collections::HashSet;
use arrayref::{array_mut_ref, array_ref, array_refs, mut_array_refs};

use solana_program::{
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    program_error::ProgramError,
};

use crate::proof::PedersenComm;

/// Mint data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mint {
    /// Mint authority.
    pub mint_authority: Pubkey, // 32 bytes
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
        let src = array_ref![src, 0, Mint::LEN];
        let (
            mint_authority, 
            supply, 
            is_initialized
        ) = array_refs![src, 32, 8, 1];

        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };

        Ok(Mint {
            mint_authority: Pubkey::new_from_array(*mint_authority),
            supply: u64::from_le_bytes(*supply),
            is_initialized,
        })
    }

    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Mint::LEN];
        let (
            mint_authority_dst,
            supply_dst,
            is_initialized_dst,
        ) = mut_array_refs![dst, 32, 8, 1];
        let &Mint {
            ref mint_authority,
            supply,
            is_initialized,
        } = self;
        mint_authority_dst.copy_from_slice(mint_authority.as_ref());
        *supply_dst = supply.to_le_bytes();
        is_initialized_dst[0] = is_initialized as u8;
    }
}

/// Account data.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Account {
    /// The mint associated with this account
    pub mint: Pubkey,
    /// Is `true` if this account has been initialized
    pub is_initialized: bool,
    // pub comm: PedersenComm,
}
impl Sealed for Account {}
impl IsInitialized for Account {
    fn is_initialized(&self) -> bool {
        self.is_initialized
    }
}
impl Pack for Account {
    const LEN: usize = 33;
    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let src = array_ref![src, 0, Account::LEN];
        let (mint, is_initialized) = array_refs![src, 32, 1];
        let is_initialized = match is_initialized {
            [0] => false,
            [1] => true,
            _ => return Err(ProgramError::InvalidAccountData),
        };
        Ok(Account {
            mint: Pubkey::new_from_array(*mint),
            is_initialized,
        })
    }
    fn pack_into_slice(&self, dst: &mut [u8]) {
        let dst = array_mut_ref![dst, 0, Account::LEN];
        let (
            mint_dst,
            is_initialized_dst,
        ) = mut_array_refs![dst, 32, 1];
        let &Account {
            ref mint,
            is_initialized,
        } = self;
        mint_dst.copy_from_slice(mint.as_ref());
        is_initialized_dst[0] = is_initialized as u8;
    }
}



