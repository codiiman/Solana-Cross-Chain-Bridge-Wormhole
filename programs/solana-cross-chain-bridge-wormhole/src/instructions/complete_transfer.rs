use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};
use crate::state::*;
use crate::errors::BridgeError;
use crate::vaa::{VaaParser, TransferPayloadParser};
use crate::utils::{TokenHelpers, address_to_pubkey, ValidationHelpers};

/// Complete a cross-chain transfer by minting wrapped tokens
/// Requires a valid VAA with verified guardian signatures
pub fn complete_transfer(
    ctx: Context<CompleteTransfer>,
) -> Result<()> {
    let bridge_state = &ctx.accounts.bridge_state;
    let vaa_account = &mut ctx.accounts.vaa_account;
    let transfer = &mut ctx.accounts.transfer;
    let clock = Clock::get()?;
    
    // Validate bridge is active
    ValidationHelpers::check_bridge_active(bridge_state.paused)?;
    
    // Check VAA not already processed
    require!(!vaa_account.processed, BridgeError::VaaAlreadyProcessed);
    
    // Check transfer not already completed
    require!(
        transfer.status == crate::state::transfer_status::PENDING,
        BridgeError::TransferAlreadyCompleted
    );
    
    // Parse transfer payload from VAA
    let transfer_payload = TransferPayloadParser::parse_transfer_payload(&vaa_account.payload)?;
    
    // Validate transfer matches VAA
    require!(
        transfer_payload.amount == transfer.amount,
        BridgeError::InvalidVaaFormat
    );
    
    require!(
        transfer_payload.recipient_chain == transfer.target_chain,
        BridgeError::ChainIdMismatch
    );
    
    // Find or create wrapped asset
    // In production, would look up wrapped asset by source_chain + source_token
    let wrapped_asset = &ctx.accounts.wrapped_asset;
    
    // Convert recipient address to Pubkey
    let recipient_pubkey = address_to_pubkey(&transfer_payload.recipient)?;
    
    // Calculate relayer reward
    let relayer_reward = if transfer.fee > 0 {
        crate::utils::calculate_relayer_reward(
            transfer.fee,
            bridge_state.relayer_reward_bps,
        )?
    } else {
        0
    };
    
    // Mint wrapped tokens to recipient
    let seeds = &[
        b"wrapped_asset",
        &transfer_payload.token_chain.to_le_bytes(),
        &transfer_payload.token_address,
        &[wrapped_asset.bump],
    ];
    let signer = &[&seeds[..]];
    
    TokenHelpers::mint_wrapped_tokens(
        ctx.accounts.wrapped_mint.to_account_info(),
        ctx.accounts.recipient_token_account.to_account_info(),
        ctx.accounts.wrapped_asset.to_account_info(),
        transfer_payload.amount,
        ctx.accounts.token_program.to_account_info(),
        signer,
    )?;
    
    // Pay relayer reward if applicable
    if relayer_reward > 0 {
        // In production, would transfer from fee account to relayer
        // For now, just emit event
    }
    
    // Update transfer status
    transfer.status = crate::state::transfer_status::COMPLETED;
    transfer.completed_at = Some(clock.unix_timestamp);
    transfer.vaa_hash = Some(vaa_account.vaa_hash);
    
    // Mark VAA as processed
    vaa_account.processed = true;
    
    // Update wrapped asset total supply
    let wrapped_asset = &mut ctx.accounts.wrapped_asset;
    wrapped_asset.total_supply = wrapped_asset.total_supply
        .checked_add(transfer_payload.amount)
        .ok_or(BridgeError::MathOverflow)?;
    
    emit!(crate::events::TokensMinted {
        transfer_id: transfer.transfer_id,
        source_chain: transfer.source_chain,
        token_mint: transfer.token_mint,
        wrapped_mint: ctx.accounts.wrapped_mint.key(),
        amount: transfer_payload.amount,
        recipient: recipient_pubkey,
        vaa_hash: vaa_account.vaa_hash,
        timestamp: clock.unix_timestamp,
    });
    
    emit!(crate::events::TransferCompleted {
        transfer_id: transfer.transfer_id,
        source_chain: transfer.source_chain,
        target_chain: transfer.target_chain,
        token_mint: transfer.token_mint,
        amount: transfer_payload.amount,
        recipient: recipient_pubkey,
        relayer: ctx.accounts.relayer.key(),
        relayer_fee: relayer_reward,
        vaa_hash: vaa_account.vaa_hash,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Transfer completed: {} tokens minted to {}", 
         transfer_payload.amount, recipient_pubkey);
    Ok(())
}

#[derive(Accounts)]
pub struct CompleteTransfer<'info> {
    #[account(
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    #[account(
        mut,
        seeds = [b"vaa", bridge_state.key().as_ref(), vaa_account.vaa_hash.as_ref()],
        bump = vaa_account.bump
    )]
    pub vaa_account: Account<'info, VaaAccount>,
    
    #[account(
        mut,
        seeds = [b"transfer", bridge_state.key().as_ref(), transfer.sequence.to_le_bytes().as_ref()],
        bump = transfer.bump
    )]
    pub transfer: Account<'info, TransferAccount>,
    
    #[account(
        mut,
        seeds = [b"wrapped_asset", transfer.token_mint.as_ref()],
        bump = wrapped_asset.bump
    )]
    pub wrapped_asset: Account<'info, WrappedAsset>,
    
    #[account(
        mut,
        constraint = wrapped_mint.key() == wrapped_asset.wrapped_mint
    )]
    pub wrapped_mint: Account<'info, Mint>,
    
    #[account(
        mut,
        constraint = recipient_token_account.mint == wrapped_mint.key()
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,
    
    pub relayer: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}
