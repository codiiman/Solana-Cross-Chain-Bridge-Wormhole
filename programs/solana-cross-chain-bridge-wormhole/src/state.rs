use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

/// Core bridge state - stores protocol-wide configuration
#[account]
#[derive(Default)]
pub struct CoreBridgeState {
    /// Bridge authority (upgrade authority or DAO)
    pub authority: Pubkey,
    
    /// Guardian set index currently in use
    pub guardian_set_index: u32,
    
    /// Bridge fee recipient (treasury)
    pub fee_recipient: Pubkey,
    
    /// Bridge pause flag
    pub paused: bool,
    
    /// Minimum bridge fee (in native token)
    pub min_bridge_fee: u64,
    
    /// Maximum bridge fee (in native token)
    pub max_bridge_fee: u64,
    
    /// Relayer reward percentage (basis points, 10000 = 100%)
    pub relayer_reward_bps: u16,
    
    /// Maximum transfer amount (0 = unlimited)
    pub max_transfer_amount: u64,
    
    /// Minimum transfer amount
    pub min_transfer_amount: u64,
    
    /// VAA expiration time (seconds)
    pub vaa_expiration_time: i64,
    
    /// Sequence number counter (incremented for each transfer)
    pub sequence: u64,
    
    /// ZK verifier program ID
    pub zk_verifier_program: Pubkey,
    
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl CoreBridgeState {
    pub const LEN: usize = 8 + // discriminator
        32 + // authority
        4 +  // guardian_set_index
        32 + // fee_recipient
        1 +  // paused
        8 +  // min_bridge_fee
        8 +  // max_bridge_fee
        2 +  // relayer_reward_bps
        8 +  // max_transfer_amount
        8 +  // min_transfer_amount
        8 +  // vaa_expiration_time
        8 +  // sequence
        32 + // zk_verifier_program
        1;   // bump
}

/// Guardian set - stores guardian public keys and quorum requirements
#[account]
pub struct GuardianSet {
    /// Guardian set index (incremented on updates)
    pub index: u32,
    
    /// List of guardian public keys
    pub keys: Vec<Pubkey>,
    
    /// Quorum threshold (number of signatures required)
    pub quorum: u8,
    
    /// Expiration time (unix timestamp, 0 = never expires)
    pub expiration_time: i64,
    
    /// Bump seed
    pub bump: u8,
}

impl GuardianSet {
    pub const MAX_GUARDIANS: usize = 19; // Wormhole standard
    
    pub fn calculate_size(num_guardians: usize) -> usize {
        8 + // discriminator
        4 + // index
        4 + (32 * num_guardians) + // keys (Vec)
        1 + // quorum
        8 + // expiration_time
        1   // bump
    }
}

/// VAA (Verifiable Action Approval) account - stores parsed VAA data
#[account]
pub struct VaaAccount {
    /// VAA hash (SHA-256 of VAA payload)
    pub vaa_hash: [u8; 32],
    
    /// Emitter chain ID
    pub emitter_chain: u16,
    
    /// Emitter address (32 bytes)
    pub emitter_address: [u8; 32],
    
    /// Sequence number
    pub sequence: u64,
    
    /// Guardian set index used for signatures
    pub guardian_set_index: u32,
    
    /// VAA timestamp
    pub timestamp: i64,
    
    /// VAA payload (variable length, max 1024 bytes)
    pub payload: Vec<u8>,
    
    /// Whether this VAA has been processed
    pub processed: bool,
    
    /// Bump seed
    pub bump: u8,
}

impl VaaAccount {
    pub const MAX_PAYLOAD_SIZE: usize = 1024;
    
    pub fn calculate_size(payload_size: usize) -> usize {
        8 + // discriminator
        32 + // vaa_hash
        2 +  // emitter_chain
        32 + // emitter_address
        8 +  // sequence
        4 +  // guardian_set_index
        8 +  // timestamp
        4 + payload_size + // payload (Vec)
        1 + // processed
        1   // bump
    }
}

/// Transfer account - tracks cross-chain token transfers
#[account]
pub struct TransferAccount {
    /// Transfer ID (hash of transfer details)
    pub transfer_id: [u8; 32],
    
    /// Source chain ID
    pub source_chain: u16,
    
    /// Target chain ID
    pub target_chain: u16,
    
    /// Token mint (source chain)
    pub token_mint: Pubkey,
    
    /// Transfer amount
    pub amount: u64,
    
    /// Recipient address on target chain (32 bytes)
    pub recipient: [u8; 32],
    
    /// Bridge fee paid
    pub fee: u64,
    
    /// Sequence number
    pub sequence: u64,
    
    /// VAA hash (if transfer completed via VAA)
    pub vaa_hash: Option<[u8; 32]>,
    
