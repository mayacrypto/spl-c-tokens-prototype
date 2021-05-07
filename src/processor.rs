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
    state::{Mint, Account, BorshPubkey},
    error::CTokenError,
    proof::{MintData, TransferData},
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

        mint.mint_authority = BorshPubkey::new(mint_authority);
        mint.is_initialized = true;

        // Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_mint(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        mint_data: MintData,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let dest_account_info = next_account_info(account_info_iter)?;

        let mut dest_account = Account::unpack(&dest_account_info.data.borrow())?;

        if dest_account.is_initialized || *mint_info.key != *dest_account.mint {
            return Err(CTokenError::AlreadyInUse.into());
        }
        mint_data.verify_proofs()?;

        let mut mint = Mint::unpack(&mint_info.data.borrow())?;

        dest_account.mint = BorshPubkey::new(*mint_info.key);
        dest_account.is_initialized = true;
        dest_account.comm = mint_data.out_comms.comms_proofs[0].comm;

        mint.supply = mint
            .supply
            .checked_add(amount)
            .ok_or(CTokenError::Overflow)?;

        // Account::pack(dest_account, &mut dest_account_info.data.borrow_mut())?;
        // Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn transfer(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        transfer_data: TransferData,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let source_account_info = next_account_info(account_info_iter)?;
        let dest_account_info = next_account_info(account_info_iter)?;

        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        let mut dest_account = Account::unpack(&dest_account_info.data.borrow())?;

        // Restricting to single 1-1 transfer for now
        if source_account.comm != transfer_data.in_comms.comms[0] {
            return Err(CTokenError::CommitmentMismatch.into());
        }
        transfer_data.verify_proofs()?;

        // TODO: Make access to commitments more ergonomic
        source_account.comm = transfer_data.out_comms.comms_proofs[0].comm;
        dest_account.comm = dest_account.comm + transfer_data.out_comms.comms_proofs[1].comm;

        // Account::pack(source_account, &mut source_account_info.data.borrow_mut());
        // Account::pack(dest_account, &mut dest_account_info.data.borrow_mut());

        Ok(())
    }
}

