use frame_support::sp_runtime::traits::AtLeast32Bit;
use sp_arithmetic::traits::{CheckedAdd, CheckedDiv, CheckedMul, IntegerSquareRoot, CheckedSub};
use crate::*;

pub fn get_lp_tokens_for_new_pool<T: AtLeast32Bit + IntegerSquareRoot + CheckedMul>(
	token_a_amount: T,
	token_b_amount: T,
) -> Option<T> {
	let k = token_a_amount.checked_mul(&token_b_amount);
	match k {
		Some(k) => k.integer_sqrt_checked(),
		None => None,
	}
}

pub fn get_lp_tokens_for_existing_pool<T: AtLeast32Bit + CheckedDiv + CheckedMul>(
	new_token_amount: T,
	current_token_amount: T,
	total_lp_token_supply: T,
) -> Option<T> {
	let lp_share = new_token_amount.checked_div(&current_token_amount);
	match lp_share {
		Some(lp_share) => lp_share.checked_mul(&total_lp_token_supply),
		None => None,
	}
}

pub fn get_token_b_amount<T: AtLeast32Bit + CheckedDiv + CheckedMul>(
	token_a_amount: T,
	liquidity_amounts: (T, T),
) -> Option<T> {
	let liquidity_ratio = liquidity_amounts.0.checked_div(&liquidity_amounts.1);
	match liquidity_ratio {
		Some(liquidity_ratio) => token_a_amount.checked_div(&liquidity_ratio),
		None => None,
	}
}

// TODO: extract to config variable
/// divide by 1_000 to get decimal percentage
/// E.g. 1 = 0.1% as 1 / 1_000 = 0.001 = 0.1%
const SWAP_FEE_PERCENTAGE: u32 = 1;
const SWAP_FEE_PERCENTAGE_DIVISOR: u32 = 1_000;

pub fn get_swap_return<T: AtLeast32Bit + CheckedDiv + CheckedMul + CheckedAdd + CheckedSub, Config>(
	token_a_amount: T,
	liquidity_amounts: (T, T),
) -> Result<T, Error<Config>> {
    let swap_fee_percentage: T = SWAP_FEE_PERCENTAGE.into();
    let swap_fee_percentage_divisor: T = SWAP_FEE_PERCENTAGE_DIVISOR.into();
    let hundred_percent: T = 1u32.into();

	let constant_product = liquidity_amounts.0.checked_mul(&liquidity_amounts.1).ok_or(Error::<Config>::MathOverflow)?;
	let new_token_a_liquidity = liquidity_amounts.0.checked_add(&token_a_amount).ok_or(Error::<Config>::MathOverflow)?;
    let new_token_b_liquidity = constant_product.checked_div(&new_token_a_liquidity).ok_or(Error::<Config>::MathOverflow)?;
    let total_b_decrease = liquidity_amounts.1.checked_sub(&new_token_b_liquidity).ok_or(Error::<Config>::MathOverflow)?;
    let fee_percentage = swap_fee_percentage.checked_div(&swap_fee_percentage_divisor).ok_or(Error::<Config>::MathOverflow)?;
    let percent_of_tokens_returned = hundred_percent.checked_sub(&fee_percentage).ok_or(Error::<Config>::MathOverflow)?;
    let returned_token_b_amount = total_b_decrease.checked_mul(&percent_of_tokens_returned).ok_or(Error::<Config>::MathOverflow)?;

    Ok(returned_token_b_amount)
}
