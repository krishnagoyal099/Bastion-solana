use crate::state::AmmPool;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token};

#[derive(Accounts)]
pub struct InitializePool<'info> {
    #[account(
        init,
        payer = authority,
        space = 8 + 242,
        seeds = [b"pool", mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump
    )]
    pub pool: Box<Account<'info, AmmPool>>,

    pub mint_a: Account<'info, Mint>,
    pub mint_b: Account<'info, Mint>,

    /// CHECK: Initialized via token program CPI in the instruction handler
    #[account(mut)]
    pub vault_a: UncheckedAccount<'info>,

    /// CHECK: Initialized via token program CPI in the instruction handler
    #[account(mut)]
    pub vault_b: UncheckedAccount<'info>,

    /// CHECK: Initialized via token program CPI in the instruction handler
    #[account(mut)]
    pub lp_mint: UncheckedAccount<'info>,

    /// CHECK: Initialized via token program CPI in the instruction handler
    #[account(mut)]
    pub pool_lp_vault: UncheckedAccount<'info>,

    #[account(mut)]
    pub authority: Signer<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle_initialize_pool(ctx: Context<InitializePool>, fee_bps: u16) -> Result<()> {
    // Derive vault_a PDA
    let pool_key = ctx.accounts.pool.key();
    let (vault_a_key, vault_a_bump) =
        Pubkey::find_program_address(&[b"vault_a", pool_key.as_ref()], ctx.program_id);
    require_keys_eq!(ctx.accounts.vault_a.key(), vault_a_key);

    // Derive vault_b PDA
    let (vault_b_key, vault_b_bump) =
        Pubkey::find_program_address(&[b"vault_b", pool_key.as_ref()], ctx.program_id);
    require_keys_eq!(ctx.accounts.vault_b.key(), vault_b_key);

    // Derive lp_mint PDA
    let (lp_mint_key, lp_mint_bump) =
        Pubkey::find_program_address(&[b"lp_mint", pool_key.as_ref()], ctx.program_id);
    require_keys_eq!(ctx.accounts.lp_mint.key(), lp_mint_key);

    // Derive lp_vault PDA
    let (lp_vault_key, lp_vault_bump) =
        Pubkey::find_program_address(&[b"lp_vault", pool_key.as_ref()], ctx.program_id);
    require_keys_eq!(ctx.accounts.pool_lp_vault.key(), lp_vault_key);

    let mint_a_key = ctx.accounts.mint_a.key();
    let mint_b_key = ctx.accounts.mint_b.key();
    let pool_bump = ctx.bumps.pool;

    let _pool_seeds: &[&[u8]] = &[
        b"pool",
        mint_a_key.as_ref(),
        mint_b_key.as_ref(),
        &[pool_bump],
    ];

    // Create vault_a token account
    let vault_a_seeds: &[&[u8]] = &[b"vault_a", pool_key.as_ref(), &[vault_a_bump]];
    create_pda_token_account(
        &ctx.accounts.authority,
        &ctx.accounts.vault_a,
        &ctx.accounts.mint_a.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        &ctx.accounts.system_program,
        &ctx.accounts.token_program,
        &ctx.accounts.rent,
        vault_a_seeds,
    )?;

    // Create vault_b token account
    let vault_b_seeds: &[&[u8]] = &[b"vault_b", pool_key.as_ref(), &[vault_b_bump]];
    create_pda_token_account(
        &ctx.accounts.authority,
        &ctx.accounts.vault_b,
        &ctx.accounts.mint_b.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        &ctx.accounts.system_program,
        &ctx.accounts.token_program,
        &ctx.accounts.rent,
        vault_b_seeds,
    )?;

    // Create LP mint
    let lp_mint_seeds: &[&[u8]] = &[b"lp_mint", pool_key.as_ref(), &[lp_mint_bump]];
    create_pda_mint(
        &ctx.accounts.authority,
        &ctx.accounts.lp_mint,
        &ctx.accounts.pool.to_account_info(),
        &ctx.accounts.system_program,
        &ctx.accounts.token_program,
        &ctx.accounts.rent,
        lp_mint_seeds,
        9,
    )?;

    // Create LP vault token account
    let lp_vault_seeds: &[&[u8]] = &[b"lp_vault", pool_key.as_ref(), &[lp_vault_bump]];
    create_pda_token_account(
        &ctx.accounts.authority,
        &ctx.accounts.pool_lp_vault,
        &ctx.accounts.lp_mint.to_account_info(),
        &ctx.accounts.pool.to_account_info(),
        &ctx.accounts.system_program,
        &ctx.accounts.token_program,
        &ctx.accounts.rent,
        lp_vault_seeds,
    )?;

    let pool = &mut ctx.accounts.pool;
    pool.authority = ctx.accounts.authority.key();
    pool.mint_a = mint_a_key;
    pool.mint_b = mint_b_key;
    pool.vault_a = vault_a_key;
    pool.vault_b = vault_b_key;
    pool.lp_mint = lp_mint_key;
    pool.fee_bps = fee_bps;
    pool.min_liquidity = 1000;
    pool.total_swaps = 0;
    pool.created_at = Clock::get()?.unix_timestamp;
    pool.paused = false;
    pool.bump = pool_bump;
    Ok(())
}

