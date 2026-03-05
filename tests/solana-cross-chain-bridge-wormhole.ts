import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SolanaCrossChainBridgeWormhole } from "../target/types/solana_cross_chain_bridge_wormhole";
import { 
  TOKEN_PROGRAM_ID, 
  getAssociatedTokenAddress,
  createAssociatedTokenAccountInstruction,
  createMint,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { 
  PublicKey, 
  Keypair, 
  SystemProgram, 
  SYSVAR_RENT_PUBKEY,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";
import { expect } from "chai";
import { sha256 } from "@noble/hashes/sha256";

describe("solana-cross-chain-bridge-wormhole", () => {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SolanaCrossChainBridgeWormhole as Program<SolanaCrossChainBridgeWormhole>;
  
  // Test accounts
  const authority = provider.wallet;
  const feeRecipient = Keypair.generate();
  const user = Keypair.generate();
  const relayer = Keypair.generate();
  const registrar = Keypair.generate();
  
  // Guardian keys
  const guardian1 = Keypair.generate();
  const guardian2 = Keypair.generate();
  const guardian3 = Keypair.generate();
  
  // PDAs
  let bridgeStatePda: PublicKey;
  let bridgeStateBump: number;
  let guardianSetPda: PublicKey;
  let guardianSetBump: number;
  let zkVerifierPda: PublicKey;
  let zkVerifierBump: number;
  
  // Test token
  let testTokenMint: PublicKey;
  let bridgeTokenAccount: PublicKey;
  let userTokenAccount: PublicKey;
  let feeTokenAccount: PublicKey;
  
  // Wrapped asset
  let wrappedAssetPda: PublicKey;
  let wrappedAssetBump: number;
  let wrappedMintPda: PublicKey;
  let wrappedMintBump: number;
  
  // Transfer
  let transferPda: PublicKey;
  let transferBump: number;
  
  // VAA
  let vaaAccountPda: PublicKey;
  let vaaAccountBump: number;

  before(async () => {
    // Airdrop SOL to test accounts
    const airdropAmount = 10 * LAMPORTS_PER_SOL;
    await provider.connection.requestAirdrop(feeRecipient.publicKey, airdropAmount);
    await provider.connection.requestAirdrop(user.publicKey, airdropAmount);
    await provider.connection.requestAirdrop(relayer.publicKey, airdropAmount);
    await provider.connection.requestAirdrop(registrar.publicKey, airdropAmount);
    
    // Wait for airdrops to confirm
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    // Find PDAs
    [bridgeStatePda, bridgeStateBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("bridge_state")],
      program.programId
    );
    
    [guardianSetPda, guardianSetBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("guardian_set"), new anchor.BN(0).toArrayLike(Buffer, "le", 4)],
      program.programId
    );
    
    [zkVerifierPda, zkVerifierBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("zk_verifier")],
      program.programId
    );
    
    // Create test token mint
    testTokenMint = await createMint(
      provider.connection,
      user,
      user.publicKey,
      null,
      9
    );
    
    // Create token accounts
    userTokenAccount = await getAssociatedTokenAddress(
      testTokenMint,
      user.publicKey
    );
    
    bridgeTokenAccount = await getAssociatedTokenAddress(
      testTokenMint,
      bridgeStatePda,
      true
    );
    
    feeTokenAccount = await getAssociatedTokenAddress(
      testTokenMint,
      feeRecipient.publicKey
    );
    
    // Create user token account if it doesn't exist
    try {
      await getAccount(provider.connection, userTokenAccount);
    } catch {
      const createIx = createAssociatedTokenAccountInstruction(
        user.publicKey,
        userTokenAccount,
        user.publicKey,
        testTokenMint
      );
      const tx = new anchor.web3.Transaction().add(createIx);
      await provider.sendAndConfirm(tx);
    }
    
    // Mint tokens to user
    await mintTo(
      provider.connection,
      user,
      testTokenMint,
      userTokenAccount,
      user.publicKey,
      1000 * 1e9 // 1000 tokens
    );
  });

  it("Initializes the bridge", async () => {
    const minBridgeFee = new anchor.BN(1000);
    const maxBridgeFee = new anchor.BN(100000);
    const relayerRewardBps = 500; // 5%
    const maxTransferAmount = new anchor.BN(0); // Unlimited
    const minTransferAmount = new anchor.BN(1000000); // 0.001 tokens
    const vaaExpirationTime = new anchor.BN(86400); // 24 hours
    const zkVerifierProgram = SystemProgram.programId; // Stub for testing

    const tx = await program.methods
      .initializeBridge(
        minBridgeFee,
        maxBridgeFee,
        relayerRewardBps,
        maxTransferAmount,
        minTransferAmount,
        vaaExpirationTime,
        zkVerifierProgram
      )
      .accounts({
        bridgeState: bridgeStatePda,
        authority: authority.publicKey,
        feeRecipient: feeRecipient.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Bridge initialized:", tx);

    // Verify bridge state
    const bridgeState = await program.account.coreBridgeState.fetch(bridgeStatePda);
    expect(bridgeState.authority.toString()).to.equal(authority.publicKey.toString());
    expect(bridgeState.feeRecipient.toString()).to.equal(feeRecipient.publicKey.toString());
    expect(bridgeState.minBridgeFee.toNumber()).to.equal(minBridgeFee.toNumber());
    expect(bridgeState.maxBridgeFee.toNumber()).to.equal(maxBridgeFee.toNumber());
    expect(bridgeState.relayerRewardBps).to.equal(relayerRewardBps);
    expect(bridgeState.paused).to.be.false;
  });

  it("Initializes guardian set", async () => {
    const guardians = [
      guardian1.publicKey,
      guardian2.publicKey,
      guardian3.publicKey,
    ];
    const quorum = 2; // 2 out of 3
    const expirationTime = new anchor.BN(0); // Never expires

    const tx = await program.methods
      .initializeGuardianSet(
        guardians,
        quorum,
        expirationTime
      )
      .accounts({
        bridgeState: bridgeStatePda,
        guardianSet: guardianSetPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Guardian set initialized:", tx);

    // Verify guardian set
    const guardianSet = await program.account.guardianSet.fetch(guardianSetPda);
    expect(guardianSet.index).to.equal(0);
    expect(guardianSet.keys.length).to.equal(3);
    expect(guardianSet.quorum).to.equal(quorum);
    expect(guardianSet.keys[0].toString()).to.equal(guardian1.publicKey.toString());
  });

  it("Registers a wrapped asset", async () => {
    const sourceChain = 2; // Ethereum
    const sourceToken = Buffer.alloc(32);
    // Ethereum address (20 bytes) padded to 32 bytes
    const ethAddress = "1234567890123456789012345678901234567890";
    sourceToken.write(ethAddress, 12, "hex"); // Write at offset 12 (32-20=12)
    
    const decimals = 18;
    const symbol = "WETH";
    const name = "Wrapped Ethereum";
    const isToken2022 = false;

    [wrappedAssetPda, wrappedAssetBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("wrapped_asset"),
        Buffer.from([sourceChain, 0]),
        sourceToken,
      ],
      program.programId
    );

    [wrappedMintPda, wrappedMintBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("wrapped_mint"), wrappedAssetPda.toBuffer()],
      program.programId
    );

    const tx = await program.methods
      .registerWrappedAsset(
        sourceChain,
        Array.from(sourceToken),
        decimals,
        symbol,
        name,
        isToken2022
      )
      .accounts({
        bridgeState: bridgeStatePda,
        wrappedAsset: wrappedAssetPda,
        wrappedMint: wrappedMintPda,
        registrar: registrar.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([registrar])
      .rpc();

    console.log("Wrapped asset registered:", tx);

    // Verify wrapped asset
    const wrappedAsset = await program.account.wrappedAsset.fetch(wrappedAssetPda);
    expect(wrappedAsset.sourceChain).to.equal(sourceChain);
    expect(wrappedAsset.symbol).to.equal(symbol);
    expect(wrappedAsset.name).to.equal(name);
    expect(wrappedAsset.decimals).to.equal(decimals);
  });

  it("Locks tokens for cross-chain transfer", async () => {
    const amount = new anchor.BN(100 * 1e9); // 100 tokens
    const targetChain = 2; // Ethereum
    const recipient = Buffer.alloc(32);
    // Ethereum address (20 bytes) padded to 32 bytes
    const ethRecipient = "abcdefabcdefabcdefabcdefabcdefabcdefabcd";
    recipient.write(ethRecipient, 12, "hex"); // Write at offset 12 (32-20=12)

    // Get current sequence
    const bridgeState = await program.account.coreBridgeState.fetch(bridgeStatePda);
    const sequence = bridgeState.sequence;

    [transferPda, transferBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("transfer"),
        bridgeStatePda.toBuffer(),
        sequence.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );

    // Create bridge token account if needed
    try {
      await getAccount(provider.connection, bridgeTokenAccount);
    } catch {
      const createIx = createAssociatedTokenAccountInstruction(
        user.publicKey,
        bridgeTokenAccount,
        bridgeStatePda,
        testTokenMint
      );
      const tx = new anchor.web3.Transaction().add(createIx);
      await provider.sendAndConfirm(tx);
    }

    // Create fee token account if needed
    try {
      await getAccount(provider.connection, feeTokenAccount);
    } catch {
      const createIx = createAssociatedTokenAccountInstruction(
        user.publicKey,
        feeTokenAccount,
        feeRecipient.publicKey,
        testTokenMint
      );
      const tx = new anchor.web3.Transaction().add(createIx);
      await provider.sendAndConfirm(tx);
    }

    const tx = await program.methods
      .lockTokens(
        amount,
        targetChain,
        Array.from(recipient)
      )
      .accounts({
        bridgeState: bridgeStatePda,
        userTokenAccount: userTokenAccount,
        bridgeTokenAccount: bridgeTokenAccount,
        feeTokenAccount: feeTokenAccount,
        tokenMint: testTokenMint,
        transfer: transferPda,
        userAuthority: user.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([user])
      .rpc();

    console.log("Tokens locked:", tx);

    // Verify transfer account
    const transfer = await program.account.transferAccount.fetch(transferPda);
    expect(transfer.sourceChain).to.equal(1); // Solana
    expect(transfer.targetChain).to.equal(targetChain);
    expect(transfer.amount.toNumber()).to.be.lessThan(amount.toNumber()); // After fee
    expect(transfer.status).to.equal(0); // PENDING

    // Verify bridge state sequence incremented
    const updatedBridgeState = await program.account.coreBridgeState.fetch(bridgeStatePda);
    expect(updatedBridgeState.sequence.toNumber()).to.equal(sequence.toNumber() + 1);
  });

  it("Posts a VAA", async () => {
    // Create a mock VAA (simplified for testing)
    // In production, this would come from guardian network
    const vaaVersion = 1;
    const guardianSetIndex = 0;
    const signaturesLen = 2; // 2 signatures
    
    // Mock VAA structure
    const emitterChain = 1; // Solana
    const emitterAddress = Buffer.alloc(32);
    emitterAddress.write(program.programId.toBase58().slice(0, 32), 0);
    
    const sequence = new anchor.BN(0);
    const consistencyLevel = 200;
    
    // Mock payload (transfer payload)
    const payloadId = 1; // Token transfer
    const amount = new anchor.BN(95 * 1e9); // After fee
    const tokenAddress = Buffer.alloc(32);
    testTokenMint.toBuffer().copy(tokenAddress);
    
    const tokenChain = 1; // Solana
    const recipient = Buffer.alloc(32);
    // Ethereum address (20 bytes) padded to 32 bytes
    const ethRecipient = "abcdefabcdefabcdefabcdefabcdefabcdefabcd";
    recipient.write(ethRecipient, 12, "hex"); // Write at offset 12
    const recipientChain = 2; // Ethereum
    const fee = new anchor.BN(5 * 1e9); // 5 tokens fee

    // Build VAA body
    const body = Buffer.concat([
      Buffer.from([emitterChain, 0]), // emitter_chain (2 bytes)
      emitterAddress, // emitter_address (32 bytes)
      sequence.toArrayLike(Buffer, "le", 8), // sequence (8 bytes)
      Buffer.from([consistencyLevel]), // consistency_level (1 byte)
      Buffer.from([payloadId]), // payload_id (1 byte)
      amount.toArrayLike(Buffer, "le", 8), // amount (8 bytes)
      tokenAddress, // token_address (32 bytes)
      Buffer.from([tokenChain, 0]), // token_chain (2 bytes)
      recipient, // recipient (32 bytes)
      Buffer.from([recipientChain, 0]), // recipient_chain (2 bytes)
      fee.toArrayLike(Buffer, "le", 8), // fee (8 bytes)
    ]);

    // Calculate VAA hash (SHA256 of body)
    const vaaHash = sha256(body);
    const vaaHashArray = Array.from(new Uint8Array(vaaHash));

    // Build full VAA (simplified - in production would include actual signatures)
    const vaaHeader = Buffer.concat([
      Buffer.from([vaaVersion]),
      Buffer.from([guardianSetIndex, 0, 0, 0]),
      Buffer.from([signaturesLen]),
    ]);
    
    // Mock signatures (66 bytes each: 1 byte index + 65 bytes signature)
    const mockSignatures = Buffer.alloc(signaturesLen * 66);
    for (let i = 0; i < signaturesLen; i++) {
      mockSignatures[i * 66] = i; // Guardian index
      // Rest would be actual signature in production
    }
    
    const vaaBytes = Buffer.concat([vaaHeader, mockSignatures, body]);

    [vaaAccountPda, vaaAccountBump] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("vaa"),
        bridgeStatePda.toBuffer(),
        Buffer.from(vaaHashArray),
      ],
      program.programId
    );

    const tx = await program.methods
      .postVaa(
        Array.from(vaaBytes),
        Array.from(vaaHashArray)
      )
      .accounts({
        bridgeState: bridgeStatePda,
        vaaAccount: vaaAccountPda,
        payer: user.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .signers([user])
      .rpc();

    console.log("VAA posted:", tx);

    // Verify VAA account
    const vaaAccount = await program.account.vaaAccount.fetch(vaaAccountPda);
    expect(vaaAccount.emitterChain).to.equal(emitterChain);
    expect(vaaAccount.sequence.toNumber()).to.equal(sequence.toNumber());
    expect(vaaAccount.processed).to.be.false;
  });

  it("Initializes ZK verifier", async () => {
    const supportedCircuits = [1, 2, 3]; // TRANSFER_PROOF, PRIVATE_TRANSFER, SOLVENCY_PROOF
    const publicInputSizes = [108, 108, 64]; // Sizes in bytes
    const zkRequired = false;
    const proofExpirationTime = new anchor.BN(3600); // 1 hour

    const tx = await program.methods
      .initializeZkVerifier(
        supportedCircuits,
        publicInputSizes,
        zkRequired,
        proofExpirationTime
      )
      .accounts({
        bridgeState: bridgeStatePda,
        zkVerifier: zkVerifierPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("ZK verifier initialized:", tx);

    // Verify ZK verifier state
    const zkVerifier = await program.account.zkVerifierState.fetch(zkVerifierPda);
    expect(zkVerifier.supportedCircuits.length).to.equal(3);
    expect(zkVerifier.zkRequired).to.be.false;
  });

  it("Submits a ZK proof", async () => {
    // Mock ZK proof (in production, this would be a real proof)
    const proof = Buffer.alloc(256); // Mock proof bytes
    proof.write("mock_zk_proof_data_for_testing", 0);
    
    // Mock public inputs
    const amount = new anchor.BN(95 * 1e9);
    const sourceChain = 1;
    const targetChain = 2;
    const recipient = Buffer.alloc(32);
    // Ethereum address (20 bytes) padded to 32 bytes
    const ethRecipient = "abcdefabcdefabcdefabcdefabcdefabcdefabcd";
    recipient.write(ethRecipient, 12, "hex"); // Write at offset 12
    const tokenAddress = Buffer.alloc(32);
    testTokenMint.toBuffer().copy(tokenAddress);
    
    const commitmentInput = Buffer.concat([
      Buffer.from("ZKCommitment"),
      amount.toArrayLike(Buffer, "le", 8),
      Buffer.from([sourceChain, 0]),
      Buffer.from([targetChain, 0]),
      recipient,
      tokenAddress,
    ]);
    const commitment = sha256(commitmentInput);
    
    const publicInputs = Buffer.concat([
      amount.toArrayLike(Buffer, "le", 8),
      Buffer.from([sourceChain, 0]),
      Buffer.from([targetChain, 0]),
      recipient,
      tokenAddress,
      Buffer.from(commitment),
    ]);
    
    const circuitId = 1; // TRANSFER_PROOF

    const tx = await program.methods
      .submitZkProof(
        Array.from(proof),
        Array.from(publicInputs),
        circuitId
      )
      .accounts({
        bridgeState: bridgeStatePda,
        zkVerifier: zkVerifierPda,
        transfer: transferPda,
        verifier: relayer.publicKey,
      })
      .signers([relayer])
      .rpc();

    console.log("ZK proof submitted:", tx);

    // Verify transfer updated with ZK proof ID
    const transfer = await program.account.transferAccount.fetch(transferPda);
    expect(transfer.zkProofId).to.not.be.null;
  });

  it("Updates bridge configuration", async () => {
    const newMinBridgeFee = new anchor.BN(2000);
    const newMaxBridgeFee = new anchor.BN(200000);

    const tx = await program.methods
      .updateBridgeConfig(
        newMinBridgeFee,
        newMaxBridgeFee,
        null, // relayer_reward_bps
        null, // max_transfer_amount
        null, // min_transfer_amount
        null  // vaa_expiration_time
      )
      .accounts({
        bridgeState: bridgeStatePda,
        authority: authority.publicKey,
      })
      .rpc();

    console.log("Bridge config updated:", tx);

    // Verify update
    const bridgeState = await program.account.coreBridgeState.fetch(bridgeStatePda);
    expect(bridgeState.minBridgeFee.toNumber()).to.equal(newMinBridgeFee.toNumber());
    expect(bridgeState.maxBridgeFee.toNumber()).to.equal(newMaxBridgeFee.toNumber());
  });

  it("Pauses and unpauses the bridge", async () => {
    // Pause
    let tx = await program.methods
      .setBridgePaused(true)
      .accounts({
        bridgeState: bridgeStatePda,
        authority: authority.publicKey,
      })
      .rpc();

    console.log("Bridge paused:", tx);

    let bridgeState = await program.account.coreBridgeState.fetch(bridgeStatePda);
    expect(bridgeState.paused).to.be.true;

    // Unpause
    tx = await program.methods
      .setBridgePaused(false)
      .accounts({
        bridgeState: bridgeStatePda,
        authority: authority.publicKey,
      })
      .rpc();

    console.log("Bridge unpaused:", tx);

    bridgeState = await program.account.coreBridgeState.fetch(bridgeStatePda);
    expect(bridgeState.paused).to.be.false;
  });

  it("Updates guardian set", async () => {
    // Add a new guardian
    const newGuardians = [
      guardian1.publicKey,
      guardian2.publicKey,
      guardian3.publicKey,
      Keypair.generate().publicKey, // New guardian
    ];
    const newQuorum = 3; // 3 out of 4
    const expirationTime = new anchor.BN(0);

    const newGuardianSetIndex = 1;
    const [newGuardianSetPda] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("guardian_set"),
        new anchor.BN(newGuardianSetIndex).toArrayLike(Buffer, "le", 4),
      ],
      program.programId
    );

    const tx = await program.methods
      .updateGuardianSet(
        newGuardians,
        newQuorum,
        expirationTime
      )
      .accounts({
        bridgeState: bridgeStatePda,
        oldGuardianSet: guardianSetPda,
        newGuardianSet: newGuardianSetPda,
        authority: authority.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Guardian set updated:", tx);

    // Verify new guardian set
    const newGuardianSet = await program.account.guardianSet.fetch(newGuardianSetPda);
    expect(newGuardianSet.index).to.equal(newGuardianSetIndex);
    expect(newGuardianSet.keys.length).to.equal(4);
    expect(newGuardianSet.quorum).to.equal(newQuorum);

    // Verify bridge state updated
    const bridgeState = await program.account.coreBridgeState.fetch(bridgeStatePda);
    expect(bridgeState.guardianSetIndex).to.equal(newGuardianSetIndex);
  });

  it("Fails to lock tokens when bridge is paused", async () => {
    // Pause bridge
    await program.methods
      .setBridgePaused(true)
      .accounts({
        bridgeState: bridgeStatePda,
        authority: authority.publicKey,
      })
      .rpc();

    const amount = new anchor.BN(50 * 1e9);
    const targetChain = 2;
    const recipient = Buffer.alloc(32);

    try {
      await program.methods
        .lockTokens(
          amount,
          targetChain,
          Array.from(recipient)
        )
        .accounts({
          bridgeState: bridgeStatePda,
          userTokenAccount: userTokenAccount,
          bridgeTokenAccount: bridgeTokenAccount,
          feeTokenAccount: feeTokenAccount,
          tokenMint: testTokenMint,
          transfer: Keypair.generate().publicKey, // New transfer
          userAuthority: user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([user])
        .rpc();

      expect.fail("Should have thrown an error");
    } catch (err) {
      expect(err.error.errorCode.code).to.equal("BridgePaused");
    }

    // Unpause for other tests
    await program.methods
      .setBridgePaused(false)
      .accounts({
        bridgeState: bridgeStatePda,
        authority: authority.publicKey,
      })
      .rpc();
  });

  it("Fails to lock tokens below minimum amount", async () => {
    const bridgeState = await program.account.coreBridgeState.fetch(bridgeStatePda);
    const minAmount = bridgeState.minTransferAmount;
    const amountBelowMin = minAmount.sub(new anchor.BN(1));
    const targetChain = 2;
    const recipient = Buffer.alloc(32);
    // Ethereum address (20 bytes) padded to 32 bytes
    const ethRecipient = "abcdefabcdefabcdefabcdefabcdefabcdefabcd";
    recipient.write(ethRecipient, 12, "hex"); // Write at offset 12

    try {
      await program.methods
        .lockTokens(
          amountBelowMin,
          targetChain,
          Array.from(recipient)
        )
        .accounts({
          bridgeState: bridgeStatePda,
          userTokenAccount: userTokenAccount,
          bridgeTokenAccount: bridgeTokenAccount,
          feeTokenAccount: feeTokenAccount,
          tokenMint: testTokenMint,
          transfer: Keypair.generate().publicKey,
          userAuthority: user.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([user])
        .rpc();

      expect.fail("Should have thrown an error");
    } catch (err) {
      expect(err.error.errorCode.code).to.equal("TransferAmountTooSmall");
    }
  });
});
