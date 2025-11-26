use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::SuperSwapError;

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ SuperSwapError::Unauthorized
    )]
    pub config: Account<'info, Config>,

    pub admin: Signer<'info>,
}

pub fn handler(ctx: Context<Pause>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.is_paused = true;

    msg!("Program paused");

    Ok(())
}

#[derive(Accounts)]
pub struct Unpause<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ SuperSwapError::Unauthorized
    )]
    pub config: Account<'info, Config>,

    pub admin: Signer<'info>,
}

pub fn handler(ctx: Context<Unpause>) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.is_paused = false;

    msg!("Program unpaused");

    Ok(())
}

