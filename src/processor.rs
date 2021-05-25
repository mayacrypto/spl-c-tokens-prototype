use solana_program::{
    program_pack::Pack,
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    pubkey::Pubkey,
    msg,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{
    instruction::CTokenInstruction,
    state::{Mint, Account, BorshPubkey},
    error::CTokenError,
    txdata::{CryptoVerRequired, MintData, TransferData, CloseAccountData},
};


/// Program state handler.
pub struct Processor {}
impl Processor {
    /// Processes an [InitializeMint] instruction.
    pub fn process_initialize_mint(
        accounts: &[AccountInfo],
        mint_authority: Pubkey,
    ) -> ProgramResult {
        // Almost identical to the process_initialize_mint function in the 
        // regular SPL token program.
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

        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_mint(
        accounts: &[AccountInfo],
        mint_data: MintData,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let dest_account_info = next_account_info(account_info_iter)?;
        
        let mut dest_account = Account::unpack_unchecked(&dest_account_info.data.borrow())?;

        // In the protocol, commitments (encrypted token amount) are immutably
        // bound to an account. This means that to mint tokens for a specific
        // user, the mint authority must open a new account for the user. The
        // user can then submit a transaction that merges two (or more) of its
        // accounts into a new merged account. Therefore, the `Mint` instruction
        // can be viewed as a merge of the `InitializeAccount` and `MintTo`
        // instruction in the regular SPL token program. The check below
        // verifies that the destination account is not already initialized
        // since mint should always initialize a new account.
        //
        // This is technically not strictly necessary for the protocol; we can
        // technically have a `MintTo` instruction as well, which can simplify
        // the API for some applications. For the prototype, we can stick to
        // `Mint` for now.
        if dest_account.is_initialized {
            return Err(CTokenError::AlreadyInUse.into());
        }

        // Verify all the crypto components:
        // - verify that each newly generated commitments are valid commitments 
        //   to a positive 64-bit number
        // - verify that the sum of all the newly generated commitments contain
        //   the claimed mint amount
        mint_data.verify_crypto()?;

        // Update the mint and newly created account
        let mut mint = Mint::unpack(&mint_info.data.borrow())?;

        dest_account.mint = BorshPubkey::new(*mint_info.key);
        dest_account.is_initialized = true;
        dest_account.comm = [0; 32];

        mint.supply = mint
            .supply
            .checked_add(mint_data.amount)
            .ok_or(CTokenError::Overflow)?;

        Account::pack(dest_account, &mut dest_account_info.data.borrow_mut())?;
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_transfer(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _transfer_data: TransferData,
    ) -> ProgramResult {
        // let account_info_iter = &mut accounts.iter();

        // let first_source_account_info = next_account_info(account_info_iter)?;
        // let second_source_account_info = next_account_info(account_info_iter)?;
        // 
        // let mut first_source_account = Account::unpack(&source_account_info.data.borrow())?;
        // let mut second_source_account = Account::unpack(&source_account_info.data.borrow())?;

        // let first_dest_account_info = next_account_info(account_info_iter)?;
        // let second_dest_account_info = next_account_info(account_info_iter)?;

        // let mut first_dest_account = Account::unpack(&source_account_info.data.borrow())?;
        // let mut second_dest_account = Account::unpack(&dest_account_info.data.borrow())?;

        // // Check that the commitments in the accounts are consistent with the commitments
        // // in the transfer data.
        // //
        // // If the accounts addresses are replaced by commitments, then this check is 
        // // not required. See txdata.rs.
        // if [first_source_account, second_source_account] != transfer_data.in_comms ||
        //     [first_dest_account, second_dest_account] != transfer_data.out_comms {
        //         return Err(CTokenError::CommitmentMismatch.into());
        // }

        // // Verify all the crypto components:
        // // - verify that each newly generated commitments are valid commitments
        // //   to a positive 64-bit number (Bulletproof)
        // // - verify that the sum of all the incoming commitments contain the same
        // //   value as the sum of all newly generated commitments
        // transfer_data.verify_crypto()?;

        // source_account.comm = transfer_data.out_comms[0].0;

        // // TODO: Make access to commitments more ergonomic
        // source_account.comm = transfer_data.out_comms[0].0;
        // // dest_account.comm = dest_account.comm + transfer_data.out_comms[0].1;

        // // Account::pack(source_account, &mut source_account_info.data.borrow_mut());
        // // Account::pack(dest_account, &mut dest_account_info.data.borrow_mut());

        Ok(())
    }

    pub fn process_close_account(
        _program_id: &Pubkey,
        _accounts: &[AccountInfo],
        _close_account_data: CloseAccountData,
    ) -> ProgramResult {
        // There are some issues that must be decided on regarding rent
        Ok(())
    }

    pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
        let instruction = CTokenInstruction::unpack(input)?;

        match instruction {
            CTokenInstruction::InitializeMint { mint_authority } => {
                msg!("Instruction: InitializeMint");
                Self::process_initialize_mint(accounts, mint_authority)
            }
            CTokenInstruction::Mint { mint_data } => {
                msg!("Instruction: Mint");
                Self::process_mint(accounts, mint_data)
            }
            CTokenInstruction::Transfer { transfer_data } => {
                msg!("Instruction: Transfer");
                Self::process_transfer(program_id, accounts, transfer_data)
            }
            CTokenInstruction::CloseAccount { close_account_data } => {
                msg!("Instruction: CloseAccount");
                Self::process_close_account(program_id, accounts, close_account_data)
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        instruction::*,
        proof::{BorshScalar, BorshRistretto, BorshRangeProof, PedersenComm, ProofKnowledge},
    };
    use solana_program::{
        account_info::IntoAccountInfo, clock::Epoch, instruction::Instruction, sysvar::rent,
        program_error::ProgramError,
    };
    use solana_sdk::account::{
        create_account_for_test, create_is_signer_account_infos, Account as SolanaAccount,
    };
    use curve25519_dalek::{
        ristretto::CompressedRistretto,
        scalar::Scalar,
    };


    fn do_process_instruction(
        instruction: Instruction,
        accounts: Vec<&mut SolanaAccount>,
    ) -> ProgramResult {
        let mut meta = instruction
            .accounts
            .iter()
            .zip(accounts)
            .map(|(account_meta, account)| (&account_meta.pubkey, account_meta.is_signer, account))
            .collect::<Vec<_>>();

        let account_infos = create_is_signer_account_infos(&mut meta);
        Processor::process(&instruction.program_id, &account_infos, &instruction.data)
    }

    #[test]
    fn test_initialize_mint() {
        let program_id = crate::id();
        let mint_authority_key = Pubkey::new_unique();
        let mint_key = Pubkey::new_unique();
        let mut mint_account = SolanaAccount::new(42, Mint::get_packed_len(), &program_id);

        let mut rent_sysvar = create_account_for_test(&Rent::default());

        // mint is not rent exempt
        assert_eq!(
            Err(CTokenError::NotRentExempt.into()),
            do_process_instruction(
                initialize_mint(&program_id, &mint_authority_key, &mint_key).unwrap(),
                vec![&mut mint_account, &mut rent_sysvar],
            )
        );
    }

    fn account_minimum_balance() -> u64 {
        Rent::default().minimum_balance(Mint::get_packed_len())
    }

    fn mint_minimum_balance() -> u64 {
        Rent::default().minimum_balance(Mint::get_packed_len())
    }

    fn rent_sysvar() -> SolanaAccount {
        create_account_for_test(&Rent::default())
    }

    #[test]
    fn test_mint() {
        let program_id = crate::id();
        
        let mint_key = Pubkey::new_unique();
        let mut mint_account = 
            SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);

        let owner_key = Pubkey::new_unique();
        let mut owner_account = SolanaAccount::default();

        let mut rent_sysvar = rent_sysvar();

        // create new mint with owner
        do_process_instruction(
            initialize_mint(&program_id, &mint_key, &owner_key).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        ).unwrap();
        
        // create account
        let account_key = Pubkey::new_unique();
        let mut account_account = SolanaAccount::new(
            account_minimum_balance(),
            Account::get_packed_len(),
            &program_id,
        );

        // build MintData
        let pedersen_comm = PedersenComm::new(
            BorshRistretto::new(CompressedRistretto([0; 32]))
        );
        let range_proof = BorshRangeProof;
        let out_comms = vec![(pedersen_comm, range_proof)];

        let proof_knowledge = ProofKnowledge { 
            nonce: BorshRistretto::new(CompressedRistretto([0; 32])),
            scalar: BorshScalar::new(Scalar::default()),
        };

        let mint_data = MintData {
            amount: 10,
            out_comms,
            proof_knowledge,
        };

        do_process_instruction(
            mint(&program_id, &mint_key, &account_key, &owner_key, mint_data).unwrap(), 
            vec![&mut mint_account, &mut account_account, &mut owner_account],
        ).unwrap();
    }
}





