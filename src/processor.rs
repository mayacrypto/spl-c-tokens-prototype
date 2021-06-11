use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program_pack::Pack,
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

use crate::{
    error::CTokenError,
    instruction::CTokenInstruction,
    state::{Account, Mint},
    txdata::{CryptoVerRequired, MintData},
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

        mint.mint_authority = mint_authority;
        mint.is_initialized = true;

        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process_mint(accounts: &[AccountInfo], mint_data: MintData) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let dest_account_info = next_account_info(account_info_iter)?;
        let expected_authority = next_account_info(account_info_iter)?;
        let rent = &Rent::from_account_info(next_account_info(account_info_iter)?)?;

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

        // Check rent exempt
        if !rent.is_exempt(dest_account_info.lamports(), dest_account_info.data_len()) {
            return Err(CTokenError::NotRentExempt.into());
        }

        // Verify all the crypto components:
        // - verify that each newly generated commitments are valid commitments
        //   to a positive 64-bit number
        // - verify that the sum of all the newly generated commitments contain
        //   the claimed mint amount
        mint_data.verify_crypto()?;

        // Validate mint authority
        let mut mint = Mint::unpack(&mint_info.data.borrow())?;
        if *expected_authority.key != mint.mint_authority {
            return Err(CTokenError::OwnerMismatch.into());
        }

        // Update the mint and newly created account
        dest_account.mint = *mint_info.key;
        dest_account.is_initialized = true;
        dest_account.comm = mint_data.out_comm;

        mint.supply = mint
            .supply
            .checked_add(mint_data.amount)
            .ok_or(CTokenError::Overflow)?;

        Account::pack(dest_account, &mut dest_account_info.data.borrow_mut())?;
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    pub fn process(_program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
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
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::instruction::*;
    use crate::txdata::{sample_mint_client_for_test};
    use solana_program::{
        account_info::IntoAccountInfo, clock::Epoch, instruction::Instruction,
        program_error::ProgramError, sysvar::rent,
    };
    use solana_sdk::account::{
        create_account_for_test, create_is_signer_account_infos, Account as SolanaAccount,
    };

    fn account_minimum_balance() -> u64 {
        Rent::default().minimum_balance(Account::get_packed_len())
    }

    fn mint_minimum_balance() -> u64 {
        Rent::default().minimum_balance(Mint::get_packed_len())
    }

    fn rent_sysvar() -> SolanaAccount {
        create_account_for_test(&Rent::default())
    }

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
        let mut mint_account = SolanaAccount::new(57, Mint::get_packed_len(), &program_id);
        let mut rent_sysvar = rent_sysvar();

        // mint is not rent exempt
        assert_eq!(
            Err(CTokenError::NotRentExempt.into()),
            do_process_instruction(
                initialize_mint(&program_id, &mint_authority_key, &mint_key).unwrap(),
                vec![&mut mint_account, &mut rent_sysvar],
            )
        );

        mint_account.lamports = mint_minimum_balance();

        // create new mint
        do_process_instruction(
            initialize_mint(&program_id, &mint_authority_key, &mint_key).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create twice
        assert_eq!(
            Err(CTokenError::AlreadyInUse.into()),
            do_process_instruction(
                initialize_mint(&program_id, &mint_key, &mint_authority_key).unwrap(),
                vec![&mut mint_account, &mut rent_sysvar]
            )
        );
    }

    #[test]
    fn test_mint() {
        let program_id = crate::id();

        let mint_key = Pubkey::new_unique();
        let mut mint_account =
            SolanaAccount::new(mint_minimum_balance(), Mint::get_packed_len(), &program_id);

        let mint_authority_key = Pubkey::new_unique();
        let mut mint_authority_account = SolanaAccount::default();

        let mut rent_sysvar = rent_sysvar();

        // create new mint with owner
        do_process_instruction(
            initialize_mint(&program_id, &mint_key, &mint_authority_key).unwrap(),
            vec![&mut mint_account, &mut rent_sysvar],
        )
        .unwrap();

        // create an account
        let account_key = Pubkey::new_unique();
        let mut account_account = SolanaAccount::new(
            account_minimum_balance(),
            Account::get_packed_len(),
            &program_id,
        );

        let mint_data = sample_mint_client_for_test(57);

        do_process_instruction(
            mint(
                &program_id,
                &mint_key,
                &account_key,
                &mint_authority_key,
                mint_data,
            )
            .unwrap(),
            vec![
                &mut mint_account,
                &mut account_account,
                &mut mint_authority_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // The mint supply should be updated.
        let mint_state = Mint::unpack_unchecked(&mint_account.data).unwrap();
        assert_eq!(mint_state.supply, 57);

        // The commitment in the associated account should be updated.
        let account = Account::unpack_unchecked(&account_account.data).unwrap();
        assert_eq!(account.comm, mint_data.out_comm);

        // test for account already-in-use error
        assert_eq!(
            Err(CTokenError::AlreadyInUse.into()),
            do_process_instruction(
                mint(
                    &program_id,
                    &mint_key,
                    &account_key,
                    &mint_authority_key,
                    mint_data
                )
                .unwrap(),
                vec![
                    &mut mint_account,
                    &mut account_account,
                    &mut mint_authority_account,
                    &mut rent_sysvar
                ],
            )
        );

        // create another account to test supply accumulation
        let account2_key = Pubkey::new_unique();
        let mut account2_account = SolanaAccount::new(
            account_minimum_balance(),
            Account::get_packed_len(),
            &program_id,
        );

        let mint_data = sample_mint_client_for_test(43);

        do_process_instruction(
            mint(
                &program_id,
                &mint_key,
                &account2_key,
                &mint_authority_key,
                mint_data,
            )
            .unwrap(),
            vec![
                &mut mint_account,
                &mut account2_account,
                &mut mint_authority_account,
                &mut rent_sysvar,
            ],
        )
        .unwrap();

        // The mint supply should be updated.
        let mint_state = Mint::unpack_unchecked(&mint_account.data).unwrap();
        assert_eq!(mint_state.supply, 100);

        // The commitment in the associated account should be updated.
        let account = Account::unpack_unchecked(&account2_account.data).unwrap();
        assert_eq!(account.comm, mint_data.out_comm);

        // TODO: Test for invalid Proof of Knowledge and Range Proof
    }
}
