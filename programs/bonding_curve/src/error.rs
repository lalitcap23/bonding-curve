use anchor_lang::prelude::*;

#[error_code]
pub enum SwifeyError {
    #[msg("Unauthorized address")]
    UnauthorizedAddress,

    #[msg("Curve limit reached")]
    CurveLimitReached,

    #[msg("Value is not in expected range")]
    IncorrectValueRange,

    #[msg("Amount out is smaller than required amount")]
    InsufficientAmountOut,

    #[msg("Insufficient funds")]
    InsufficientFunds,

    #[msg("Incorrect fee recipient")]
    IncorrectFeeRecipient,

    #[msg("An overflow or underflow occurred during calculation")]
    InvalidReserves,

    #[msg("Curve is not initialized")]
    CurveNotInitialized,

    #[msg("Curve is not completed")]
    CurveNotCompleted,

    #[msg("Already migrated to Raydium")]
    AlreadyMigrated,

    #[msg("Mathematical operation overflow")]
    MathOverflow,

    #[msg("Insufficient SOL balance")]
    InsufficientSolBalance,

    #[msg("Insufficient token balance")]
    InsufficientTokenBalance,

    #[msg("Invalid pool owner")]
    InvalidPoolOwner,

    #[msg("Invalid pool state")]
    InvalidPoolState,

    #[msg("Invalid pool tokens")]
    InvalidPoolTokens,

    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,

    #[msg("Division by zero not allowed")]
    DivisionByZero,

    #[msg("Invalid token allocation - must allocate at least 80% to bonding curve")]
    InvalidTokenAllocation,

    #[msg("Invalid curve limit - must be exactly 42 SOL")]
    InvalidCurveLimit,

    #[msg("Invalid initial SOL reserve - must be exactly 12.33 SOL")]
    InvalidInitialSolReserve,
}
