# SuperSwap Solana Development Guide

This guide provides step-by-step instructions for developing and testing the SuperSwap Solana program.

## Table of Contents

1. [Development Environment Setup](#development-environment-setup)
2. [Understanding the Architecture](#understanding-the-architecture)
3. [Step-by-Step Development](#step-by-step-development)
4. [Testing Strategy](#testing-strategy)
5. [Across Integration Details](#across-integration-details)
6. [Jupiter Integration Details](#jupiter-integration-details)
7. [Debugging Guide](#debugging-guide)

## Development Environment Setup

### 1. Install Prerequisites

#### Install Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup default stable
rustup component add rustfmt clippy
```

Verify installation:
```bash
rustc --version
cargo --version
```

#### Install Solana CLI
```bash
sh -c "$(curl -sSfL https://release.solana.com/v1.18.22/install)"
export PATH="/home/ck/.local/share/solana/install/active_release/bin:$PATH"
```

Verify installation:
```bash
solana --version
# Expected: solana-cli 1.18.22
```

#### Install Anchor
```bash
cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
avm install 0.30.1
avm use 0.30.1
```

Verify installation:
```bash
anchor --version
# Expected: anchor-cli 0.30.1
```

#### Install Node.js and Yarn
```bash
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs
npm install -g yarn
```

Verify installation:
```bash
node --version
yarn --version
```

### 2. Set Up Development Keypair

```bash
# Generate a new keypair for development
solana-keygen new --outfile ~/.config/solana/id.json

# Set to localnet
solana config set --url localhost

# Verify configuration
solana config get
```

### 3. Install Project Dependencies

```bash
cd /home/ck/Documents/superswap-sol
yarn install
```

## Understanding the Architecture

### Program Structure

```
SuperSwap Program
│
├── Config (PDA)
│   ├── Admin authority
│   ├── Across handler
│   ├── Jupiter program ID
│   ├── USDC mint
│   ├── Fee configuration
│   └── Pause state
│
└── SwapOrder (PDA per order)
    ├── Order ID
    ├── Recipient
    ├── Amounts
    ├── Deadline
    └── Status
```

### Flow Diagram

```
┌──────────────────────────────────────────────────────────────┐
│                    EVM Chain (Base)                          │
│  User initiates swap: cbBTC -> PUMP (Solana)                │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     │ 1. Swap cbBTC -> USDC on Base
                     │    (via SuperSwap EVM contract)
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│                    Across Protocol                           │
│  - Bridge USDC from Base to Solana                          │
│  - Include message with swap instructions                    │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     │ 2. Bridge USDC with calldata
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│                SuperSwap Solana Program                      │
│  - Receive USDC from Across                                  │
│  - Parse swap instructions                                   │
│  - Execute Jupiter swap                                      │
│  - Send tokens to user OR refund on failure                 │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     │ 3. Execute swap via CPI
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│                    Jupiter Aggregator                        │
│  Swap USDC -> PUMP with optimal routing                     │
└────────────────────┬─────────────────────────────────────────┘
                     │
                     │ 4. Transfer swapped tokens
                     │
                     ▼
┌──────────────────────────────────────────────────────────────┐
│                      User Wallet                             │
│  Receives PUMP tokens on Solana                             │
└──────────────────────────────────────────────────────────────┘
```

## Step-by-Step Development

### Phase 1: Build and Deploy Locally

#### Step 1.1: Start Local Validator

In terminal 1:
```bash
solana-test-validator --reset
```

In terminal 2 (monitor logs):
```bash
solana logs
```

#### Step 1.2: Build the Program

In terminal 3:
```bash
cd /home/ck/Documents/superswap-sol
anchor build
```

This creates:
- `target/deploy/superswap_sol.so` - The compiled program
- `target/idl/superswap_sol.json` - Interface definition
- `target/types/superswap_sol.ts` - TypeScript types

#### Step 1.3: Update Program ID

After first build, get the program ID:
```bash
solana-keygen pubkey target/deploy/superswap_sol-keypair.json
```

Update the program ID in:
1. `Anchor.toml` - `[programs.localnet]` section
2. `programs/superswap-sol/src/lib.rs` - `declare_id!()` macro

Then rebuild:
```bash
anchor build
```

#### Step 1.4: Deploy Locally

```bash
anchor deploy
```

Verify deployment:
```bash
solana program show <PROGRAM_ID>
```

### Phase 2: Initialize and Test Basic Functions

#### Step 2.1: Run Initialization Test

```bash
anchor test --skip-build
```

This will:
1. Initialize the program configuration
2. Set admin, Across handler, Jupiter program ID
3. Configure fees

#### Step 2.2: Test Configuration Updates

Run specific test:
```bash
anchor test --skip-build --skip-deploy tests/superswap-sol.ts
```

Verify:
- Admin can update configuration
- Non-admin cannot update (should fail)
- Pause/unpause works correctly

### Phase 3: Implement Jupiter Integration

#### Step 3.1: Understand Jupiter V6 API

Jupiter V6 uses a swap instruction format:

```typescript
interface JupiterSwapInstruction {
  programId: PublicKey;
  accounts: AccountMeta[];
  data: Buffer;
}
```

To get swap instructions:

```typescript
import { createJupiterApiClient } from '@jup-ag/api';

const jupiterApi = createJupiterApiClient();

// Get quote
const quote = await jupiterApi.quoteGet({
  inputMint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', // USDC
  outputMint: destinationMint,
  amount: 1000000, // 1 USDC
  slippageBps: 50,
});

// Get swap instructions
const { swapInstruction } = await jupiterApi.swapInstructionsPost({
  swapRequest: {
    quoteResponse: quote,
    userPublicKey: programPDA.toString(),
  },
});
```

#### Step 3.2: Test Jupiter Swap Locally

Create a test file `tests/jupiter-swap.ts`:

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SuperswapSol } from "../target/types/superswap_sol";
import { createJupiterApiClient } from '@jup-ag/api';

describe("Jupiter swap integration", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  
  const program = anchor.workspace.SuperswapSol as Program<SuperswapSol>;
  const jupiterApi = createJupiterApiClient();
  
  it("Executes a real Jupiter swap", async () => {
    // Get Jupiter quote
    const quote = await jupiterApi.quoteGet({
      inputMint: USDC_MINT,
      outputMint: DESTINATION_MINT,
      amount: 1000000,
      slippageBps: 100,
    });
    
    // Get swap instructions
    const { swapInstruction } = await jupiterApi.swapInstructionsPost({
      swapRequest: {
        quoteResponse: quote,
        userPublicKey: configPda.toString(),
      },
    });
    
    // Deserialize instruction
    const ix = deserializeInstruction(swapInstruction);
    
    // Call program with Jupiter swap data
    await program.methods
      .processBridgeAndSwap({
        // ... params
        jupiterSwapData: ix.data,
      })
      .accounts({
        // ... accounts
      })
      .remainingAccounts(ix.accounts)
      .rpc();
  });
});
```

#### Step 3.3: Implement CPI to Jupiter

The key challenge is executing Jupiter via CPI. In `process_bridge_and_swap.rs`:

```rust
use crate::utils::jupiter::execute_jupiter_swap;

// After fee deduction
let jupiter_accounts: Vec<AccountInfo> = ctx.remaining_accounts.to_vec();

// Execute Jupiter swap
execute_jupiter_swap(
    ctx.accounts.jupiter_program.as_ref(),
    &params.jupiter_swap_data,
    &jupiter_accounts,
    &[&[b"config", &[config.bump]]],
)?;

// Verify output amount
let output_amount = get_output_amount(&recipient_destination_account)?;
require!(
    output_amount >= params.min_output_amount,
    SuperSwapError::InsufficientOutputAmount
);
```

### Phase 4: Implement Refund Logic

#### Step 4.1: Add Refund Instruction

In `instructions/process_bridge_and_swap.rs`:

```rust
// Wrap Jupiter swap in error handling
match execute_jupiter_swap(...) {
    Ok(_) => {
        // Verify output
        if output_amount >= params.min_output_amount {
            swap_order.status = OrderStatus::Completed;
            msg!("Swap completed successfully");
        } else {
            // Insufficient output, refund
            refund_usdc(
                &config,
                &mut swap_order,
                &program_usdc_account,
                &recipient_usdc_account,
                &token_program,
            )?;
        }
    },
    Err(e) => {
        // Swap failed, refund
        msg!("Swap failed: {:?}", e);
        refund_usdc(
            &config,
            &mut swap_order,
            &program_usdc_account,
            &recipient_usdc_account,
            &token_program,
        )?;
    }
}
```

#### Step 4.2: Test Refund Logic

Create test cases:
1. Swap with impossible slippage (should refund)
2. Swap with expired deadline (should refund)
3. Swap with invalid token mint (should refund)

```typescript
it("Refunds USDC on swap failure", async () => {
  const initialBalance = await getTokenBalance(recipientUsdcAccount);
  
  // Trigger swap with impossible parameters
  await program.methods
    .processBridgeAndSwap({
      // ... params with minOutputAmount too high
    })
    .accounts({...})
    .rpc();
  
  const finalBalance = await getTokenBalance(recipientUsdcAccount);
  
  // User should receive refund
  assert.equal(finalBalance, initialBalance + usdcAmount);
  
  // Order should be marked as refunded
  const swapOrder = await program.account.swapOrder.fetch(swapOrderPda);
  assert.equal(swapOrder.status, OrderStatus.Refunded);
});
```

### Phase 5: Across Integration

#### Step 5.1: Understand Across Message Format

Across uses a message passing system. On Solana, the message is delivered as calldata.

Message structure:
```rust
pub struct AcrossMessage {
    pub order_id: u64,
    pub recipient: Pubkey,
    pub usdc_amount: u64,
    pub min_output_amount: u64,
    pub destination_mint: Pubkey,
    pub deadline: i64,
    pub jupiter_swap_data: Vec<u8>,
}
```

#### Step 5.2: Set Up Across Handler

The Across handler is the account that calls your program when USDC arrives on Solana.

In production:
```rust
pub fn process_bridge_and_swap(
    ctx: Context<ProcessBridgeAndSwap>,
    params: ProcessBridgeAndSwapParams,
) -> Result<()> {
    // Verify caller is Across handler
    require!(
        ctx.accounts.across_handler.key() == ctx.accounts.config.across_handler,
        SuperSwapError::InvalidAcrossHandler
    );
    
    // ... rest of logic
}
```

#### Step 5.3: Test with Mock Across Handler

For local testing, use a mock Across handler:

```typescript
// In tests, any keypair can be the "Across handler"
const mockAcrossHandler = Keypair.generate();

// Initialize with mock handler
await program.methods
  .initialize({
    acrossHandler: mockAcrossHandler.publicKey,
    // ... other params
  })
  .rpc();

// Simulate Across delivery
await program.methods
  .processBridgeAndSwap({...})
  .accounts({
    acrossHandler: mockAcrossHandler.publicKey,
    // ...
  })
  .signers([mockAcrossHandler])
  .rpc();
```

#### Step 5.4: Integrate with Real Across (Devnet)

1. Deploy program to devnet:
   ```bash
   anchor deploy --provider.cluster devnet
   ```

2. Get Across handler address for Solana devnet from Across documentation

3. Update configuration:
   ```typescript
   await program.methods
     .updateConfig({
       newAcrossHandler: ACROSS_HANDLER_DEVNET,
       // ...
     })
     .rpc();
   ```

4. Test with real Across bridge:
   - Bridge USDC from EVM testnet to Solana devnet
   - Include message with swap parameters
   - Monitor Solana for execution

## Testing Strategy

### 1. Unit Tests

Test each instruction independently:

```bash
# Test initialization
anchor test tests/initialize.spec.ts

# Test config updates
anchor test tests/config.spec.ts

# Test pause functionality
anchor test tests/pause.spec.ts
```

### 2. Integration Tests

Test full swap flow:

```bash
# Test complete swap flow
anchor test tests/integration.spec.ts

# Test refund logic
anchor test tests/refund.spec.ts
```

### 3. Devnet Testing

Deploy to devnet and test with real protocols:

```bash
# Deploy to devnet
anchor deploy --provider.cluster devnet

# Run devnet tests
anchor test --provider.cluster devnet
```

### 4. Mainnet Fork Testing

Test against mainnet state without deploying:

```bash
# Clone mainnet state
solana-test-validator --clone <JUPITER_PROGRAM> --clone <USDC_MINT>

# Run tests
anchor test --skip-deploy
```

## Debugging Guide

### Common Issues

#### Issue: "Account not found"

**Cause:** Program not deployed or wrong cluster

**Solution:**
```bash
solana config get
anchor deploy
```

#### Issue: "Custom program error: 0x1"

**Cause:** Anchor error, usually account validation failed

**Solution:**
1. Check program logs: `solana logs`
2. Verify account addresses match expected PDAs
3. Check account ownership

#### Issue: "Jupiter swap failed"

**Cause:** Various - slippage, liquidity, invalid route

**Solution:**
1. Check Jupiter quote is fresh (< 30 seconds old)
2. Increase slippage tolerance
3. Verify token accounts exist
4. Check program logs for specific error

### Debugging Tools

#### 1. Program Logs

```bash
# Watch all logs
solana logs

# Filter for your program
solana logs | grep <PROGRAM_ID>

# Save logs to file
solana logs > debug.log
```

#### 2. Account Inspector

```bash
# Inspect program account
solana account <ACCOUNT_ADDRESS>

# Decode account data
anchor account <ACCOUNT_TYPE> <ACCOUNT_ADDRESS>
```

#### 3. Transaction Inspector

```bash
# Get transaction details
solana confirm -v <TRANSACTION_SIGNATURE>
```

#### 4. Anchor Test with Logs

```bash
# Run tests with verbose output
RUST_LOG=debug anchor test
```

## Performance Optimization

### 1. Compute Units

Monitor compute units usage:

```rust
use anchor_lang::solana_program::log::sol_log_compute_units;

pub fn process_bridge_and_swap(ctx: Context<ProcessBridgeAndSwap>, params: ProcessBridgeAndSwapParams) -> Result<()> {
    sol_log_compute_units();
    
    // ... your logic
    
    sol_log_compute_units();
    Ok(())
}
```

### 2. Optimize Account Access

Minimize account reads:

```rust
// Bad: Multiple fetches
let config = ctx.accounts.config.load()?;
let fee = config.fee_bps;
let admin = config.admin;

// Good: Single fetch, multiple uses
let config = ctx.accounts.config.load()?;
let (fee, admin) = (config.fee_bps, config.admin);
```

### 3. Batch Operations

Use transaction batching when possible:

```typescript
// Instead of multiple transactions
await program.methods.updateConfig1().rpc();
await program.methods.updateConfig2().rpc();

// Batch into one transaction
const tx = new Transaction();
tx.add(program.methods.updateConfig1().instruction());
tx.add(program.methods.updateConfig2().instruction());
await provider.sendAndConfirm(tx);
```

## Next Steps

1. **Complete Jupiter Integration**
   - Implement full CPI to Jupiter
   - Handle all Jupiter error cases
   - Test with multiple token pairs

2. **Production Hardening**
   - Security audit
   - Stress testing
   - Rate limiting considerations

3. **SVM → EVM Flow**
   - Design reverse bridge flow
   - Implement Solana → EVM instructions
   - Integrate with Across for reverse bridging

4. **Frontend Integration**
   - Build SDK for easy integration
   - Create example frontend
   - Document API for developers

## Resources

- [Anchor Book](https://book.anchor-lang.com/)
- [Solana Cookbook](https://solanacookbook.com/)
- [Jupiter Documentation](https://station.jup.ag/docs)
- [Across Protocol Docs](https://docs.across.to/)
- [Solana Program Library](https://spl.solana.com/)