    /// ZK proof ID (if transfer uses ZK proof)
    pub zk_proof_id: Option<[u8; 32]>,
    
    /// Transfer status: 0 = pending, 1 = completed, 2 = failed
    pub status: u8,
    
    /// Timestamp when transfer was initiated
    pub created_at: i64,
    
    /// Timestamp when transfer was completed
    pub completed_at: Option<i64>,
    
    /// Bump seed
    pub bump: u8,
}

impl TransferAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // transfer_id
        2 +  // source_chain
        2 +  // target_chain
        32 + // token_mint
        8 +  // amount
        32 + // recipient
        8 +  // fee
        8 +  // sequence
        1 + 32 + // vaa_hash (Option)
        1 + 32 + // zk_proof_id (Option)
        1 +  // status
        8 +  // created_at
        1 + 8 + // completed_at (Option)
        1;   // bump
}

/// Wrapped asset metadata - stores information about wrapped tokens
#[account]
pub struct WrappedAsset {
    /// Source chain ID
    pub source_chain: u16,
    
    /// Source token address (32 bytes)
    pub source_token: [u8; 32],
    
    /// Wrapped token mint on Solana
    pub wrapped_mint: Pubkey,
    
    /// Token decimals
    pub decimals: u8,
    
    /// Token symbol (max 16 chars)
    pub symbol: String,
    
    /// Token name (max 64 chars)
    pub name: String,
    
    /// Whether this is a Token-2022 mint
    pub is_token2022: bool,
    
    /// Total supply of wrapped tokens
    pub total_supply: u64,
    
    /// Bump seed
    pub bump: u8,
}

impl WrappedAsset {
    pub const MAX_SYMBOL_LENGTH: usize = 16;
    pub const MAX_NAME_LENGTH: usize = 64;
    
    pub const LEN: usize = 8 + // discriminator
        2 +  // source_chain
        32 + // source_token
        32 + // wrapped_mint
        1 +  // decimals
        4 + Self::MAX_SYMBOL_LENGTH + // symbol
        4 + Self::MAX_NAME_LENGTH + // name
        1 +  // is_token2022
        8 +  // total_supply
        1;   // bump
}

/// ZK verifier state - stores ZK proof verification configuration
#[account]
pub struct ZkVerifierState {
    /// Verifier program ID
    pub verifier_program: Pubkey,
    
    /// Circuit IDs supported (max 8 circuits)
    pub supported_circuits: Vec<u8>,
    
    /// Public input size for each circuit (bytes)
    pub public_input_sizes: Vec<u16>,
    
    /// Whether ZK proofs are required for transfers
    pub zk_required: bool,
    
    /// ZK proof expiration time (seconds)
    pub proof_expiration_time: i64,
    
    /// Bump seed
    pub bump: u8,
}

impl ZkVerifierState {
    pub const MAX_CIRCUITS: usize = 8;
    
    pub const LEN: usize = 8 + // discriminator
        32 + // verifier_program
        4 + (Self::MAX_CIRCUITS * 1) + // supported_circuits (Vec)
        4 + (Self::MAX_CIRCUITS * 2) + // public_input_sizes (Vec)
        1 +  // zk_required
        8 +  // proof_expiration_time
        1;   // bump
}

/// Relayer account - tracks relayer information and rewards
#[account]
pub struct RelayerAccount {
    /// Relayer wallet
    pub relayer: Pubkey,
    
    /// Total transfers relayed
    pub total_transfers: u64,
    
    /// Total rewards earned
    pub total_rewards: u64,
    
    /// Whether relayer is authorized (if false, anyone can relay)
    pub authorized: bool,
    
    /// Last activity timestamp
    pub last_activity: i64,
    
    /// Bump seed
    pub bump: u8,
}

impl RelayerAccount {
    pub const LEN: usize = 8 + // discriminator
        32 + // relayer
        8 +  // total_transfers
        8 +  // total_rewards
        1 +  // authorized
        8 +  // last_activity
        1;   // bump
}

/// Chain ID constants (following Wormhole standard)
pub mod chain_ids {
    pub const SOLANA: u16 = 1;
    pub const ETHEREUM: u16 = 2;
    pub const TERRA: u16 = 3;
    pub const BSC: u16 = 4;
    pub const POLYGON: u16 = 5;
    pub const AVALANCHE: u16 = 6;
    pub const FANTOM: u16 = 10;
    pub const CELO: u16 = 14;
    pub const MOONBEAM: u16 = 16;
    pub const NEAR: u16 = 15;
    pub const ARBITRUM: u16 = 23;
    pub const OPTIMISM: u16 = 24;
    pub const BASE: u16 = 30;
}

/// Transfer status constants
pub mod transfer_status {
    pub const PENDING: u8 = 0;
    pub const COMPLETED: u8 = 1;
    pub const FAILED: u8 = 2;
}
