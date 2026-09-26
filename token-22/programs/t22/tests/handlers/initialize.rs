use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

pub fn init_mint(
    svm: &mut LiteSVM,
    payer: &Keypair,
) -> Keypair {
    let mint = Keypair::new();

    let ix = Instruction {
        program_id: t22::ID,
        accounts: t22::accounts::InitializeMint {
            payer: payer.pubkey(),
            mint: mint.pubkey(),
            system_program: anchor_lang::system_program::ID,
            token_program: t22new::ID,
        }
        .to_account_metas(None),
        data: t22::instruction::Initialize {}.data(),
    };

    crate::send(
        svm,
        payer,
        &[&mint],
        vec![ix],
    );

    mint
}

pub fn mint_data(
    svm: &LiteSVM,
    mint: &Keypair,
) -> Vec<u8> {
    svm.get_account(&mint.pubkey())
        .unwrap()
        .data
        .clone()
}