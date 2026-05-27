use crate::errors::AmmError;
use crate::math::calculate_lp_shares;
use crate::state::{AmmPool, LpPosition};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, MintTo, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    #[account(
        mut,
        seeds = [b"pool", pool.mint_a.as_ref(), pool.mint_b.as_ref()],
        bump = pool.bump,
        constraint = !pool.paused @ AmmError::ProtocolPaused
    )]
    pub pool: Box<Account<'info, AmmPool>>,

    #[account(
        init_if_needed,
        payer = provider,
        space = 8 + 81,
        seeds = [b"lp", pool.key().as_ref(), provider.key().as_ref()],
        bump
    )]
    pub lp_position: Box<Account<'info, LpPosition>>,

    #[account(mut, address = pool.vault_a)]
    pub vault_a: Box<Account<'info, TokenAccount>>,
    #[account(mut, address = pool.vault_b)]
    pub vault_b: Box<Account<'info, TokenAccount>>,

    #[account(mut, address = pool.lp_mint)]
    pub lp_mint: Box<Account<'info, Mint>>,

    #[account(mut, seeds = [b"lp_vault", pool.key().as_ref()], bump)]
    pub pool_lp_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub provider_lp_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub provider_token_a_ata: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub provider_token_b_ata: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub provider: Signer<'info>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

pub fn handle_add_liquidity(
    ctx: Context<AddLiquidity>,
    amount_a: u64,
    amount_b: u64,
    min_lp_shares: u64,
) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, AmmError::InvalidAmount);

    let reserve_a = ctx.accounts.vault_a.amount;
    let reserve_b = ctx.accounts.vault_b.amount;
    let total_supply = ctx.accounts.lp_mint.supply;

    let shares = calculate_lp_shares(amount_a, amount_b, reserve_a, reserve_b, total_supply)?;

    require!(shares >= min_lp_shares, AmmError::SlippageExceeded);

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.provider_token_a_ata.to_account_info(),
                to: ctx.accounts.vault_a.to_account_info(),
                authority: ctx.accounts.provider.to_account_info(),
            },
        ),
        amount_a,
    )?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.provider_token_b_ata.to_account_info(),
                to: ctx.accounts.vault_b.to_account_info(),
                authority: ctx.accounts.provider.to_account_info(),
            },
        ),
        amount_b,
    )?;

    let mint_a = ctx.accounts.pool.mint_a;
    let mint_b = ctx.accounts.pool.mint_b;
    let bump = ctx.accounts.pool.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[b"pool", mint_a.as_ref(), mint_b.as_ref(), &[bump]]];

    if total_supply == 0 {
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.pool_lp_vault.to_account_info(),
                    authority: ctx.accounts.pool.to_account_info(),
                },
                signer_seeds,
            ),
            ctx.accounts.pool.min_liquidity,
        )?;
    }

    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.provider_lp_ata.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        shares,
    )?;

    let position = &mut ctx.accounts.lp_position;
    if position.pool == Pubkey::default() {
        position.pool = ctx.accounts.pool.key();
        position.owner = ctx.accounts.provider.key();
        position.deposited_at = Clock::get()?.unix_timestamp;
        position.bump = ctx.bumps.lp_position;
    }
    position.lp_shares = position
        .lp_shares
        .checked_add(shares)
        .ok_or(AmmError::MathOverflow)?;

    Ok(())
}
