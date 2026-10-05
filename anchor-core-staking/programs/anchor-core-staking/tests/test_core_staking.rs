use anchor_lang::{Id, InstructionData, ToAccountMetas};
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::ID as TOKEN_PROGRAM_ID,
};
use mpl_core::ID as MPL_CORE_ID;

use anchor_core_staking::{
    accounts::{
        BurnStakedNft,
        ClaimRewards,
        CreateCollection,
        Initialize,
        MintAsset,
        Stake,
        Unstake,
    },
    instruction,
    ID as PROGRAM_ID,
};

#[test]
fn test_pda_derivations() {
    let collection = Pubkey::new_unique();

    let (config_pda, config_bump) =
        Pubkey::find_program_address(
            &[b"config", collection.as_ref()],
            &PROGRAM_ID,
        );

    let (update_authority_pda, update_authority_bump) =
        Pubkey::find_program_address(
            &[b"update_authority", collection.as_ref()],
            &PROGRAM_ID,
        );

    let (rewards_mint_pda, rewards_bump) =
        Pubkey::find_program_address(
            &[b"rewards_mint", config_pda.as_ref()],
            &PROGRAM_ID,
        );

    let asset = Pubkey::new_unique();

    let (stake_state_pda, stake_state_bump) =
        Pubkey::find_program_address(
            &[b"stake_state", asset.as_ref()],
            &PROGRAM_ID,
        );

    assert_ne!(config_pda, Pubkey::default());
    assert_ne!(update_authority_pda, Pubkey::default());
    assert_ne!(rewards_mint_pda, Pubkey::default());
    assert_ne!(stake_state_pda, Pubkey::default());

    println!("Config PDA: {} (bump {})", config_pda, config_bump);
    println!(
        "Update Authority PDA: {} (bump {})",
        update_authority_pda,
        update_authority_bump
    );
    println!(
        "Rewards Mint PDA: {} (bump {})",
        rewards_mint_pda,
        rewards_bump
    );
    println!(
        "Stake State PDA: {} (bump {})",
        stake_state_pda,
        stake_state_bump
    );

    assert!(config_bump <= 255);
    assert!(update_authority_bump <= 255);
    assert!(rewards_bump <= 255);
    assert!(stake_state_bump <= 255);
}

#[test]
fn test_instruction_data_encoding() {
    let initialize_data = instruction::Initialize {
        rewards_bps: 500,
        freeze_period: 7,
    }
    .data();

    assert!(!initialize_data.is_empty());

    let create_collection_data = instruction::CreateCollection {
        name: "Test Collection".to_string(),
        uri: "https://example.com/collection.json".to_string(),
    }
    .data();

    assert!(!create_collection_data.is_empty());

    let mint_asset_data = instruction::MintAsset {
        name: "NFT #1".to_string(),
        uri: "https://example.com/1.json".to_string(),
    }
    .data();

    assert!(!mint_asset_data.is_empty());

    let stake_data = instruction::Stake {}.data();
    assert!(!stake_data.is_empty());

    let unstake_data = instruction::Unstake {}.data();
    assert!(!unstake_data.is_empty());

    let claim_rewards_data = instruction::ClaimRewards {}.data();
    assert!(!claim_rewards_data.is_empty());

    let burn_staked_nft_data =
        instruction::BurnStakedNft {}.data();

    assert!(!burn_staked_nft_data.is_empty());
}

#[test]
fn test_burn_to_earn_rewards_calculation() {
    let decimals = 6u32;
    let rewards_bps = 500u64;
    let burn_bonus_multiplier = 5u64;
    let seconds_per_day = 86_400u64;

    let staked_duration_seconds =
        10 * seconds_per_day;

    let staked_days =
        (staked_duration_seconds / seconds_per_day).max(1);

    let base_reward = staked_days
        .checked_mul(rewards_bps)
        .unwrap()
        .checked_mul(10u64.pow(decimals))
        .unwrap()
        .checked_div(10_000)
        .unwrap();

    let total_burn_payout = base_reward
        .checked_mul(burn_bonus_multiplier)
        .unwrap();

    assert_eq!(base_reward, 500_000);
    assert_eq!(total_burn_payout, 2_500_000);
}

