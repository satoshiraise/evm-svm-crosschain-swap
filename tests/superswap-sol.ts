import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SuperswapSol } from "../target/types/superswap_sol";
import { PublicKey, Keypair, SystemProgram, LAMPORTS_PER_SOL } from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  createAccount,
  mintTo,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("superswap-sol", () => {
  // Configure the client to use the local cluster
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SuperswapSol as Program<SuperswapSol>;
  
  // Test accounts
  const admin = provider.wallet;
  const acrossHandler = Keypair.generate();
  const feeRecipient = Keypair.generate();
  const user = Keypair.generate();
  
  let usdcMint: PublicKey;
  let destinationMint: PublicKey;
  let configPda: PublicKey;
  let configBump: number;

  // Jupiter program ID (mainnet)
  const jupiterProgramId = new PublicKey("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4");

  before(async () => {
    // Airdrop SOL to test accounts
    await provider.connection.requestAirdrop(
      user.publicKey,
      2 * LAMPORTS_PER_SOL
    );
    
    await provider.connection.requestAirdrop(
      acrossHandler.publicKey,
      2 * LAMPORTS_PER_SOL
    );

    await provider.connection.requestAirdrop(
      feeRecipient.publicKey,
      1 * LAMPORTS_PER_SOL
    );

    // Wait for airdrops to confirm
    await new Promise((resolve) => setTimeout(resolve, 1000));

    // Create USDC mint (6 decimals like real USDC)
    usdcMint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      6
    );

    // Create destination token mint
    destinationMint = await createMint(
      provider.connection,
      admin.payer,
      admin.publicKey,
      null,
      9
    );

    // Derive config PDA
    [configPda, configBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("config")],
      program.programId
    );
  });

  it("Initializes the program", async () => {
    const tx = await program.methods
      .initialize({
        acrossHandler: acrossHandler.publicKey,
        jupiterProgram: jupiterProgramId,
        usdcMint: usdcMint,
        feeRecipient: feeRecipient.publicKey,
        feeBps: 30, // 0.3% fee
      })
      .accounts({
        config: configPda,
        admin: admin.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    console.log("Initialize transaction:", tx);

    // Fetch and verify config
    const config = await program.account.config.fetch(configPda);
    assert.ok(config.admin.equals(admin.publicKey));
    assert.ok(config.acrossHandler.equals(acrossHandler.publicKey));
    assert.ok(config.jupiterProgram.equals(jupiterProgramId));
    assert.ok(config.usdcMint.equals(usdcMint));
    assert.ok(config.feeRecipient.equals(feeRecipient.publicKey));
    assert.equal(config.feeBps, 30);
    assert.equal(config.isPaused, false);
  });

  it("Updates config", async () => {
    const newFeeRecipient = Keypair.generate().publicKey;

    await program.methods
      .updateConfig({
        newAdmin: null,
        newAcrossHandler: null,
        newJupiterProgram: null,
        newFeeRecipient: newFeeRecipient,
        newFeeBps: 50,
      })
      .accounts({
        config: configPda,
        admin: admin.publicKey,
      })
      .rpc();

    const config = await program.account.config.fetch(configPda);
    assert.ok(config.feeRecipient.equals(newFeeRecipient));
    assert.equal(config.feeBps, 50);
  });

  it("Pauses the program", async () => {
    await program.methods
      .pause()
      .accounts({
        config: configPda,
        admin: admin.publicKey,
      })
      .rpc();

    const config = await program.account.config.fetch(configPda);
    assert.equal(config.isPaused, true);
  });

  it("Unpauses the program", async () => {
    await program.methods
      .unpause()
      .accounts({
        config: configPda,
        admin: admin.publicKey,
      })
      .rpc();

    const config = await program.account.config.fetch(configPda);
    assert.equal(config.isPaused, false);
  });

  describe("Process bridge and swap", () => {
    let sourceUsdcAccount: PublicKey;
    let programUsdcAccount: PublicKey;
    let recipientUsdcAccount: PublicKey;
    let recipientDestinationAccount: PublicKey;
    let feeRecipientAccount: PublicKey;
    let swapOrderPda: PublicKey;

    const orderId = Date.now();
    const usdcAmount = 1000000; // 1 USDC (6 decimals)
    const minOutputAmount = 950000; // 0.95 destination tokens

    before(async () => {
      // Create token accounts
      sourceUsdcAccount = await createAccount(
        provider.connection,
        admin.payer,
        usdcMint,
        acrossHandler.publicKey
      );

      // Mint USDC to source account (simulating Across bridge)
      await mintTo(
        provider.connection,
        admin.payer,
        usdcMint,
        sourceUsdcAccount,
        admin.publicKey,
        usdcAmount
      );

      // Get associated token accounts
      programUsdcAccount = await anchor.utils.token.associatedAddress({
        mint: usdcMint,
        owner: configPda,
      });

      recipientUsdcAccount = await anchor.utils.token.associatedAddress({
        mint: usdcMint,
        owner: user.publicKey,
      });

      recipientDestinationAccount = await anchor.utils.token.associatedAddress({
        mint: destinationMint,
        owner: user.publicKey,
      });

      feeRecipientAccount = await anchor.utils.token.associatedAddress({
        mint: usdcMint,
        owner: feeRecipient.publicKey,
      });

      // Derive swap order PDA
      [swapOrderPda] = PublicKey.findProgramAddressSync(
        [
          Buffer.from("swap_order"),
          new anchor.BN(orderId).toArrayLike(Buffer, "le", 8),
        ],
        program.programId
      );
    });

    it("Processes bridge and swap", async () => {
      const deadline = Math.floor(Date.now() / 1000) + 300; // 5 minutes from now

      // Create mock Jupiter swap data
      // In production, this would come from Jupiter API
      const jupiterSwapData = Buffer.from([
        // Mock instruction data
        // In real implementation, this would be generated by Jupiter SDK
      ]);

      const config = await program.account.config.fetch(configPda);

      const tx = await program.methods
        .processBridgeAndSwap({
          orderId: new anchor.BN(orderId),
          recipient: user.publicKey,
          usdcAmount: new anchor.BN(usdcAmount),
          minOutputAmount: new anchor.BN(minOutputAmount),
          destinationMint: destinationMint,
          deadline: new anchor.BN(deadline),
          jupiterSwapData: jupiterSwapData,
        })
        .accounts({
          config: configPda,
          swapOrder: swapOrderPda,
          acrossHandler: acrossHandler.publicKey,
          recipient: user.publicKey,
          usdcMint: usdcMint,
          sourceUsdcAccount: sourceUsdcAccount,
          programUsdcAccount: programUsdcAccount,
          destinationMint: destinationMint,
          recipientDestinationAccount: recipientDestinationAccount,
          recipientUsdcAccount: recipientUsdcAccount,
          feeRecipientAccount: feeRecipientAccount,
          jupiterProgram: config.jupiterProgram,
          payer: admin.publicKey,
          tokenProgram: TOKEN_PROGRAM_ID,
          associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([acrossHandler])
        .rpc();

      console.log("Process bridge and swap transaction:", tx);

      // Verify swap order was created
      const swapOrder = await program.account.swapOrder.fetch(swapOrderPda);
      assert.equal(swapOrder.orderId.toNumber(), orderId);
      assert.ok(swapOrder.recipient.equals(user.publicKey));
      assert.equal(swapOrder.usdcAmount.toNumber(), usdcAmount);
      assert.equal(swapOrder.minOutputAmount.toNumber(), minOutputAmount);
      assert.ok(swapOrder.destinationMint.equals(destinationMint));

      // Verify fee was collected
      const feeAccount = await getAccount(
        provider.connection,
        feeRecipientAccount
      );
      const expectedFee = Math.floor((usdcAmount * config.feeBps) / 10000);
      assert.equal(Number(feeAccount.amount), expectedFee);
    });
  });
});

