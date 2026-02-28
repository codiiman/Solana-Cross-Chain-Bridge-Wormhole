use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount};
use crate::state::*;
use crate::errors::BridgeError;
use crate::utils::{calculate_transfer_id, calculate_bridge_fee, TokenHelpers, ValidationHelpers};

/// Lock tokens on source chain (Solana) for cross-chain transfer
/// Transfers tokens to bridge custody and creates a transfer record
pub fn lock_tokens(
    ctx: Context<LockTokens>,
    amount: u64,
    target_chain: u16,
    recipient: [u8; 32],
) -> Result<()> {
    let bridge_state = &mut ctx.accounts.bridge_state;
    let clock = Clock::get()?;
    
    // Validate bridge is active
    ValidationHelpers::check_bridge_active(bridge_state.paused)?;
    
    // Validate transfer amount
    ValidationHelpers::validate_transfer_amount(
        amount,
        bridge_state.min_transfer_amount,
        bridge_state.max_transfer_amount,
    )?;
    
    // Validate target chain
    crate::utils::validate_chain_id(target_chain)?;
    require!(
        target_chain != crate::state::chain_ids::SOLANA,
        BridgeError::InvalidTargetChainAddress
    );
    
    // Calculate bridge fee
    let bridge_fee = calculate_bridge_fee(
        amount,
        bridge_state.min_bridge_fee,
        bridge_state.max_bridge_fee,
        10, // 0.1% fee (1000 bps)
    )?;
    
    let transfer_amount = amount
        .checked_sub(bridge_fee)
        .ok_or(BridgeError::MathUnderflow)?;
    
    require!(transfer_amount > 0, BridgeError::TransferAmountTooSmall);
    
    // Increment sequence number
    let sequence = bridge_state.sequence;
    bridge_state.sequence = bridge_state.sequence
        .checked_add(1)
        .ok_or(BridgeError::MathOverflow)?;
    
    // Calculate transfer ID
    let transfer_id = calculate_transfer_id(
        crate::state::chain_ids::SOLANA,
        target_chain,
        &ctx.accounts.token_mint.key(),
        transfer_amount,
        &recipient,
        sequence,
    )?;
    
    // Lock tokens (transfer to bridge custody)
    TokenHelpers::lock_tokens(
        ctx.accounts.user_token_account.to_account_info(),
        ctx.accounts.bridge_token_account.to_account_info(),
        ctx.accounts.user_authority.to_account_info(),
        amount,
        ctx.accounts.token_program.to_account_info(),
    )?;
    
    // Transfer fee to fee recipient
    if bridge_fee > 0 {
        let seeds = &[
            b"bridge_state",
            &[bridge_state.bump],
        ];
        let signer = &[&seeds[..]];
        
        TokenHelpers::transfer_tokens(
            ctx.accounts.bridge_token_account.to_account_info(),
            ctx.accounts.fee_token_account.to_account_info(),
            ctx.accounts.bridge_state.to_account_info(),
            bridge_fee,
            ctx.accounts.token_program.to_account_info(),
            Some(signer),
        )?;
    }
    
    // Create transfer account
    let transfer = &mut ctx.accounts.transfer;
    transfer.transfer_id = transfer_id;
    transfer.source_chain = crate::state::chain_ids::SOLANA;
    transfer.target_chain = target_chain;
    transfer.token_mint = ctx.accounts.token_mint.key();
    transfer.amount = transfer_amount;
    transfer.recipient = recipient;
    transfer.fee = bridge_fee;
    transfer.sequence = sequence;
    transfer.vaa_hash = None;
    transfer.zk_proof_id = None;
    transfer.status = crate::state::transfer_status::PENDING;
    transfer.created_at = clock.unix_timestamp;
    transfer.completed_at = None;
    transfer.bump = ctx.bumps.transfer;
    
    emit!(crate::events::TokensLocked {
        transfer_id,
        source_chain: crate::state::chain_ids::SOLANA,
        target_chain,
        token_mint: ctx.accounts.token_mint.key(),
        amount: transfer_amount,
        recipient,
        fee: bridge_fee,
        sequence,
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Locked {} tokens for transfer to chain {} (fee: {})", 
         transfer_amount, target_chain, bridge_fee);
    Ok(())
}

#[derive(Accounts)]
#[instruction(amount: u64, target_chain: u16, recipient: [u8; 32])]
pub struct LockTokens<'info> {
    #[account(
        mut,
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    #[account(
        mut,
        constraint = user_token_account.mint == token_mint.key()
    )]
    pub user_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = bridge_token_account.mint == token_mint.key()
    )]
    pub bridge_token_account: Account<'info, TokenAccount>,
    
    #[account(
        mut,
        constraint = fee_token_account.mint == token_mint.key(),
        constraint = fee_token_account.owner == bridge_state.fee_recipient
    )]
    pub fee_token_account: Account<'info, TokenAccount>,
    
    /// CHECK: Token mint
    pub token_mint: UncheckedAccount<'info>,
    
    #[account(
        init,
        payer = user_authority,
        space = TransferAccount::LEN,
        seeds = [b"transfer", bridge_state.key().as_ref(), bridge_state.sequence.to_le_bytes().as_ref()],
        bump
    )]
    pub transfer: Account<'info, TransferAccount>,
    
    pub user_authority: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}
