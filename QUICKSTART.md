# SuperSwap Solana - Quick Start Guide

Get up and running with SuperSwap Solana in 10 minutes.

## Prerequisites Check

Verify you have everything installed:

```bash
# Check Rust
rustc --version
# Expected: rustc 1.75.0 or later

# Check Solana
solana --version
# Expected: solana-cli 1.18.22

# Check Anchor
anchor --version
# Expected: anchor-cli 0.30.1

# Check Node.js
node --version
# Expected: v18.x or later
```

If anything is missing, see [README.md](./README.md#prerequisites) for installation instructions.

## Quick Setup (Local Development)

### 1. Start Local Validator

Open a new terminal and run:

```bash
solana-test-validator --reset
```

Keep this running. In a new terminal:

```bash
solana config set --url localhost
solana config get
```

### 2. Build the Program

```bash
cd /home/ck/Documents/superswap-sol
anchor build
```

This takes 2-5 minutes on first build.

### 3. Update Program ID

Get your program ID:

```bash
solana-keygen pubkey target/deploy/superswap_sol-keypair.json
```

Copy the output and update these files:

**File 1: `Anchor.toml`**
```toml
[programs.localnet]
superswap_sol = "YOUR_PROGRAM_ID_HERE"
```

**File 2: `programs/superswap-sol/src/lib.rs`**
```rust
declare_id!("YOUR_PROGRAM_ID_HERE");
```

Rebuild:

```bash
anchor build
```

### 4. Deploy Locally

```bash
anchor deploy
```

### 5. Run Tests

Install dependencies:

```bash
yarn install
```

Run tests:

```bash
anchor test --skip-deploy --skip-local-validator
```

You should see all tests passing! ✅

## What Just Happened?

You've successfully:
1. ✅ Started a local Solana validator
2. ✅ Built the SuperSwap program
3. ✅ Deployed it locally
4. ✅ Ran the test suite

The program can now:
- Receive USDC from Across Protocol
- Parse swap instructions
- Execute Jupiter swaps
- Refund on failure

## Next Steps

### Option A: Explore the Code

**Start with:**
1. `programs/superswap-sol/src/lib.rs` - Program entry point
2. `programs/superswap-sol/src/state.rs` - Account structures
3. `programs/superswap-sol/src/instructions/process_bridge_and_swap.rs` - Main logic

### Option B: Test with Real Data

**Devnet deployment:**

```bash
# Switch to devnet
solana config set --url devnet

# Airdrop SOL for fees
solana airdrop 2

# Deploy to devnet
./scripts/deploy.sh devnet

# Initialize the program
ts-node scripts/initialize.ts
```

**Update initialize.ts first with real addresses:**
- Across handler (get from Across docs)
- Jupiter program: `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`
- USDC mint: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`

### Option C: Integrate with Frontend

**Create a simple swap interface:**

```typescript
import { AcrossClient } from '@across-protocol/sdk';
import { createJupiterApiClient } from '@jup-ag/api';

// 1. Get Jupiter quote
const jupiterApi = createJupiterApiClient();
const quote = await jupiterApi.quoteGet({
  inputMint: USDC_MINT,
  outputMint: destinationMint,
  amount: 1000000,
  slippageBps: 50,
});

// 2. Get swap instruction
const { swapInstruction } = await jupiterApi.swapInstructionsPost({
  swapRequest: {
    quoteResponse: quote,
    userPublicKey: PROGRAM_CONFIG_PDA,
  },
});

// 3. Build message for Across
const message = buildSuperSwapMessage({
  recipient: userSolanaAddress,
  destinationMint,
  amount: 1000000,
  minOutput: quote.otherAmountThreshold,
  jupiterInstruction: Buffer.from(swapInstruction, 'base64'),
});

// 4. Bridge via Across
const acrossClient = new AcrossClient();
await acrossClient.deposit({
  fromChain: 8453, // Base
  toChain: SOLANA_CHAIN_ID,
  token: USDC_ADDRESS,
  amount: '1000000',
  recipient: PROGRAM_CONFIG_PDA,
  message: message.toString('hex'),
});
```

See [ACROSS_INTEGRATION.md](./ACROSS_INTEGRATION.md) for complete examples.

## Common Commands

```bash
# Build program
anchor build

# Deploy to localnet
anchor deploy

# Deploy to devnet
anchor deploy --provider.cluster devnet

# Run all tests
anchor test

# Run tests without redeploying
anchor test --skip-deploy

# Watch program logs
solana logs | grep superswap

# Check program info
solana program show <PROGRAM_ID>

# Get account info
anchor account config <CONFIG_PDA>

# Clean build artifacts
anchor clean
```

## Project Structure Overview

```
superswap-sol/
├── programs/superswap-sol/    # Rust program code
│   └── src/
│       ├── lib.rs             # Entry point
│       ├── state.rs           # Account structures
│       ├── error.rs           # Error definitions
│       ├── instructions/      # Instruction handlers
│       │   ├── initialize.rs
│       │   ├── process_bridge_and_swap.rs  # Main logic
│       │   └── ...
│       └── utils/             # Helper functions
│           ├── jupiter.rs
│           └── refund.rs
│
├── tests/                     # TypeScript tests
│   └── superswap-sol.ts
│
├── scripts/                   # Utility scripts
│   ├── deploy.sh             # Deployment script
│   └── initialize.ts         # Initialization script
│
├── target/                    # Build artifacts
│   ├── deploy/               # Compiled programs
│   ├── idl/                  # Interface definitions
│   └── types/                # TypeScript types
│
└── Documentation
    ├── README.md             # Main documentation
    ├── QUICKSTART.md         # This file
    ├── DEVELOPMENT_GUIDE.md  # Detailed development guide
    ├── ARCHITECTURE.md       # Technical architecture
    └── ACROSS_INTEGRATION.md # Across integration guide
```

## Understanding the Flow

### EVM → Solana Swap

```
User (Base) 
    │
    │ 1. Swap cbBTC → USDC
    ▼
SuperSwap EVM Contract
    │
    │ 2. Bridge USDC + message
    ▼
Across Protocol
    │
    │ 3. Deliver USDC + message
    ▼
SuperSwap Solana Program
    │
    │ 4. Execute Jupiter swap
    ▼
Jupiter Aggregator
    │
    │ 5. Return swapped tokens
    ▼
User Wallet (Solana)
```

### What Happens in the Program

1. **Receive**: Across delivers USDC to program
2. **Parse**: Extract swap instructions from message
3. **Validate**: Check deadline, amounts, signatures
4. **Fee**: Deduct protocol fee
5. **Swap**: Execute Jupiter swap via CPI
6. **Verify**: Check output ≥ minimum
7. **Transfer**: Send tokens to user
8. **Refund**: If anything fails, return USDC

## Troubleshooting

### Build fails

```bash
# Clean and rebuild
cargo clean
anchor clean
anchor build
```

### Tests fail with "account not found"

```bash
# Make sure validator is running
solana-test-validator --reset

# In another terminal
anchor test --skip-local-validator
```

### Deploy fails with "insufficient funds"

```bash
# Check balance
solana balance

# Airdrop SOL (localnet/devnet only)
solana airdrop 2
```

### Program ID mismatch

```bash
# Get the program ID from keypair
solana-keygen pubkey target/deploy/superswap_sol-keypair.json

# Update in Anchor.toml and lib.rs
# Then rebuild
anchor build
```

## Learn More

- **Full Documentation**: [README.md](./README.md)
- **Development Guide**: [DEVELOPMENT_GUIDE.md](./DEVELOPMENT_GUIDE.md)
- **Architecture**: [ARCHITECTURE.md](./ARCHITECTURE.md)
- **Across Integration**: [ACROSS_INTEGRATION.md](./ACROSS_INTEGRATION.md)

## Resources

- [Anchor Book](https://book.anchor-lang.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [Jupiter Docs](https://station.jup.ag/docs)
- [Across Docs](https://docs.across.to/)

## Getting Help

1. Check the documentation files
2. Review test files for examples
3. Check program logs: `solana logs`
4. Open an issue on GitHub

## What to Build Next

Ideas for extending SuperSwap:

1. **Frontend DApp**: Build a UI for easy swaps
2. **SVM → EVM**: Implement reverse flow (Solana → EVM)
3. **Analytics**: Track swap volume, fees, success rate
4. **Monitoring**: Set up alerting for failures
5. **Batching**: Support multiple swaps per transaction
6. **Limit Orders**: Add order book functionality

Happy building! 🚀

