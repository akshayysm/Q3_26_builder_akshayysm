use {
    anchor_lang::{
        prelude::msg,
        solana_program::instruction::Instruction,
        solana_program::program_pack::Pack,
        system_program::ID as SYSTEM_PROGRAM_ID,
        AccountDeserialize,
        InstructionData,
        ToAccountMetas,
    },
    anchor_spl::{
        associated_token::{self, ID as ASSOCIATED_TOKEN_PROGRAM_ID},
        token::spl_token,
    },
    litesvm::LiteSVM,
    litesvm_token::{
        spl_token::ID as TOKEN_PROGRAM_ID,
        CreateAssociatedTokenAccount,
        CreateMint,
        MintTo,
    },
    solana_keypair::Keypair,
    solana_message::Message,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
};

fn setup() -> (LiteSVM, Keypair) {
    let program_id = escrowq32026::id();
    let payer = Keypair::new();

    let mut svm = LiteSVM::new();

    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/escrowq32026.so"
    ));

    svm.add_program(program_id, bytes).unwrap();

    svm.airdrop(&payer.pubkey(), 1_000_000_000).unwrap();

    (svm, payer)
}

#[test]
fn test_make_and_update_and_refund() {
    let (mut program, payer) = setup();

    let maker = payer.pubkey();

    let mint_a = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut program, &payer)
        .decimals(6)
        .authority(&maker)
        .send()
        .unwrap();

    let maker_ata_a = CreateAssociatedTokenAccount::new(
        &mut program,
        &payer,
        &mint_a,
    )
    .owner(&maker)
    .send()
    .unwrap();

    let escrow = Pubkey::find_program_address(
        &[
            b"escrow",
            maker.as_ref(),
            &123u64.to_le_bytes(),
        ],
        &escrowq32026::id(),
    )
    .0;

    let vault =
        associated_token::get_associated_token_address(&escrow, &mint_a);

    MintTo::new(
        &mut program,
        &payer,
        &mint_a,
        &maker_ata_a,
        1_000_000_000,
    )
    .send()
    .unwrap();

    // MAKE


    let make_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Make {
            maker,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Make {
            deposit: 10_000_000,
            seed: 123u64,
            receive: 10_000_000,
            expiration: 1_778_020_6209,
        }
        .data(),
    };

    let message = Message::new(
        &[make_ix],
        Some(&maker),
    );

    let transaction =
        Transaction::new(&[&payer], message, program.latest_blockhash());

    program.send_transaction(transaction).unwrap();

    msg!("Make successful");

    // Verify Make
    let vault_account = program.get_account(&vault).unwrap();

    let vault_data =
        spl_token::state::Account::unpack(&vault_account.data).unwrap();

    assert_eq!(vault_data.amount, 10_000_000);
    assert_eq!(vault_data.owner, escrow);
    assert_eq!(vault_data.mint, mint_a);

    let escrow_account = program.get_account(&escrow).unwrap();

    let escrow_data =
        escrowq32026::state::Escrow::try_deserialize(
            &mut escrow_account.data.as_ref(),
        )
        .unwrap();

    assert_eq!(escrow_data.seed, 123u64);
    assert_eq!(escrow_data.maker, maker);
    assert_eq!(escrow_data.mint_a, mint_a);
    assert_eq!(escrow_data.mint_b, mint_b);
    assert_eq!(escrow_data.receive, 10_000_000);

    // UPDATE

    let update_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Update {
            maker,
            escrow,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Update {
            receive: 20_000_000,
        }
        .data(),
    };

    let message = Message::new(
        &[update_ix],
        Some(&maker),
    );

    let transaction =
        Transaction::new(&[&payer], message, program.latest_blockhash());

    program.send_transaction(transaction).unwrap();

    msg!("Update successful");

    // Verify Update
    let escrow_account = program.get_account(&escrow).unwrap();

    let escrow_data =
        escrowq32026::state::Escrow::try_deserialize(
            &mut escrow_account.data.as_ref(),
        )
        .unwrap();

    assert_eq!(escrow_data.receive, 20_000_000);

    // REFUND

    let refund_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Refund {
            maker,
            mint_a,
            maker_ata_a,
            escrow,
            vault,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Refund {}.data(),
    };

    let message = Message::new(
        &[refund_ix],
        Some(&maker),
    );

    let transaction =
        Transaction::new(&[&payer], message, program.latest_blockhash());

    program.send_transaction(transaction).unwrap();

    msg!("Refund successful");

    assert!(program.get_account(&escrow).is_none());
    assert!(program.get_account(&vault).is_none());
}
#[test]
fn test_make_and_take() {
    let (mut program, maker) = setup();

    let maker_pubkey = maker.pubkey();

    let taker = Keypair::new();
    let taker_pubkey = taker.pubkey();

    program
        .airdrop(&taker_pubkey, 1_000_000_000)
        .unwrap();

    // Create Mint A and Mint B
    let mint_a = CreateMint::new(&mut program, &maker)
        .decimals(6)
        .authority(&maker_pubkey)
        .send()
        .unwrap();

    let mint_b = CreateMint::new(&mut program, &maker)
        .decimals(6)
        .authority(&maker_pubkey)
        .send()
        .unwrap();

    // Maker's Token A account
    let maker_ata_a =
        CreateAssociatedTokenAccount::new(&mut program, &maker, &mint_a)
            .owner(&maker_pubkey)
            .send()
            .unwrap();

    // Maker's Token B account
    let maker_ata_b =
        CreateAssociatedTokenAccount::new(&mut program, &maker, &mint_b)
            .owner(&maker_pubkey)
            .send()
            .unwrap();

    // Taker's Token A account
    let taker_ata_a =
        CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_a)
            .owner(&taker_pubkey)
            .send()
            .unwrap();

    // Taker's Token B account
    let taker_ata_b =
        CreateAssociatedTokenAccount::new(&mut program, &taker, &mint_b)
            .owner(&taker_pubkey)
            .send()
            .unwrap();

    // Give maker 10 Token A
    MintTo::new(
        &mut program,
        &maker,
        &mint_a,
        &maker_ata_a,
        10_000_000,
    )
    .send()
    .unwrap();

    // Give taker 20 Token B
    MintTo::new(
        &mut program,
        &maker,
        &mint_b,
        &taker_ata_b,
        20_000_000,
    )
    .send()
    .unwrap();

    let seed = 123u64;

    let escrow = Pubkey::find_program_address(
        &[
            b"escrow",
            maker_pubkey.as_ref(),
            &seed.to_le_bytes(),
        ],
        &escrowq32026::id(),
    )
    .0;

    let vault =
        associated_token::get_associated_token_address(&escrow, &mint_a);

    // MAKE

    let make_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Make {
            maker: maker_pubkey,
            mint_a,
            mint_b,
            maker_ata_a,
            escrow,
            vault,
            associated_token_program: ASSOCIATED_TOKEN_PROGRAM_ID,
            token_program: TOKEN_PROGRAM_ID,
            system_program: SYSTEM_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Make {
            deposit: 10_000_000,
            seed,
            receive: 20_000_000,
            expiration: 1_778_020_6209,
        }
        .data(),
    };

    let message = Message::new(
        &[make_ix],
        Some(&maker_pubkey),
    );

    let transaction =
        Transaction::new(&[&maker], message, program.latest_blockhash());

    program.send_transaction(transaction).unwrap();

    // TAKE

    let take_ix = Instruction {
        program_id: escrowq32026::id(),
        accounts: escrowq32026::accounts::Take {
            taker: taker_pubkey,
            maker: maker_pubkey,
            escrow,
            mint_a,
            mint_b,
            taker_ata_b,
            taker_ata_a,
            maker_ata_b,
            vault,
            token_program: TOKEN_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: escrowq32026::instruction::Take {}.data(),
    };

    let message = Message::new(
        &[take_ix],
        Some(&taker_pubkey),
    );

    let transaction =
        Transaction::new(&[&taker], message, program.latest_blockhash());

    program.send_transaction(transaction).unwrap();

    
    // VERIFY
    

    let taker_a_account =
        program.get_account(&taker_ata_a).unwrap();

    let taker_a =
        spl_token::state::Account::unpack(&taker_a_account.data).unwrap();

    assert_eq!(taker_a.amount, 10_000_000);

    let maker_b_account =
        program.get_account(&maker_ata_b).unwrap();

    let maker_b =
        spl_token::state::Account::unpack(&maker_b_account.data).unwrap();

    assert_eq!(maker_b.amount, 20_000_000);

    assert!(program.get_account(&escrow).is_none());
    assert!(program.get_account(&vault).is_none());
}