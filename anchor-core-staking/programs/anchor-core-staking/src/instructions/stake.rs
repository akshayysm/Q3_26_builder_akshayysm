use anchor_lang::prelude::*;
use mpl_core::{
    accounts::{BaseAssetV1, BaseCollectionV1},
    fetch_plugin,
    instructions::{
        AddCollectionPluginV1CpiBuilder,
        AddPluginV1CpiBuilder,
        UpdateCollectionPluginV1CpiBuilder,
        UpdatePluginV1CpiBuilder,
    },
    types::{
        Attribute,
        Attributes,
        FreezeDelegate,
        Plugin,
        PluginAuthority,
        PluginType,
        UpdateAuthority,
    },
    ID as MPL_CORE_ID,
};

use crate::error::ErrorCode;
use crate::state::{Config, StakeState};

#[derive(Accounts)]
pub struct Stake<'info> {
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
        init,
        payer = owner,
        space = StakeState::DISCRIMINATOR.len() + StakeState::INIT_SPACE,
        seeds = [b"stake_state", asset.key().as_ref()],
        bump,
    )]
    pub stake_state: Account<'info, StakeState>,

    pub system_program: Program<'info, System>,

    /// CHECK: Metaplex Core program.
    #[account(address = MPL_CORE_ID)]
    pub mpl_core_program: UncheckedAccount<'info>,
}

pub fn handler(ctx: Context<Stake>) -> Result<()> {
    let attributes_fetched =
        fetch_plugin::<BaseAssetV1, Attributes>(
            &ctx.accounts.asset.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_, attrs, _)| attrs);

    let mut attributes_list = Vec::new();

    if let Some(attributes) = &attributes_fetched {
        for attribute in &attributes.attribute_list {
            if attribute.key == "staked" {
                require!(
                    attribute.value == "false",
                    ErrorCode::AlreadyStaked
                );
            } else if attribute.key != "staked_at" {
                attributes_list.push(attribute.clone());
            }
        }
    }

    let current_timestamp = Clock::get()?.unix_timestamp;

    attributes_list.push(Attribute {
        key: "staked".to_string(),
        value: "true".to_string(),
    });

    attributes_list.push(Attribute {
        key: "staked_at".to_string(),
        value: current_timestamp.to_string(),
    });

    let collection_key = ctx.accounts.collection.key();

    let signer_seeds = &[
        b"update_authority",
        collection_key.as_ref(),
        &[ctx.bumps.update_authority],
    ];

    if attributes_fetched.is_none() {
        AddPluginV1CpiBuilder::new(
            &ctx.accounts.mpl_core_program.to_account_info(),
        )
        .asset(&ctx.accounts.asset.to_account_info())
        .collection(Some(&ctx.accounts.collection.to_account_info()))
        .payer(&ctx.accounts.owner.to_account_info())
        .authority(Some(
            &ctx.accounts.update_authority.to_account_info(),
        ))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes {
            attribute_list: attributes_list,
        }))
        .init_authority(PluginAuthority::UpdateAuthority)
        .invoke_signed(&[signer_seeds])?;
    } else {
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
        .plugin(Plugin::Attributes(Attributes {
            attribute_list: attributes_list,
        }))
        .invoke_signed(&[signer_seeds])?;
    }

    AddPluginV1CpiBuilder::new(
        &ctx.accounts.mpl_core_program.to_account_info(),
    )
    .asset(&ctx.accounts.asset.to_account_info())
    .collection(Some(&ctx.accounts.collection.to_account_info()))
    .payer(&ctx.accounts.owner.to_account_info())
    .authority(Some(&ctx.accounts.owner.to_account_info()))
    .system_program(&ctx.accounts.system_program.to_account_info())
    .plugin(Plugin::FreezeDelegate(FreezeDelegate { frozen: true }))
    .init_authority(PluginAuthority::UpdateAuthority)
    .invoke()?;

    let collection_attributes_fetched =
        fetch_plugin::<BaseCollectionV1, Attributes>(
            &ctx.accounts.collection.to_account_info(),
            PluginType::Attributes,
        )
        .ok()
        .map(|(_, attrs, _)| attrs);

    let had_collection_attributes = collection_attributes_fetched.is_some();

    let mut total_staked = 1u64;
    let mut collection_attributes = Vec::new();

    if let Some(attributes) = collection_attributes_fetched {
        for attribute in attributes.attribute_list {
            if attribute.key == "total_staked" {
                total_staked = attribute
                    .value
                    .parse::<u64>()
                    .unwrap_or(0)
                    .saturating_add(1);
            } else {
                collection_attributes.push(attribute);
            }
        }
    }

    collection_attributes.push(Attribute {
        key: "total_staked".to_string(),
        value: total_staked.to_string(),
    });

    if had_collection_attributes {
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
            attribute_list: collection_attributes,
        }))
        .invoke_signed(&[signer_seeds])?;
    } else {
        AddCollectionPluginV1CpiBuilder::new(
            &ctx.accounts.mpl_core_program.to_account_info(),
        )
        .collection(&ctx.accounts.collection.to_account_info())
        .payer(&ctx.accounts.owner.to_account_info())
        .authority(Some(
            &ctx.accounts.update_authority.to_account_info(),
        ))
        .system_program(&ctx.accounts.system_program.to_account_info())
        .plugin(Plugin::Attributes(Attributes {
            attribute_list: collection_attributes,
        }))
        .init_authority(PluginAuthority::UpdateAuthority)
        .invoke_signed(&[signer_seeds])?;
    }

    ctx.accounts.stake_state.set_inner(StakeState {
        owner: ctx.accounts.owner.key(),
        last_claimed_at: current_timestamp,
        bump: ctx.bumps.stake_state,
    });

    Ok(())
}