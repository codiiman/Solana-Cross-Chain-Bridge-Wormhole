use anchor_lang::prelude::*;

/// Custom error codes for the cross-chain bridge program
/// Inspired by Wormhole protocol error handling patterns
#[error_code]
pub enum BridgeError {
    // ========== General Errors ==========
    #[msg("Bridge is paused")]
    BridgePaused,
    
    #[msg("Bridge not initialized")]
    BridgeNotInitialized,
    
    #[msg("Invalid bridge state")]
    InvalidBridgeState,
    
    // ========== VAA Errors ==========
    #[msg("Invalid VAA format")]
    InvalidVaaFormat,
    
    #[msg("VAA already processed")]
    VaaAlreadyProcessed,
    
    #[msg("VAA sequence number invalid")]
    InvalidVaaSequence,
    
    #[msg("VAA timestamp too old")]
    VaaTimestampTooOld,
    
    #[msg("VAA timestamp too far in future")]
    VaaTimestampTooFar,
    
    #[msg("VAA guardian set index mismatch")]
    GuardianSetMismatch,
    
    #[msg("VAA chain ID mismatch")]
    ChainIdMismatch,
    
    #[msg("VAA emitter address mismatch")]
    EmitterMismatch,
    
    // ========== Guardian Errors ==========
    #[msg("Insufficient guardian signatures")]
    InsufficientGuardianSignatures,
    
    #[msg("Invalid guardian signature")]
    InvalidGuardianSignature,
    
    #[msg("Guardian set not found")]
    GuardianSetNotFound,
    
    #[msg("Guardian quorum not met")]
    GuardianQuorumNotMet,
    
    #[msg("Guardian set expired")]
    GuardianSetExpired,
    
    #[msg("Unauthorized guardian operation")]
    UnauthorizedGuardian,
    
    // ========== ZK Proof Errors ==========
    #[msg("ZK proof verification failed")]
    ZkProofVerificationFailed,
    
    #[msg("Invalid ZK proof format")]
    InvalidZkProofFormat,
    
    #[msg("ZK proof circuit mismatch")]
    ZkCircuitMismatch,
    
    #[msg("ZK proof public inputs invalid")]
    ZkPublicInputsInvalid,
    
    #[msg("ZK proof commitment mismatch")]
    ZkCommitmentMismatch,
    
    #[msg("ZK verifier not initialized")]
    ZkVerifierNotInitialized,
    
    // ========== Transfer Errors ==========
    #[msg("Transfer not found")]
    TransferNotFound,
    
    #[msg("Transfer already completed")]
    TransferAlreadyCompleted,
    
    #[msg("Transfer amount too small")]
    TransferAmountTooSmall,
    
    #[msg("Transfer amount exceeds limit")]
    TransferAmountExceedsLimit,
    
    #[msg("Invalid transfer recipient")]
    InvalidTransferRecipient,
    
    #[msg("Insufficient funds for transfer")]
    InsufficientFunds,
    
    #[msg("Token account mismatch")]
    TokenAccountMismatch,
    
    #[msg("Invalid token mint")]
    InvalidTokenMint,
    
    // ========== Relayer Errors ==========
    #[msg("Unauthorized relayer")]
    UnauthorizedRelayer,
    
    #[msg("Relayer fee insufficient")]
    RelayerFeeInsufficient,
    
    #[msg("Relayer reward calculation failed")]
    RelayerRewardCalculationFailed,
    
    // ========== Wrapped Asset Errors ==========
    #[msg("Wrapped asset not found")]
    WrappedAssetNotFound,
    
    #[msg("Wrapped asset already exists")]
    WrappedAssetAlreadyExists,
    
    #[msg("Invalid wrapped asset metadata")]
    InvalidWrappedAssetMetadata,
    
    #[msg("Token-2022 extension not supported")]
    Token2022ExtensionNotSupported,
    
    // ========== Replay Protection Errors ==========
    #[msg("Sequence number already used")]
    SequenceNumberAlreadyUsed,
    
    #[msg("Invalid sequence number")]
    InvalidSequenceNumber,
    
    #[msg("Replay attack detected")]
    ReplayAttackDetected,
    
    // ========== Fee Errors ==========
    #[msg("Insufficient bridge fee")]
    InsufficientBridgeFee,
    
    #[msg("Fee calculation overflow")]
    FeeCalculationOverflow,
    
    #[msg("Invalid fee recipient")]
    InvalidFeeRecipient,
    
    // ========== Math Errors ==========
    #[msg("Math overflow")]
    MathOverflow,
    
    #[msg("Math underflow")]
    MathUnderflow,
    
    #[msg("Division by zero")]
    DivisionByZero,
    
    // ========== Signature Errors ==========
    #[msg("Signature verification failed")]
    SignatureVerificationFailed,
    
    #[msg("Invalid signature format")]
    InvalidSignatureFormat,
    
    #[msg("Signature threshold not met")]
    SignatureThresholdNotMet,
    
    // ========== Cross-Chain Errors ==========
    #[msg("Unsupported target chain")]
    UnsupportedTargetChain,
    
    #[msg("Invalid target chain address")]
    InvalidTargetChainAddress,
    
    #[msg("Cross-chain message failed")]
    CrossChainMessageFailed,
}
