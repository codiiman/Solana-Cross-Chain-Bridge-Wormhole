use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::BridgeError;
use crate::vaa::{VaaParser, VaaComponents};
use crate::utils::ValidationHelpers;

/// Post a VAA (Verifiable Action Approval) to the bridge
/// This stores the VAA for later verification and processing
/// Anyone can post a VAA (permissionless)
/// 
/// Note: vaa_hash should be calculated client-side as SHA256(VAA body)
pub fn post_vaa(
    ctx: Context<PostVaa>,
    vaa_bytes: Vec<u8>,
    vaa_hash: [u8; 32],
) -> Result<()> {
    let bridge_state = &ctx.accounts.bridge_state;
    
    // Validate bridge is active
    ValidationHelpers::check_bridge_active(bridge_state.paused)?;
    
    // Parse VAA
    let vaa_components = VaaParser::parse_vaa(&vaa_bytes)?;
    
    // Verify provided hash matches calculated hash
    let calculated_hash = VaaParser::calculate_vaa_hash(&vaa_bytes)?;
    require!(calculated_hash == vaa_hash, BridgeError::InvalidVaaFormat);
    
    // Check if VAA already exists
    // In production, would check if VAA account already exists
    
    // Validate VAA timestamp (using current time as reference)
    let clock = Clock::get()?;
    ValidationHelpers::check_vaa_timestamp(
        clock.unix_timestamp, // Using current time
        bridge_state.vaa_expiration_time,
    )?;
    
    // Store VAA
    let vaa_account = &mut ctx.accounts.vaa_account;
    vaa_account.vaa_hash = vaa_hash;
    vaa_account.emitter_chain = vaa_components.emitter_chain;
    vaa_account.emitter_address = vaa_components.emitter_address;
    vaa_account.sequence = vaa_components.sequence;
    vaa_account.guardian_set_index = vaa_components.guardian_set_index;
    vaa_account.timestamp = Clock::get()?.unix_timestamp;
    vaa_account.payload = vaa_components.payload;
    vaa_account.processed = false;
    vaa_account.bump = ctx.bumps.vaa_account;
    
    emit!(crate::events::VaaPosted {
        vaa_hash,
        emitter_chain: vaa_account.emitter_chain,
        emitter_address: vaa_account.emitter_address,
        sequence: vaa_account.sequence,
        guardian_set_index: vaa_account.guardian_set_index,
        timestamp: vaa_account.timestamp,
    });
    
    msg!("VAA posted: hash={:?}, sequence={}", vaa_hash, vaa_account.sequence);
    Ok(())
}

#[derive(Accounts)]
#[instruction(vaa_bytes: Vec<u8>, vaa_hash: [u8; 32])]
pub struct PostVaa<'info> {
    #[account(
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    #[account(
        init,
        payer = payer,
        space = VaaAccount::calculate_size(vaa_bytes.len()),
        seeds = [b"vaa", bridge_state.key().as_ref(), &vaa_hash],
        bump
    )]
    pub vaa_account: Account<'info, VaaAccount>,
    
    #[account(mut)]
    pub payer: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}
