use anchor_lang::prelude::*;

pub mod error;
pub mod instructions;
pub mod state;
pub mod utils;

use instructions::*;

declare_id!("EzUq3vK7g8JvTLQzKvNAzBCjRz6wNJaZMWZPQVRz7nJq");

#[program]
pub mod superswap_sol {
    use super::*;

    /// Initialize the SuperSwap program configuration
    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        instructions::initialize::handler(ctx, params)
    }

    /// Update program configuration (admin only)
    pub fn update_config(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
        instructions::update_config::handler(ctx, params)
    }

    /// Process bridged USDC from Across and execute Jupiter swap
    /// This is called by the Across handler account
    pub fn process_bridge_and_swap(
        ctx: Context<ProcessBridgeAndSwap>,
        params: ProcessBridgeAndSwapParams,
    ) -> Result<()> {
        instructions::process_bridge_and_swap::handler(ctx, params)
    }

    /// Execute a Jupiter swap using provided instructions
    /// Internal instruction used by process_bridge_and_swap
    pub fn execute_jupiter_swap(
        ctx: Context<ExecuteJupiterSwap>,
        params: ExecuteJupiterSwapParams,
    ) -> Result<()> {
        instructions::execute_jupiter_swap::handler(ctx, params)
    }

    /// Emergency function to recover stuck funds (admin only)
    pub fn recover_funds(ctx: Context<RecoverFunds>, params: RecoverFundsParams) -> Result<()> {
        instructions::recover_funds::handler(ctx, params)
    }

    /// Pause the program (admin only)
    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        instructions::pause::handler(ctx)
    }

    /// Unpause the program (admin only)
    pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
        instructions::unpause::handler(ctx)
    }
}