#[test]
fn test_collection_level_counter_logic() {
    let mut total_staked: u64 = 0;

    total_staked = total_staked.saturating_add(1);
    assert_eq!(total_staked, 1);

    total_staked = total_staked.saturating_add(1);
    assert_eq!(total_staked, 2);

    total_staked = total_staked.saturating_add(1);
    assert_eq!(total_staked, 3);

    total_staked = total_staked.saturating_sub(1);
    assert_eq!(total_staked, 2);

    total_staked = total_staked.saturating_sub(1);
    assert_eq!(total_staked, 1);

    total_staked = total_staked.saturating_sub(1);
    assert_eq!(total_staked, 0);

    total_staked = total_staked.saturating_sub(1);
    assert_eq!(total_staked, 0);
}

#[test]
fn test_account_metas_structure() {
    let owner = Pubkey::new_unique();
    let collection = Pubkey::new_unique();
    let asset = Pubkey::new_unique();

    let (config, _) =
        Pubkey::find_program_address(
            &[b"config", collection.as_ref()],
            &PROGRAM_ID,
        );

    let (update_authority, _) =
        Pubkey::find_program_address(
            &[b"update_authority", collection.as_ref()],
            &PROGRAM_ID,
        );

    let (rewards_mint, _) =
        Pubkey::find_program_address(
            &[b"rewards_mint", config.as_ref()],
            &PROGRAM_ID,
        );

    let (stake_state, _) =
        Pubkey::find_program_address(
            &[b"stake_state", asset.as_ref()],
            &PROGRAM_ID,
        );

    let user_rewards_ata = Pubkey::new_unique();

    let initialize_accounts = Initialize {
        admin: owner,
        config,
        collection,
        update_authority,
        rewards_mint,
        system_program: anchor_lang::system_program::System::id(),
        token_program: TOKEN_PROGRAM_ID,
        mpl_core_program: MPL_CORE_ID,
    };

    let initialize_metas =
        initialize_accounts.to_account_metas(None);

    assert_eq!(initialize_metas.len(), 8);

    let create_collection_accounts = CreateCollection {
        payer: owner,
        collection,
        update_authority,
        system_program: anchor_lang::system_program::System::id(),
        mpl_core_program: MPL_CORE_ID,
    };

    let create_collection_metas =
        create_collection_accounts.to_account_metas(None);

    assert_eq!(create_collection_metas.len(), 5);

    let mint_asset_accounts = MintAsset {
        user: owner,
        asset,
        collection,
        update_authority,
        system_program: anchor_lang::system_program::System::id(),
        mpl_core_program: MPL_CORE_ID,
    };

    let mint_asset_metas =
        mint_asset_accounts.to_account_metas(None);

    assert_eq!(mint_asset_metas.len(), 6);

    let stake_accounts = Stake {
        owner,
        config,
        asset,
        collection,
        update_authority,
        stake_state,
        system_program: anchor_lang::system_program::System::id(),
        mpl_core_program: MPL_CORE_ID,
    };

    let stake_metas =
        stake_accounts.to_account_metas(None);

    assert_eq!(stake_metas.len(), 8);

    let unstake_accounts = Unstake {
        owner,
        config,
        asset,
        collection,
        update_authority,
        stake_state,
        rewards_mint,
        user_rewards_ata,
        token_program: TOKEN_PROGRAM_ID,
        associated_token_program: AssociatedToken::id(),
        system_program: anchor_lang::system_program::System::id(),
        mpl_core_program: MPL_CORE_ID,
    };

    let unstake_metas =
        unstake_accounts.to_account_metas(None);

    assert_eq!(unstake_metas.len(), 12);

    let claim_rewards_accounts = ClaimRewards {
        owner,
        collection,
        update_authority,
        config,
        stake_state,
        asset,
        rewards_mint,
        user_rewards_ata,
        token_program: TOKEN_PROGRAM_ID,
        associated_token_program: AssociatedToken::id(),
        system_program: anchor_lang::system_program::System::id(),
    };

    let claim_rewards_metas =
        claim_rewards_accounts.to_account_metas(None);

    assert_eq!(claim_rewards_metas.len(), 11);

    let burn_accounts = BurnStakedNft {
        owner,
        config,
        asset,
        collection,
        update_authority,
        rewards_mint,
        user_rewards_ata,
        token_program: TOKEN_PROGRAM_ID,
        associated_token_program: AssociatedToken::id(),
        system_program: anchor_lang::system_program::System::id(),
        mpl_core_program: MPL_CORE_ID,
    };

    let burn_metas =
        burn_accounts.to_account_metas(None);

    assert_eq!(burn_metas.len(), 11);

    assert!(burn_metas[0].is_signer);
    assert!(burn_metas[0].is_writable);
    assert!(burn_metas[2].is_writable);
    assert!(burn_metas[3].is_writable);
}