use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::BridgeError;

/// Pause or unpause the bridge (authority only)
pub fn set_bridge_paused(
    ctx: Context<SetBridgePaused>,
    paused: bool,
) -> Result<()> {
    let bridge_state = &mut ctx.accounts.bridge_state;
    
    require!(
        ctx.accounts.authority.key() == bridge_state.authority,
        BridgeError::Unauthorized
    );
    
    bridge_state.paused = paused;
    
    msg!("Bridge {} by {}", 
         if paused { "paused" } else { "unpaused" }, 
         ctx.accounts.authority.key());
    Ok(())
}

/// Update bridge configuration (authority only)
pub fn update_bridge_config(
    ctx: Context<UpdateBridgeConfig>,
    new_min_bridge_fee: Option<u64>,
    new_max_bridge_fee: Option<u64>,
    new_relayer_reward_bps: Option<u16>,
    new_max_transfer_amount: Option<u64>,
    new_min_transfer_amount: Option<u64>,
    new_vaa_expiration_time: Option<i64>,
) -> Result<()> {
    let bridge_state = &mut ctx.accounts.bridge_state;
    
    require!(
        ctx.accounts.authority.key() == bridge_state.authority,
        BridgeError::Unauthorized
    );
    
    if let Some(fee) = new_min_bridge_fee {
        bridge_state.min_bridge_fee = fee;
    }
    
    if let Some(fee) = new_max_bridge_fee {
        require!(fee >= bridge_state.min_bridge_fee, BridgeError::InvalidBridgeState);
        bridge_state.max_bridge_fee = fee;
    }
    
    if let Some(reward) = new_relayer_reward_bps {
        require!(reward <= 10000, BridgeError::InvalidBridgeState);
        bridge_state.relayer_reward_bps = reward;
    }
    
    if let Some(max) = new_max_transfer_amount {
        bridge_state.max_transfer_amount = max;
    }
    
    if let Some(min) = new_min_transfer_amount {
        bridge_state.min_transfer_amount = min;
    }
    
    if let Some(expiration) = new_vaa_expiration_time {
        require!(expiration > 0, BridgeError::InvalidBridgeState);
        bridge_state.vaa_expiration_time = expiration;
    }
    
    msg!("Bridge configuration updated");
    Ok(())
}

/// Initialize ZK verifier state
pub fn initialize_zk_verifier(
    ctx: Context<InitializeZkVerifier>,
    supported_circuits: Vec<u8>,
    public_input_sizes: Vec<u16>,
    zk_required: bool,
    proof_expiration_time: i64,
) -> Result<()> {
    let bridge_state = &ctx.accounts.bridge_state;
    let zk_verifier = &mut ctx.accounts.zk_verifier;
    
    require!(
        ctx.accounts.authority.key() == bridge_state.authority,
        BridgeError::Unauthorized
    );
    
    require!(
        supported_circuits.len() == public_input_sizes.len(),
        BridgeError::InvalidBridgeState
    );
    
    require!(
        supported_circuits.len() <= crate::state::ZkVerifierState::MAX_CIRCUITS,
        BridgeError::InvalidBridgeState
    );
    
    zk_verifier.verifier_program = bridge_state.zk_verifier_program;
    zk_verifier.supported_circuits = supported_circuits;
    zk_verifier.public_input_sizes = public_input_sizes;
    zk_verifier.zk_required = zk_required;
    zk_verifier.proof_expiration_time = proof_expiration_time;
    zk_verifier.bump = ctx.bumps.zk_verifier;
    
    msg!("ZK verifier initialized with {} circuits", zk_verifier.supported_circuits.len());
    Ok(())
}

#[derive(Accounts)]
pub struct SetBridgePaused<'info> {
    #[account(
        mut,
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct UpdateBridgeConfig<'info> {
    #[account(
        mut,
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct InitializeZkVerifier<'info> {
    #[account(
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    #[account(
        init,
        payer = authority,
        space = crate::state::ZkVerifierState::LEN,
        seeds = [b"zk_verifier"],
        bump
    )]
    pub zk_verifier: Account<'info, ZkVerifierState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
