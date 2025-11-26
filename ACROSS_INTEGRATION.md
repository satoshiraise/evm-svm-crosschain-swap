# Across Protocol Integration Guide

This document details how to integrate with Across Protocol for cross-chain bridging to the SuperSwap Solana program.

## Overview

Across Protocol enables fast, secure bridging between EVM chains and Solana. For SuperSwap, Across serves as the bridge layer, delivering USDC to Solana along with swap instructions.

## Across Solana Support

### Supported Assets

- **USDC**: Fully supported for bridging to/from Solana
- **WETH**: Not currently supported for Solana
- **Other tokens**: Not supported for Solana

For swaps involving other tokens, you must:
1. Swap to USDC on the source chain
2. Bridge USDC via Across
3. Swap from USDC on Solana

## Message Passing

### Across Message Structure

Across supports embedding arbitrary data in bridge transactions. This data is delivered to the destination chain along with the bridged assets.

```typescript
interface AcrossBridgeRequest {
  fromChain: number;           // Source chain ID
  toChain: number;             // Destination chain ID (Solana)
  token: string;               // Token address (USDC)
  amount: string;              // Amount to bridge
  recipient: string;           // Recipient on destination (SuperSwap program)
  destinationChainId: number;  // Solana chain ID
  message?: Buffer;            // Optional: Embedded message data
  relayerFeePct?: string;      // Optional: Relayer fee percentage
}
```

### SuperSwap Message Format

SuperSwap expects messages in the following format:

```rust
pub struct SuperSwapMessage {
    pub order_id: u64,              // Unique identifier
    pub recipient: [u8; 32],        // Solana address (32 bytes)
    pub usdc_amount: u64,           // USDC amount (6 decimals)
    pub min_output_amount: u64,     // Minimum tokens expected
    pub destination_mint: [u8; 32], // Token mint address (32 bytes)
    pub deadline: i64,              // Unix timestamp
    pub jupiter_swap_data: Vec<u8>, // Serialized Jupiter instruction
}
```

### Serialization

Use Borsh for serialization (compatible with Solana):

```typescript
import * as borsh from 'borsh';

class SuperSwapMessage {
  orderId: bigint;
  recipient: Uint8Array;
  usdcAmount: bigint;
  minOutputAmount: bigint;
  destinationMint: Uint8Array;
  deadline: bigint;
  jupiterSwapData: Uint8Array;

  constructor(props: {
    orderId: bigint;
    recipient: Uint8Array;
    usdcAmount: bigint;
    minOutputAmount: bigint;
    destinationMint: Uint8Array;
    deadline: bigint;
    jupiterSwapData: Uint8Array;
  }) {
    Object.assign(this, props);
  }
}

const messageSchema = new Map([
  [SuperSwapMessage, {
    kind: 'struct',
    fields: [
      ['orderId', 'u64'],
      ['recipient', [32]],
      ['usdcAmount', 'u64'],
      ['minOutputAmount', 'u64'],
      ['destinationMint', [32]],
      ['deadline', 'i64'],
      ['jupiterSwapData', ['u8']],
    ],
  }],
]);

// Serialize message
const message = new SuperSwapMessage({
  orderId: BigInt(Date.now()),
  recipient: recipientPubkey.toBytes(),
  usdcAmount: BigInt(1000000), // 1 USDC
  minOutputAmount: BigInt(950000),
  destinationMint: destMintPubkey.toBytes(),
  deadline: BigInt(Math.floor(Date.now() / 1000) + 1800),
  jupiterSwapData: jupiterInstructionData,
});

const serialized = borsh.serialize(messageSchema, message);
```

## Frontend Integration

### Step-by-Step Integration

#### Step 1: Install Dependencies

```bash
npm install @across-protocol/sdk @coral-xyz/anchor @jup-ag/api borsh
```

#### Step 2: Initialize Clients

```typescript
import { AcrossClient } from '@across-protocol/sdk';
import { createJupiterApiClient } from '@jup-ag/api';
import { Connection, PublicKey } from '@solana/web3.js';

const acrossClient = new AcrossClient({
  // Configure with your RPC endpoints
});

const jupiterApi = createJupiterApiClient();
const solanaConnection = new Connection('https://api.mainnet-beta.solana.com');
```

#### Step 3: Get Jupiter Quote

```typescript
async function getJupiterQuote(
  inputMint: string,
  outputMint: string,
  amount: number,
  slippageBps: number = 50 // 0.5%
) {
  const quote = await jupiterApi.quoteGet({
    inputMint,
    outputMint,
    amount,
    slippageBps,
  });

  if (!quote) {
    throw new Error('No Jupiter quote available');
  }

  return quote;
}
```

#### Step 4: Generate Jupiter Swap Instruction

```typescript
async function getJupiterSwapInstruction(
  quote: any,
  userPublicKey: string
) {
  const { swapInstruction } = await jupiterApi.swapInstructionsPost({
    swapRequest: {
      quoteResponse: quote,
      userPublicKey,
      wrapAndUnwrapSol: false,
      dynamicComputeUnitLimit: true,
    },
  });

  // Decode base64 instruction
  return Buffer.from(swapInstruction, 'base64');
}
```

