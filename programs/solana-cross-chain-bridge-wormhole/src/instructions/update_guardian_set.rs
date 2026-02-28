use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::BridgeError;

/// Update guardian set (add/remove guardians or change quorum)
/// Only callable by bridge authority
pub fn update_guardian_set(
    ctx: Context<UpdateGuardianSet>,
    new_guardians: Vec<Pubkey>,
    new_quorum: u8,
    expiration_time: i64,
) -> Result<()> {
    let bridge_state = &mut ctx.accounts.bridge_state;
    let old_guardian_set = &ctx.accounts.old_guardian_set;
    let new_guardian_set = &mut ctx.accounts.new_guardian_set;
    let clock = Clock::get()?;
    
    // Validate authority
    require!(
        ctx.accounts.authority.key() == bridge_state.authority,
        BridgeError::Unauthorized
    );
    
    // Validate new guardian set
    require!(
        new_guardians.len() <= GuardianSet::MAX_GUARDIANS,
        BridgeError::InvalidBridgeState
    );
    
    require!(
        new_quorum > 0 && new_quorum <= new_guardians.len() as u8,
        BridgeError::InvalidBridgeState
    );
    
    // Create new guardian set
    let new_index = old_guardian_set.index
        .checked_add(1)
        .ok_or(BridgeError::MathOverflow)?;
    new_guardian_set.index = new_index;
    new_guardian_set.keys = new_guardians.clone();
    new_guardian_set.quorum = new_quorum;
    new_guardian_set.expiration_time = expiration_time;
    new_guardian_set.bump = ctx.bumps.new_guardian_set;
    
    // Update bridge state to use new guardian set
    bridge_state.guardian_set_index = new_index;
    
    emit!(crate::events::GuardianSetUpdated {
        guardian_set_index: new_index,
        guardians: new_guardians,
        quorum: new_quorum,
        expiration_time,
        updated_by: ctx.accounts.authority.key(),
        timestamp: clock.unix_timestamp,
    });
    
    msg!("Guardian set updated: index={}, guardians={}, quorum={}", 
         new_index, new_guardian_set.keys.len(), new_quorum);
    Ok(())
}

#[derive(Accounts)]
pub struct UpdateGuardianSet<'info> {
    #[account(
        mut,
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    #[account(
        seeds = [b"guardian_set", bridge_state.guardian_set_index.to_le_bytes().as_ref()],
        bump = old_guardian_set.bump
    )]
    pub old_guardian_set: Account<'info, GuardianSet>,
    
    #[account(
        init,
        payer = authority,
        space = GuardianSet::calculate_size(new_guardians.len()),
        seeds = [b"guardian_set", &new_index.to_le_bytes()],
        bump
    )]
    pub new_guardian_set: Account<'info, GuardianSet>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
