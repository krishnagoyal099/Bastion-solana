use crate::errors::PoolError;
use crate::state::{BastionConfig, OrderCommitment, OrderStatus, UserDeposit};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Token, TokenAccount, Transfer};

/// Order expiry timeout in seconds (24 hours)
const ORDER_EXPIRY_SECONDS: i64 = 86400;

#[derive(Accounts)]
pub struct ExpireOrder<'info> {
    #[account(
        mut,
        seeds = [b"order", order.commitment.as_ref()],
        bump,
        constraint = order.status == OrderStatus::Committed @ PoolError::InvalidOrderStatus
    )]
    pub order: Box<Account<'info, OrderCommitment>>,

    #[account(
        mut,
        seeds = [b"deposit", order.beneficiary.as_ref()],
        bump
    )]
    pub user_deposit: Box<Account<'info, UserDeposit>>,

    #[account(
        mut,
        constraint = escrow.key() == order.escrow_vault @ PoolError::InvalidEscrow
    )]
    pub escrow: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [b"token_vault", config.key().as_ref()],
        bump,
    )]
    pub program_vault: Box<Account<'info, TokenAccount>>,

    /// CHECK: Receives rent refund from closed escrow
    #[account(mut)]
    pub rent_receiver: AccountInfo<'info>,

    /// Anyone can call expire after timeout (cranker-friendly)
    pub cranker: Signer<'info>,

    #[account(seeds = [b"config"], bump)]
    pub config: Box<Account<'info, BastionConfig>>,

    pub token_program: Program<'info, Token>,
}

pub fn handle_expire_order(ctx: Context<ExpireOrder>) -> Result<()> {
    let now = Clock::get()?.unix_timestamp;
    let order_age = now
        .checked_sub(ctx.accounts.order.created_at)
        .ok_or(error!(PoolError::MathOverflow))?;

    require!(order_age >= ORDER_EXPIRY_SECONDS, PoolError::InvalidOrderStatus);

    let deposit_mint = ctx.accounts.order.deposit_mint;
    let deposit_amount = ctx.accounts.order.deposit_amount;
    let commitment = ctx.accounts.order.commitment;
    let bump = ctx.accounts.order.bump;
    let wsol_mint = ctx.accounts.config.wsol_mint;

    // Restore deposit balance
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

    // Return funds from escrow to program vault
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

    // Close escrow to reclaim rent
    token::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.escrow.to_account_info(),
            destination: ctx.accounts.rent_receiver.to_account_info(),
            authority: ctx.accounts.order.to_account_info(),
        },
        signer_seeds,
    ))?;

    let order = &mut ctx.accounts.order;
    order.status = OrderStatus::Expired;

    Ok(())
}