#### Step 5: Build SuperSwap Message

```typescript
import * as borsh from 'borsh';
import { PublicKey } from '@solana/web3.js';

async function buildSuperSwapMessage(
  recipientAddress: string,
  destinationMint: string,
  usdcAmount: number,
  minOutputAmount: number,
  jupiterInstruction: Buffer
): Promise<Buffer> {
  // Convert addresses to bytes
  const recipientPubkey = new PublicKey(recipientAddress);
  const destMintPubkey = new PublicKey(destinationMint);

  // Create message
  const message = {
    orderId: BigInt(Date.now()),
    recipient: Array.from(recipientPubkey.toBytes()),
    usdcAmount: BigInt(usdcAmount),
    minOutputAmount: BigInt(minOutputAmount),
    destinationMint: Array.from(destMintPubkey.toBytes()),
    deadline: BigInt(Math.floor(Date.now() / 1000) + 1800), // 30 minutes
    jupiterSwapData: Array.from(jupiterInstruction),
  };

  // Define schema
  const schema = {
    struct: {
      orderId: 'u64',
      recipient: { array: { type: 'u8', len: 32 } },
      usdcAmount: 'u64',
      minOutputAmount: 'u64',
      destinationMint: { array: { type: 'u8', len: 32 } },
      deadline: 'i64',
      jupiterSwapData: { array: { type: 'u8' } },
    },
  };

  // Serialize
  return Buffer.from(borsh.serialize(schema, message));
}
```

#### Step 6: Execute Across Bridge

```typescript
async function executeCrossChainSwap(
  fromChainId: number,
  tokenAddress: string,
  amount: string,
  recipientSolanaAddress: string,
  destinationMint: string,
  signer: any
) {
  // 1. Get Jupiter quote
  const quote = await getJupiterQuote(
    SOLANA_USDC_MINT,
    destinationMint,
    parseInt(amount),
    50 // 0.5% slippage
  );

  console.log('Jupiter quote:', quote);

  // 2. Get Jupiter swap instruction
  const SUPERSWAP_PROGRAM_PDA = 'YOUR_PROGRAM_CONFIG_PDA_HERE';
  const jupiterInstruction = await getJupiterSwapInstruction(
    quote,
    SUPERSWAP_PROGRAM_PDA
  );

  // 3. Build SuperSwap message
  const message = await buildSuperSwapMessage(
    recipientSolanaAddress,
    destinationMint,
    parseInt(amount),
    parseInt(quote.otherAmountThreshold),
    jupiterInstruction
  );

  console.log('Message size:', message.length, 'bytes');

  // 4. Execute Across bridge
  const tx = await acrossClient.deposit({
    fromChain: fromChainId,
    toChain: 1151111081099710, // Solana chain ID
    token: tokenAddress,
    amount: amount,
    recipient: SUPERSWAP_PROGRAM_PDA,
    message: message.toString('hex'),
    relayerFeePct: '0.0001', // 0.01% relayer fee
    signer: signer,
  });

  console.log('Bridge transaction:', tx.hash);

  return tx;
}
```

### Complete Example

