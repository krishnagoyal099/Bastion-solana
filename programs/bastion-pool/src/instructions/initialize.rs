use crate::state::BastionConfig;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 232,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, BastionConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handle_initialize(
    ctx: Context<Initialize>,
    treasury: Pubkey,
    token_mint: Pubkey,
    wsol_mint: Pubkey,
    amm_program: Pubkey,
) -> Result<()> {
    let config = &mut ctx.accounts.config;
    config.authority = ctx.accounts.authority.key();
    config.amm_program = amm_program;
    config.treasury = treasury;
    config.token_mint = token_mint;
    config.wsol_mint = wsol_mint;
    config.paused = false;
    config.fee_bps = 30; // 0.3%
    config.min_order_size = 100_000_000;
    config.max_order_size = u64::MAX;
    config.total_orders = 0;
    config.total_volume = 0;
    config.version = 1;
    config.bump = ctx.bumps.config;
    Ok(())
}
