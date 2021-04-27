use std::borrow::BorrowMut;

use solana_program::{
    program_pack::{IsInitialized, Pack},
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program_error::ProgramError,
    pubkey::Pubkey,
    msg,
};

use curve25519_dalek::{
    ristretto::CompressedRistretto,
    traits::Identity,
};

use crate::{
    instruction::ConfTXInstruction,
    state::WorldState,
    state::MintTX,
    state::TransferTX,

    state::AggPubkey,
};


pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey, 
        accounts: &[AccountInfo], 
        instruction_data: &[u8]
    ) -> ProgramResult {
        let instruction = ConfTXInstruction::unpack(instruction_data)?;

        match instruction {
            ConfTXInstruction::Initialize => {
                msg!("Instruction: Initialize");
                Self::process_initialize(accounts, program_id)
            }
            ConfTXInstruction::Mint { tx } => {
                msg!("Instruction: Mint");
                Self::process_mint(accounts, tx, program_id)
            }
            ConfTXInstruction::Transfer { tx } => {
                msg!("Instruction: Transfer");
                Self::process_transfer(accounts, tx, program_id)
            }
        }

    }

    fn process_initialize(
        accounts: &[AccountInfo],
        _program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer = next_account_info(account_info_iter)?;

        if !initializer.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let world_state_account = next_account_info(account_info_iter)?;
        let mut world_state = WorldState::unpack_unchecked(&world_state_account.data.borrow())?;
        if world_state.is_initialized() {
            return Err(ProgramError::AccountAlreadyInitialized);
        }

        world_state.is_initialized = true;
        world_state.initializer = *initializer.key;
        world_state.supply = 0;
        world_state.agg_pk = AggPubkey::new(CompressedRistretto::identity());

        // WorldState::pack(world_state, &mut world_state_account.data.borrow_mut())?;

        Ok(())

    }

    fn process_mint(
        _account: &[AccountInfo],
        _st: MintTX,
        _program_id: &Pubkey,
        ) -> ProgramResult {
        Ok(())
    }

    fn process_transfer(
        _account: &[AccountInfo],
        _st: TransferTX,
        _program_id: &Pubkey,
        ) -> ProgramResult {
        Ok(())
    }
}

