use anchor_lang::prelude::*;
use anchor_lang::solana_program::{instruction::Instruction, program::invoke_signed};
use crate::state::*;
use crate::error::SuperSwapError;

#[derive(Accounts)]
pub struct ExecuteJupiterSwap<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump,
    )]
    pub config: Account<'info, Config>,

    /// CHECK: Jupiter program ID
    pub jupiter_program: UncheckedAccount<'info>,

    // Note: Additional accounts required for Jupiter swap will be passed as remaining_accounts
    // These include:
    // - Source token account
    // - Destination token account
    // - Various DEX program accounts
    // - Swap route accounts
}

pub fn handler(ctx: Context<ExecuteJupiterSwap>, params: ExecuteJupiterSwapParams) -> Result<()> {
    let config = &ctx.accounts.config;

    // Validate Jupiter program
    require!(
        ctx.accounts.jupiter_program.key() == config.jupiter_program,
        SuperSwapError::InvalidJupiterProgram
    );

    msg!("Executing Jupiter swap");
    msg!("Swap data length: {}", params.swap_data.len());

    // Parse the Jupiter swap instruction from the swap_data
    // Jupiter V6 uses a specific instruction format that needs to be deserialized
    
    // The swap_data should contain:
    // 1. Instruction discriminator (8 bytes for Anchor)
    // 2. Serialized instruction parameters
    
    // Build the remaining accounts vector for CPI
    let remaining_accounts: Vec<AccountInfo> = ctx.remaining_accounts.to_vec();

    msg!("Number of remaining accounts: {}", remaining_accounts.len());

    // Create the Jupiter instruction
    let jupiter_instruction = Instruction {
        program_id: ctx.accounts.jupiter_program.key(),
        accounts: remaining_accounts
            .iter()
            .map(|account| AccountMeta {
                pubkey: account.key(),
                is_signer: account.is_signer,
                is_writable: account.is_writable,
            })
            .collect(),
        data: params.swap_data.clone(),
    };

    // Execute CPI with program authority
    let config_key = config.key();
    let seeds = &[b"config".as_ref(), &[config.bump]];
    let signer_seeds = &[&seeds[..]];

    invoke_signed(
        &jupiter_instruction,
        &remaining_accounts,
        signer_seeds,
    )?;

    msg!("Jupiter swap executed successfully");

    Ok(())
}

