use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    #[msg("Slippage limit exceeded")]
    SlippageExceeded,
    #[msg("Insufficient liquidity in the pool")]
    InsufficientLiquidity,
    #[msg("Insufficient shares to remove")]
    InsufficientShares,
    #[msg("Amount must be greater than zero")]
    InvalidAmount,
    #[msg("Math operation overflow")]
    MathOverflow,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Protocol is paused")]
    ProtocolPaused,
}
