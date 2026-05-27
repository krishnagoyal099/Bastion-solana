use anchor_lang::prelude::*;

#[error_code]
pub enum PoolError {
    #[msg("Protocol is paused")]
    ProtocolPaused,
    #[msg("Insufficient deposit amount")]
    InsufficientDeposit,
    #[msg("Invalid order status")]
    InvalidOrderStatus,
    #[msg("Unauthorized access")]
    Unauthorized,
    #[msg("Pending orders exist")]
    PendingOrdersExist,
    #[msg("Invalid side or amount")]
    InvalidOrderDetails,
    #[msg("Math operation overflow")]
    MathOverflow,
    #[msg("Commitment hash does not match revealed values")]
    InvalidCommitment,
    #[msg("Invalid proof hash")]
    InvalidProof,
    #[msg("Escrow account does not match order")]
    InvalidEscrow,
    #[msg("Invalid program vault")]
    InvalidVault,
    #[msg("Beneficiary ATA does not match order beneficiary")]
    InvalidBeneficiary,
    #[msg("Order has expired")]
    OrderExpired,
    #[msg("Deposit mint does not match expected token")]
    InvalidDepositMint,
}
