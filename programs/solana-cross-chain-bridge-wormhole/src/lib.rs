use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod state;
pub mod vaa;
pub mod zk;
pub mod utils;
pub mod instructions;

use instructions::*;

declare_id!("BriDgE1111111111111111111111111111111111111");

/// Solana Cross-Chain Bridge (Wormhole-inspired with ZK Proof Integration)
/// 
/// A production-grade cross-chain bridge protocol enabling secure token transfers
/// across multiple blockchains with guardian attestations, VAA verification,
/// and zero-knowledge proof support for enhanced privacy and security.
///
/// Key Features:
/// - Cross-chain token transfers (lock on source, mint wrapped on target)
/// - Guardian-based VAA (Verifiable Action Approval) system
/// - ZK proof integration for private transfers and proof of solvency
/// - Permissionless relaying
/// - Token-2022 support for wrapped RWA/DeFi assets
/// - Comprehensive security features and replay protection
#[program]
pub mod solana_cross_chain_bridge_wormhole {
    use super::*;

    /// Initialize the core bridge state
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
        instructions::initialize::initialize_bridge(
            ctx,
            min_bridge_fee,
            max_bridge_fee,
            relayer_reward_bps,
            max_transfer_amount,
            min_transfer_amount,
            vaa_expiration_time,
            zk_verifier_program,
        )
    }

    /// Initialize the first guardian set
    pub fn initialize_guardian_set(
        ctx: Context<InitializeGuardianSet>,
        guardians: Vec<Pubkey>,
        quorum: u8,
        expiration_time: i64,
    ) -> Result<()> {
        instructions::initialize::initialize_guardian_set(
            ctx,
            guardians,
            quorum,
            expiration_time,
        )
    }

    /// Post a VAA to the bridge
    /// vaa_hash should be SHA256(VAA body) calculated client-side
    pub fn post_vaa(
        ctx: Context<PostVaa>,
        vaa_bytes: Vec<u8>,
        vaa_hash: [u8; 32],
    ) -> Result<()> {
        instructions::post_vaa::post_vaa(ctx, vaa_bytes, vaa_hash)
    }

    /// Verify guardian signatures on a VAA
    pub fn verify_signatures(
        ctx: Context<VerifySignatures>,
        vaa_bytes: Vec<u8>,
    ) -> Result<()> {
        instructions::verify_signatures::verify_signatures(ctx, vaa_bytes)
    }

    /// Lock tokens on source chain for cross-chain transfer
    pub fn lock_tokens(
        ctx: Context<LockTokens>,
        amount: u64,
        target_chain: u16,
        recipient: [u8; 32],
    ) -> Result<()> {
        instructions::lock_tokens::lock_tokens(ctx, amount, target_chain, recipient)
    }

    /// Complete a cross-chain transfer by minting wrapped tokens
    pub fn complete_transfer(
        ctx: Context<CompleteTransfer>,
    ) -> Result<()> {
        instructions::complete_transfer::complete_transfer(ctx)
    }

    /// Submit and verify a ZK proof
    pub fn submit_zk_proof(
        ctx: Context<SubmitZkProof>,
        proof: Vec<u8>,
        public_inputs: Vec<u8>,
        circuit_id: u8,
    ) -> Result<()> {
        instructions::submit_zk_proof::submit_zk_proof(ctx, proof, public_inputs, circuit_id)
    }

    /// Register a wrapped asset
    pub fn register_wrapped_asset(
        ctx: Context<RegisterWrappedAsset>,
        source_chain: u16,
        source_token: [u8; 32],
        decimals: u8,
        symbol: String,
        name: String,
        is_token2022: bool,
    ) -> Result<()> {
        instructions::register_wrapped_asset::register_wrapped_asset(
            ctx,
            source_chain,
            source_token,
            decimals,
            symbol,
            name,
            is_token2022,
        )
    }

    /// Update guardian set
    pub fn update_guardian_set(
        ctx: Context<UpdateGuardianSet>,
        new_guardians: Vec<Pubkey>,
        new_quorum: u8,
        expiration_time: i64,
    ) -> Result<()> {
        instructions::update_guardian_set::update_guardian_set(
            ctx,
            new_guardians,
            new_quorum,
            expiration_time,
        )
    }

    /// Pause or unpause the bridge
    pub fn set_bridge_paused(
        ctx: Context<SetBridgePaused>,
        paused: bool,
    ) -> Result<()> {
        instructions::admin::set_bridge_paused(ctx, paused)
    }

    /// Update bridge configuration
    pub fn update_bridge_config(
        ctx: Context<UpdateBridgeConfig>,
        new_min_bridge_fee: Option<u64>,
        new_max_bridge_fee: Option<u64>,
        new_relayer_reward_bps: Option<u16>,
        new_max_transfer_amount: Option<u64>,
        new_min_transfer_amount: Option<u64>,
        new_vaa_expiration_time: Option<i64>,
    ) -> Result<()> {
        instructions::admin::update_bridge_config(
            ctx,
            new_min_bridge_fee,
            new_max_bridge_fee,
            new_relayer_reward_bps,
            new_max_transfer_amount,
            new_min_transfer_amount,
            new_vaa_expiration_time,
        )
    }

    /// Initialize ZK verifier state
    pub fn initialize_zk_verifier(
        ctx: Context<InitializeZkVerifier>,
        supported_circuits: Vec<u8>,
        public_input_sizes: Vec<u16>,
        zk_required: bool,
        proof_expiration_time: i64,
    ) -> Result<()> {
        instructions::admin::initialize_zk_verifier(
            ctx,
            supported_circuits,
            public_input_sizes,
            zk_required,
            proof_expiration_time,
        )
    }
}
