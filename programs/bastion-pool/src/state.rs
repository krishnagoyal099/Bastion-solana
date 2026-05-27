use anchor_lang::prelude::*;

#[account]
pub struct BastionConfig {
    pub authority: Pubkey,   // 32
    pub amm_program: Pubkey, // 32
    pub treasury: Pubkey,    // 32
    pub token_mint: Pubkey,  // 32
    pub wsol_mint: Pubkey,   // 32
    pub paused: bool,        // 1
    pub fee_bps: u16,        // 2
    pub min_order_size: u64, // 8
    pub max_order_size: u64, // 8
    pub total_orders: u64,   // 8
    pub total_volume: u64,   // 8
    pub version: u32,        // 4
    pub bump: u8,            // 1
    pub _reserved: [u8; 32], // 32
}

#[account]
pub struct UserDeposit {
    pub owner: Pubkey,        // 32
    pub sol_deposited: u64,   // 8
    pub token_deposited: u64, // 8
    pub pending_orders: u8,   // 1
    pub total_orders: u32,    // 4
    pub created_at: i64,      // 8
    pub bump: u8,             // 1
    pub _reserved: [u8; 16],  // 16
}

#[account]
pub struct OrderCommitment {
    pub commitment: [u8; 32],        // 32
    pub nullifier_hash: [u8; 32],    // 32
    pub submitter: Pubkey,           // 32
    pub beneficiary: Pubkey,         // 32
    pub escrow_vault: Pubkey,        // 32
    pub amount_commitment: [u8; 32], // 32
    pub side_commitment: [u8; 32],   // 32
    pub price_commitment: [u8; 32],  // 32
    pub status: OrderStatus,         // 1
    pub deposit_mint: Pubkey,        // 32
    pub deposit_amount: u64,         // 8
    pub created_at: i64,             // 8
    pub settled_at: i64,             // 8
    pub proof_hash: [u8; 32],        // 32
    pub bump: u8,                    // 1
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum OrderStatus {
    Committed,
    Revealed,
    Matched,
    Settled,
    Cancelled,
    Expired,
}

#[account]
pub struct NullifierRecord {
    pub nullifier_hash: [u8; 32],   // 32
    pub order_commitment: [u8; 32], // 32
    pub used_at: i64,               // 8
    pub bump: u8,                   // 1
}
