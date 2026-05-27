use crate::errors::AmmError;
use crate::state::AmmPool;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct PausePool<'info> {
    #[account(
        mut,
        seeds = [b"pool", pool.mint_a.as_ref(), pool.mint_b.as_ref()],
        bump = pool.bump,
        constraint = pool.authority == authority.key() @ AmmError::Unauthorized
    )]
    pub pool: Account<'info, AmmPool>,

    pub authority: Signer<'info>,
}

pub fn handle_pause_pool(ctx: Context<PausePool>, paused: bool) -> Result<()> {
    ctx.accounts.pool.paused = paused;
    Ok(())
}
