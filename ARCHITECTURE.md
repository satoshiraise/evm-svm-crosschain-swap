# SuperSwap Solana Architecture

## Overview

This document describes the technical architecture of the SuperSwap Solana program for cross-chain swaps.

## System Architecture

### High-Level Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         Frontend/DApp                           │
│  - Generate Jupiter quote                                       │
│  - Create Across bridge transaction with message                │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                     EVM Chain (Base/Arbitrum/etc)               │
│  - SuperSwap EVM Contract                                       │
│  - Swap source token → USDC                                     │
│  - Call Across bridge with message                              │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Across Protocol                            │
│  - Verify bridge transaction                                    │
│  - Settle USDC on Solana                                        │
│  - Deliver message to SuperSwap program                         │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                   SuperSwap Solana Program                      │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │ 1. Receive USDC from Across                               │ │
│  │ 2. Parse swap instructions from message                   │ │
│  │ 3. Validate parameters (deadline, amounts, etc)           │ │
│  │ 4. Deduct protocol fee                                    │ │
│  │ 5. Execute Jupiter swap via CPI                           │ │
│  │ 6. Verify output amount ≥ minimum                         │ │
│  │ 7. Transfer tokens to user                                │ │
│  │ 8. If failure: Refund USDC to user                        │ │
│  └───────────────────────────────────────────────────────────┘ │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Jupiter Aggregator                         │
│  - Route swap through optimal DEXs                              │
│  - Execute swap instructions                                    │
│  - Return swapped tokens                                        │
└────────────────────┬────────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                        User Wallet                              │
│  - Receives swapped tokens                                      │
│  - OR receives USDC refund if swap failed                       │
└─────────────────────────────────────────────────────────────────┘
```

## Program Architecture

### Account Structure

#### 1. Config Account (PDA)

**Purpose:** Global program configuration  
**Seeds:** `["config"]`  
**Size:** 178 bytes

```rust
pub struct Config {
    pub admin: Pubkey,              // 32 bytes - Program administrator
    pub across_handler: Pubkey,     // 32 bytes - Authorized Across handler
    pub jupiter_program: Pubkey,    // 32 bytes - Jupiter program ID
    pub usdc_mint: Pubkey,          // 32 bytes - USDC mint address
    pub fee_recipient: Pubkey,      // 32 bytes - Fee collection address
    pub fee_bps: u16,               // 2 bytes  - Fee in basis points (max 1000 = 10%)
    pub is_paused: bool,            // 1 byte   - Emergency pause flag
    pub bump: u8,                   // 1 byte   - PDA bump seed
}
```

**Access Control:**
- `admin`: Can update all configuration
- `across_handler`: Can trigger swap execution
- Only one Config account per program

#### 2. SwapOrder Account (PDA)

**Purpose:** Track individual swap orders  
**Seeds:** `["swap_order", order_id.to_le_bytes()]`  
**Size:** 98 bytes

```rust
pub struct SwapOrder {
    pub order_id: u64,              // 8 bytes  - Unique identifier
    pub recipient: Pubkey,          // 32 bytes - Token recipient
    pub usdc_amount: u64,           // 8 bytes  - USDC bridged
    pub min_output_amount: u64,     // 8 bytes  - Minimum tokens expected
    pub destination_mint: Pubkey,   // 32 bytes - Output token mint
    pub deadline: i64,              // 8 bytes  - Expiration timestamp
    pub status: OrderStatus,        // 1 byte   - Current status
    pub bump: u8,                   // 1 byte   - PDA bump seed
}

