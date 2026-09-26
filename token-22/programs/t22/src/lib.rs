use anchor_lang::prelude::*;

pub mod constants;
pub mod instructions;

pub use constants::*;
use instructions::*;

pub const AE_CIPHERTEXT_LEN: usize = 36;

declare_id!("3kjBCVZPCqW6e6JLFCBkTWPocDH1TD6GgB6XEiLw1qVc");

#[program]
pub mod t22 {
    use super::*;

    pub fn initialize(ctx: Context<InitializeMint>) -> Result<()> {
        ctx.accounts.initialize()
    }

    pub fn transfer_with_fee(
        ctx: Context<TransferWithFee>,
        amount: u64,
        decimals: u8,
    ) -> Result<()> {
        ctx.accounts.transfer(amount, decimals)
    }

    pub fn unfreeze_kyc_account(
        ctx: Context<UnfreezeKycAccount>,
    ) -> Result<()> {
        ctx.accounts.unfreeze()
    }

    pub fn initialize_confidential_mint(
        ctx: Context<InitializeConfidentialMint>,
        withdraw_withheld_authority_elgamal_pubkey: [u8; 32],
    ) -> Result<()> {
        ctx.accounts
            .initialize(withdraw_withheld_authority_elgamal_pubkey)
    }

    pub fn deposit_confidential(
        ctx: Context<DepositConfidential>,
        amount: u64,
        decimals: u8,
    ) -> Result<()> {
        ctx.accounts.deposit(amount, decimals)
    }

    pub fn apply_pending_balance(
        ctx: Context<ApplyPendingBalance>,
        expected_pending_balance_credit_counter: u64,
        new_decryptable_available_balance: [u8; 36],
    ) -> Result<()> {
        ctx.accounts.apply(
            expected_pending_balance_credit_counter,
            new_decryptable_available_balance,
        )
    }
}