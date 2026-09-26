use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

use t22new::{
    extension::{
        transfer_fee::TransferFeeAmount,
        BaseStateWithExtensions,
        ExtensionType,
        StateWithExtensions,
    },
    state::{Account as TokenAccount, AccountState},
};

use anchor_spl::token_interface::spl_token_2022::ID as TOKEN_2022_PROGRAM_ID;

pub fn create_token_account(
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

    crate::send(
        svm,
        payer,
        &[],
        vec![
            t22new::instruction::thaw_account(
                &TOKEN_2022_PROGRAM_ID,
                &account.pubkey(),
                &mint.pubkey(),
                &payer.pubkey(),
                &[],
            )
            .unwrap(),
        ],
    );

    account.pubkey()
}

pub fn mint_to(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Keypair,
    destination: &Pubkey,
    amount: u64,
) {
    crate::send(
        svm,
        payer,
        &[],
        vec![
            t22new::instruction::mint_to(
                &TOKEN_2022_PROGRAM_ID,
                &mint.pubkey(),
                destination,
                &payer.pubkey(),
                &[],
                amount,
            )
            .unwrap(),
        ],
    );
}

pub fn transfer_via_program(
    svm: &mut LiteSVM,
    payer: &Keypair,
    owner: &Keypair,
    source: &Pubkey,
    destination: &Pubkey,
    mint: &Keypair,
    amount: u64,
) {
    let ix = Instruction {
        program_id: t22::ID,
        accounts: t22::accounts::TransferWithFee {
            authority: owner.pubkey(),
            source: *source,
            destination: *destination,
            mint: mint.pubkey(),
            token_program: TOKEN_2022_PROGRAM_ID,
        }
        .to_account_metas(None),
        data: t22::instruction::TransferWithFee {
            amount,
            decimals: t22::DECIMALS,
        }
        .data(),
    };

    crate::send(
        svm,
        payer,
        &[owner],
        vec![ix],
    );
}

pub fn balances(
    svm: &LiteSVM,
    account: &Pubkey,
) -> (u64, u64) {
    let data =
        svm.get_account(account).unwrap().data.clone();

    let state =
        StateWithExtensions::<TokenAccount>::unpack(&data)
            .unwrap();

    let amount = state.base.amount;

    let withheld = u64::from(
        state
            .get_extension::<TransferFeeAmount>()
            .unwrap()
            .withheld_amount,
    );

    (amount, withheld)
}

pub fn is_unfrozen(
    svm: &LiteSVM,
    account: &Pubkey,
) -> bool {
    let data =
        svm.get_account(account).unwrap().data.clone();

    StateWithExtensions::<TokenAccount>::unpack(&data)
        .unwrap()
        .base
        .state
        == AccountState::Initialized
}