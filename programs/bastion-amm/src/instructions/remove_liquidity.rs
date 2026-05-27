use crate::errors::AmmError;
use crate::math::calculate_remove_liquidity;
use crate::state::{AmmPool, LpPosition};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Burn, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    #[account(
        mut,
        seeds = [b"pool", pool.mint_a.as_ref(), pool.mint_b.as_ref()],
        bump = pool.bump,
        constraint = !pool.paused @ AmmError::ProtocolPaused
    )]
    pub pool: Box<Account<'info, AmmPool>>,

    #[account(
        mut,
        seeds = [b"lp", pool.key().as_ref(), provider.key().as_ref()],
        bump = lp_position.bump,
        constraint = lp_position.owner == provider.key() @ AmmError::Unauthorized
    )]
    pub lp_position: Box<Account<'info, LpPosition>>,

    #[account(mut, address = pool.vault_a)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = pool.vault_b)]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = pool.lp_mint)]
    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(mut)]
    pub provider_lp_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub provider_token_a_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub provider_token_b_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub provider: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

pub fn handle_remove_liquidity(
    ctx: Context<RemoveLiquidity>,
    shares: u64,
    min_amount_a: u64,
    min_amount_b: u64,
) -> Result<()> {
    require!(shares > 0, AmmError::InvalidAmount);
    require!(
        ctx.accounts.lp_position.lp_shares >= shares,
        AmmError::InsufficientShares
    );

    let reserve_a = ctx.accounts.vault_a.amount;
    let reserve_b = ctx.accounts.vault_b.amount;
    let total_supply = ctx.accounts.lp_mint.supply;

    let (amount_a, amount_b) =
        calculate_remove_liquidity(shares, reserve_a, reserve_b, total_supply)?;

    // Slippage protection
    require!(amount_a >= min_amount_a, AmmError::SlippageExceeded);
    require!(amount_b >= min_amount_b, AmmError::SlippageExceeded);

    let mint_a = ctx.accounts.pool.mint_a;
    let mint_b = ctx.accounts.pool.mint_b;
    let bump = ctx.accounts.pool.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[b"pool", mint_a.as_ref(), mint_b.as_ref(), &[bump]]];

    // Transfer token A from vault to provider
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_a.to_account_info(),
                to: ctx.accounts.provider_token_a_ata.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount_a,
    )?;

    // Transfer token B from vault to provider
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.vault_b.to_account_info(),
                to: ctx.accounts.provider_token_b_ata.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount_b,
    )?;

    // Burn LP tokens from provider
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.provider_lp_ata.to_account_info(),
                authority: ctx.accounts.provider.to_account_info(),
            },
        ),
        shares,
    )?;

    // Update LP position
    let position = &mut ctx.accounts.lp_position;
    position.lp_shares = position
        .lp_shares
        .checked_sub(shares)
        .ok_or(AmmError::MathOverflow)?;

    Ok(())
}
