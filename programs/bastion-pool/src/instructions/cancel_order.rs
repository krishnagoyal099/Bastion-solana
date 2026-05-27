use crate::errors::PoolError;
use crate::state::{BastionConfig, OrderCommitment, OrderStatus, UserDeposit};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct CancelOrder<'info> {
    #[account(
        mut,
        seeds = [b"order", order.commitment.as_ref()],
        bump,
        constraint = order.status == OrderStatus::Committed @ PoolError::InvalidOrderStatus,
        constraint = order.beneficiary == beneficiary.key() @ PoolError::Unauthorized
    )]
    pub order: Box<Account<'info, OrderCommitment>>,

    #[account(
        mut,
        seeds = [b"deposit", beneficiary.key().as_ref()],
        bump
    )]
    pub user_deposit: Box<Account<'info, UserDeposit>>,

    // F3/F11 FIX: Validate escrow matches order's escrow_vault
    #[account(
        mut,
        constraint = escrow.key() == order.escrow_vault @ PoolError::InvalidEscrow
    )]
    pub escrow: Box<Account<'info, TokenAccount>>,

    // F11 FIX: program_vault validated via PDA seeds
    #[account(
        mut,
        seeds = [b"token_vault", config.key().as_ref()],
        bump,
    )]
    pub program_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub beneficiary: Signer<'info>,

    #[account(seeds = [b"config"], bump)]
    pub config: Box<Account<'info, BastionConfig>>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_cancel_order(ctx: Context<CancelOrder>) -> Result<()> {
    // Read values before mutable borrows
    let deposit_mint = ctx.accounts.order.deposit_mint;
    let deposit_amount = ctx.accounts.order.deposit_amount;
    let commitment = ctx.accounts.order.commitment;
    let bump = ctx.accounts.order.bump;
    let wsol_mint = ctx.accounts.config.wsol_mint;

    // Restore virtual deposit limit
    let user_deposit = &mut ctx.accounts.user_deposit;
    if deposit_mint == wsol_mint {
        user_deposit.sol_deposited = user_deposit
            .sol_deposited
            .checked_add(deposit_amount)
            .ok_or(error!(PoolError::MathOverflow))?;
    } else {
        user_deposit.token_deposited = user_deposit
            .token_deposited
            .checked_add(deposit_amount)
            .ok_or(error!(PoolError::MathOverflow))?;
    }
    user_deposit.pending_orders = user_deposit
        .pending_orders
        .checked_sub(1)
        .ok_or(error!(PoolError::MathOverflow))?;

    let signer_seeds: &[&[&[u8]]] = &[&[b"order", commitment.as_ref(), &[bump]]];

    // Move funds from escrow back to program vault
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow.to_account_info(),
                to: ctx.accounts.program_vault.to_account_info(),
                authority: ctx.accounts.order.to_account_info(),
            },
            signer_seeds,
        ),
        deposit_amount,
    )?;

    // F13 FIX: Close escrow token account to reclaim rent
    token::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.escrow.to_account_info(),
            destination: ctx.accounts.beneficiary.to_account_info(),
            authority: ctx.accounts.order.to_account_info(),
        },
        signer_seeds,
    ))?;

    let order = &mut ctx.accounts.order;
    order.status = OrderStatus::Cancelled;

    Ok(())
}
