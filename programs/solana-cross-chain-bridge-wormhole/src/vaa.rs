use anchor_lang::prelude::*;
use sha2::{Sha256, Digest};
use crate::state::VaaAccount;
use crate::errors::BridgeError;

/// VAA (Verifiable Action Approval) parsing and validation utilities
/// 
/// VAA format (following Wormhole standard):
/// - Header (6 bytes): version (1), guardian_set_index (4), signatures_len (1)
/// - Signatures: Each signature is 66 bytes (guardian_index (1) + signature (65))
/// - Body: emitter_chain (2), emitter_address (32), sequence (8), consistency_level (1), payload (variable)
pub struct VaaParser;

impl VaaParser {
    /// Parse VAA from raw bytes
    /// Returns parsed components for validation
    pub fn parse_vaa(vaa_bytes: &[u8]) -> Result<VaaComponents> {
        require!(vaa_bytes.len() >= 51, BridgeError::InvalidVaaFormat); // Minimum size
        
        let mut offset = 0;
        
        // Parse header
        let version = vaa_bytes[offset];
        offset += 1;
        require!(version == 1, BridgeError::InvalidVaaFormat);
        
        let guardian_set_index = u32::from_le_bytes([
            vaa_bytes[offset],
            vaa_bytes[offset + 1],
            vaa_bytes[offset + 2],
            vaa_bytes[offset + 3],
        ]);
        offset += 4;
        
        let signatures_len = vaa_bytes[offset] as usize;
        offset += 1;
        
        require!(
            signatures_len > 0 && signatures_len <= 19,
            BridgeError::InvalidVaaFormat
        );
        
        // Parse signatures (each is 66 bytes)
        let signatures_start = offset;
        let signatures_end = offset + (signatures_len * 66);
        require!(signatures_end <= vaa_bytes.len(), BridgeError::InvalidVaaFormat);
        
        let signatures = vaa_bytes[signatures_start..signatures_end].to_vec();
        offset = signatures_end;
        
        // Parse body
        require!(offset + 43 <= vaa_bytes.len(), BridgeError::InvalidVaaFormat);
        
        let emitter_chain = u16::from_le_bytes([
            vaa_bytes[offset],
            vaa_bytes[offset + 1],
        ]);
        offset += 2;
        
        let mut emitter_address = [0u8; 32];
        emitter_address.copy_from_slice(&vaa_bytes[offset..offset + 32]);
        offset += 32;
        
        let sequence = u64::from_le_bytes([
            vaa_bytes[offset],
            vaa_bytes[offset + 1],
            vaa_bytes[offset + 2],
            vaa_bytes[offset + 3],
            vaa_bytes[offset + 4],
            vaa_bytes[offset + 5],
            vaa_bytes[offset + 6],
            vaa_bytes[offset + 7],
        ]);
        offset += 8;
        
        let consistency_level = vaa_bytes[offset];
        offset += 1;
        
        // Payload is the rest
        let payload = vaa_bytes[offset..].to_vec();
        
        Ok(VaaComponents {
            version,
            guardian_set_index,
            signatures_len,
            signatures,
            emitter_chain,
            emitter_address,
            sequence,
            consistency_level,
            payload,
        })
    }
    
    /// Calculate VAA hash (SHA-256 of body)
    pub fn calculate_vaa_hash(vaa_bytes: &[u8]) -> Result<[u8; 32]> {
        // Hash is computed over the body (everything after signatures)
        let components = Self::parse_vaa(vaa_bytes)?;
        
        // Reconstruct body for hashing
        let mut body = Vec::new();
        body.extend_from_slice(&components.emitter_chain.to_le_bytes());
        body.extend_from_slice(&components.emitter_address);
        body.extend_from_slice(&components.sequence.to_le_bytes());
        body.push(components.consistency_level);
        body.extend_from_slice(&components.payload);
        
        let mut hasher = Sha256::new();
        hasher.update(&body);
        let hash = hasher.finalize();
        
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash);
        Ok(hash_array)
    }
    
    /// Validate VAA structure
    pub fn validate_vaa_structure(vaa_bytes: &[u8]) -> Result<()> {
        Self::parse_vaa(vaa_bytes)?;
        Ok(())
    }
    
    /// Extract message hash from VAA (for guardian signature verification)
    pub fn extract_message_hash(vaa_bytes: &[u8]) -> Result<[u8; 32]> {
        // Message hash is SHA-256 of: "VAAMessage" + body
        let components = Self::parse_vaa(vaa_bytes)?;
        
        let mut body = Vec::new();
        body.extend_from_slice(b"VAAMessage");
        body.extend_from_slice(&components.emitter_chain.to_le_bytes());
        body.extend_from_slice(&components.emitter_address);
        body.extend_from_slice(&components.sequence.to_le_bytes());
        body.push(components.consistency_level);
        body.extend_from_slice(&components.payload);
        
        let mut hasher = Sha256::new();
        hasher.update(&body);
        let hash = hasher.finalize();
        
        let mut hash_array = [0u8; 32];
        hash_array.copy_from_slice(&hash);
        Ok(hash_array)
    }
}

