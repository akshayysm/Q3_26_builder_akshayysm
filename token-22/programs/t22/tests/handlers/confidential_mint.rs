use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::{InstructionData, ToAccountMetas};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_signer::Signer;

use zk::encryption::elgamal::ElGamalKeypair;

pub fn init_confidential_mint(
    svm: &mut LiteSVM,
    payer: &Keypair,
    fee_authority: &ElGamalKeypair,
) -> Keypair {
    let mint = Keypair::new();

    let ix = Instruction {
        program_id: t22::ID,
        accounts: t22::accounts::InitializeConfidentialMint {
            payer: payer.pubkey(),
            mint: mint.pubkey(),
            system_program: anchor_lang::system_program::ID,
            token_program: t22new::ID,
        }
        .to_account_metas(None),
        data: t22::instruction::InitializeConfidentialMint {
            withdraw_withheld_authority_elgamal_pubkey:
                fee_authority.pubkey().into(),
        }
        .data(),
    };

    crate::send(
        svm,
        payer,
        &[&mint],
        vec![ix],
    );

    mint
}