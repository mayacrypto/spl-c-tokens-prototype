#![cfg(feature = "test-bpf")]

use {
    solana_program::{pubkey::Pubkey, program_pack::Pack, sysvar::rent::Rent},
    solana_program_test::*,
    solana_sdk::{signature::Signer, transaction::Transaction, signature::Keypair},
    spl_c_tokens_prototype::{id, instruction, processor::Processor, state::{Mint, Account}},
    spl_c_tokens_prototype::txdata::sample_mint_client_for_test,
    solana_sdk::account::{
        create_account_for_test, create_is_signer_account_infos, Account as SolanaAccount,
    }
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

#[tokio::test]
async fn test_initialize_mint() {
    let mut pc = ProgramTest::new("spl_c_tokens_prototype", id(), processor!(Processor::process));

    pc.set_bpf_compute_max_units(200_000);

    let mint_id = crate::id();
    let mint_key = Pubkey::new_unique();
    let mut mint_account = SolanaAccount::new(57, Mint::get_packed_len(), &mint_id);
    mint_account.lamports = mint_minimum_balance();
    pc.add_account(mint_key, mint_account);

    let (mut banks_client, payer, recent_blockhash) = pc.start().await;

    let mint_authority_key = Pubkey::new_unique();
    let mut transaction = Transaction::new_with_payer(
        &[instruction::initialize_mint(&id(), &mint_key, &mint_authority_key).unwrap()],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}

#[tokio::test]
async fn test_mint() {
    let mut pc = ProgramTest::new("spl_c_tokens_prototype", id(), processor!(Processor::process));

    pc.set_bpf_compute_max_units(200_000);

    let mint_id = crate::id();
    let mint_key = Pubkey::new_unique();
    let mut mint_account = SolanaAccount::new(
        mint_minimum_balance(), 
        Mint::get_packed_len(), 
        &mint_id
    );

    // Generate mint authority and add to test environment
    let mint_authority_keypair = Keypair::new();
    let mut mint_authority_account = SolanaAccount::default();
    pc.add_account(mint_authority_keypair.pubkey(), mint_authority_account);

    // Initialize mint and add to test environment
    let mut mint = Mint::unpack_unchecked(&mint_account.data).unwrap();
    mint.mint_authority = mint_authority_keypair.pubkey();
    mint.is_initialized = true;

    Mint::pack(mint, &mut mint_account.data).unwrap();
    pc.add_account(mint_key, mint_account);

    // Create an account to mint to
    let account_key = Pubkey::new_unique();
    let mut account_account = SolanaAccount::new(
        account_minimum_balance(),
        Account::get_packed_len(),
        &mint_id,
    );
    pc.add_account(account_key, account_account);

    let mint_data = sample_mint_client_for_test(57);

    let (mut banks_client, payer, recent_blockhash) = pc.start().await;

    let mut transaction = Transaction::new_with_payer(
        &[instruction::mint(
            &id(), 
            &mint_key, 
            &account_key, 
            &mint_authority_keypair.pubkey(), 
            mint_data
        ).unwrap()
        ],
        Some(&payer.pubkey()),
    );
    transaction.sign(&[&payer, &mint_authority_keypair], recent_blockhash);
    banks_client.process_transaction(transaction).await.unwrap();
}
