# Test Suite Documentation

This directory contains comprehensive test cases for the Solana Cross-Chain Bridge program.

## Test Coverage

The test suite covers the following functionality:

### Core Functionality Tests

1. **Bridge Initialization**
   - Initialize bridge state with configuration
   - Verify all parameters are set correctly

2. **Guardian Set Management**
   - Initialize first guardian set
   - Update guardian set with new guardians
   - Verify quorum requirements

3. **Wrapped Asset Registration**
   - Register wrapped assets for foreign tokens
   - Support for Token-2022 (future)
   - Verify metadata storage

4. **Token Locking**
   - Lock tokens for cross-chain transfer
   - Calculate and deduct bridge fees
   - Create transfer records
   - Verify sequence number increment

5. **VAA Posting**
   - Post VAA to bridge storage
   - Verify VAA parsing and validation
   - Check hash calculation

6. **ZK Proof Submission**
   - Submit ZK proofs for transfers
   - Verify proof validation
   - Test commitment hashing

7. **Bridge Administration**
   - Update bridge configuration
   - Pause/unpause bridge
   - Verify access control

### Error Handling Tests

1. **Bridge Paused**
   - Verify operations fail when bridge is paused
   - Test proper error codes

2. **Minimum Amount Validation**
   - Verify transfers below minimum amount fail
   - Test proper error messages

## Running Tests

```bash
# Install dependencies
npm install

# Run all tests
anchor test

# Run with verbose output
anchor test -- --nocapture

# Run specific test file
anchor test tests/solana-cross-chain-bridge-wormhole.ts
```

## Test Setup

The test suite:
- Creates test accounts (authority, users, relayers, guardians)
- Sets up test token mints and accounts
- Initializes bridge and guardian sets
- Provides helper functions for common operations

## Mock Data

Some test data is mocked for demonstration:
- **VAA Signatures**: Guardian signatures are mocked (in production, these would be real ED25519 signatures)
- **ZK Proofs**: ZK proofs are mocked (in production, these would be real circuit proofs)
- **Cross-Chain Addresses**: Ethereum addresses are padded to 32 bytes

## Notes

- Tests use a local validator (automatically started by Anchor)
- All accounts are created fresh for each test run
- Some operations require sequential execution (e.g., initialize bridge before locking tokens)
- Guardian signature verification is stubbed in the program (needs real implementation for production)

## Future Test Additions

- Complete transfer flow (lock → VAA → verify → complete)
- Guardian signature verification with real signatures
- ZK proof verification with real circuits
- Relayer reward distribution
- Fee collection
- Wrapped token minting/burning
- Multi-chain transfer scenarios
- Replay attack prevention
- Edge cases and stress tests
