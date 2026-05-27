use crate::errors::PoolError;
use crate::state::{BastionConfig, OrderCommitment, OrderStatus, UserDeposit};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, CloseAccount, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct MatchSettle<'info> {
    #[account(seeds = [b"config"], bump)]
    pub config: Box<Account<'info, BastionConfig>>,

    #[account(
        mut,
        seeds = [b"order", order_a.commitment.as_ref()],
        bump,
        constraint = order_a.status == OrderStatus::Committed || order_a.status == OrderStatus::Revealed
    )]
    pub order_a: Box<Account<'info, OrderCommitment>>,

    #[account(
        mut,
        seeds = [b"order", order_b.commitment.as_ref()],
        bump,
        constraint = order_b.status == OrderStatus::Committed || order_b.status == OrderStatus::Revealed
    )]
    pub order_b: Box<Account<'info, OrderCommitment>>,

    // Escrow accounts — validated against order state
    #[account(
        mut,
        constraint = escrow_a.key() == order_a.escrow_vault @ PoolError::InvalidEscrow
    )]
    pub escrow_a: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        constraint = escrow_b.key() == order_b.escrow_vault @ PoolError::InvalidEscrow
    )]
    pub escrow_b: Box<Account<'info, TokenAccount>>,

    // Beneficiary ATAs — validated against order beneficiary
    #[account(
        mut,
        token::authority = order_a.beneficiary,
    )]
    pub beneficiary_a_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::authority = order_b.beneficiary,
    )]
    pub beneficiary_b_ata: Box<Account<'info, TokenAccount>>,

    // User deposit accounts to decrement pending_orders
    #[account(
        mut,
        seeds = [b"deposit", order_a.beneficiary.as_ref()],
        bump
    )]
    pub user_deposit_a: Box<Account<'info, UserDeposit>>,

    #[account(
        mut,
        seeds = [b"deposit", order_b.beneficiary.as_ref()],
        bump
    )]
    pub user_deposit_b: Box<Account<'info, UserDeposit>>,

    pub token_program: Program<'info, Token>,

    // Only the protocol authority can trigger settlement
    #[account(
        constraint = match_authority.key() == config.authority @ PoolError::Unauthorized
    )]
    pub match_authority: Signer<'info>,
}

pub fn handle_match_settle(
    ctx: Context<MatchSettle>,
    _match_proof: Vec<u8>,
    _execution_price: u64,
) -> Result<()> {
    require!(!ctx.accounts.config.paused, PoolError::ProtocolPaused);

    // --- Transfer from escrow A to Beneficiary B ---
    let commitment_a = ctx.accounts.order_a.commitment;
    let bump_a = ctx.accounts.order_a.bump;
    let signer_seeds_a: &[&[&[u8]]] = &[&[b"order", commitment_a.as_ref(), &[bump_a]]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_a.to_account_info(),
                to: ctx.accounts.beneficiary_b_ata.to_account_info(),
                authority: ctx.accounts.order_a.to_account_info(),
            },
            signer_seeds_a,
        ),
        ctx.accounts.order_a.deposit_amount,
    )?;

    // Close escrow A — reclaim rent to match_authority
    token::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.escrow_a.to_account_info(),
            destination: ctx.accounts.match_authority.to_account_info(),
            authority: ctx.accounts.order_a.to_account_info(),
        },
        signer_seeds_a,
    ))?;

    // --- Transfer from escrow B to Beneficiary A ---
    let commitment_b = ctx.accounts.order_b.commitment;
    let bump_b = ctx.accounts.order_b.bump;
    let signer_seeds_b: &[&[&[u8]]] = &[&[b"order", commitment_b.as_ref(), &[bump_b]]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.escrow_b.to_account_info(),
                to: ctx.accounts.beneficiary_a_ata.to_account_info(),
                authority: ctx.accounts.order_b.to_account_info(),
            },
            signer_seeds_b,
        ),
        ctx.accounts.order_b.deposit_amount,
    )?;

    // Close escrow B
    token::close_account(CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        CloseAccount {
            account: ctx.accounts.escrow_b.to_account_info(),
            destination: ctx.accounts.match_authority.to_account_info(),
            authority: ctx.accounts.order_b.to_account_info(),
        },
        signer_seeds_b,
    ))?;

    // --- Update order states ---
    let now = Clock::get()?.unix_timestamp;

    let order_a = &mut ctx.accounts.order_a;
    order_a.status = OrderStatus::Settled;
    order_a.settled_at = now;

    let order_b = &mut ctx.accounts.order_b;
    order_b.status = OrderStatus::Settled;
    order_b.settled_at = now;

    // Decrement pending orders on both user deposits
    let deposit_a = &mut ctx.accounts.user_deposit_a;
    deposit_a.pending_orders = deposit_a
        .pending_orders
        .checked_sub(1)
        .ok_or(error!(PoolError::MathOverflow))?;

    let deposit_b = &mut ctx.accounts.user_deposit_b;
    deposit_b.pending_orders = deposit_b
        .pending_orders
        .checked_sub(1)
        .ok_or(error!(PoolError::MathOverflow))?;

    Ok(())
}
