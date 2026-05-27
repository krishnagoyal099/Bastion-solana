use crate::errors::PoolError;
use crate::state::{BastionConfig, UserDeposit};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct DepositSol<'info> {
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + 110,
        seeds = [b"deposit", user.key().as_ref()],
        bump
    )]
    pub user_deposit: Account<'info, UserDeposit>,

    #[account(mut)]
    pub user: Signer<'info>,

    // F6 FIX: Validate WSOL account is the program vault with correct mint and authority
    #[account(
        mut,
        token::mint = wsol_mint,
        seeds = [b"wsol_vault", config.key().as_ref()],
        bump
    )]
    pub wsol_vault: Account<'info, TokenAccount>,

    pub wsol_mint: Account<'info, Mint>,

    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, BastionConfig>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct DepositToken<'info> {
    #[account(
        mut,
        seeds = [b"deposit", user.key().as_ref()],
        bump
    )]
    pub user_deposit: Account<'info, UserDeposit>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        token::authority = user,
        token::mint = token_mint,
    )]
    pub user_token_account: Account<'info, TokenAccount>,

    // F11 FIX: Validate program_vault is the correct PDA
    #[account(
        mut,
        seeds = [b"token_vault", config.key().as_ref()],
        bump,
        token::mint = token_mint,
    )]
    pub program_vault: Account<'info, TokenAccount>,

    pub token_mint: Account<'info, Mint>,

    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, BastionConfig>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
    require!(amount > 0, PoolError::InvalidOrderDetails);

    // Transfer SOL to WSOL vault
    let ix = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.wsol_vault.key(),
        amount,
    );
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.wsol_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Sync WSOL balance
    let ix_sync = anchor_spl::token::spl_token::instruction::sync_native(
        &anchor_spl::token::spl_token::id(),
        &ctx.accounts.wsol_vault.key(),
    )?;
    anchor_lang::solana_program::program::invoke(
        &ix_sync,
        &[
            ctx.accounts.wsol_vault.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
        ],
    )?;

    let deposit = &mut ctx.accounts.user_deposit;
    if deposit.owner == Pubkey::default() {
        deposit.owner = ctx.accounts.user.key();
        deposit.created_at = Clock::get()?.unix_timestamp;
        deposit.bump = ctx.bumps.user_deposit;
    }
    deposit.sol_deposited = deposit
        .sol_deposited
        .checked_add(amount)
        .ok_or(error!(PoolError::MathOverflow))?;
    Ok(())
}

pub fn handle_deposit_token(ctx: Context<DepositToken>, amount: u64) -> Result<()> {
    require!(amount > 0, PoolError::InvalidOrderDetails);

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.program_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    let deposit = &mut ctx.accounts.user_deposit;
    deposit.token_deposited = deposit
        .token_deposited
        .checked_add(amount)
        .ok_or(error!(PoolError::MathOverflow))?;
    Ok(())
}
