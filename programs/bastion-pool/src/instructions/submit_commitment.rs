use crate::errors::PoolError;
use crate::state::{BastionConfig, NullifierRecord, OrderCommitment, OrderStatus, UserDeposit};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer};

#[derive(Accounts)]
#[instruction(commitment: [u8; 32], nullifier_hash: [u8; 32])]
pub struct SubmitCommitment<'info> {
    #[account(seeds = [b"config"], bump)]
    pub config: Box<Account<'info, BastionConfig>>,

    #[account(
        mut,
        seeds = [b"deposit", beneficiary.key().as_ref()],
        bump,
    )]
    pub user_deposit: Box<Account<'info, UserDeposit>>,

    #[account(
        init,
        payer = submitter,
        space = 8 + 338,
        seeds = [b"order", commitment.as_ref()],
        bump
    )]
    pub order: Box<Account<'info, OrderCommitment>>,

    #[account(
        init,
        payer = submitter,
        space = 8 + 73,
        seeds = [b"nullifier", nullifier_hash.as_ref()],
        bump
    )]
    pub nullifier: Box<Account<'info, NullifierRecord>>,

    /// CHECK: Beneficiary of the order — must match user_deposit owner
    #[account(
        constraint = beneficiary.key() == user_deposit.owner @ PoolError::Unauthorized
    )]
    pub beneficiary: AccountInfo<'info>,

    #[account(mut)]
    pub submitter: Signer<'info>,

    // F11 FIX: program_vault validated via PDA seeds
    #[account(
        mut,
        seeds = [b"token_vault", config.key().as_ref()],
        bump,
        token::mint = deposit_mint,
    )]
    pub program_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = submitter,
        seeds = [b"escrow", commitment.as_ref()],
        bump,
        token::mint = deposit_mint,
        token::authority = order
    )]
    pub escrow_vault: Box<Account<'info, TokenAccount>>,

    pub deposit_mint: Box<Account<'info, Mint>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
}

pub fn handle_submit_commitment(
    ctx: Context<SubmitCommitment>,
    commitment: [u8; 32],
    nullifier_hash: [u8; 32],
    amount_commitment: [u8; 32],
    side_commitment: [u8; 32],
    price_commitment: [u8; 32],
    proof_hash: [u8; 32],
    deposit_amount: u64,
) -> Result<()> {
    require!(!ctx.accounts.config.paused, PoolError::ProtocolPaused);
    require!(deposit_amount > 0, PoolError::InvalidOrderDetails);

    // F7 FIX: Check the CORRECT deposit balance based on token type (AND, not OR)
    let is_sol = ctx.accounts.deposit_mint.key() == ctx.accounts.config.wsol_mint;
    if is_sol {
        require!(
            ctx.accounts.user_deposit.sol_deposited >= deposit_amount,
            PoolError::InsufficientDeposit
        );
    } else {
        require!(
            ctx.accounts.deposit_mint.key() == ctx.accounts.config.token_mint,
            PoolError::InvalidDepositMint
        );
        require!(
            ctx.accounts.user_deposit.token_deposited >= deposit_amount,
            PoolError::InsufficientDeposit
        );
    }

    // F1 PARTIAL FIX: Reject zero proof_hash/nullifier/commitment
    let zero_hash = [0u8; 32];
    require!(proof_hash != zero_hash, PoolError::InvalidProof);
    require!(nullifier_hash != zero_hash, PoolError::InvalidProof);
    require!(commitment != zero_hash, PoolError::InvalidCommitment);

    let config_key = ctx.accounts.config.key();
    let config_bump = ctx.bumps.config;
    let signer_seeds: &[&[&[u8]]] = &[&[b"config", &[config_bump]]];

    // Lock deposit into escrow
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.program_vault.to_account_info(),
                to: ctx.accounts.escrow_vault.to_account_info(),
                authority: ctx.accounts.config.to_account_info(),
            },
            signer_seeds,
        ),
        deposit_amount,
    )?;

    // Decrement the correct deposit balance
    let user_deposit = &mut ctx.accounts.user_deposit;
    if is_sol {
        user_deposit.sol_deposited = user_deposit
            .sol_deposited
            .checked_sub(deposit_amount)
            .ok_or(error!(PoolError::MathOverflow))?;
    } else {
        user_deposit.token_deposited = user_deposit
            .token_deposited
            .checked_sub(deposit_amount)
            .ok_or(error!(PoolError::MathOverflow))?;
    }
    user_deposit.pending_orders = user_deposit
        .pending_orders
        .checked_add(1)
        .ok_or(error!(PoolError::MathOverflow))?;
    user_deposit.total_orders = user_deposit
        .total_orders
        .checked_add(1)
        .ok_or(error!(PoolError::MathOverflow))?;

    let order = &mut ctx.accounts.order;
    order.commitment = commitment;
    order.nullifier_hash = nullifier_hash;
    order.submitter = ctx.accounts.submitter.key();
    order.beneficiary = ctx.accounts.beneficiary.key();
    order.escrow_vault = ctx.accounts.escrow_vault.key();
    order.amount_commitment = amount_commitment;
    order.side_commitment = side_commitment;
    order.price_commitment = price_commitment;
    order.status = OrderStatus::Committed;
    order.deposit_mint = ctx.accounts.deposit_mint.key();
    order.deposit_amount = deposit_amount;
    order.created_at = Clock::get()?.unix_timestamp;
    order.proof_hash = proof_hash;
    order.bump = ctx.bumps.order;

    let nullifier = &mut ctx.accounts.nullifier;
    nullifier.nullifier_hash = nullifier_hash;
    nullifier.order_commitment = commitment;
    nullifier.used_at = Clock::get()?.unix_timestamp;
    nullifier.bump = ctx.bumps.nullifier;

    Ok(())
}