pub enum OrderStatus {
    Pending,    // Being processed
    Completed,  // Successfully swapped
    Refunded,   // Failed, USDC returned
    Failed,     // Error occurred
}
```

**Lifecycle:**
1. Created when Across delivers USDC
2. Status: `Pending`
3. Jupiter swap executed
4. Status: `Completed` OR `Refunded` on failure
5. Remains on-chain for record keeping

### Instruction Flow

#### 1. Initialize

**Purpose:** Set up program configuration  
**Authority:** Admin only  
**Called:** Once at deployment

```
initialize(params: InitializeParams)
├─ Create Config PDA
├─ Set admin authority
├─ Set Across handler
├─ Set Jupiter program ID
├─ Set USDC mint
├─ Configure fees
└─ Set paused = false
```

**Validation:**
- Config PDA doesn't already exist
- Fee BPS ≤ 1000 (max 10%)
- All pubkeys are valid

#### 2. Process Bridge and Swap

**Purpose:** Execute swap for bridged USDC  
**Authority:** Across handler only  
**Called:** By Across when USDC arrives

```
process_bridge_and_swap(params: ProcessBridgeAndSwapParams)
├─ Validate caller is Across handler
├─ Check program not paused
├─ Verify deadline not exceeded
├─ Create SwapOrder PDA
│
├─ Transfer USDC from Across to program
│
├─ Calculate and deduct fee
│  ├─ Fee = (amount × fee_bps) / 10000
│  └─ Transfer fee to fee_recipient
│
├─ Execute Jupiter swap via CPI
│  ├─ Parse Jupiter instruction data
│  ├─ Build account list
│  ├─ Execute CPI with program authority
│  └─ Catch errors
│
├─ Verify output amount
│  └─ If output ≥ min_output_amount: SUCCESS
│
└─ Handle result
   ├─ SUCCESS: Transfer tokens to user, mark Completed
   └─ FAILURE: Refund USDC to user, mark Refunded
```

**Accounts Required:**
- Config (read)
- SwapOrder (write, init)
- Across handler (signer)
- Recipient (read)
- USDC mint (read)
- Source USDC account (write)
- Program USDC account (write)
- Destination token mint (read)
- Recipient destination account (write, init_if_needed)
- Recipient USDC account (write, init_if_needed)
- Fee recipient account (write, init_if_needed)
- Jupiter program (read)
- Token program
- Associated token program
- System program

**Error Handling:**
All errors result in USDC refund to user:
- Jupiter swap failure
- Slippage exceeded
- Deadline exceeded
- Invalid token accounts
- Math overflow

#### 3. Execute Jupiter Swap (Internal)

**Purpose:** CPI to Jupiter aggregator  
**Authority:** Program internal  
**Called:** By process_bridge_and_swap

```
execute_jupiter_swap(params: ExecuteJupiterSwapParams)
├─ Validate Jupiter program ID
├─ Deserialize swap instruction
├─ Build account metas from remaining_accounts
├─ Create instruction
│  ├─ program_id: Jupiter
│  ├─ accounts: From instruction
│  └─ data: Serialized swap params
│
└─ invoke_signed with program authority
   └─ Seeds: ["config", bump]
```

**Jupiter V6 Instruction Format:**
```rust
// Instruction discriminator (first 8 bytes)
[237, 48, 91, 112, 159, 137, 27, 8]

// Followed by:
struct RouteSwapArgs {
    amount_in: u64,
    minimum_amount_out: u64,
}
```

#### 4. Update Config

**Purpose:** Modify program settings  
**Authority:** Admin only  
**Called:** As needed

```
update_config(params: UpdateConfigParams)
├─ Verify caller is admin
├─ Update admin (if provided)
├─ Update across_handler (if provided)
├─ Update jupiter_program (if provided)
├─ Update fee_recipient (if provided)
└─ Update fee_bps (if provided)
   └─ Validate ≤ 1000
```

#### 5. Pause / Unpause

**Purpose:** Emergency circuit breaker  
**Authority:** Admin only  
**Called:** In emergency situations

```
pause()
├─ Verify caller is admin
└─ Set is_paused = true

unpause()
├─ Verify caller is admin
└─ Set is_paused = false
```

**Effect:**
- When paused, `process_bridge_and_swap` fails immediately
- All other admin functions still work
- Existing swaps not affected

#### 6. Recover Funds (Emergency)

**Purpose:** Recover stuck tokens  
**Authority:** Admin only  
**Called:** Only in emergency

```
recover_funds(params: RecoverFundsParams)
├─ Verify caller is admin
├─ Validate token accounts
└─ Transfer tokens from program to destination
   └─ Use program authority (PDA)
