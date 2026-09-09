use anchor_lang::prelude::*;
use anchor_spl::token_interface::{
    close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TokenInterface,
    TransferChecked,
};

use crate::{state::Escrow, ESCROW_SEED};

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,

    /// CHECK: Validated by the escrow's has_one constraint.
    pub maker: UncheckedAccount<'info>,

    #[account(
        mut,
        close = taker,
        has_one = maker,
        has_one = mint_a,
        has_one = mint_b,
        seeds = [
            ESCROW_SEED,
            maker.key().as_ref(),
            escrow.seed.to_le_bytes().as_ref()
        ],
        bump = escrow.bump,
    )]
    pub escrow: Account<'info, Escrow>,

    pub mint_a: InterfaceAccount<'info, Mint>,

    pub mint_b: InterfaceAccount<'info, Mint>,

    #[account(
        mut,
        token::mint = mint_b,
        token::authority = taker,
        token::token_program = token_program,
    )]
    pub taker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = mint_a,
        token::authority = taker,
        token::token_program = token_program,
    )]
    pub taker_ata_a: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = mint_b,
        token::authority = maker,
        token::token_program = token_program,
    )]
    pub maker_ata_b: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = mint_a,
        token::authority = escrow,
        token::token_program = token_program,
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> Take<'info> {
    pub fn take(&mut self) -> Result<()> {
        let cpi_program = self.token_program.key();

        // Taker sends Token B to maker.
        let transfer_to_maker = TransferChecked {
            from: self.taker_ata_b.to_account_info(),
            mint: self.mint_b.to_account_info(),
            to: self.maker_ata_b.to_account_info(),
            authority: self.taker.to_account_info(),
        };

        let cpi_ctx = CpiContext::new(
            cpi_program,
            transfer_to_maker,
        );

        transfer_checked(
            cpi_ctx,
            self.escrow.receive,
            self.mint_b.decimals,
        )?;

        // Escrow PDA signs for the vault.
        let signer_seeds: [&[&[u8]]; 1] = [&[
            ESCROW_SEED,
            self.maker.key.as_ref(),
            &self.escrow.seed.to_le_bytes()[..],
            &[self.escrow.bump],
        ]];

        // Vault sends Token A to taker.
        let transfer_to_taker = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.taker_ata_a.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program,
            transfer_to_taker,
            &signer_seeds,
        );

        transfer_checked(
            cpi_ctx,
            self.vault.amount,
            self.mint_a.decimals,
        )?;

        // Close the vault.
        let close_vault = CloseAccount {
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info(),
            authority: self.escrow.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            cpi_program,
            close_vault,
            &signer_seeds,
        );

        close_account(cpi_ctx)?;

        Ok(())
    }
}