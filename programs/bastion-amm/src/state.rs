use anchor_lang::prelude::*;

#[account]
pub struct AmmPool {
    pub authority: Pubkey,   // 32 — F22: pool authority for pause/admin
    pub mint_a: Pubkey,      // 32
    pub mint_b: Pubkey,      // 32
    pub vault_a: Pubkey,     // 32
    pub vault_b: Pubkey,     // 32
    pub lp_mint: Pubkey,     // 32
    pub fee_bps: u16,        // 2
    pub min_liquidity: u64,  // 8
    pub total_swaps: u64,    // 8
    pub created_at: i64,     // 8
    pub paused: bool,        // 1  — F22: pause mechanism
    pub bump: u8,            // 1
    pub _reserved: [u8; 22], // 22
}

#[account]
pub struct LpPosition {
    pub pool: Pubkey,      // 32
    pub owner: Pubkey,     // 32
    pub lp_shares: u64,    // 8
    pub deposited_at: i64, // 8
    pub bump: u8,          // 1
}
