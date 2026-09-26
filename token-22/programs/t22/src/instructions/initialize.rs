use anchor_lang::prelude::*;
use anchor_spl::{
    token_2022::spl_token_2022::{
        extension::ExtensionType,
        state::{AccountState, Mint as MintState},
    },
    token_interface::{
        default_account_state_initialize,
        initialize_mint2,
        metadata_pointer_initialize,
        mint_close_authority_initialize,
        token_metadata_initialize,
        transfer_fee_initialize,
        DefaultAccountStateInitialize,
        InitializeMint2,
        MetadataPointerInitialize,
        MintCloseAuthorityInitialize,
        TokenInterface,
        TokenMetadataInitialize,
        TransferFeeInitialize,
    },
};
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;

use crate::{
    DECIMALS,
    MAXIMUM_FEE,
    TOKEN_NAME,
    TOKEN_SYMBOL,
    TOKEN_URI,
    TRANSFER_FEE_BPS,
};

const TLV_HEADER_LEN: usize = 4;

#[derive(Accounts)]
pub struct InitializeMint<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub mint: Signer<'info>,

    pub system_program: Program<'info, System>,
    pub token_program: Interface<'info, TokenInterface>,
}

impl<'info> InitializeMint<'info> {
    pub fn initialize(&mut self) -> Result<()> {
        let extensions = [
            ExtensionType::TransferFeeConfig,
            ExtensionType::MintCloseAuthority,
            ExtensionType::DefaultAccountState,
            ExtensionType::MetadataPointer,
        ];

        let metadata = TokenMetadata {
            update_authority:
                spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(
                    Some(self.payer.key()),
                )
                .unwrap(),
            mint: self.mint.key(),
            name: TOKEN_NAME.to_string(),
            symbol: TOKEN_SYMBOL.to_string(),
            uri: TOKEN_URI.to_string(),
            additional_metadata: vec![],
        };

        let space =
            ExtensionType::try_calculate_account_len::<MintState>(&extensions)?;

        let metadata_size =
            TLV_HEADER_LEN + metadata.get_packed_len()?;

        let final_size = space + metadata_size;

        let initial_lamports = Rent::get()?.minimum_balance(space);

        anchor_lang::system_program::create_account(
            CpiContext::new(
                self.system_program.key(),
                anchor_lang::system_program::CreateAccount {
                    from: self.payer.to_account_info(),
                    to: self.mint.to_account_info(),
                },
            ),
            initial_lamports,
            space as u64,
            &self.token_program.key(),
        )?;

        self.transfer_fee_config()?;
        self.mint_close_config()?;
        self.default_state_config()?;
        self.metadata_pointer_config()?;

        initialize_mint2(
            CpiContext::new(
                self.token_program.key(),
                InitializeMint2 {
                    mint: self.mint.to_account_info(),
                },
            ),
            DECIMALS,
            &self.payer.key(),
            Some(&self.payer.key()),
        )?;

        let final_lamports = Rent::get()?.minimum_balance(final_size);
        let additional_lamports =
            final_lamports.saturating_sub(self.mint.to_account_info().lamports());

        if additional_lamports > 0 {
            anchor_lang::system_program::transfer(
                CpiContext::new(
                    self.system_program.key(),
                    anchor_lang::system_program::Transfer {
                        from: self.payer.to_account_info(),
                        to: self.mint.to_account_info(),
                    },
                ),
                additional_lamports,
            )?;
        }

        self.token_metadata_config()?;

        Ok(())
    }

    fn transfer_fee_config(&self) -> Result<()> {
        transfer_fee_initialize(
            CpiContext::new(
                self.token_program.key(),
                TransferFeeInitialize {
                    mint: self.mint.to_account_info(),
                    token_program_id: self.token_program.to_account_info(),
                },
            ),
            Some(&self.payer.key()),
            Some(&self.payer.key()),
            TRANSFER_FEE_BPS,
            MAXIMUM_FEE,
        )
    }

    fn mint_close_config(&self) -> Result<()> {
        mint_close_authority_initialize(
            CpiContext::new(
                self.token_program.key(),
                MintCloseAuthorityInitialize {
                    mint: self.mint.to_account_info(),
                    token_program_id: self.token_program.to_account_info(),
                },
            ),
            Some(&self.payer.key()),
        )
    }

    fn default_state_config(&self) -> Result<()> {
        default_account_state_initialize(
            CpiContext::new(
                self.token_program.key(),
                DefaultAccountStateInitialize {
                    mint: self.mint.to_account_info(),
                    token_program_id: self.token_program.to_account_info(),
                },
            ),
            &AccountState::Frozen,
        )
    }

    fn metadata_pointer_config(&self) -> Result<()> {
        metadata_pointer_initialize(
            CpiContext::new(
                self.token_program.key(),
                MetadataPointerInitialize {
                    mint: self.mint.to_account_info(),
                    token_program_id: self.token_program.to_account_info(),
                },
            ),
            Some(self.payer.key()),
            Some(self.mint.key()),
        )
    }

    fn token_metadata_config(&self) -> Result<()> {
        token_metadata_initialize(
            CpiContext::new(
                self.token_program.key(),
                TokenMetadataInitialize {
                    mint: self.mint.to_account_info(),
                    mint_authority: self.payer.to_account_info(),
                    update_authority: self.payer.to_account_info(),
                    program_id: self.token_program.to_account_info(),
                    metadata: self.mint.to_account_info(),
                },
            ),
            TOKEN_NAME.to_string(),
            TOKEN_SYMBOL.to_string(),
            TOKEN_URI.to_string(),
        )
    }
}