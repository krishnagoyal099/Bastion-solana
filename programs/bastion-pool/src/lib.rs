pub mod errors;
pub mod instructions;
pub mod state;

use anchor_lang::prelude::*;
use instructions::*;

declare_id!("CbHS6twCMkYyodaEUtvonRV6HVBZnkGjekohLqXJziU5");

#[program]
pub mod bastion_pool {
    use super::*;

    pub fn initialize(
        ctx: Context<Initialize>,
        treasury: Pubkey,
        token_mint: Pubkey,
        wsol_mint: Pubkey,
        amm_program: Pubkey,
    ) -> Result<()> {
        handle_initialize(ctx, treasury, token_mint, wsol_mint, amm_program)
    }

    pub fn create_vaults(ctx: Context<CreateVaults>) -> Result<()> {
        handle_create_vaults(ctx)
    }

    pub fn deposit_sol(ctx: Context<DepositSol>, amount: u64) -> Result<()> {
        handle_deposit_sol(ctx, amount)
    }

    pub fn deposit_token(ctx: Context<DepositToken>, amount: u64) -> Result<()> {
        handle_deposit_token(ctx, amount)
    }

    pub fn submit_commitment(
        ctx: Context<SubmitCommitment>,
        commitment: [u8; 32],
        nullifier_hash: [u8; 32],
        amount_commitment: [u8; 32],
        side_commitment: [u8; 32],
        price_commitment: [u8; 32],
        proof_hash: [u8; 32],
        deposit_amount: u64,
    ) -> Result<()> {
        handle_submit_commitment(
            ctx,
            commitment,
            nullifier_hash,
            amount_commitment,
            side_commitment,
            price_commitment,
            proof_hash,
            deposit_amount,
        )
    }

    pub fn reveal_order(
        ctx: Context<RevealOrder>,
        amount: u64,
        side: u8,
        price: u64,
        nonce: [u8; 32],
    ) -> Result<()> {
        handle_reveal_order(ctx, amount, side, price, nonce)
    }

    pub fn match_settle(
        ctx: Context<MatchSettle>,
        match_proof: Vec<u8>,
        execution_price: u64,
    ) -> Result<()> {
        handle_match_settle(ctx, match_proof, execution_price)
    }

    pub fn cancel_order(ctx: Context<CancelOrder>) -> Result<()> {
        handle_cancel_order(ctx)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64, is_sol: bool) -> Result<()> {
        handle_withdraw(ctx, amount, is_sol)
    }

    pub fn expire_order(ctx: Context<ExpireOrder>) -> Result<()> {
        handle_expire_order(ctx)
    }
}
