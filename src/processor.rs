use solana_program::{
    account_info::AccountInfo,
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
};

use crate::{
    instruction::ConfTXInstruction,
    state::TXState,
    state::Transaction,
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
            ConfTXInstruction::Transact { tx } => {
                msg!("Instruction: Transact");
                Self::process_transact(accounts, tx, program_id)
            }
        }

    }

    fn process_initialize(
        _account: &[AccountInfo],
        _st: TXState,
        _program_id: &Pubkey,
        ) -> ProgramResult {
        Ok(())
    }

    fn process_transact(
        _account: &[AccountInfo],
        _st: Transaction,
        _program_id: &Pubkey,
        ) -> ProgramResult {
        Ok(())
    }
}

