# SuperSwap Solana Program

A Solana program that enables cross-chain swaps from EVM chains to Solana using Across Protocol for bridging and Jupiter for DEX aggregation.

## Overview

SuperSwap-Sol is a trustless, on-chain solution for executing cross-chain swaps between EVM chains and Solana. The program:

1. Receives bridged USDC from Across Protocol
2. Parses the embedded swap instructions
3. Executes swaps via Jupiter aggregator
4. Delivers tokens to the end user
5. Automatically refunds USDC if swaps fail

## Architecture

```
┌─────────────┐         ┌─────────────┐         ┌─────────────┐
│  EVM Chain  │────────▶│   Across    │────────▶│   Solana    │
│   (Base)    │         │  Protocol   │         │  SuperSwap  │
└─────────────┘         └─────────────┘         └─────────────┘
                                                        │
                                                        ▼
                                                 ┌─────────────┐
                                                 │   Jupiter   │
                                                 │   Swap      │
                                                 └─────────────┘
                                                        │
                                                        ▼
                                                 ┌─────────────┐
                                                 │    User     │
                                                 └─────────────┘
```

## Features

- ✅ Trustless cross-chain swaps (EVM → Solana)
- ✅ Integration with Across Protocol for bridging
- ✅ Integration with Jupiter for optimal swap routing
- ✅ Automatic refunds on swap failure
- ✅ Configurable fees
- ✅ Emergency pause mechanism
- ✅ Admin controls for configuration updates
- ✅ Comprehensive error handling

## Project Structure

```
superswap-sol/
├── programs/
│   └── superswap-sol/
│       ├── src/
│       │   ├── lib.rs                 # Program entry point
│       │   ├── error.rs               # Custom error definitions
│       │   ├── state.rs               # Account structures
│       │   ├── instructions/          # Instruction handlers
│       │   │   ├── mod.rs
│       │   │   ├── initialize.rs      # Program initialization
│       │   │   ├── update_config.rs   # Config management
│       │   │   ├── process_bridge_and_swap.rs  # Main swap logic
│       │   │   ├── execute_jupiter_swap.rs     # Jupiter integration
│       │   │   ├── recover_funds.rs   # Emergency recovery
│       │   │   └── pause.rs           # Pause/unpause
│       │   └── utils/                 # Helper utilities
│       │       ├── mod.rs
│       │       ├── jupiter.rs         # Jupiter helpers
│       │       └── refund.rs          # Refund logic
│       ├── Cargo.toml
│       └── Xargo.toml
├── tests/
│   └── superswap-sol.ts              # Integration tests
├── Anchor.toml                        # Anchor configuration
├── Cargo.toml                         # Workspace configuration
├── package.json                       # Node.js dependencies
└── tsconfig.json                      # TypeScript configuration
```

## Prerequisites

Before you begin, ensure you have the following installed:

