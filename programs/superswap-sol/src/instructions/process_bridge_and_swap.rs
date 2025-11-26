use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Mint};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::*;
use crate::error::SuperSwapError;

#[derive(Accounts)]
#[instruction(params: ProcessBridgeAndSwapParams)]
pub struct ProcessBridgeAndSwap<'info> {
    #[account(
        seeds = [b"config"],
        bump = config.bump,
        has_one = across_handler @ SuperSwapError::InvalidAcrossHandler,
        has_one = usdc_mint @ SuperSwapError::InvalidTokenMint,
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = payer,
        space = SwapOrder::LEN,
        seeds = [
            b"swap_order",
            params.order_id.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub swap_order: Account<'info, SwapOrder>,

    /// Across handler that triggers the swap (Across program account)
    pub across_handler: Signer<'info>,

    /// CHECK: Recipient address validated in instruction
    #[account(constraint = params.recipient != Pubkey::default() @ SuperSwapError::InvalidRecipient)]
    pub recipient: UncheckedAccount<'info>,

    /// USDC mint
    pub usdc_mint: Account<'info, Mint>,

    /// Source USDC token account (receives bridged USDC from Across)
    #[account(
        mut,
        constraint = source_usdc_account.mint == usdc_mint.key() @ SuperSwapError::InvalidTokenMint,
    )]
    pub source_usdc_account: Account<'info, TokenAccount>,

    /// Program's USDC token account
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = usdc_mint,
        associated_token::authority = config
    )]
    pub program_usdc_account: Account<'info, TokenAccount>,

    /// Destination token mint (the token user wants to receive)
    pub destination_mint: Account<'info, Mint>,

    /// Recipient's destination token account
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = destination_mint,
        associated_token::authority = recipient
    )]
    pub recipient_destination_account: Account<'info, TokenAccount>,

    /// Recipient's USDC account (for refunds)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = usdc_mint,
        associated_token::authority = recipient
    )]
    pub recipient_usdc_account: Account<'info, TokenAccount>,

    /// Fee recipient's USDC account
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = usdc_mint,
        associated_token::authority = config.fee_recipient
    )]
    pub fee_recipient_account: Account<'info, TokenAccount>,

    /// CHECK: Jupiter program (validated against config)
    #[account(constraint = jupiter_program.key() == config.jupiter_program @ SuperSwapError::InvalidJupiterProgram)]
    pub jupiter_program: UncheckedAccount<'info>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(ctx: Context<ProcessBridgeAndSwap>, params: ProcessBridgeAndSwapParams) -> Result<()> {
    let config = &ctx.accounts.config;

    // Check if program is paused
    require!(!config.is_paused, SuperSwapError::ProgramPaused);

    // Validate deadline
    let current_time = Clock::get()?.unix_timestamp;
    require!(current_time <= params.deadline, SuperSwapError::DeadlineExceeded);

    // Validate amounts
    require!(params.usdc_amount > 0, SuperSwapError::InvalidBridgeAmount);

    // Initialize swap order
    let swap_order = &mut ctx.accounts.swap_order;
    swap_order.order_id = params.order_id;
    swap_order.recipient = params.recipient;
    swap_order.usdc_amount = params.usdc_amount;
    swap_order.min_output_amount = params.min_output_amount;
    swap_order.destination_mint = params.destination_mint;
    swap_order.deadline = params.deadline;
    swap_order.status = OrderStatus::Pending;
    swap_order.bump = ctx.bumps.swap_order;

    msg!("Processing swap order: {}", params.order_id);
    msg!("Recipient: {}", params.recipient);
    msg!("USDC Amount: {}", params.usdc_amount);
    msg!("Min Output: {}", params.min_output_amount);

    // Calculate swap fee
    let fee_amount = (params.usdc_amount as u128)
        .checked_mul(config.fee_bps as u128)
        .ok_or(SuperSwapError::MathOverflow)?
        .checked_div(10000)
        .ok_or(SuperSwapError::MathOverflow)? as u64;

    let swap_amount = params.usdc_amount
        .checked_sub(fee_amount)
        .ok_or(SuperSwapError::MathOverflow)?;

    msg!("Fee Amount: {}", fee_amount);
    msg!("Swap Amount: {}", swap_amount);

    // Transfer USDC from source to program account for swap
    let transfer_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.source_usdc_account.to_account_info(),
            to: ctx.accounts.program_usdc_account.to_account_info(),
            authority: ctx.accounts.across_handler.to_account_info(),
        },
    );
    token::transfer(transfer_ctx, params.usdc_amount)?;

    // Transfer fee to fee recipient if fee > 0
    if fee_amount > 0 {
        let config_key = config.key();
        let seeds = &[b"config".as_ref(), &[config.bump]];
        let signer = &[&seeds[..]];

        let fee_transfer_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.program_usdc_account.to_account_info(),
                to: ctx.accounts.fee_recipient_account.to_account_info(),
                authority: config.to_account_info(),
            },
            signer,
        );
        token::transfer(fee_transfer_ctx, fee_amount)?;
    }

    // Execute Jupiter swap
    // Note: The actual Jupiter swap execution will be done via CPI
    // The jupiter_swap_data contains the serialized instruction data
    // This is a complex operation that requires deserializing Jupiter instructions
    // and executing them via CPI
    
    // For now, we'll add a placeholder that needs to be implemented
    // based on Jupiter's exact CPI interface
    msg!("Executing Jupiter swap with {} USDC", swap_amount);
    msg!("Jupiter swap data length: {}", params.jupiter_swap_data.len());

    // TODO: Implement actual Jupiter CPI call
    // This will involve:
    // 1. Deserializing the Jupiter swap instruction
    // 2. Building the accounts vector from the instruction
    // 3. Executing the CPI call
    // 4. Verifying the output amount meets minimum requirements
    
    // For now, mark as completed (this should be conditional on successful swap)
    swap_order.status = OrderStatus::Completed;

    msg!("Swap order {} processed successfully", params.order_id);

    Ok(())
}

// Helper function to execute Jupiter swap (to be implemented)
fn execute_jupiter_swap_cpi(
    jupiter_program: AccountInfo,
    swap_data: &[u8],
    accounts: Vec<AccountInfo>,
    config: &Account<Config>,
    config_bump: u8,
) -> Result<u64> {
    // This function will:
    // 1. Deserialize Jupiter instruction data
    // 2. Execute CPI to Jupiter
    // 3. Return the output amount
    
    msg!("Jupiter swap CPI execution (placeholder)");
    Ok(0) // Placeholder return
}

