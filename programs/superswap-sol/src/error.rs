use anchor_lang::prelude::*;

#[error_code]
pub enum SuperSwapError {
    #[msg("Program is currently paused")]
    ProgramPaused,

    #[msg("Unauthorized: caller is not admin")]
    Unauthorized,

    #[msg("Invalid Across message handler")]
    InvalidAcrossHandler,

    #[msg("Invalid recipient address")]
    InvalidRecipient,

    #[msg("Invalid swap calldata")]
    InvalidSwapCalldata,

    #[msg("Swap execution failed")]
    SwapExecutionFailed,

    #[msg("Insufficient output amount")]
    InsufficientOutputAmount,

    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,

    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Refund failed")]
    RefundFailed,

    #[msg("Invalid Jupiter program")]
    InvalidJupiterProgram,

    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Invalid bridge amount")]
    InvalidBridgeAmount,

    #[msg("Deadline exceeded")]
    DeadlineExceeded,

    #[msg("Invalid instruction data")]
    InvalidInstructionData,

    #[msg("USDC token account not found")]
    UsdcTokenAccountNotFound,

    #[msg("Destination token account not found")]
    DestinationTokenAccountNotFound,

    #[msg("Invalid fee configuration")]
    InvalidFeeConfiguration,

    #[msg("Fee calculation failed")]
    FeeCalculationFailed,
}

