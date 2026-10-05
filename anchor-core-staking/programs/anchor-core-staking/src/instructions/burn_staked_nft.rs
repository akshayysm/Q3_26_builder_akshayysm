use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
    token_interface::{mint_to_checked, MintToChecked},
};
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{
        BurnV1CpiBuilder,
        UpdateCollectionPluginV1CpiBuilder,
        UpdatePluginV1CpiBuilder,
    },
    types::{
        Attribute,
        Attributes,
        FreezeDelegate,
        Plugin,
        PluginType,
        UpdateAuthority,
    },
    ID as MPL_CORE_ID,
};

use crate::error::ErrorCode;
use crate::state::Config;

const SECONDS_PER_DAY: i64 = 86_400;
const BURN_BONUS_MULTIPLIER: u64 = 5;

#[derive(Accounts)]
pub struct BurnStakedNft<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        seeds = [b"config", collection.key().as_ref()],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    #[account(
        mut,
        has_one = owner @ ErrorCode::InvalidOwner,
        constraint = asset.update_authority
            == UpdateAuthority::Collection(collection.key())
            @ ErrorCode::InvalidUpdateAuthority,
    )]
    pub asset: Account<'info, BaseAssetV1>,

    #[account(
        mut,
        has_one = update_authority @ ErrorCode::InvalidUpdateAuthority,
    )]
    pub collection: Account<'info, BaseCollectionV1>,

    /// CHECK: PDA used as the collection update authority.
    #[account(
        seeds = [b"update_authority", collection.key().as_ref()],
        bump,
    )]
    pub update_authority: UncheckedAccount<'info>,

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

    /// CHECK: Metaplex Core program.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<BurnStakedNft>) -> Result<()> {
    let attributes = fetch_plugin::<BaseAssetV1, Attributes>(
        &ctx.accounts.asset.to_account_info(),
        PluginType::Attributes,
    )
    .ok()
    .map(|(_, attrs, _)| attrs)
    .ok_or(ErrorCode::AssetNotStaked)?;

    let now = Clock::get()?.unix_timestamp;
    let mut staked_at = None;

    for attribute in &attributes.attribute_list {
        if attribute.key == "staked" {
            require!(
                attribute.value == "true",
                ErrorCode::AssetNotStaked
            );
        }

        if attribute.key == "staked_at" {
            staked_at = Some(
                attribute
                    .value
                    .parse::<i64>()
                    .map_err(|_| ErrorCode::InvalidTimestamp)?,
            );
        }
    }

    let staked_at = staked_at.ok_or(ErrorCode::InvalidTimestamp)?;

    let elapsed = now
        .checked_sub(staked_at)
        .ok_or(ErrorCode::InvalidTimestamp)?;

    let days = (elapsed / SECONDS_PER_DAY).max(1) as u64;

    let base_reward = days
        .checked_mul(ctx.accounts.config.rewards_bps as u64)
        .ok_or(ErrorCode::InvalidRewardsBps)?
        .checked_mul(10u64.pow(ctx.accounts.rewards_mint.decimals as u32))
        .ok_or(ErrorCode::InvalidRewardsBps)?
        .checked_div(10_000)
        .ok_or(ErrorCode::InvalidRewardsBps)?;

    let burn_reward = base_reward
        .checked_mul(BURN_BONUS_MULTIPLIER)
        .ok_or(ErrorCode::InvalidRewardsBps)?;

    let collection_key = ctx.accounts.collection.key();

    let signer_seeds = &[
        b"update_authority",
        collection_key.as_ref(),
        &[ctx.bumps.update_authority],
    ];

    UpdatePluginV1CpiBuilder::new(
        &ctx.accounts.mpl_core_program.to_account_info(),
    )
    .asset(&ctx.accounts.asset.to_account_info())
    .collection(Some(&ctx.accounts.collection.to_account_info()))
    .payer(&ctx.accounts.owner.to_account_info())
    .authority(Some(
        &ctx.accounts.update_authority.to_account_info(),
    ))
    .system_program(&ctx.accounts.system_program.to_account_info())
    .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: false }))
    .invoke_signed(&[signer_seeds])?;

    BurnV1CpiBuilder::new(
        &ctx.accounts.mpl_core_program.to_account_info(),
    )
    .asset(&ctx.accounts.asset.to_account_info())
    .collection(Some(&ctx.accounts.collection.to_account_info()))
    .payer(&ctx.accounts.owner.to_account_info())
    .authority(Some(&ctx.accounts.owner.to_account_info()))
    .system_program(Some(
        &ctx.accounts.system_program.to_account_info(),
    ))
    .invoke()?;

    if let Some(collection_attributes) =
        fetch_plugin::<BaseCollectionV1, Attributes>(
            &ctx.accounts.collection.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_, attrs, _)| attrs)
    {
        let mut list = Vec::new();
        let mut total = 0u64;

        for attribute in collection_attributes.attribute_list {
            if attribute.key == "total_staked" {
                total = attribute
                    .value
                    .parse::<u64>()
                    .unwrap_or(0)
                    .saturating_sub(1);
            } else {
                list.push(attribute);
            }
        }

        list.push(Attribute {
            key: "total_staked".to_string(),
            value: total.to_string(),
        });

        UpdateCollectionPluginV1CpiBuilder::new(
            &ctx.accounts.mpl_core_program.to_account_info(),
        )
        .collection(&ctx.accounts.collection.to_account_info())
        .payer(&ctx.accounts.owner.to_account_info())
        .authority(Some(
            &ctx.accounts.update_authority.to_account_info(),
        ))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes {
            attribute_list: list,
        }))
        .invoke_signed(&[signer_seeds])?;
    }

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
        burn_reward,
        ctx.accounts.rewards_mint.decimals,
    )?;

    Ok(())
}