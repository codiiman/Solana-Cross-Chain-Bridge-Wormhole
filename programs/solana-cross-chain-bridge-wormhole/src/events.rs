use anchor_lang::prelude::*;

/// Event emitted when a VAA is posted to the bridge
#[event]
pub struct VaaPosted {
    pub vaa_hash: [u8; 32],
    pub emitter_chain: u16,
    pub emitter_address: [u8; 32],
    pub sequence: u64,
    pub guardian_set_index: u32,
    pub timestamp: i64,
}

/// Event emitted when guardian signatures are verified
#[event]
pub struct SignaturesVerified {
    pub vaa_hash: [u8; 32],
    pub guardian_set_index: u32,
    pub signatures_count: u8,
    pub quorum_met: bool,
    pub timestamp: i64,
}

/// Event emitted when tokens are locked on source chain
#[event]
pub struct TokensLocked {
    pub transfer_id: [u8; 32],
    pub source_chain: u16,
    pub target_chain: u16,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub recipient: [u8; 32], // Target chain address
    pub fee: u64,
    pub sequence: u64,
    pub timestamp: i64,
}

/// Event emitted when wrapped tokens are minted on target chain
#[event]
pub struct TokensMinted {
    pub transfer_id: [u8; 32],
    pub source_chain: u16,
    pub token_mint: Pubkey,
    pub wrapped_mint: Pubkey,
    pub amount: u64,
    pub recipient: Pubkey,
    pub vaa_hash: [u8; 32],
    pub timestamp: i64,
}

/// Event emitted when wrapped tokens are burned for redemption
#[event]
pub struct TokensBurned {
    pub transfer_id: [u8; 32],
    pub target_chain: u16,
    pub wrapped_mint: Pubkey,
    pub amount: u64,
    pub recipient: [u8; 32], // Target chain address
    pub fee: u64,
    pub sequence: u64,
    pub timestamp: i64,
}

/// Event emitted when a transfer is completed
#[event]
pub struct TransferCompleted {
    pub transfer_id: [u8; 32],
    pub source_chain: u16,
    pub target_chain: u16,
    pub token_mint: Pubkey,
    pub amount: u64,
    pub recipient: Pubkey,
    pub relayer: Pubkey,
    pub relayer_fee: u64,
    pub vaa_hash: [u8; 32],
    pub timestamp: i64,
}

/// Event emitted when a ZK proof is submitted and verified
#[event]
pub struct ZkProofVerified {
    pub proof_id: [u8; 32],
    pub circuit_id: u8,
    pub public_inputs_hash: [u8; 32],
    pub verifier: Pubkey,
    pub verified_by: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when a wrapped asset is registered
#[event]
pub struct WrappedAssetRegistered {
    pub source_chain: u16,
    pub source_token: [u8; 32],
    pub wrapped_mint: Pubkey,
    pub decimals: u8,
    pub symbol: String,
    pub name: String,
    pub registered_by: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when guardian set is updated
#[event]
pub struct GuardianSetUpdated {
    pub guardian_set_index: u32,
    pub guardians: Vec<Pubkey>,
    pub quorum: u8,
    pub expiration_time: i64,
    pub updated_by: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when a relayer is rewarded
#[event]
pub struct RelayerRewarded {
    pub relayer: Pubkey,
    pub transfer_id: [u8; 32],
    pub reward_amount: u64,
    pub reward_token: Pubkey,
    pub timestamp: i64,
}

/// Event emitted when bridge fees are collected
#[event]
pub struct FeesCollected {
    pub fee_type: u8, // 0 = bridge fee, 1 = relayer fee
    pub amount: u64,
    pub token: Pubkey,
    pub treasury: Pubkey,
    pub timestamp: i64,
}