1. **Rust** (1.75.0 or later)
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source $HOME/.cargo/env
   ```

2. **Solana CLI** (1.18.22 or later)
   ```bash
   sh -c "$(curl -sSfL https://release.solana.com/v1.18.22/install)"
   ```

3. **Anchor CLI** (0.30.1)
   ```bash
   cargo install --git https://github.com/coral-xyz/anchor avm --locked --force
   avm install 0.30.1
   avm use 0.30.1
   ```

4. **Node.js** (18.x or later) and **Yarn**
   ```bash
   curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
   sudo apt-get install -y nodejs
   npm install -g yarn
   ```

## Installation

1. **Clone the repository:**
   ```bash
   cd /home/ck/Documents/superswap-sol
   ```

2. **Install dependencies:**
   ```bash
   yarn install
   ```

3. **Build the program:**
   ```bash
   anchor build
   ```

4. **Generate TypeScript types:**
   ```bash
   anchor build
   ```

## Configuration

### 1. Set up Solana Keypair

Generate a keypair for local development:

```bash
solana-keygen new --outfile ~/.config/solana/id.json
```

### 2. Configure Network

For local development:
```bash
solana config set --url localhost
```

For devnet:
```bash
solana config set --url devnet
```

For mainnet:
```bash
solana config set --url mainnet-beta
```

### 3. Airdrop SOL (Devnet/Localhost only)

```bash
solana airdrop 2
```

## Development Workflow

### Step 1: Start Local Validator

For local testing, start a local Solana validator:

```bash
solana-test-validator
```

In a separate terminal, monitor logs:
```bash
solana logs
```

### Step 2: Build the Program

```bash
anchor build
```

This generates:
- Program binary: `target/deploy/superswap_sol.so`
- IDL: `target/idl/superswap_sol.json`
- TypeScript types: `target/types/superswap_sol.ts`

### Step 3: Deploy the Program

For local testing:
```bash
anchor deploy
```

For devnet:
```bash
anchor deploy --provider.cluster devnet
```

### Step 4: Run Tests

Run the full test suite:
```bash
anchor test
```

Run tests without restarting validator:
```bash
anchor test --skip-local-validator
```

### Step 5: Initialize the Program

After deployment, initialize the program with configuration:

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SuperswapSol } from "../target/types/superswap_sol";

const program = anchor.workspace.SuperswapSol as Program<SuperswapSol>;
const provider = anchor.AnchorProvider.env();

// Derive config PDA
const [configPda] = PublicKey.findProgramAddressSync(
  [Buffer.from("config")],
  program.programId
);

// Initialize
await program.methods
  .initialize({
    acrossHandler: acrossHandlerPublicKey,
    jupiterProgram: new PublicKey("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4"),
    usdcMint: new PublicKey("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
    feeRecipient: feeRecipientPublicKey,
    feeBps: 30, // 0.3%
  })
  .accounts({
    config: configPda,
    admin: provider.wallet.publicKey,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

## Testing Strategy

### Unit Tests

Test individual instructions in isolation:

```bash
anchor test tests/initialize.spec.ts
anchor test tests/config.spec.ts
```

### Integration Tests

Test the full swap flow:

```bash
anchor test tests/superswap-sol.ts
```

### Test with Jupiter Devnet

To test with real Jupiter swaps on devnet:

1. Update `Anchor.toml` to use devnet
2. Get devnet USDC from faucet
3. Use Jupiter API to generate swap instructions
4. Test the full flow

Example:
```typescript
// Generate Jupiter swap instructions
const quoteResponse = await fetch(
  `https://quote-api.jup.ag/v6/quote?inputMint=${usdcMint}&outputMint=${destinationMint}&amount=${amount}`
).then(res => res.json());

const swapResponse = await fetch('https://quote-api.jup.ag/v6/swap', {
  method: 'POST',
  headers: { 'Content-Type': 'application/json' },
  body: JSON.stringify({
    quoteResponse,
    userPublicKey: userPublicKey.toString(),
  }),
}).then(res => res.json());