```

## Jupiter Integration

### CPI Architecture

```
SuperSwap Program
│
├─ Has authority over USDC (via PDA)
│
└─ Calls Jupiter via CPI
   │
   └─ Jupiter executes swap
      │
      ├─ Transfers USDC from program
      │
      ├─ Routes through DEXs (Orca, Raydium, etc.)
      │
      └─ Transfers output tokens to program
         │
         └─ Program transfers to user
```

### Account Management

Jupiter requires many accounts:
- Source token account (USDC)
- Destination token account
- DEX program accounts
- AMM/pool accounts
- Oracle accounts (if applicable)

**Challenge:** These accounts are dynamic (different per swap)

**Solution:** Use `remaining_accounts`
```rust
#[derive(Accounts)]
pub struct ExecuteJupiterSwap<'info> {
    // Fixed accounts
    pub config: Account<'info, Config>,
    pub jupiter_program: UncheckedAccount<'info>,
    
    // Note: Additional accounts in remaining_accounts
}

// Access remaining accounts
ctx.remaining_accounts.iter()
```

### Instruction Generation

Frontend generates Jupiter instruction:

```typescript
// 1. Get quote from Jupiter
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
    userPublicKey: PROGRAM_PDA.toString(),
  },
});

// 3. Serialize instruction for Across message
const jupiterSwapData = Buffer.from(swapInstruction, 'base64');
```

## Across Integration

### Message Passing

Across supports passing messages with bridge transactions:

```
EVM Chain                    Solana
┌────────┐                  ┌────────┐
│  User  │─────Bridge──────▶│ Across │
└────────┘   + Message       │Handler │
                             └───┬────┘
                                 │
                         Calls SuperSwap
                                 │
                                 ▼
                         ┌───────────────┐
                         │   SuperSwap   │
                         │   Program     │
                         └───────────────┘
```

### Message Format

```rust
// Serialized with Borsh
pub struct AcrossMessage {
    pub order_id: u64,              // Unique ID (use timestamp)
    pub recipient: Pubkey,          // User's Solana address
    pub usdc_amount: u64,           // Amount bridged (6 decimals)
    pub min_output_amount: u64,     // Minimum tokens expected
    pub destination_mint: Pubkey,   // Token to receive
    pub deadline: i64,              // Unix timestamp
    pub jupiter_swap_data: Vec<u8>, // Serialized Jupiter instruction
}
```

### Frontend Integration

```typescript
import { AcrossClient } from '@across-protocol/sdk';
import * as borsh from 'borsh';

// 1. Get Jupiter quote and instruction
const jupiterData = await getJupiterSwapData(destinationMint, amount);

// 2. Create message
const message = {
  orderId: Date.now(),
  recipient: userSolanaAddress,
  usdcAmount: amount,
  minOutputAmount: jupiterData.minOutput,
  destinationMint: destinationMint,
  deadline: Math.floor(Date.now() / 1000) + 1800,
  jupiterSwapData: jupiterData.instruction,
};

