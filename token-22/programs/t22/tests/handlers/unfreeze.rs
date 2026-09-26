use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

use anchor_spl::token_interface::spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

use t22new::{
    extension::ExtensionType,
    state::Account as TokenAccount,
};

pub fn create_frozen_account(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Keypair,
    owner: &Keypair,
) -> Pubkey {
    let account = Keypair::new();

    let space =
        ExtensionType::try_calculate_account_len::<TokenAccount>(&[
            ExtensionType::ImmutableOwner,
            ExtensionType::TransferFeeAmount,
        ])
        .unwrap();

    let lamports =
        svm.minimum_balance_for_rent_exemption(space);

    crate::send(
        svm,
        payer,
        &[&account],
        vec![
            anchor_lang::solana_program::system_instruction::create_account(
                &payer.pubkey(),
                &account.pubkey(),
                lamports,
                space as u64,
                &TOKEN_2022_PROGRAM_ID,
            ),
            t22new::instruction::initialize_account3(
                &TOKEN_2022_PROGRAM_ID,
                &account.pubkey(),
                &mint.pubkey(),
                &owner.pubkey(),
            )
            .unwrap(),
        ],
    );

    account.pubkey()
}

pub fn unfreeze_via_program(
    svm: &mut LiteSVM,
    payer: &Keypair,
    freeze_authority: &Keypair,
    token_account: &Pubkey,
    mint: &Keypair,
) {
    let ix = Instruction {
        program_id: t22::ID,
        accounts: t22::accounts::UnfreezeKycAccount {
            freeze_authority: freeze_authority.pubkey(),
            token_account: *token_account,
            mint: mint.pubkey(),
            token_program: TOKEN_2022_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: t22::instruction::UnfreezeKycAccount {}.data(),
    };

    crate::send(
        svm,
        payer,
        &[freeze_authority],
        vec![ix],
    );
}