```typescript
import { ethers } from 'ethers';
import { AcrossClient } from '@across-protocol/sdk';
import { createJupiterApiClient } from '@jup-ag/api';
import { PublicKey } from '@solana/web3.js';
import * as borsh from 'borsh';

// Configuration
const BASE_CHAIN_ID = 8453;
const SOLANA_CHAIN_ID = 1151111081099710;
const USDC_BASE = '0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913';
const USDC_SOLANA = 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v';
const SUPERSWAP_PROGRAM = 'YOUR_CONFIG_PDA_HERE';

async function swapBaseToSolana(
  userEthereumSigner: ethers.Signer,
  userSolanaAddress: string,
  destinationTokenMint: string,
  amountUsdc: string
) {
  // Initialize clients
  const acrossClient = new AcrossClient();
  const jupiterApi = createJupiterApiClient();

  // 1. Get Jupiter quote for USDC → Destination token
  console.log('Getting Jupiter quote...');
  const quote = await jupiterApi.quoteGet({
    inputMint: USDC_SOLANA,
    outputMint: destinationTokenMint,
    amount: parseInt(amountUsdc),
    slippageBps: 50,
  });

  console.log('Quote received:');
  console.log('  Input:', quote.inAmount, 'USDC');
  console.log('  Output:', quote.outAmount, 'tokens');
  console.log('  Min output:', quote.otherAmountThreshold);

  // 2. Get Jupiter swap instruction
  console.log('Getting Jupiter swap instruction...');
  const { swapInstruction } = await jupiterApi.swapInstructionsPost({
    swapRequest: {
      quoteResponse: quote,
      userPublicKey: SUPERSWAP_PROGRAM,
    },
  });

  const jupiterInstructionData = Buffer.from(swapInstruction, 'base64');

  // 3. Build SuperSwap message
  console.log('Building SuperSwap message...');
  const message = {
    orderId: BigInt(Date.now()),
    recipient: Array.from(new PublicKey(userSolanaAddress).toBytes()),
    usdcAmount: BigInt(amountUsdc),
    minOutputAmount: BigInt(quote.otherAmountThreshold),
    destinationMint: Array.from(new PublicKey(destinationTokenMint).toBytes()),
    deadline: BigInt(Math.floor(Date.now() / 1000) + 1800),
    jupiterSwapData: Array.from(jupiterInstructionData),
  };

  const schema = {
    struct: {
      orderId: 'u64',
      recipient: { array: { type: 'u8', len: 32 } },
      usdcAmount: 'u64',
      minOutputAmount: 'u64',
      destinationMint: { array: { type: 'u8', len: 32 } },
      deadline: 'i64',
      jupiterSwapData: { array: { type: 'u8' } },
    },
  };

  const serializedMessage = Buffer.from(borsh.serialize(schema, message));
  console.log('Message size:', serializedMessage.length, 'bytes');

  // 4. Approve USDC spending (if needed)
  const usdcContract = new ethers.Contract(
    USDC_BASE,
    ['function approve(address spender, uint256 amount) returns (bool)'],
    userEthereumSigner
  );

  console.log('Approving USDC...');
  const approveTx = await usdcContract.approve(
    acrossClient.spokePoolAddress(BASE_CHAIN_ID),
    amountUsdc
  );
  await approveTx.wait();
  console.log('USDC approved');

  // 5. Execute Across bridge
  console.log('Executing bridge...');
  const bridgeTx = await acrossClient.deposit({
    fromChain: BASE_CHAIN_ID,
    toChain: SOLANA_CHAIN_ID,
    token: USDC_BASE,
    amount: amountUsdc,
    recipient: SUPERSWAP_PROGRAM,
    message: serializedMessage.toString('hex'),
    signer: userEthereumSigner,
  });

  console.log('Bridge transaction:', bridgeTx.hash);
  console.log('Waiting for confirmation...');

  await bridgeTx.wait();

  console.log('✅ Bridge transaction confirmed!');
  console.log('Your tokens will arrive on Solana within 1-5 minutes.');
  console.log('Recipient:', userSolanaAddress);

  return bridgeTx;
}

// Usage
const provider = new ethers.providers.JsonRpcProvider('YOUR_BASE_RPC_URL');
const signer = new ethers.Wallet('YOUR_PRIVATE_KEY', provider);

swapBaseToSolana(
  signer,
  'YOUR_SOLANA_ADDRESS',
  'DESTINATION_TOKEN_MINT',
  '1000000' // 1 USDC
).then(() => console.log('Done'));
```

## Testing

### Local Testing

For local development, use a mock Across handler:

```typescript
// In tests
const mockAcrossHandler = Keypair.generate();

// Fund the handler
await connection.requestAirdrop(mockAcrossHandler.publicKey, 2 * LAMPORTS_PER_SOL);

// Initialize program with mock handler
await program.methods
  .initialize({
    acrossHandler: mockAcrossHandler.publicKey,
    // ... other params
  })
  .rpc();

// Simulate Across delivery
await program.methods
  .processBridgeAndSwap(params)
  .accounts({
    acrossHandler: mockAcrossHandler.publicKey,
    // ... other accounts
  })
  .signers([mockAcrossHandler])
  .rpc();
```

### Devnet Testing

1. Deploy SuperSwap program to devnet
2. Get Across handler address for Solana devnet
3. Update program config with real Across handler
4. Use Across testnet bridge to send test transactions

### Mainnet Integration

1. Complete security audit
2. Deploy to mainnet
3. Get production Across handler from Across team
4. Configure with production parameters:
   - USDC mint: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
   - Across handler: (obtain from Across)
5. Test with small amounts first

## Troubleshooting

### Message Too Large

**Error:** Message exceeds maximum size

**Solution:** 
- Jupiter instructions can be large (500+ bytes)
- Use simpler routes when possible
- Consider compressing instruction data

### Invalid Message Format

**Error:** Deserialization failed

**Solution:**
- Verify Borsh serialization matches Rust struct
- Check byte order (little-endian)
- Validate all PublicKeys are 32 bytes

### Across Handler Mismatch

**Error:** `InvalidAcrossHandler`

**Solution:**
- Verify `across_handler` in config matches caller
- Update config if Across handler changes
- Check correct network (devnet vs mainnet)

### Bridge Not Completing

**Issue:** USDC bridged but swap not executed

**Debug:**
1. Check Solana explorer for program logs
2. Verify SuperSwap program address is correct
3. Check Across message was included
4. Verify USDC arrived at program

## Resources

- [Across Documentation](https://docs.across.to/)
- [Across Solana Integration](https://docs.across.to/exclusive/add-solana-support-to-your-bridge)
- [Across SDK](https://github.com/across-protocol/sdk)
- [Jupiter API](https://station.jup.ag/docs/apis/swap-api)
- [Borsh Specification](https://borsh.io/)

## Support

For Across-specific issues:
- Across Discord: https://discord.gg/across
- Across Documentation: https://docs.across.to/

For SuperSwap integration:
- GitHub Issues: [SuperSwap Issues](https://github.com/superswap/superswap-sol/issues)
- Documentation: This repository

