use anchor_lang::prelude::*;

/// Global configuration for the SuperSwap program
#[account]
pub struct Config {
    /// Program admin who can update configuration
    pub admin: Pubkey,
    
    /// Across handler account that can trigger swaps
    pub across_handler: Pubkey,
    
    /// Jupiter program ID for swaps
    pub jupiter_program: Pubkey,
    
    /// USDC mint address on Solana
    pub usdc_mint: Pubkey,
    
    /// Fee recipient address
    pub fee_recipient: Pubkey,
    
    /// Fee in basis points (1 bp = 0.01%)
    pub fee_bps: u16,
    
    /// Whether the program is paused
    pub is_paused: bool,
    
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl Config {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        32 + // across_handler
        32 + // jupiter_program
        32 + // usdc_mint
        32 + // fee_recipient
        2 + // fee_bps
        1 + // is_paused
        1; // bump
}

/// Represents a swap order being processed
#[account]
pub struct SwapOrder {
    /// Order ID (derived from Across message)
    pub order_id: u64,
    
    /// User receiving the swapped tokens
    pub recipient: Pubkey,
    
    /// Amount of USDC bridged
    pub usdc_amount: u64,
    
    /// Minimum output amount expected
    pub min_output_amount: u64,
    
    /// Destination token mint
    pub destination_mint: Pubkey,
    
    /// Deadline timestamp
    pub deadline: i64,
    
    /// Status of the order
    pub status: OrderStatus,
    
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl SwapOrder {
    pub const LEN: usize = 8 + // discriminator
        8 + // order_id
        32 + // recipient
        8 + // usdc_amount
        8 + // min_output_amount
        32 + // destination_mint
        8 + // deadline
        1 + // status
        1; // bump
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    /// Order is being processed
    Pending,
    /// Swap completed successfully
    Completed,
    /// Swap failed, USDC refunded
    Refunded,
    /// Order failed with error
    Failed,
}

/// Parameters for initialization
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub across_handler: Pubkey,
    pub jupiter_program: Pubkey,
    pub usdc_mint: Pubkey,
    pub fee_recipient: Pubkey,
    pub fee_bps: u16,
}

/// Parameters for updating configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateConfigParams {
    pub new_admin: Option<Pubkey>,
    pub new_across_handler: Option<Pubkey>,
    pub new_jupiter_program: Option<Pubkey>,
    pub new_fee_recipient: Option<Pubkey>,
    pub new_fee_bps: Option<u16>,
}

/// Parameters for processing bridge and swap
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ProcessBridgeAndSwapParams {
    pub order_id: u64,
    pub recipient: Pubkey,
    pub usdc_amount: u64,
    pub min_output_amount: u64,
    pub destination_mint: Pubkey,
    pub deadline: i64,
    pub jupiter_swap_data: Vec<u8>,
}

/// Parameters for executing Jupiter swap
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteJupiterSwapParams {
    pub swap_data: Vec<u8>,
}

/// Parameters for recovering funds
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RecoverFundsParams {
    pub token_mint: Pubkey,
    pub amount: u64,
}

