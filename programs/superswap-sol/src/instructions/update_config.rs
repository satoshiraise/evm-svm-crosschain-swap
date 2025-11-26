use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::SuperSwapError;

#[derive(Accounts)]
pub struct UpdateConfig<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ SuperSwapError::Unauthorized
    )]
    pub config: Account<'info, Config>,

    pub admin: Signer<'info>,
}

pub fn handler(ctx: Context<UpdateConfig>, params: UpdateConfigParams) -> Result<()> {
    let config = &mut ctx.accounts.config;

    if let Some(new_admin) = params.new_admin {
        config.admin = new_admin;
        msg!("Admin updated to: {}", new_admin);
    }

    if let Some(new_across_handler) = params.new_across_handler {
        config.across_handler = new_across_handler;
        msg!("Across handler updated to: {}", new_across_handler);
    }

    if let Some(new_jupiter_program) = params.new_jupiter_program {
        config.jupiter_program = new_jupiter_program;
        msg!("Jupiter program updated to: {}", new_jupiter_program);
    }

    if let Some(new_fee_recipient) = params.new_fee_recipient {
        config.fee_recipient = new_fee_recipient;
        msg!("Fee recipient updated to: {}", new_fee_recipient);
    }

    if let Some(new_fee_bps) = params.new_fee_bps {
        require!(new_fee_bps <= 1000, SuperSwapError::InvalidFeeConfiguration);
        config.fee_bps = new_fee_bps;
        msg!("Fee BPS updated to: {}", new_fee_bps);
    }

    Ok(())
}

