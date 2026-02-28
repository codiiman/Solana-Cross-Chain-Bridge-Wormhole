use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::BridgeError;

/// Initialize the core bridge state
/// Only callable by the program authority
pub fn initialize_bridge(
    ctx: Context<InitializeBridge>,
    min_bridge_fee: u64,
    max_bridge_fee: u64,
    relayer_reward_bps: u16,
    max_transfer_amount: u64,
    min_transfer_amount: u64,
    vaa_expiration_time: i64,
    zk_verifier_program: Pubkey,
) -> Result<()> {
    let bridge_state = &mut ctx.accounts.bridge_state;
    
    bridge_state.authority = ctx.accounts.authority.key();
    bridge_state.guardian_set_index = 0;
    bridge_state.fee_recipient = ctx.accounts.fee_recipient.key();
    bridge_state.paused = false;
    bridge_state.min_bridge_fee = min_bridge_fee;
    bridge_state.max_bridge_fee = max_bridge_fee;
    bridge_state.relayer_reward_bps = relayer_reward_bps;
    bridge_state.max_transfer_amount = max_transfer_amount;
    bridge_state.min_transfer_amount = min_transfer_amount;
    bridge_state.vaa_expiration_time = vaa_expiration_time;
    bridge_state.sequence = 0;
    bridge_state.zk_verifier_program = zk_verifier_program;
    bridge_state.bump = ctx.bumps.bridge_state;
    
    msg!("Bridge initialized with authority: {}", bridge_state.authority);
    Ok(())
}

/// Initialize the first guardian set
pub fn initialize_guardian_set(
    ctx: Context<InitializeGuardianSet>,
    guardians: Vec<Pubkey>,
    quorum: u8,
    expiration_time: i64,
) -> Result<()> {
    let bridge_state = &ctx.accounts.bridge_state;
    let guardian_set = &mut ctx.accounts.guardian_set;
    
    require!(
        ctx.accounts.authority.key() == bridge_state.authority,
        BridgeError::Unauthorized
    );
    
    require!(
        guardians.len() <= GuardianSet::MAX_GUARDIANS,
        BridgeError::InvalidBridgeState
    );
    
    require!(
        quorum > 0 && quorum <= guardians.len() as u8,
        BridgeError::InvalidBridgeState
    );
    
    guardian_set.index = 0;
    guardian_set.keys = guardians;
    guardian_set.quorum = quorum;
    guardian_set.expiration_time = expiration_time;
    guardian_set.bump = ctx.bumps.guardian_set;
    
    // Update bridge state to use this guardian set
    let bridge_state = &mut ctx.accounts.bridge_state;
    bridge_state.guardian_set_index = 0;
    
    emit!(crate::events::GuardianSetUpdated {
        guardian_set_index: 0,
        guardians: guardian_set.keys.clone(),
        quorum: guardian_set.quorum,
        expiration_time: guardian_set.expiration_time,
        updated_by: ctx.accounts.authority.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    msg!("Guardian set initialized with {} guardians, quorum: {}", 
         guardian_set.keys.len(), quorum);
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeBridge<'info> {
    #[account(
        init,
        payer = authority,
        space = CoreBridgeState::LEN,
        seeds = [b"bridge_state"],
        bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: Fee recipient (treasury)
    pub fee_recipient: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct InitializeGuardianSet<'info> {
    #[account(
        mut,
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    #[account(
        init,
        payer = authority,
        space = GuardianSet::calculate_size(guardians.len()),
        seeds = [b"guardian_set", bridge_state.guardian_set_index.to_le_bytes().as_ref()],
        bump
    )]
    pub guardian_set: Account<'info, GuardianSet>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// CHECK: Guardians vector (passed as instruction data)
    /// In production, this would be validated more carefully
    pub system_program: Program<'info, System>,
}
