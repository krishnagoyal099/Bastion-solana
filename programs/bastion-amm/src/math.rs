use crate::errors::AmmError;
use anchor_lang::prelude::*;

/// Integer square root using Newton's method (F9 FIX)
/// Replaces f64 sqrt which loses precision for large u128 values
pub fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

pub fn calculate_lp_shares(
    amount_a: u64,
    amount_b: u64,
    reserve_a: u64,
    reserve_b: u64,
    total_supply: u64,
) -> Result<u64> {
    if total_supply == 0 {
        let amount_a_u128 = amount_a as u128;
        let amount_b_u128 = amount_b as u128;
        let product = amount_a_u128
            .checked_mul(amount_b_u128)
            .ok_or(error!(AmmError::MathOverflow))?;

        // F9 FIX: Use integer sqrt instead of f64 cast
        let shares = isqrt(product);
        let shares_u64 = u64::try_from(shares).map_err(|_| error!(AmmError::MathOverflow))?;

        require!(shares_u64 > 1000, AmmError::InsufficientLiquidity);
        Ok(shares_u64 - 1000) // Lock minimum liquidity to prevent manipulation
    } else {
        let amount_a_u128 = amount_a as u128;
        let amount_b_u128 = amount_b as u128;
        let total_supply_u128 = total_supply as u128;
        let reserve_a_u128 = reserve_a as u128;
        let reserve_b_u128 = reserve_b as u128;

        require!(reserve_a_u128 > 0, AmmError::InsufficientLiquidity);
        require!(reserve_b_u128 > 0, AmmError::InsufficientLiquidity);

        let share_a = amount_a_u128
            .checked_mul(total_supply_u128)
            .ok_or(error!(AmmError::MathOverflow))?
            .checked_div(reserve_a_u128)
            .ok_or(error!(AmmError::MathOverflow))?;

        let share_b = amount_b_u128
            .checked_mul(total_supply_u128)
            .ok_or(error!(AmmError::MathOverflow))?
            .checked_div(reserve_b_u128)
            .ok_or(error!(AmmError::MathOverflow))?;

        let shares = std::cmp::min(share_a, share_b);
        let shares_u64 = u64::try_from(shares).map_err(|_| error!(AmmError::MathOverflow))?;
        Ok(shares_u64)
    }
}

/// F10 FIX: Use fee_bps parameter instead of hardcoded 997/1000
pub fn calculate_swap_output(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_bps: u16,
) -> Result<u64> {
    require!(amount_in > 0, AmmError::InvalidAmount);
    require!(
        reserve_in > 0 && reserve_out > 0,
        AmmError::InsufficientLiquidity
    );
    // Cap fee_bps to prevent absurd values
    require!(fee_bps <= 10000, AmmError::InvalidAmount);

    let amount_in_u128 = amount_in as u128;
    let reserve_in_u128 = reserve_in as u128;
    let reserve_out_u128 = reserve_out as u128;

    // fee_factor = 10000 - fee_bps (e.g., 30 bps → 9970)
    let fee_factor = 10000u128
        .checked_sub(fee_bps as u128)
        .ok_or(error!(AmmError::MathOverflow))?;

    let amount_in_with_fee = amount_in_u128
        .checked_mul(fee_factor)
        .ok_or(error!(AmmError::MathOverflow))?;

    let numerator = amount_in_with_fee
        .checked_mul(reserve_out_u128)
        .ok_or(error!(AmmError::MathOverflow))?;
    let denominator = reserve_in_u128
        .checked_mul(10000)
        .ok_or(error!(AmmError::MathOverflow))?
        .checked_add(amount_in_with_fee)
        .ok_or(error!(AmmError::MathOverflow))?;

    let amount_out = numerator
        .checked_div(denominator)
        .ok_or(error!(AmmError::MathOverflow))?;
    let amount_out_u64 = u64::try_from(amount_out).map_err(|_| error!(AmmError::MathOverflow))?;
    Ok(amount_out_u64)
}

pub fn calculate_remove_liquidity(
    shares: u64,
    reserve_a: u64,
    reserve_b: u64,
    total_supply: u64,
) -> Result<(u64, u64)> {
    require!(shares > 0, AmmError::InvalidAmount);
    require!(total_supply > 0, AmmError::InsufficientLiquidity);
    require!(shares <= total_supply, AmmError::InsufficientShares);

    let shares_u128 = shares as u128;
    let reserve_a_u128 = reserve_a as u128;
    let reserve_b_u128 = reserve_b as u128;
    let total_supply_u128 = total_supply as u128;

    let amount_a = shares_u128
        .checked_mul(reserve_a_u128)
        .ok_or(error!(AmmError::MathOverflow))?
        .checked_div(total_supply_u128)
        .ok_or(error!(AmmError::MathOverflow))?;
    let amount_b = shares_u128
        .checked_mul(reserve_b_u128)
        .ok_or(error!(AmmError::MathOverflow))?
        .checked_div(total_supply_u128)
        .ok_or(error!(AmmError::MathOverflow))?;

    let a_u64 = u64::try_from(amount_a).map_err(|_| error!(AmmError::MathOverflow))?;
    let b_u64 = u64::try_from(amount_b).map_err(|_| error!(AmmError::MathOverflow))?;

    Ok((a_u64, b_u64))
}