// Use swapResponse.swapTransaction for jupiter_swap_data
```

## Across Integration

### Message Format

When bridging USDC via Across, include swap parameters in the message:

```typescript
interface AcrossMessage {
  orderId: number;          // Unique order identifier
  recipient: string;         // Solana address (base58)
  minOutputAmount: number;   // Minimum tokens expected
  destinationMint: string;   // Token mint address (base58)
  deadline: number;          // Unix timestamp
  jupiterSwapData: Buffer;   // Serialized Jupiter instruction
}
```

### Frontend Integration

1. **Get Jupiter Quote:**
   ```typescript
   const quote = await fetch(
     `https://quote-api.jup.ag/v6/quote?inputMint=${USDC_MINT}&outputMint=${destinationMint}&amount=${amount}&slippageBps=50`
   ).then(r => r.json());
   ```

2. **Generate Swap Transaction:**
   ```typescript
   const swap = await fetch('https://quote-api.jup.ag/v6/swap-instructions', {
     method: 'POST',
     headers: { 'Content-Type': 'application/json' },
     body: JSON.stringify({
       quoteResponse: quote,
       userPublicKey: superswapProgramAddress,
     }),
   }).then(r => r.json());
   ```

3. **Encode Message for Across:**
   ```typescript
   const message = {
     orderId: Date.now(),
     recipient: userSolanaAddress,
     minOutputAmount: quote.otherAmountThreshold,
     destinationMint: destinationTokenMint,
     deadline: Math.floor(Date.now() / 1000) + 1800, // 30 minutes
     jupiterSwapData: Buffer.from(swap.swapInstruction, 'base64'),
   };
   
   const encodedMessage = borsh.serialize(MessageSchema, message);
   ```

4. **Bridge via Across:**
   ```typescript
   // Use Across SDK to bridge with message
   await acrossClient.bridge({
     fromChain: 'base',
     toChain: 'solana',
     token: USDC_ADDRESS,
     amount: amount,
     recipient: SUPERSWAP_PROGRAM_ADDRESS,
     message: encodedMessage,
   });
   ```

## Jupiter Integration

### Swap Execution

The program executes Jupiter swaps via CPI (Cross-Program Invocation):

1. Receives serialized Jupiter instruction in `jupiter_swap_data`
2. Validates the instruction format
3. Builds account list from instruction
4. Executes CPI to Jupiter program
5. Validates output meets minimum requirements
6. Transfers output tokens to user

### Jupiter V6 API

Use Jupiter V6 for optimal routing:

```typescript
// Get quote
const quote = await jupiterApi.quoteGet({
  inputMint: 'EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', // USDC
  outputMint: destinationMint,
  amount: amount,
  slippageBps: 50,
});

// Get swap instructions
const swapInstructions = await jupiterApi.swapInstructionsPost({
  swapRequest: {
    quoteResponse: quote,
    userPublicKey: programAddress,
  },
});
```

## Error Handling

The program includes comprehensive error handling:

### Swap Failures

If a Jupiter swap fails:
1. Program catches the error
2. Marks order as `Failed`
3. Automatically refunds USDC to user
4. Emits failure event

### Refund Logic

```rust
// Automatic refund on swap failure
if swap_result.is_err() {
    msg!("Swap failed, initiating refund");
    refund_usdc(
        &config,
        &mut swap_order,
        &program_usdc_account,
        &recipient_usdc_account,
        &token_program,
    )?;
}
```

### Error Types

| Error | Description | Action |
|-------|-------------|--------|
| `ProgramPaused` | Program is paused | Wait for unpause |
| `SlippageExceeded` | Output below minimum | Increase slippage or retry |
| `DeadlineExceeded` | Transaction too slow | Increase deadline |
| `InvalidSwapCalldata` | Malformed Jupiter data | Regenerate swap data |

## Security Considerations

### 1. Authority Validation

- Only Across handler can trigger swaps
- Only admin can update configuration
- PDA-based authority for token transfers

### 2. Amount Validation

- Minimum output enforced
- Deadline checks prevent stale transactions
- Fee calculations checked for overflow

### 3. Refund Protection

- Automatic refunds on failure
- User always receives either tokens or USDC
- No funds can be stuck

### 4. Emergency Controls

- Admin can pause program
- Admin can recover stuck funds
- Configuration updates require admin signature

## Deployment Checklist

### Devnet Deployment

- [ ] Build program: `anchor build`
- [ ] Update program ID in `Anchor.toml` and `lib.rs`
- [ ] Deploy: `anchor deploy --provider.cluster devnet`
- [ ] Initialize program with test configuration
- [ ] Test with small amounts
- [ ] Verify refund logic

### Mainnet Deployment

- [ ] Complete security audit
- [ ] Test thoroughly on devnet
- [ ] Prepare admin keypair (use hardware wallet)
- [ ] Set up fee recipient
- [ ] Deploy program
- [ ] Initialize with production config:
  - Across handler: Verified Across program address
  - Jupiter program: `JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4`
  - USDC mint: `EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v`
  - Fee: 0.3% (30 bps) or as decided
- [ ] Test with small real transaction
- [ ] Monitor logs for 24 hours
- [ ] Gradually increase limits

## Monitoring

### Logs

Monitor program execution:
```bash
solana logs | grep "superswap"
```

### RPC Calls

Query program state:
```typescript
// Get config
const config = await program.account.config.fetch(configPda);

