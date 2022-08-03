use crate::*;
use frame_support::sp_runtime::traits::AtLeast32Bit;
use sp_arithmetic::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub, IntegerSquareRoot};

const MULTIPLIER: u32 = 1_000;

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
	let current_token_amount: T =
		if current_token_amount == 0u32.into() { 1u32.into() } else { current_token_amount };
	let total_lp_token_supply: T =
		if total_lp_token_supply == 0u32.into() { 1u32.into() } else { total_lp_token_supply };

	let multiplier: T = MULTIPLIER.into();
	let large_new_token_amount = new_token_amount.checked_mul(&multiplier);

	let lp_share = large_new_token_amount?.checked_div(&current_token_amount);
	match lp_share {
		Some(lp_share) => lp_share.checked_mul(&total_lp_token_supply)?.checked_div(&multiplier),
		None => None,
	}
}

pub fn get_token_b_amount<T: AtLeast32Bit + CheckedDiv + CheckedMul>(
	token_a_amount: T,
	liquidity_amounts: (T, T),
) -> Option<T> {
	let multiplier: T = MULTIPLIER.into();

	let liquidity_a_amount: T =
		if liquidity_amounts.0 == 0u32.into() { 1u32.into() } else { liquidity_amounts.0 };
	let liquidity_b_amount: T =
		if liquidity_amounts.1 == 0u32.into() { 1u32.into() } else { liquidity_amounts.1 };

	let liquidity_ratio =
		liquidity_a_amount.checked_mul(&multiplier)?.checked_div(&liquidity_b_amount);

	match liquidity_ratio {
		Some(liquidity_ratio) =>
			token_a_amount.checked_mul(&multiplier)?.checked_div(&liquidity_ratio),
		None => None,
	}
}

// TODO: extract to config variable
/// divide by 1_000 to get decimal percentage
/// E.g. 1 = 0.1% as 1 / 1_000 = 0.001 = 0.1%
/// E.g. 10 = 1% as 10 / 1_000 = 0.01 = 1%
/// E.g. 100 = 10% as 100 / 1_000 = 0.1 = 10%
const SWAP_FEE_PERCENTAGE: u32 = 100;
const SWAP_FEE_PERCENTAGE_DIVISOR: u32 = MULTIPLIER;

pub fn get_swap_return<
	T: AtLeast32Bit + CheckedDiv + CheckedMul + CheckedAdd + CheckedSub,
	Config,
>(
	token_a_amount: T,
	liquidity_amounts: (T, T),
) -> Result<T, Error<Config>> {
	let swap_fee_percentage: T = SWAP_FEE_PERCENTAGE.into();
	let swap_fee_percentage_divisor: T = SWAP_FEE_PERCENTAGE_DIVISOR.into();

	let liquidity_a_amount: T =
		if liquidity_amounts.0 == 0u32.into() { 1u32.into() } else { liquidity_amounts.0 };
	let liquidity_b_amount: T =
		if liquidity_amounts.1 == 0u32.into() { 1u32.into() } else { liquidity_amounts.1 };

	let returned_fee_percentage_multiplier: T = swap_fee_percentage_divisor
		.checked_sub(&swap_fee_percentage)
		.ok_or(Error::<Config>::MathOverflow)?;

	let constant_product = liquidity_a_amount
		.checked_mul(&liquidity_b_amount)
		.ok_or(Error::<Config>::MathOverflow)?;

	let new_token_a_liquidity = liquidity_a_amount
		.checked_add(&token_a_amount)
		.ok_or(Error::<Config>::MathOverflow)?;

	let new_token_b_liquidity = constant_product
		.checked_div(&new_token_a_liquidity)
		.ok_or(Error::<Config>::MathOverflow)?;

	let total_b_decrease = liquidity_b_amount
		.checked_sub(&new_token_b_liquidity)
		.ok_or(Error::<Config>::MathOverflow)?;

	let returned_large_amount = total_b_decrease
		.checked_mul(&returned_fee_percentage_multiplier)
		.ok_or(Error::<Config>::MathOverflow)?;

	let returned_token_b_amount_minus_fee = returned_large_amount
		.checked_div(&swap_fee_percentage_divisor)
		.ok_or(Error::<Config>::MathOverflow)?;

	Ok(returned_token_b_amount_minus_fee)
}

pub fn get_redeemed_token_balance<T: AtLeast32Bit + CheckedDiv + CheckedMul>(
	lp_tokens: T,
	total_lp_token_supply: T,
	liquidity_amounts: (T, T),
) -> Option<(T, T)> {
	let multiplier: T = MULTIPLIER.into();

	let total_lp_token_supply: T =
		if total_lp_token_supply == 0u32.into() { 1u32.into() } else { total_lp_token_supply };
	let liquidity_a_amount: T =
		if liquidity_amounts.0 == 0u32.into() { 1u32.into() } else { liquidity_amounts.0 };
	let liquidity_b_amount: T =
		if liquidity_amounts.1 == 0u32.into() { 1u32.into() } else { liquidity_amounts.1 };

	let large_new_lp_token_amount = lp_tokens.checked_mul(&multiplier);

	let lp_share = large_new_lp_token_amount?.checked_div(&total_lp_token_supply);

	match lp_share {
		Some(lp_share) => Some((
			lp_share.checked_mul(&liquidity_a_amount)?.checked_div(&multiplier)?,
			lp_share.checked_mul(&liquidity_b_amount)?.checked_div(&multiplier)?,
		)),
		None => None,
	}
}
