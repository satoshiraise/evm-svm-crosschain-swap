use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction,
    program::invoke_signed,
};

/// Executes a Jupiter swap via CPI
/// 
/// # Arguments
/// * `jupiter_program` - Jupiter program account
/// * `swap_data` - Serialized Jupiter instruction data
/// * `accounts` - Accounts required for the swap
/// * `signer_seeds` - Seeds for PDA signing
///
/// # Returns
/// * `Result<()>` - Success or error
pub fn execute_jupiter_swap(
    jupiter_program: &AccountInfo,
    swap_data: &[u8],
    accounts: &[AccountInfo],
    signer_seeds: &[&[&[u8]]],
) -> Result<()> {
    msg!("Executing Jupiter swap via CPI");
    msg!("Jupiter program: {}", jupiter_program.key());
    msg!("Number of accounts: {}", accounts.len());
    msg!("Swap data length: {}", swap_data.len());

    // Build account metas for the instruction
    let account_metas: Vec<AccountMeta> = accounts
        .iter()
        .map(|account| AccountMeta {
            pubkey: account.key(),
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        })
        .collect();

    // Create the Jupiter instruction
    let jupiter_instruction = Instruction {
        program_id: *jupiter_program.key(),
        accounts: account_metas,
        data: swap_data.to_vec(),
    };

    // Execute the CPI
    invoke_signed(
        &jupiter_instruction,
        accounts,
        signer_seeds,
    )?;

    msg!("Jupiter swap executed successfully");

    Ok(())
}

/// Validates Jupiter swap output meets minimum requirements
pub fn validate_swap_output(
    actual_output: u64,
    min_output: u64,
) -> Result<()> {
    require!(
        actual_output >= min_output,
        crate::error::SuperSwapError::InsufficientOutputAmount
    );
    
    msg!("Swap output validated: {} >= {}", actual_output, min_output);
    
    Ok(())
}

/// Parse Jupiter V6 swap instruction data
/// 
/// Jupiter V6 uses the following instruction format:
/// - First 8 bytes: Instruction discriminator
/// - Following bytes: Instruction parameters
///
/// This function helps parse and validate the instruction data
pub fn parse_jupiter_swap_data(data: &[u8]) -> Result<JupiterSwapParams> {
    require!(data.len() >= 8, crate::error::SuperSwapError::InvalidSwapCalldata);
    
    // In production, you would deserialize the full instruction here
    // For now, we return a placeholder
    Ok(JupiterSwapParams {
        amount_in: 0,
        minimum_amount_out: 0,
    })
}

#[derive(Debug)]
pub struct JupiterSwapParams {
    pub amount_in: u64,
    pub minimum_amount_out: u64,
}