// 3. Serialize message
const messageSchema = new Map([
  [AcrossMessage, {
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
const encodedMessage = borsh.serialize(messageSchema, message);

// 4. Bridge with Across
const tx = await acrossClient.deposit({
  fromChain: 8453, // Base
  toChain: 1151111081099710, // Solana
  token: USDC_ADDRESS,
  amount: amount,
  recipient: SUPERSWAP_PROGRAM_ADDRESS,
  message: encodedMessage,
});
```

## Security Model

### Authority Hierarchy

```
Admin
  │
  ├─── Can update configuration
  ├─── Can pause/unpause
  ├─── Can recover funds
  └─── Cannot execute swaps

Across Handler
  │
  └─── Can trigger swaps only

Program (PDA)
  │
  ├─── Owns USDC tokens
  └─── Signs CPI to Jupiter

User
  │
  └─── Receives output tokens
```

### Trust Assumptions

1. **Admin:** Trusted to:
   - Set correct Across handler
   - Set correct Jupiter program ID
   - Not steal funds (can only recover, not redirect)

2. **Across:** Trusted to:
   - Deliver USDC correctly
   - Only call with valid messages
   - Not call with fake messages

3. **Jupiter:** Trusted to:
   - Execute swaps fairly
   - Not exploit CPI

### Attack Vectors & Mitigations

| Attack | Mitigation |
|--------|------------|
| Fake Across handler calls | Verify `across_handler` matches config |
| Admin stealing funds | Admin can only recover to any address, but all actions are on-chain and auditable |
| Swap manipulation | Use Jupiter (trusted aggregator), verify minimum output |
| Deadline attack | Check deadline before execution |
| Reentrancy | Solana's single-threaded execution prevents reentrancy |
| Integer overflow | Use checked math throughout |

### Audit Checklist

- [ ] All math operations use checked arithmetic
- [ ] All PDAs use correct seeds
- [ ] All authority checks are enforced
- [ ] All token transfers are validated
- [ ] Deadline checks before expensive operations
- [ ] Refund logic always works
- [ ] No way to lock funds permanently
- [ ] Admin actions are logged
- [ ] CPI calls have proper signing

## Performance Considerations

### Compute Units

Estimated compute units per instruction:
- `initialize`: ~10,000 CU
- `process_bridge_and_swap`: ~200,000 CU (varies with Jupiter route)
- `update_config`: ~5,000 CU
- `pause/unpause`: ~3,000 CU
- `recover_funds`: ~15,000 CU

**Note:** Jupiter swaps can use 100,000-400,000 CU depending on route complexity.

### Account Size Optimization

- Config: 178 bytes (very cheap)
- SwapOrder: 98 bytes per order
- Rent exemption: ~0.0014 SOL per order

### Transaction Size

Maximum transaction size: 1232 bytes

Typical `process_bridge_and_swap` transaction:
- Instruction data: ~200 bytes (varies with Jupiter data)
- Accounts: ~800 bytes (varies with Jupiter route)
- Signatures: 64 bytes
- Total: ~1,064 bytes ✅ Under limit

## Future Enhancements

### 1. SVM → EVM Flow

**Architecture:**
```
Solana              Across              EVM Chain
┌────────┐         ┌──────┐           ┌────────┐
│  User  │────────▶│Bridge│──────────▶│  User  │
└────────┘         └──────┘           └────────┘
    │                                      │
    │ Swap to USDC                         │ Swap from USDC
    ▼                                      ▼
┌────────────┐                      ┌──────────────┐
│  Jupiter   │                      │ UniswapV3/V2 │
└────────────┘                      └──────────────┘
```

**New Instructions:**
- `initiate_evm_swap`: Swap tokens → USDC on Solana
- `bridge_to_evm`: Bridge USDC to EVM with swap calldata

### 2. Batching

Support multiple swaps in one transaction:
```rust
pub fn process_batch(
    ctx: Context<ProcessBatch>,
    orders: Vec<ProcessBridgeAndSwapParams>,
) -> Result<()>
```

### 3. Limit Orders

Add order book functionality:
```rust
pub struct LimitOrder {
    pub maker: Pubkey,
    pub input_mint: Pubkey,
    pub output_mint: Pubkey,
    pub input_amount: u64,
    pub min_output_amount: u64,
    pub expiry: i64,
}
```

### 4. Fee Tiers

Implement volume-based fee discounts:
```rust
pub struct FeeTier {
    pub min_volume: u64,
    pub fee_bps: u16,
}
```

## Conclusion

The SuperSwap Solana program provides a secure, efficient, and trustless way to execute cross-chain swaps from EVM to Solana. By integrating with Across for bridging and Jupiter for optimal routing, it offers users the best pricing with automatic refund protection.

Key architectural decisions:
1. **PDA-based authority**: Secure token custody
2. **Modular design**: Easy to extend and audit
3. **Robust error handling**: Users never lose funds
4. **CPI to Jupiter**: Leverages best-in-class routing
5. **Admin controls**: Emergency response capability

The architecture supports future enhancements including reverse (SVM→EVM) flow, batching, and limit orders.

