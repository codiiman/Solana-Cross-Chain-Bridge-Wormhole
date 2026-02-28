use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::BridgeError;
use crate::zk::{ZkVerifier, circuit_types};

/// Submit and verify a ZK proof for a cross-chain transfer
/// Enables privacy-preserving transfers or proof of solvency
pub fn submit_zk_proof(
    ctx: Context<SubmitZkProof>,
    proof: Vec<u8>,
    public_inputs: Vec<u8>,
    circuit_id: u8,
) -> Result<()> {
    let bridge_state = &ctx.accounts.bridge_state;
    let zk_verifier = &ctx.accounts.zk_verifier;
    
    // Validate bridge is active
    crate::utils::ValidationHelpers::check_bridge_active(bridge_state.paused)?;
    
    // Validate circuit ID is supported
    require!(
        zk_verifier.supported_circuits.contains(&circuit_id),
        BridgeError::ZkCircuitMismatch
    );
    
    // Verify ZK proof
    let is_valid = ZkVerifier::verify_proof(
        &proof,
        &public_inputs,
        circuit_id,
        &zk_verifier.verifier_program,
    )?;
    
    require!(is_valid, BridgeError::ZkProofVerificationFailed);
    
    // Calculate proof ID (hash of proof + public inputs)
    let proof_id = ZkVerifier::calculate_commitment(&public_inputs)?;
    
    // Verify commitment matches expected
    let commitment_valid = ZkVerifier::verify_commitment(
        &proof,
        &public_inputs,
        &proof_id,
    )?;
    
    require!(commitment_valid, BridgeError::ZkCommitmentMismatch);
    
    // Extract public inputs for transfer processing
    let zk_inputs = ZkVerifier::extract_public_inputs(&public_inputs)?;
    
    // If this is a transfer proof, validate against transfer account
    if circuit_id == circuit_types::TRANSFER_PROOF {
        let transfer = &ctx.accounts.transfer;
        
        require!(
            zk_inputs.amount == transfer.amount,
            BridgeError::ZkPublicInputsInvalid
        );
        
        require!(
            zk_inputs.source_chain == transfer.source_chain,
            BridgeError::ZkPublicInputsInvalid
        );
        
        require!(
            zk_inputs.target_chain == transfer.target_chain,
            BridgeError::ZkPublicInputsInvalid
        );
        
        // Update transfer with ZK proof ID
        let transfer = &mut ctx.accounts.transfer;
        transfer.zk_proof_id = Some(proof_id);
    }
    
    emit!(crate::events::ZkProofVerified {
        proof_id,
        circuit_id,
        public_inputs_hash: proof_id,
        verifier: zk_verifier.verifier_program,
        verified_by: ctx.accounts.verifier.key(),
        timestamp: Clock::get()?.unix_timestamp,
    });
    
    msg!("ZK proof verified: circuit_id={}, proof_id={:?}", circuit_id, proof_id);
    Ok(())
}

#[derive(Accounts)]
pub struct SubmitZkProof<'info> {
    #[account(
        seeds = [b"bridge_state"],
        bump = bridge_state.bump
    )]
    pub bridge_state: Account<'info, CoreBridgeState>,
    
    #[account(
        seeds = [b"zk_verifier"],
        bump = zk_verifier.bump
    )]
    pub zk_verifier: Account<'info, ZkVerifierState>,
    
    /// CHECK: Transfer account (optional, for transfer proofs)
    #[account(mut)]
    pub transfer: Option<Account<'info, TransferAccount>>,
    
    pub verifier: Signer<'info>,
}
