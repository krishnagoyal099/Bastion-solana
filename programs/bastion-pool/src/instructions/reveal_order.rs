use crate::errors::PoolError;
use crate::state::{BastionConfig, OrderCommitment, OrderStatus};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::hashv;

#[derive(Accounts)]
pub struct RevealOrder<'info> {
    #[account(
        mut,
        seeds = [b"order", order.commitment.as_ref()],
        bump,
        constraint = order.status == OrderStatus::Committed @ PoolError::InvalidOrderStatus,
        constraint = order.beneficiary == beneficiary.key() @ PoolError::Unauthorized
    )]
    pub order: Account<'info, OrderCommitment>,

    pub beneficiary: Signer<'info>, // Only beneficiary can reveal

    #[account(seeds = [b"config"], bump)]
    pub config: Account<'info, BastionConfig>,
}

pub fn handle_reveal_order(
    ctx: Context<RevealOrder>,
    amount: u64,
    side: u8,
    price: u64,
    nonce: [u8; 32],
) -> Result<()> {
    // Validate side is valid (0 = Buy, 1 = Sell)
    require!(side <= 1, PoolError::InvalidOrderDetails);

    // Validate amount bounds
    require!(
        amount >= ctx.accounts.config.min_order_size,
        PoolError::InvalidOrderDetails
    );
    require!(
        amount <= ctx.accounts.config.max_order_size,
        PoolError::InvalidOrderDetails
    );

    // Validate price is non-zero
    require!(price > 0, PoolError::InvalidOrderDetails);

    // CRITICAL FIX (F4): Verify commitment hash binding
    // commitment = sha256(amount || side || price || nonce)
    let computed_hash = hashv(&[
        &amount.to_le_bytes(),
        &[side],
        &price.to_le_bytes(),
        &nonce,
    ]);

    require!(
        computed_hash.to_bytes() == ctx.accounts.order.commitment,
        PoolError::InvalidCommitment
    );

    let order = &mut ctx.accounts.order;
    order.status = OrderStatus::Revealed;

    Ok(())
}
