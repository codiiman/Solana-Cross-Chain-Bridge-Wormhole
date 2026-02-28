use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, MintTo, Burn};
use sha2::{Sha256, Digest};
use crate::errors::BridgeError;

/// Utility functions for bridge operations

/// Calculate transfer ID from transfer details
pub fn calculate_transfer_id(
    source_chain: u16,
    target_chain: u16,
    token_mint: &Pubkey,
    amount: u64,
    recipient: &[u8; 32],
    sequence: u64,
) -> Result<[u8; 32]> {
    let mut hasher = Sha256::new();
    hasher.update(b"TransferID");
    hasher.update(&source_chain.to_le_bytes());
    hasher.update(&target_chain.to_le_bytes());
    hasher.update(token_mint.as_ref());
    hasher.update(&amount.to_le_bytes());
    hasher.update(recipient);
    hasher.update(&sequence.to_le_bytes());
    
    let hash = hasher.finalize();
    let mut hash_array = [0u8; 32];
    hash_array.copy_from_slice(&hash);
    Ok(hash_array)
}

/// Calculate bridge fee
pub fn calculate_bridge_fee(
    amount: u64,
    min_fee: u64,
    max_fee: u64,
    fee_bps: u16, // Fee in basis points (10000 = 100%)
) -> Result<u64> {
    let fee = amount
        .checked_mul(fee_bps as u64)
        .ok_or(BridgeError::MathOverflow)?
        .checked_div(10000)
        .ok_or(BridgeError::DivisionByZero)?;
    
    let fee = fee.max(min_fee).min(max_fee);
    Ok(fee)
}

/// Calculate relayer reward
pub fn calculate_relayer_reward(
    bridge_fee: u64,
    reward_bps: u16,
) -> Result<u64> {
    let reward = bridge_fee
        .checked_mul(reward_bps as u64)
        .ok_or(BridgeError::MathOverflow)?
        .checked_div(10000)
        .ok_or(BridgeError::DivisionByZero)?;
    
    Ok(reward)
}

/// Convert 32-byte address to Solana Pubkey
pub fn address_to_pubkey(address: &[u8; 32]) -> Result<Pubkey> {
    Pubkey::try_from(address.as_slice())
        .map_err(|_| BridgeError::InvalidTargetChainAddress.into())
}

/// Convert Solana Pubkey to 32-byte address
pub fn pubkey_to_address(pubkey: &Pubkey) -> [u8; 32] {
    let mut address = [0u8; 32];
    address.copy_from_slice(&pubkey.to_bytes());
    address
}

/// Validate chain ID
pub fn validate_chain_id(chain_id: u16) -> Result<()> {
    // Supported chains (following Wormhole standard)
    let supported_chains = [
        1,  // Solana
        2,  // Ethereum
        3,  // Terra
        4,  // BSC
        5,  // Polygon
        6,  // Avalanche
        10, // Fantom
        14, // Celo
        15, // NEAR
        16, // Moonbeam
        23, // Arbitrum
        24, // Optimism
        30, // Base
    ];
    
    require!(
        supported_chains.contains(&chain_id),
        BridgeError::UnsupportedTargetChain
    );
    
    Ok(())
}

/// CPI helpers for token operations
pub struct TokenHelpers;

impl TokenHelpers {
    /// Lock tokens (transfer to bridge custody)
    pub fn lock_tokens<'info>(
        from: Account<'info, TokenAccount>,
        to: Account<'info, TokenAccount>,
        authority: AccountInfo<'info>,
        amount: u64,
        token_program: Program<'info, Token>,
    ) -> Result<()> {
        let cpi_accounts = token::Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: authority.to_account_info(),
        };
        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }
    
    /// Mint wrapped tokens
    pub fn mint_wrapped_tokens<'info>(
        mint: Account<'info, Mint>,
        to: Account<'info, TokenAccount>,
        authority: AccountInfo<'info>,
        amount: u64,
        token_program: Program<'info, Token>,
        seeds: &[&[&[u8]]],
    ) -> Result<()> {
        let cpi_accounts = MintTo {
            mint: mint.to_account_info(),
            to: to.to_account_info(),
            authority: authority.to_account_info(),
        };
        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds);
        token::mint_to(cpi_ctx, amount)?;
        Ok(())
    }
    
    /// Burn wrapped tokens
    pub fn burn_wrapped_tokens<'info>(
        mint: Account<'info, Mint>,
        from: Account<'info, TokenAccount>,
        authority: AccountInfo<'info>,
        amount: u64,
        token_program: Program<'info, Token>,
    ) -> Result<()> {
        let cpi_accounts = Burn {
            mint: mint.to_account_info(),
            from: from.to_account_info(),
            authority: authority.to_account_info(),
        };
        let cpi_program = token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::burn(cpi_ctx, amount)?;
        Ok(())
    }
    
    /// Transfer tokens (for fee collection, etc.)
    pub fn transfer_tokens<'info>(
        from: Account<'info, TokenAccount>,
        to: Account<'info, TokenAccount>,
        authority: AccountInfo<'info>,
        amount: u64,
        token_program: Program<'info, Token>,
        seeds: Option<&[&[&[u8]]]>,
    ) -> Result<()> {
        let cpi_accounts = token::Transfer {
            from: from.to_account_info(),
            to: to.to_account_info(),
            authority: authority.to_account_info(),
        };
        let cpi_program = token_program.to_account_info();
        
        let cpi_ctx = if let Some(seeds) = seeds {
            CpiContext::new_with_signer(cpi_program, cpi_accounts, seeds)
        } else {
            CpiContext::new(cpi_program, cpi_accounts)
        };
        
        token::transfer(cpi_ctx, amount)?;
        Ok(())
    }
}

/// Validation helpers
pub struct ValidationHelpers;

impl ValidationHelpers {
    /// Check if bridge is not paused
    pub fn check_bridge_active(paused: bool) -> Result<()> {
        require!(!paused, BridgeError::BridgePaused);
        Ok(())
    }
    
    /// Validate transfer amount
    pub fn validate_transfer_amount(
        amount: u64,
        min_amount: u64,
        max_amount: u64,
    ) -> Result<()> {
        require!(amount >= min_amount, BridgeError::TransferAmountTooSmall);
        if max_amount > 0 {
            require!(amount <= max_amount, BridgeError::TransferAmountExceedsLimit);
        }
        Ok(())
    }
    
    /// Check VAA timestamp is not expired
    pub fn check_vaa_timestamp(
        vaa_timestamp: i64,
        expiration_time: i64,
    ) -> Result<()> {
        let clock = Clock::get()?;
        let current_time = clock.unix_timestamp;
        
        require!(
            vaa_timestamp <= current_time + 300, // Allow 5 min future tolerance
            BridgeError::VaaTimestampTooFar
        );
        
        require!(
            current_time - vaa_timestamp <= expiration_time,
            BridgeError::VaaTimestampTooOld
        );
        
        Ok(())
    }
    
    /// Validate sequence number (replay protection)
    pub fn validate_sequence(
        sequence: u64,
        last_sequence: u64,
    ) -> Result<()> {
        require!(
            sequence > last_sequence,
            BridgeError::InvalidSequenceNumber
        );
        Ok(())
    }
}
