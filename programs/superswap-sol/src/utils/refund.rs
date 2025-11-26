use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{Config, SwapOrder, OrderStatus};
use crate::error::SuperSwapError;

/// Refunds USDC to the recipient in case of swap failure
///
/// # Arguments
/// * `config` - Program configuration account
/// * `swap_order` - The swap order to refund
/// * `program_usdc_account` - Program's USDC account (source)
/// * `recipient_usdc_account` - Recipient's USDC account (destination)
/// * `token_program` - SPL Token program
///
/// # Returns
/// * `Result<()>` - Success or error
pub fn refund_usdc<'info>(
    config: &Account<'info, Config>,
    swap_order: &mut Account<'info, SwapOrder>,
    program_usdc_account: &Account<'info, TokenAccount>,
    recipient_usdc_account: &Account<'info, TokenAccount>,
    token_program: &Program<'info, Token>,
) -> Result<()> {
    msg!("Initiating USDC refund for order {}", swap_order.order_id);
    msg!("Refund amount: {}", swap_order.usdc_amount);
    msg!("Recipient: {}", swap_order.recipient);

    // Validate accounts
    require!(
        program_usdc_account.mint == config.usdc_mint,
        SuperSwapError::InvalidTokenMint
    );
    require!(
        recipient_usdc_account.mint == config.usdc_mint,
        SuperSwapError::InvalidTokenMint
    );
    require!(
        recipient_usdc_account.owner == swap_order.recipient,
        SuperSwapError::InvalidRecipient
    );

    // Check if order is in a refundable state
    require!(
        swap_order.status == OrderStatus::Pending || swap_order.status == OrderStatus::Failed,
        SuperSwapError::RefundFailed
    );

    // Calculate refund amount (includes fee that was deducted)
    let refund_amount = swap_order.usdc_amount;

    // Prepare signer seeds
    let config_key = config.key();
    let seeds = &[b"config".as_ref(), &[config.bump]];
    let signer = &[&seeds[..]];

    // Execute transfer
    let transfer_ctx = CpiContext::new_with_signer(
        token_program.to_account_info(),
        Transfer {
            from: program_usdc_account.to_account_info(),
            to: recipient_usdc_account.to_account_info(),
            authority: config.to_account_info(),
        },
        signer,
    );

    token::transfer(transfer_ctx, refund_amount)?;

    // Update swap order status
    swap_order.status = OrderStatus::Refunded;

    msg!("Refund completed successfully");
    msg!("Amount refunded: {}", refund_amount);

    Ok(())
}

/// Calculates the fee amount based on fee_bps
pub fn calculate_fee(amount: u64, fee_bps: u16) -> Result<u64> {
    let fee = (amount as u128)
        .checked_mul(fee_bps as u128)
        .ok_or(SuperSwapError::MathOverflow)?
        .checked_div(10000)
        .ok_or(SuperSwapError::MathOverflow)? as u64;
    
    Ok(fee)
}

/// Calculates the net amount after fee deduction
pub fn calculate_net_amount(amount: u64, fee_bps: u16) -> Result<u64> {
    let fee = calculate_fee(amount, fee_bps)?;
    amount
        .checked_sub(fee)
        .ok_or(SuperSwapError::MathOverflow.into())
}

