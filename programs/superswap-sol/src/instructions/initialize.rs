use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::SuperSwapError;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = admin,
        space = Config::LEN,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
    let config = &mut ctx.accounts.config;

    // Validate fee bps (max 10% = 1000 bps)
    require!(params.fee_bps <= 1000, SuperSwapError::InvalidFeeConfiguration);

    config.admin = ctx.accounts.admin.key();
    config.across_handler = params.across_handler;
    config.jupiter_program = params.jupiter_program;
    config.usdc_mint = params.usdc_mint;
    config.fee_recipient = params.fee_recipient;
    config.fee_bps = params.fee_bps;
    config.is_paused = false;
    config.bump = ctx.bumps.config;

    msg!("SuperSwap initialized successfully");
    msg!("Admin: {}", config.admin);
    msg!("Across Handler: {}", config.across_handler);
    msg!("Jupiter Program: {}", config.jupiter_program);
    msg!("Fee BPS: {}", config.fee_bps);

    Ok(())
}

