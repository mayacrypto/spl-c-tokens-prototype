use borsh::{BorshSerialize, BorshDeserialize};
use std::io::{Write, Error};
use std::io;
use std::ops::Deref;

use solana_program::{
    program_pack::{IsInitialized, Pack, Sealed},
    pubkey::Pubkey,
    program_error::ProgramError,
};
use arrayref::array_ref;

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
    pub comm: [u8; 32], // 32 bytes
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

/// Type wrapper of Pubkey: to implement the Borsh Serialize/Deserialize traits
/// using the New Type Pattern.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BorshPubkey(Pubkey);
impl BorshPubkey {
    pub fn new(pubkey: Pubkey) -> Self {
        Self(pubkey)
    }
}
impl Deref for BorshPubkey {
    type Target = Pubkey;
    
    fn deref(&self) -> &Pubkey {
        let Self(pubkey) = self;
        pubkey
    }
}
impl BorshSerialize for BorshPubkey {
    fn serialize<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        let Self(pubkey) = self;
        let pubkey_bytes = pubkey.to_bytes();
        writer.write(&pubkey_bytes)?;
        Ok(())
    }
}
impl BorshDeserialize for BorshPubkey {
    fn deserialize(buf: &mut &[u8]) -> io::Result<Self> {
        let pubkey = Pubkey::new(array_ref![buf, 0, 32]);
        *buf = &buf[32..];
        Ok(BorshPubkey(pubkey))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pack_unpack_mint() {
        let check = Mint {
            mint_authority: BorshPubkey::new(Pubkey::new(&[1; 32])),
            supply: 42,
            is_initialized: true,
        };
        let mut packed = vec![0; Mint::get_packed_len() + 1];
        assert_eq!(
            Err(ProgramError::InvalidAccountData),
            Mint::pack(check, &mut packed)
        );
        let mut packed = vec![0; Mint::get_packed_len() - 1];
        assert_eq!(
            Err(ProgramError::InvalidAccountData),
            Mint::pack(check, &mut packed),
        );
        let mut packed = vec![0; Mint::get_packed_len()];
        Mint::pack(check, &mut packed).unwrap();
        let expect = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 
            1, 1, 1, // 32 bytes for mint authority pubkey
            42, 0, 0, 0, 0, 0, 0, 0, // 8 bytes for supply
            1, // 1 byte for is_initialized
        ];
        assert_eq!(packed, expect);
        let unpacked = Mint::unpack(&packed).unwrap();
        assert_eq!(unpacked, check);
    }

    #[test]
    fn test_pack_unpack_account() {
        let check = Account {
            mint: BorshPubkey::new(Pubkey::new(&[1; 32])),
            is_initialized: true,
            comm: [0; 32],
        };
        let mut packed = vec![0; Account::get_packed_len() + 1];
        assert_eq!(
            Err(ProgramError::InvalidAccountData),
            Account::pack(check, &mut packed)
        );
        let mut packed = vec![0; Account::get_packed_len() - 1];
        assert_eq!(
            Err(ProgramError::InvalidAccountData),
            Account::pack(check, &mut packed)
        );
        let mut packed = vec![0; Account::get_packed_len()];
        Account::pack(check, &mut packed).unwrap();
        let expect = vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 
            1, 1, 1, // 32 bytes for mint pubkey
            1, // 1 byte for is_initialized
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, // 32 bytes for commitment associated with account
        ];
        assert_eq!(packed, expect);
        let unpacked = Account::unpack(&packed).unwrap();
        assert_eq!(unpacked, check);
    }
}
