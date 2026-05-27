use crate::errors::AmmError;
use crate::math::calculate_swap_output;
use crate::state::AmmPool;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(
        mut,
        seeds = [b"pool", pool.mint_a.as_ref(), pool.mint_b.as_ref()],
        bump = pool.bump,
        constraint = !pool.paused @ AmmError::ProtocolPaused
    )]
    pub pool: Account<'info, AmmPool>,

    #[account(mut, address = pool.vault_a)]
    pub vault_a: Account<'info, TokenAccount>,
    #[account(mut, address = pool.vault_b)]
    pub vault_b: Account<'info, TokenAccount>,

    #[account(mut)]
    pub trader_token_a_ata: Account<'info, TokenAccount>,
    #[account(mut)]
    pub trader_token_b_ata: Account<'info, TokenAccount>,

    #[account(mut)]
    pub trader: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handle_swap_a_to_b(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
    require!(amount_in > 0, AmmError::InvalidAmount);
    let reserve_a = ctx.accounts.vault_a.amount;
    let reserve_b = ctx.accounts.vault_b.amount;

    let amount_out =
        calculate_swap_output(amount_in, reserve_a, reserve_b, ctx.accounts.pool.fee_bps)?;
    require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.trader_token_a_ata.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.trader.to_account_info(),
            },
        ),
        amount_in,
    )?;

    let mint_a = ctx.accounts.pool.mint_a;
    let mint_b = ctx.accounts.pool.mint_b;
    let bump = ctx.accounts.pool.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[b"pool", mint_a.as_ref(), mint_b.as_ref(), &[bump]]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.trader_token_b_ata.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount_out,
    )?;

    let pool = &mut ctx.accounts.pool;
    pool.total_swaps = pool
        .total_swaps
        .checked_add(1)
        .ok_or(error!(AmmError::MathOverflow))?;

    Ok(())
}

pub fn handle_swap_b_to_a(ctx: Context<Swap>, amount_in: u64, min_amount_out: u64) -> Result<()> {
    require!(amount_in > 0, AmmError::InvalidAmount);
    let reserve_a = ctx.accounts.vault_a.amount;
    let reserve_b = ctx.accounts.vault_b.amount;

    let amount_out =
        calculate_swap_output(amount_in, reserve_b, reserve_a, ctx.accounts.pool.fee_bps)?;
    require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.trader_token_b_ata.to_account_info(),
                to: ctx.accounts.vault_b.to_account_info(),
                authority: ctx.accounts.trader.to_account_info(),
            },
        ),
        amount_in,
    )?;

    let mint_a = ctx.accounts.pool.mint_a;
    let mint_b = ctx.accounts.pool.mint_b;
    let bump = ctx.accounts.pool.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[b"pool", mint_a.as_ref(), mint_b.as_ref(), &[bump]]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.trader_token_a_ata.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount_out,
    )?;

    let pool = &mut ctx.accounts.pool;
    pool.total_swaps = pool
        .total_swaps
        .checked_add(1)
        .ok_or(error!(AmmError::MathOverflow))?;

    Ok(())
}