/// Parsed VAA components
#[derive(Clone)]
pub struct VaaComponents {
    pub version: u8,
    pub guardian_set_index: u32,
    pub signatures_len: usize,
    pub signatures: Vec<u8>,
    pub emitter_chain: u16,
    pub emitter_address: [u8; 32],
    pub sequence: u64,
    pub consistency_level: u8,
    pub payload: Vec<u8>,
}

/// Guardian signature verification
pub struct GuardianVerifier;

impl GuardianVerifier {
    /// Verify guardian signatures against guardian set
    /// Returns number of valid signatures
    pub fn verify_signatures(
        message_hash: &[u8; 32],
        signatures: &[u8],
        guardian_set: &[Pubkey],
        quorum: u8,
    ) -> Result<u8> {
        let mut valid_signatures = 0u8;
        let mut used_guardians = std::collections::HashSet::new();
        
        // Each signature is 66 bytes: guardian_index (1) + signature (65)
        let num_signatures = signatures.len() / 66;
        require!(num_signatures > 0, BridgeError::InsufficientGuardianSignatures);
        
        for i in 0..num_signatures {
            let sig_start = i * 66;
            let guardian_index = signatures[sig_start] as usize;
            
            // Check guardian index is valid
            require!(
                guardian_index < guardian_set.len(),
                BridgeError::InvalidGuardianSignature
            );
            
            // Check guardian not already used
            require!(
                !used_guardians.contains(&guardian_index),
                BridgeError::InvalidGuardianSignature
            );
            
            // Extract signature (65 bytes, ED25519)
            let sig_bytes = &signatures[sig_start + 1..sig_start + 66];
            
            // Verify signature (stub - in production would use ed25519-dalek)
            // For now, we'll do a basic validation
            if Self::verify_ed25519_signature(
                &guardian_set[guardian_index],
                message_hash,
                sig_bytes,
            )? {
                valid_signatures = valid_signatures
                    .checked_add(1)
                    .ok_or(BridgeError::MathOverflow)?;
                used_guardians.insert(guardian_index);
            }
        }
        
        require!(
            valid_signatures >= quorum,
            BridgeError::GuardianQuorumNotMet
        );
        
        Ok(valid_signatures)
    }
    
    /// Verify ED25519 signature (stub implementation)
    /// In production, this would use ed25519-dalek or similar
    fn verify_ed25519_signature(
        _public_key: &Pubkey,
        _message: &[u8; 32],
        _signature: &[u8],
    ) -> Result<bool> {
        // Stub: In production, this would verify the signature
        // For now, return true for demonstration
        // TODO: Implement actual ED25519 verification
        Ok(true)
    }
}

/// Transfer payload parsing (for token transfers)
pub struct TransferPayloadParser;

impl TransferPayloadParser {
    /// Parse transfer payload from VAA
    /// Transfer payload format:
    /// - payload_id (1): 1 = token transfer, 2 = token transfer with payload
    /// - amount (8): transfer amount
    /// - token_address (32): source token address
    /// - token_chain (2): source chain ID
    /// - recipient (32): target chain recipient address
    /// - recipient_chain (2): target chain ID
    /// - fee (8): bridge fee
    pub fn parse_transfer_payload(payload: &[u8]) -> Result<TransferPayload> {
        require!(payload.len() >= 89, BridgeError::InvalidVaaFormat);
        
        let mut offset = 0;
        
        let payload_id = payload[offset];
        offset += 1;
        require!(payload_id == 1 || payload_id == 2, BridgeError::InvalidVaaFormat);
        
        let amount = u64::from_le_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
            payload[offset + 4],
            payload[offset + 5],
            payload[offset + 6],
            payload[offset + 7],
        ]);
        offset += 8;
        
        let mut token_address = [0u8; 32];
        token_address.copy_from_slice(&payload[offset..offset + 32]);
        offset += 32;
        
        let token_chain = u16::from_le_bytes([
            payload[offset],
            payload[offset + 1],
        ]);
        offset += 2;
        
        let mut recipient = [0u8; 32];
        recipient.copy_from_slice(&payload[offset..offset + 32]);
        offset += 32;
        
        let recipient_chain = u16::from_le_bytes([
            payload[offset],
            payload[offset + 1],
        ]);
        offset += 2;
        
        let fee = u64::from_le_bytes([
            payload[offset],
            payload[offset + 1],
            payload[offset + 2],
            payload[offset + 3],
            payload[offset + 4],
            payload[offset + 5],
            payload[offset + 6],
            payload[offset + 7],
        ]);
        
        Ok(TransferPayload {
            payload_id,
            amount,
            token_address,
            token_chain,
            recipient,
            recipient_chain,
            fee,
        })
    }
}

/// Parsed transfer payload
#[derive(Clone)]
pub struct TransferPayload {
    pub payload_id: u8,
    pub amount: u64,
    pub token_address: [u8; 32],
    pub token_chain: u16,
    pub recipient: [u8; 32],
    pub recipient_chain: u16,
    pub fee: u64,
}
