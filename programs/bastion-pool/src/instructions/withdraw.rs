use crate::errors::PoolError;
use crate::state::{BastionConfig, UserDeposit};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(
        mut,
        seeds = [b"deposit", user.key().as_ref()],
        bump,
        constraint = user_deposit.owner == user.key() @ PoolError::Unauthorized,
        constraint = user_deposit.pending_orders == 0 @ PoolError::PendingOrdersExist
    )]
    pub user_deposit: Account<'info, UserDeposit>,

    #[account(mut)]
    pub user: Signer<'info>,

    // Accepts either wsol_vault or token_vault — validated in handler
    #[account(mut)]
    pub program_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::authority = user,
    )]
    pub user_ata: Account<'info, TokenAccount>,

    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, BastionConfig>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_withdraw(ctx: Context<Withdraw>, amount: u64, is_sol: bool) -> Result<()> {
    require!(amount > 0, PoolError::InvalidOrderDetails);

    let config_key = ctx.accounts.config.key();

    // Validate the vault is the correct PDA
    let expected_seed = if is_sol { b"wsol_vault".as_ref() } else { b"token_vault".as_ref() };
    let (expected_vault, _) = Pubkey::find_program_address(
        &[expected_seed, config_key.as_ref()],
        ctx.program_id,
    );
    require_keys_eq!(
        ctx.accounts.program_vault.key(),
        expected_vault,
        PoolError::Unauthorized
    );

    let user_deposit = &mut ctx.accounts.user_deposit;

    if is_sol {
        require!(
            user_deposit.sol_deposited >= amount,
            PoolError::InsufficientDeposit
        );
        user_deposit.sol_deposited = user_deposit
            .sol_deposited
            .checked_sub(amount)
            .ok_or(error!(PoolError::MathOverflow))?;
    } else {
        require!(
            user_deposit.token_deposited >= amount,
            PoolError::InsufficientDeposit
        );
        user_deposit.token_deposited = user_deposit
            .token_deposited
            .checked_sub(amount)
            .ok_or(error!(PoolError::MathOverflow))?;
    }

    let config_bump = ctx.accounts.config.bump;
    let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[config_bump]]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.program_vault.to_account_info(),
                to: ctx.accounts.user_ata.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        amount,
    )?;

    Ok(())
}
