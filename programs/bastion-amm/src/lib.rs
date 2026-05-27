pub mod errors;
pub mod instructions;
pub mod math;
pub mod state;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("BvFgtfCEeCcMHoN1PRHSXkdzVYTka1NsrVBTeHmnDN2D");

#[program]
pub mod bastion_amm {
    use super::*;

    pub fn initialize_pool(ctx: Context<InitializePool>, fee_bps: u16) -> Result<()> {
        handle_initialize_pool(ctx, fee_bps)
    }

    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
        min_lp_shares: u64,
    ) -> Result<()> {
        handle_add_liquidity(ctx, amount_a, amount_b, min_lp_shares)
    }

    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        shares: u64,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> Result<()> {
        handle_remove_liquidity(ctx, shares, min_amount_a, min_amount_b)
    }

    pub fn swap_a_to_b(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
        handle_swap_a_to_b(ctx, amount_in, min_amount_out)
    }

    pub fn swap_b_to_a(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
        handle_swap_b_to_a(ctx, amount_in, min_amount_out)
    }

    pub fn pause_pool(ctx: Context<PausePool>, paused: bool) -> Result<()> {
        handle_pause_pool(ctx, paused)
    }
}
