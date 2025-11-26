import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SuperswapSol } from "../target/types/superswap_sol";
import { PublicKey, SystemProgram } from "@solana/web3.js";

/**
 * Script to initialize the SuperSwap program
 * 
 * Usage:
 *   ts-node scripts/initialize.ts
 */

async function main() {
  // Configure the client
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.SuperswapSol as Program<SuperswapSol>;
  
  console.log("Program ID:", program.programId.toString());
  console.log("Provider:", provider.wallet.publicKey.toString());

  // Configuration parameters
  const config = {
    // Replace with actual Across handler for your deployment
    acrossHandler: new PublicKey("AcrossHandlerPubkeyHere111111111111111111111"),
    
    // Jupiter V6 program ID (mainnet)
    jupiterProgram: new PublicKey("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"),
    
    // USDC mint (mainnet)
    usdcMint: new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    
    // Fee recipient (replace with your address)
    feeRecipient: provider.wallet.publicKey,
    
    // Fee in basis points (30 = 0.3%)
    feeBps: 30,
  };

  // Derive config PDA
  const [configPda, configBump] = PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    program.programId
  );

  console.log("\nConfig PDA:", configPda.toString());
  console.log("Config Bump:", configBump);

  // Check if already initialized
  try {
    const existingConfig = await program.account.config.fetch(configPda);
    console.log("\n⚠️  Program already initialized!");
    console.log("Existing config:", {
      admin: existingConfig.admin.toString(),
      acrossHandler: existingConfig.acrossHandler.toString(),
      jupiterProgram: existingConfig.jupiterProgram.toString(),
      usdcMint: existingConfig.usdcMint.toString(),
      feeRecipient: existingConfig.feeRecipient.toString(),
      feeBps: existingConfig.feeBps,
      isPaused: existingConfig.isPaused,
    });
    return;
  } catch (err) {
    // Not initialized, continue
    console.log("\nInitializing program...");
  }

  // Initialize the program
  const tx = await program.methods
    .initialize(config)
    .accounts({
      config: configPda,
      admin: provider.wallet.publicKey,
      systemProgram: SystemProgram.programId,
    })
    .rpc();

  console.log("\n✅ Program initialized successfully!");
  console.log("Transaction signature:", tx);

  // Fetch and display config
  const configAccount = await program.account.config.fetch(configPda);
  console.log("\nProgram configuration:");
  console.log("  Admin:", configAccount.admin.toString());
  console.log("  Across Handler:", configAccount.acrossHandler.toString());
  console.log("  Jupiter Program:", configAccount.jupiterProgram.toString());
  console.log("  USDC Mint:", configAccount.usdcMint.toString());
  console.log("  Fee Recipient:", configAccount.feeRecipient.toString());
  console.log("  Fee BPS:", configAccount.feeBps);
  console.log("  Is Paused:", configAccount.isPaused);
}

main()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error("Error:", err);
    process.exit(1);
  });

