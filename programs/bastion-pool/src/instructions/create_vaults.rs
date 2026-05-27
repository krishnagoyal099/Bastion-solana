use crate::state::BastionConfig;
use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

#[derive(Accounts)]
pub struct CreateVaults<'info> {
    #[account(
        mut,
        seeds = [b"config"],
        bump = config.bump,
        has_one = authority
    )]
    pub config: Account<'info, BastionConfig>,

    #[account(mut)]
    pub authority: Signer<'info>,

    /// WSOL vault - PDA token account for SOL deposits
    /// CHECK: Created via CPI in the handler
    #[account(
        mut,
        seeds = [b"wsol_vault", config.key().as_ref()],
        bump
    )]
    pub wsol_vault: UncheckedAccount<'info>,

    /// Token vault - PDA token account for USDC deposits
    /// CHECK: Created via CPI in the handler
    #[account(
        mut,
        seeds = [b"token_vault", config.key().as_ref()],
        bump
    )]
    pub token_vault: UncheckedAccount<'info>,

    /// WSOL mint (native mint)
    pub wsol_mint: Account<'info, Mint>,

    /// Token mint (e.g., USDC)
    pub token_mint: Account<'info, Mint>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handle_create_vaults(ctx: Context<CreateVaults>) -> Result<()> {
    let config_key = ctx.accounts.config.key();

    // Create WSOL vault
    let (_, wsol_bump) = Pubkey::find_program_address(
        &[b"wsol_vault", config_key.as_ref()],
        ctx.program_id,
    );
    let wsol_seeds: &[&[u8]] = &[b"wsol_vault", config_key.as_ref(), &[wsol_bump]];

    create_pda_token_account(
        &ctx.accounts.authority,
        &ctx.accounts.wsol_vault,
        &ctx.accounts.wsol_mint.to_account_info(),
        &ctx.accounts.config.to_account_info(),
        &ctx.accounts.system_program,
        &ctx.accounts.token_program,
        wsol_seeds,
    )?;

    // Create token vault
    let (_, token_bump) = Pubkey::find_program_address(
        &[b"token_vault", config_key.as_ref()],
        ctx.program_id,
    );
    let token_seeds: &[&[u8]] = &[b"token_vault", config_key.as_ref(), &[token_bump]];

    create_pda_token_account(
        &ctx.accounts.authority,
        &ctx.accounts.token_vault,
        &ctx.accounts.token_mint.to_account_info(),
        &ctx.accounts.config.to_account_info(),
        &ctx.accounts.system_program,
        &ctx.accounts.token_program,
        token_seeds,
    )?;

    msg!("Vaults created: WSOL + token");
    Ok(())
}

fn create_pda_token_account<'info>(
    payer: &Signer<'info>,
    account: &UncheckedAccount<'info>,
    mint: &AccountInfo<'info>,
    authority: &AccountInfo<'info>,
    system_program: &Program<'info, System>,
    token_program: &Program<'info, Token>,
    seeds: &[&[u8]],
) -> Result<()> {
    let rent = Rent::get()?;
    let space = TokenAccount::LEN;
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
