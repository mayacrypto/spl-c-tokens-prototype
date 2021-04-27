use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
};

use crate::{
    instruction::ConfTXInstruction,
    state::WorldState,
    state::MintTX,
    state::TransferTX,
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
            ConfTXInstruction::Initialize { st } => {
                msg!("Instruction: Initialize");
                Self::process_initialize(accounts, st, program_id)
            }
            ConfTXInstruction::Mint { tx } => {
                msg!("Instruction: Transact");
                Self::process_mint(accounts, tx, program_id)
            }
            ConfTXInstruction::Transfer { tx } => {
                msg!("Instruction: Transact");
                Self::process_transfer(accounts, tx, program_id)
            }
        }

    }

    fn process_initialize(
        _account: &[AccountInfo],
        _st: WorldState,
        _program_id: &Pubkey,
        ) -> ProgramResult {
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