// Get swap order
const swapOrder = await program.account.swapOrder.fetch(swapOrderPda);

// Check order status
console.log("Status:", swapOrder.status);
```

### Metrics to Track

- Total volume processed
- Success rate of swaps
- Average swap time
- Fee revenue
- Failed transactions (refunds)

## Troubleshooting

### Build Errors

**Error: "anchor-lang version mismatch"**
```bash
cargo clean
anchor build
```

**Error: "program ID mismatch"**
Update the program ID in:
- `Anchor.toml`
- `programs/superswap-sol/src/lib.rs` (declare_id!)

### Test Failures

**Error: "insufficient funds"**
```bash
solana airdrop 2
```

**Error: "account not found"**
Ensure validator is running and program is deployed:
```bash
solana program show <PROGRAM_ID>
```

### Runtime Errors

**Error: "ProgramPaused"**
Program is paused by admin. Check with:
```typescript
const config = await program.account.config.fetch(configPda);
console.log("Is paused:", config.isPaused);
```

**Error: "SlippageExceeded"**
Jupiter swap output below minimum. Increase slippage tolerance or update price.

## Future Enhancements

### SVM → EVM Integration

To enable Solana → EVM swaps:

1. Add instruction to initiate SVM → EVM bridge
2. Swap tokens to USDC on Solana via Jupiter
3. Bridge USDC to EVM chain via Across
4. Include calldata for EVM DEX swap (e.g., Uniswap)

### Multi-Hop Swaps

Enable complex routing:
- Swap through multiple intermediate tokens
- Use multiple DEXs for better pricing
- Split large orders across venues

### Order Book Integration

Add limit order functionality:
- Users set desired price
- Execute when price is met
- Cancel/modify pending orders

## API Reference

### Instructions

#### `initialize`
Initializes the program configuration.

**Parameters:**
- `across_handler: Pubkey` - Across handler authority
- `jupiter_program: Pubkey` - Jupiter program ID
- `usdc_mint: Pubkey` - USDC token mint
- `fee_recipient: Pubkey` - Fee collection address
- `fee_bps: u16` - Fee in basis points (0-1000)

#### `process_bridge_and_swap`
Processes bridged USDC and executes swap.

**Parameters:**
- `order_id: u64` - Unique order identifier
- `recipient: Pubkey` - Token recipient
- `usdc_amount: u64` - Amount of USDC bridged
- `min_output_amount: u64` - Minimum tokens expected
- `destination_mint: Pubkey` - Destination token mint
- `deadline: i64` - Expiration timestamp
- `jupiter_swap_data: Vec<u8>` - Serialized Jupiter instruction

#### `update_config`
Updates program configuration (admin only).

#### `pause` / `unpause`
Pauses/unpauses program (admin only).

#### `recover_funds`
Recovers stuck funds (admin only, emergency use).

### Accounts

#### `Config`
Global program configuration.

#### `SwapOrder`
Represents an individual swap order.

**Status:**
- `Pending` - Order being processed
- `Completed` - Swap successful
- `Refunded` - Swap failed, USDC refunded
- `Failed` - Order failed with error

## Contributing

Contributions welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Support

For issues and questions:
- GitHub Issues: [SuperSwap Issues](https://github.com/superswap/superswap-sol/issues)
- Discord: [SuperSwap Community](https://discord.gg/superswap)
- Documentation: [docs.superswap.io](https://docs.superswap.io)

## Resources

- [Anchor Documentation](https://www.anchor-lang.com/)
- [Solana Documentation](https://docs.solana.com/)
- [Jupiter Documentation](https://docs.jup.ag/)
- [Across Protocol Documentation](https://docs.across.to/)

