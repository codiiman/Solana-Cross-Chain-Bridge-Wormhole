use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::BridgeError;
use crate::vaa::{VaaParser, GuardianVerifier};

/// Verify guardian signatures on a VAA
/// This validates that enough guardians have signed the VAA
pub fn verify_signatures(
    ctx: Context<VerifySignatures>,
    vaa_bytes: Vec<u8>,
) -> Result<()> {
    let bridge_state = &ctx.accounts.bridge_state;
    let guardian_set = &ctx.accounts.guardian_set;
    let vaa_account = &mut ctx.accounts.vaa_account;
    
    // Check VAA matches guardian set
    require!(
        vaa_account.guardian_set_index == guardian_set.index,
        BridgeError::GuardianSetMismatch
    );
    
    // Check guardian set not expired
    if guardian_set.expiration_time > 0 {
        let clock = Clock::get()?;
        require!(
            clock.unix_timestamp < guardian_set.expiration_time,
            BridgeError::GuardianSetExpired
        );
    }
    
    // Parse VAA to extract signatures
    let vaa_components = VaaParser::parse_vaa(&vaa_bytes)?;
    
    // Extract message hash for signature verification
    let message_hash = VaaParser::extract_message_hash(&vaa_bytes)?;
    
    // Verify signatures
    let valid_signatures = GuardianVerifier::verify_signatures(
        &message_hash,
        &vaa_components.signatures,
        &guardian_set.keys,
        guardian_set.quorum,
    )?;
    
    // Mark VAA as verified (in production, would have separate verified flag)
    // For now, we'll just check signatures are valid
    
    emit!(crate::events::SignaturesVerified {
        vaa_hash: vaa_account.vaa_hash,
        guardian_set_index: guardian_set.index,
        signatures_count: valid_signatures,
        quorum_met: valid_signatures >= guardian_set.quorum,
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    msg!("Verified {} signatures on VAA (quorum: {})", 
         valid_signatures, guardian_set.quorum);
    Ok(())
}

#[derive(Accounts)]
pub struct VerifySignatures<'info> {
    #[account(
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    #[account(
        seeds = [b"guardian_set", bridge_state.guardian_set_index.to_le_bytes().as_ref()],
        bump = guardian_set.bump
    )]
    pub guardian_set: Account<'info, GuardianSet>,
    
    #[account(
        mut,
        seeds = [b"vaa", bridge_state.key().as_ref(), vaa_account.vaa_hash.as_ref()],
        bump = vaa_account.bump
    )]
    pub vaa_account: Account<'info, VaaAccount>,
    
    /// CHECK: VAA bytes (passed as instruction data)
    pub system_program: Program<'info, System>,
}
