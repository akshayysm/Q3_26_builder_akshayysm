use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
    token_interface::{mint_to_checked, MintToChecked},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    types::{Attributes, PluginType},
};

use crate::error::ErrorCode;
use crate::state::{Config, StakeState};

const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        has_one = update_authority @ ErrorCode::InvalidUpdateAuthority
    )]
    pub collection: Account<'info, BaseCollectionV1>,

    /// CHECK: Collection update authority PDA.
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump,
    )]
    pub update_authority: UncheckedAccount<'info>,

    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        seeds = [b"stake_state", asset.key().as_ref()],
        bump = stake_state.bump,
        has_one = owner @ ErrorCode::InvalidOwner,
    )]
    pub stake_state: Account<'info, StakeState>,

    /// CHECK: Staked Metaplex Core asset.
    pub asset: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [b"rewards_mint", config.key().as_ref()],
        bump = config.rewards_bump,
    )]
    pub rewards_mint: Account<'info, Mint>,

    #[account(
        init_if_needed,
        payer = owner,
        associated_token::mint = rewards_mint,
        associated_token::authority = owner,
    )]
    pub user_rewards_ata: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<ClaimRewards>) -> Result<()> {
    let attributes = fetch_plugin::<BaseAssetV1, Attributes>(
        &ctx.accounts.asset.to_account_info(),
        PluginType::Attributes,
    )
    .ok()
    .map(|(_, attrs, _)| attrs)
    .ok_or(ErrorCode::AssetNotStaked)?;

    let mut is_staked = false;

    for attribute in attributes.attribute_list {
        if attribute.key == "staked" {
            is_staked = attribute.value == "true";
        }
    }

    require!(is_staked, ErrorCode::AssetNotStaked);

    let now = Clock::get()?.unix_timestamp;

    let elapsed = now
        .checked_sub(ctx.accounts.stake_state.last_claimed_at)
        .ok_or(ErrorCode::InvalidTimestamp)?;

    let days = elapsed
        .checked_div(SECONDS_PER_DAY)
        .ok_or(ErrorCode::InvalidTimestamp)?;

    let amount = (days as u64)
        .checked_mul(ctx.accounts.config.rewards_bps as u64)
        .ok_or(ErrorCode::InvalidRewardsBps)?
        .checked_mul(10u64.pow(ctx.accounts.rewards_mint.decimals as u32))
        .ok_or(ErrorCode::InvalidRewardsBps)?
        .checked_div(10_000)
        .ok_or(ErrorCode::InvalidRewardsBps)?;

    let collection_key = ctx.accounts.collection.key();

    let config_seeds = &[
        b"config",
        collection_key.as_ref(),
        &[ctx.accounts.config.bump],
    ];

    mint_to_checked(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintToChecked {
                mint: ctx.accounts.rewards_mint.to_account_info(),
                to: ctx.accounts.user_rewards_ata.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            &[&config_seeds[..]],
        ),
        amount,
        ctx.accounts.rewards_mint.decimals,
    )?;

    ctx.accounts.stake_state.last_claimed_at = now;

    Ok(())
}