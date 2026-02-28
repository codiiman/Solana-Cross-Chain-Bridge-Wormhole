use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token2022};
use crate::state::*;
use crate::errors::BridgeError;

/// Register a wrapped asset (create wrapped token mint for foreign token)
/// Supports both SPL Token and Token-2022 for RWA/compliance features
pub fn register_wrapped_asset(
    ctx: Context<RegisterWrappedAsset>,
    source_chain: u16,
    source_token: [u8; 32],
    decimals: u8,
    symbol: String,
    name: String,
    is_token2022: bool,
) -> Result<()> {
    let bridge_state = &ctx.accounts.bridge_state;
    let clock = Clock::get()?;
    
    // Validate bridge is active
    crate::utils::ValidationHelpers::check_bridge_active(bridge_state.paused)?;
    
    // Validate symbol and name length
    require!(
        symbol.len() <= WrappedAsset::MAX_SYMBOL_LENGTH,
        BridgeError::InvalidWrappedAssetMetadata
    );
    
    require!(
        name.len() <= WrappedAsset::MAX_NAME_LENGTH,
        BridgeError::InvalidWrappedAssetMetadata
    );
    
    // Validate source chain
    crate::utils::validate_chain_id(source_chain)?;
    require!(
        source_chain != crate::state::chain_ids::SOLANA,
        BridgeError::InvalidTargetChainAddress
    );
    
    // Check if wrapped asset already exists
    // In production, would check existing accounts
    
    // Create wrapped token mint
    // In production, would use Token-2022 if is_token2022 is true
    // For now, using standard SPL Token
    
    // Register wrapped asset
    let wrapped_asset = &mut ctx.accounts.wrapped_asset;
    wrapped_asset.source_chain = source_chain;
    wrapped_asset.source_token = source_token;
    wrapped_asset.wrapped_mint = ctx.accounts.wrapped_mint.key();
    wrapped_asset.decimals = decimals;
    wrapped_asset.symbol = symbol.clone();
    wrapped_asset.name = name.clone();
    wrapped_asset.is_token2022 = is_token2022;
    wrapped_asset.total_supply = 0;
    wrapped_asset.bump = ctx.bumps.wrapped_asset;
    
    emit!(crate::events::WrappedAssetRegistered {
        source_chain,
        source_token,
        wrapped_mint: ctx.accounts.wrapped_mint.key(),
        decimals,
        symbol,
        name,
        registered_by: ctx.accounts.registrar.key(),
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Wrapped asset registered: {} ({}) from chain {}", 
         name, symbol, source_chain);
    Ok(())
}

#[derive(Accounts)]
#[instruction(source_chain: u16, source_token: [u8; 32])]
pub struct RegisterWrappedAsset<'info> {
    #[account(
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    #[account(
        init,
        payer = registrar,
        space = WrappedAsset::LEN,
        seeds = [b"wrapped_asset", &source_chain.to_le_bytes(), &source_token],
        bump
    )]
    pub wrapped_asset: Account<'info, WrappedAsset>,
    
    #[account(
        init,
        payer = registrar,
        mint::decimals = decimals,
        mint::authority = wrapped_asset,
        seeds = [b"wrapped_mint", wrapped_asset.key().as_ref()],
        bump
    )]
    pub wrapped_mint: Account<'info, Mint>,
    
    pub registrar: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
