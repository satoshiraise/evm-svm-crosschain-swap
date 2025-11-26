use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::*;
use crate::error::SuperSwapError;

#[derive(Accounts)]
pub struct RecoverFunds<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = admin @ SuperSwapError::Unauthorized
    )]
    pub config: Account<'info, Config>,

    pub admin: Signer<'info>,

    #[account(mut)]
    pub source_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(ctx: Context<RecoverFunds>, params: RecoverFundsParams) -> Result<()> {
    let config = &ctx.accounts.config;

    // Validate token accounts
    require!(
        ctx.accounts.source_token_account.mint == params.token_mint,
        SuperSwapError::InvalidTokenMint
    );
    require!(
        ctx.accounts.destination_token_account.mint == params.token_mint,
        SuperSwapError::InvalidTokenMint
    );

    msg!("Recovering {} tokens", params.amount);
    msg!("Token mint: {}", params.token_mint);

    let config_key = config.key();
    let seeds = &[b"config".as_ref(), &[config.bump]];
    let signer = &[&seeds[..]];

    let transfer_ctx = CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.source_token_account.to_account_info(),
            to: ctx.accounts.destination_token_account.to_account_info(),
            authority: config.to_account_info(),
        },
        signer,
    );

    token::transfer(transfer_ctx, params.amount)?;

    msg!("Funds recovered successfully");

    Ok(())
}

