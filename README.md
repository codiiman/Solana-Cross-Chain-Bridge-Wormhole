# Solana Cross-Chain Bridge (Wormhole-Inspired with ZK Proofs)

[![Anchor](https://img.shields.io/badge/Anchor-0.30.1-blue)](https://www.anchor-lang.com/)
[![Solana](https://img.shields.io/badge/Solana-1.18-purple)](https://solana.com/)
[![Rust](https://img.shields.io/badge/Rust-2021-orange)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-MIT-green)](LICENSE)

> **Wormhole-inspired cross-chain bridge with ZK proof integration for secure, verifiable DeFi/RWA transfers**

A production-grade Solana smart contract for cross-chain token transfers, inspired by Wormhole's core protocol architecture. Features guardian attestations, VAA (Verifiable Action Approval) messages, zero-knowledge proof integration, and support for wrapped assets using Token-2022.

## 📋 Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Architecture](#architecture)
- [Key Algorithms](#key-algorithms)
- [Installation](#installation)
- [Build & Test](#build--test)
- [Deployment](#deployment)
- [Usage Examples](#usage-examples)
- [Contact & Contributions](#contact)

## 🎯 Overview

This program implements a sophisticated cross-chain bridge protocol that enables secure token transfers between Solana and other blockchains (Ethereum, BSC, Polygon, Avalanche, etc.). The bridge uses:

- **Guardian Network**: A set of trusted validators that attest to cross-chain messages
- **VAA System**: Verifiable Action Approvals that contain guardian signatures
- **ZK Proofs**: Zero-knowledge proofs for privacy-preserving transfers and proof of solvency
- **Wrapped Assets**: Native representation of foreign tokens on Solana
- **Permissionless Relaying**: Anyone can relay messages to complete transfers

Users can lock tokens on the source chain and receive wrapped tokens on the target chain, with full security guarantees through guardian attestations and optional ZK verification.

## ✨ Features

### Core Functionality

- **Cross-Chain Transfers**: Lock tokens on source chain, mint wrapped tokens on target chain
- **VAA System**: Verifiable Action Approvals with guardian signatures for message verification
- **Guardian Network**: Configurable set of guardians with quorum-based attestation
- **ZK Proof Integration**: Support for zero-knowledge proofs for private transfers and solvency proofs
- **Wrapped Assets**: Automatic creation and management of wrapped token representations
- **Token-2022 Support**: Enhanced token features for RWA and compliance use cases
- **Permissionless Relaying**: Anyone can relay VAAs to complete transfers
- **Replay Protection**: Sequence numbers and VAA hash tracking prevent double-spending
- **Fee Management**: Bridge fees and relayer rewards with configurable parameters

### Supported Chains

Following Wormhole's chain ID standard:
- Solana (1)
- Ethereum (2)
- BSC (4)
- Polygon (5)
- Avalanche (6)
- Fantom (10)
- Celo (14)
- NEAR (15)
- Moonbeam (16)
- Arbitrum (23)
- Optimism (24)
- Base (30)

### Security Features

- ✅ Guardian signature verification with quorum requirements
- ✅ VAA timestamp validation and expiration checks
- ✅ Replay protection via sequence numbers
- ✅ ZK proof verification for enhanced security
- ✅ Overflow/underflow protection with checked math
- ✅ Authority validation on privileged operations
- ✅ Pause functionality for emergency stops
- ✅ Comprehensive error handling

## 🏗️ Architecture

### Account Structure

```
CoreBridgeState (Program-wide configuration)
├── authority: Pubkey
├── guardian_set_index: u32
├── fee_recipient: Pubkey
├── min_bridge_fee: u64
├── max_bridge_fee: u64
├── relayer_reward_bps: u16
├── sequence: u64 (replay protection)
└── zk_verifier_program: Pubkey

GuardianSet (Guardian network configuration)
├── index: u32
├── keys: Vec<Pubkey> (max 19 guardians)
├── quorum: u8 (signature threshold)
└── expiration_time: i64

VaaAccount (Stored VAA data)
├── vaa_hash: [u8; 32]
├── emitter_chain: u16
├── emitter_address: [u8; 32]
├── sequence: u64
├── guardian_set_index: u32
├── payload: Vec<u8>
└── processed: bool

TransferAccount (Cross-chain transfer record)
├── transfer_id: [u8; 32]
├── source_chain: u16
├── target_chain: u16
├── token_mint: Pubkey
├── amount: u64
├── recipient: [u8; 32]
├── vaa_hash: Option<[u8; 32]>
├── zk_proof_id: Option<[u8; 32]>
└── status: u8 (pending/completed/failed)

WrappedAsset (Wrapped token metadata)
├── source_chain: u16
├── source_token: [u8; 32]
├── wrapped_mint: Pubkey
├── decimals: u8
├── symbol: String
├── name: String
└── total_supply: u64
```

### Instruction Flow

```
┌─────────────────┐
│  Initialize     │ → Setup bridge & guardian set
└─────────────────┘

┌─────────────────┐
│  Lock Tokens    │ → Transfer to bridge → Create transfer record
└─────────────────┘
         │
         ↓ (Guardians observe & sign)
┌─────────────────┐
│  Post VAA       │ → Store VAA on target chain
└─────────────────┘
         │
         ↓
┌─────────────────┐
│ Verify Signatures│ → Validate guardian signatures
└─────────────────┘
         │
         ↓
┌─────────────────┐
│ Complete Transfer│ → Mint wrapped tokens → Pay relayer
└─────────────────┘

┌─────────────────┐
│ Submit ZK Proof │ → Verify ZK proof (optional, for privacy)
└─────────────────┘
```

### Module Structure

```
programs/solana-cross-chain-bridge-wormhole/src/
├── lib.rs              # Program entry point & instruction handlers
├── state.rs            # Account structs (CoreBridgeState, GuardianSet, VAA, etc.)
├── errors.rs           # Custom error codes
├── events.rs           # Event definitions
├── vaa.rs              # VAA parsing, validation, guardian verification
├── zk.rs               # ZK proof verification utilities
├── utils.rs            # Helper functions (fees, transfers, validation)
└── instructions/
    ├── initialize.rs    # Bridge & guardian set initialization
    ├── post_vaa.rs      # Post VAA to bridge
    ├── verify_signatures.rs # Verify guardian signatures
    ├── lock_tokens.rs   # Lock tokens on source chain
    ├── complete_transfer.rs # Complete transfer & mint wrapped tokens
    ├── submit_zk_proof.rs # Submit & verify ZK proofs
    ├── register_wrapped_asset.rs # Register wrapped asset
    ├── update_guardian_set.rs # Update guardian network
    └── admin.rs         # Bridge configuration & pause
```

## 🔢 Key Algorithms

### VAA Validation

**VAA Structure:**
```
Header (6 bytes):
  - version (1 byte)
  - guardian_set_index (4 bytes)
  - signatures_len (1 byte)

Signatures (66 bytes each):
  - guardian_index (1 byte)
  - ED25519 signature (65 bytes)

Body:
  - emitter_chain (2 bytes)
  - emitter_address (32 bytes)
  - sequence (8 bytes)
  - consistency_level (1 byte)
  - payload (variable)
```

**VAA Hash Calculation:**
```
vaa_hash = SHA256(body)
message_hash = SHA256("VAAMessage" || body)
```

**Guardian Verification:**
```
valid_signatures = 0
for each signature in VAA:
  if verify_ed25519(guardian_pubkey, message_hash, signature):
    valid_signatures++
  
require(valid_signatures >= quorum)
```

### Transfer ID Calculation

```
transfer_id = SHA256(
  "TransferID" ||
  source_chain ||
  target_chain ||
  token_mint ||
  amount ||
  recipient ||
  sequence
)
```

### Bridge Fee Calculation

```
fee = (amount × fee_bps) / 10000
fee = max(min_fee, min(max_fee, fee))
transfer_amount = amount - fee
```

### Relayer Reward

```
relayer_reward = (bridge_fee × relayer_reward_bps) / 10000
```

### ZK Proof Verification

**Public Inputs Format:**
```
- amount (8 bytes)
- source_chain (2 bytes)
- target_chain (2 bytes)
- recipient (32 bytes)
- token_address (32 bytes)
- commitment (32 bytes)
```

**Commitment Hash:**
```
commitment = SHA256("ZKCommitment" || public_inputs)
```

**Verification:**
```
1. Verify proof format & size
2. Extract public inputs
3. Calculate commitment hash
4. Call ZK verifier program (CPI)
5. Validate commitment matches
6. Check public inputs match transfer
```

## 🚀 Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable)
- [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) (v1.18+)
- [Anchor](https://www.anchor-lang.com/docs/installation) (v0.30.1+)
- [Node.js](https://nodejs.org/) (for tests)

### Setup

1. **Clone the repository:**
```bash
git clone <repository-url>
cd Solana-Cross-Chain-Bridge-Wormhole
```

2. **Install dependencies:**
```bash
anchor build
```

3. **Verify installation:**
```bash
anchor --version
solana --version
```

## 🔨 Build & Test

### Build the Program

```bash
# Build the program
anchor build

# Build with optimizations (for mainnet)
anchor build --release
```

### Run Tests

```bash
# Run all tests
anchor test

# Run tests with logs
anchor test -- --nocapture

# Run specific test file
anchor test tests/<test-file>.ts
```

### Test Coverage

The program includes comprehensive test scenarios:

- ✅ Bridge initialization and configuration
- ✅ Guardian set management
- ✅ VAA posting and parsing
- ✅ Guardian signature verification
- ✅ Token locking and transfer creation
- ✅ Transfer completion with wrapped token minting
- ✅ ZK proof submission and verification
- ✅ Wrapped asset registration
- ✅ Replay protection (sequence numbers)
- ✅ Fee calculation and relayer rewards
- ✅ Error handling and edge cases

## 📦 Deployment

### Localnet Deployment

```bash
# Start local validator
solana-test-validator

# In another terminal, deploy
anchor deploy

# Or use Anchor's test environment
anchor localnet
```

### Devnet Deployment

```bash
# Set cluster
solana config set --url devnet

# Airdrop SOL (if needed)
solana airdrop 2

# Deploy
anchor deploy --provider.cluster devnet
```

### Mainnet Deployment

⚠️ **WARNING**: This code is for educational purposes. Do not deploy to mainnet without comprehensive security audits.

```bash
# Set cluster
solana config set --url mainnet-beta

# Deploy (requires sufficient SOL for rent + fees)
anchor deploy --provider.cluster mainnet-beta
```

## 💻 Usage Examples

### Initialize Bridge

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaCrossChainBridgeWormhole } from "../target/types/solana_cross_chain_bridge_wormhole";

// Initialize bridge state
await program.methods
  .initializeBridge(
    new anchor.BN(1000),        // min_bridge_fee
    new anchor.BN(100000),     // max_bridge_fee
    500,                       // relayer_reward_bps (5%)
    new anchor.BN(0),          // max_transfer_amount (0 = unlimited)
    new anchor.BN(1000000),    // min_transfer_amount
    new anchor.BN(86400),      // vaa_expiration_time (24 hours)
    zkVerifierProgramId        // ZK verifier program ID
  )
  .accounts({
    bridgeState: bridgeStatePda,
    authority: authority.publicKey,
    feeRecipient: treasury.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();

// Initialize guardian set
const guardians = [
  guardian1.publicKey,
  guardian2.publicKey,
  guardian3.publicKey,
  // ... up to 19 guardians
];

await program.methods
  .initializeGuardianSet(
    guardians,
    2,                         // quorum (2 out of 3)
    new anchor.BN(0)           // expiration_time (0 = never)
  )
  .accounts({
    bridgeState: bridgeStatePda,
    guardianSet: guardianSetPda,
    authority: authority.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

### Lock Tokens for Cross-Chain Transfer

```typescript
// Lock 100 SOL for transfer to Ethereum
const amount = new anchor.BN(100 * 1e9); // 100 SOL
const targetChain = 2; // Ethereum
const recipient = Buffer.alloc(32); // Ethereum address (20 bytes + 12 zero padding)

await program.methods
  .lockTokens(
    amount,
    targetChain,
    recipient
  )
  .accounts({
    bridgeState: bridgeStatePda,
    userTokenAccount: userTokenAccount,
    bridgeTokenAccount: bridgeTokenAccount,
    feeTokenAccount: feeTokenAccount,
    tokenMint: solMint,
    transfer: transferPda,
    userAuthority: user.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  })
  .rpc();
```

### Post and Verify VAA

```typescript
// Post VAA (from guardian network)
const vaaBytes = Buffer.from(/* VAA bytes from guardians */);

await program.methods
  .postVaa(Array.from(vaaBytes))
  .accounts({
    bridgeState: bridgeStatePda,
    vaaAccount: vaaAccountPda,
    payer: payer.publicKey,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();

// Verify guardian signatures
await program.methods
  .verifySignatures(Array.from(vaaBytes))
  .accounts({
    bridgeState: bridgeStatePda,
    guardianSet: guardianSetPda,
    vaaAccount: vaaAccountPda,
    systemProgram: anchor.web3.SystemProgram.programId,
  })
  .rpc();
```

### Complete Transfer

```typescript
// Complete transfer and mint wrapped tokens
await program.methods
  .completeTransfer()
  .accounts({
    bridgeState: bridgeStatePda,
    vaaAccount: vaaAccountPda,
    transfer: transferPda,
    wrappedAsset: wrappedAssetPda,
    wrappedMint: wrappedMintPda,
    recipientTokenAccount: recipientTokenAccount,
    relayer: relayer.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

### Submit ZK Proof

```typescript
// Submit ZK proof for privacy-preserving transfer
const proof = Buffer.from(/* ZK proof bytes */);
const publicInputs = Buffer.from(/* Public inputs */);
const circuitId = 1; // TRANSFER_PROOF

await program.methods
  .submitZkProof(
    Array.from(proof),
    Array.from(publicInputs),
    circuitId
  )
  .accounts({
    bridgeState: bridgeStatePda,
    zkVerifier: zkVerifierPda,
    transfer: transferPda,
    verifier: verifier.publicKey,
  })
  .rpc();
```

### Register Wrapped Asset

```typescript
// Register wrapped asset for foreign token
const sourceChain = 2; // Ethereum
const sourceToken = Buffer.alloc(32); // Ethereum token address
const decimals = 18;
const symbol = "WETH";
const name = "Wrapped Ethereum";

await program.methods
  .registerWrappedAsset(
    sourceChain,
    Array.from(sourceToken),
    decimals,
    symbol,
    name,
    false // is_token2022
  )
  .accounts({
    bridgeState: bridgeStatePda,
    wrappedAsset: wrappedAssetPda,
    wrappedMint: wrappedMintPda,
    registrar: registrar.publicKey,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: anchor.web3.SystemProgram.programId,
    rent: anchor.web3.SYSVAR_RENT_PUBKEY,
  })
  .rpc();
```

## 📞 Contact & Contributions

- telegram: https://t.me/codiiman
- twitter:  https://x.com/codiiman_
