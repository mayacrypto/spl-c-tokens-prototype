use std::borrow::BorrowMut;

use solana_program::{
    program_pack::Pack,
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{
    instruction::CTokenInstruction,
    state::Mint,

    error::CTokenError,
};

/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Processes an [InitializeMint] instruction.
    pub fn process_initialize_mint(
        accounts: &[AccountInfo],
        mint_authority: Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let mint_data_len = mint_info.data_len();
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

        let mut mint = Mint::unpack_unchecked(&mint_info.data.borrow())?;
        if mint.is_initialized {
            return Err(CTokenError::AlreadyInUse.into());
        }

        if !rent.is_exempt(mint_info.lamports(), mint_data_len) {
            return Err(CTokenError::NotRentExempt.into());
        }

        mint.mint_authority = mint_authority;
        mint.is_initialized = true;

        // Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }



}