fn create_pda_token_account<'info>(
    payer: &Signer<'info>,
    account: &UncheckedAccount<'info>,
    mint: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    system_program: &Program<'info, System>,
    token_program: &Program<'info, Token>,
    _rent_sysvar: &Sysvar<'info, Rent>,
    seeds: &[&[u8]],
) -> Result<()> {
    let rent = Rent::get()?;
    let space = anchor_spl::token::TokenAccount::LEN;
    let lamports = rent.minimum_balance(space);

    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::create_account(
            &payer.key(),
            &account.key(),
            lamports,
            space as u64,
            &anchor_spl::token::spl_token::id(),
        ),
        &[
            payer.to_account_info(),
            account.to_account_info(),
            system_program.to_account_info(),
        ],
        &[seeds],
    )?;

    anchor_lang::solana_program::program::invoke(
        &anchor_spl::token::spl_token::instruction::initialize_account3(
            &anchor_spl::token::spl_token::id(),
            &account.key(),
            &mint.key(),
            &authority.key(),
        )?,
        &[
            account.to_account_info(),
            mint.to_account_info(),
            token_program.to_account_info(),
        ],
    )?;

    Ok(())
}

fn create_pda_mint<'info>(
    payer: &Signer<'info>,
    account: &UncheckedAccount<'info>,
    authority: &AccountInfo<'info>,
    system_program: &Program<'info, System>,
    token_program: &Program<'info, Token>,
    _rent_sysvar: &Sysvar<'info, Rent>,
    seeds: &[&[u8]],
    decimals: u8,
) -> Result<()> {
    let rent = Rent::get()?;
    let space = anchor_spl::token::Mint::LEN;
    let lamports = rent.minimum_balance(space);

    anchor_lang::solana_program::program::invoke_signed(
        &anchor_lang::solana_program::system_instruction::create_account(
            &payer.key(),
            &account.key(),
            lamports,
            space as u64,
            &anchor_spl::token::spl_token::id(),
        ),
        &[
            payer.to_account_info(),
            account.to_account_info(),
            system_program.to_account_info(),
        ],
        &[seeds],
    )?;

    anchor_lang::solana_program::program::invoke_signed(
        &anchor_spl::token::spl_token::instruction::initialize_mint2(
            &anchor_spl::token::spl_token::id(),
            &account.key(),
            &authority.key(),
            None,
            decimals,
        )?,
        &[account.to_account_info(), token_program.to_account_info()],
        &[seeds],
    )?;

    Ok(())
}
