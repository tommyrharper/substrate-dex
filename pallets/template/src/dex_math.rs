use frame_support::sp_runtime::traits::AtLeast32Bit;
use sp_arithmetic::traits::{IntegerSquareRoot, CheckedDiv, CheckedAdd, CheckedMul};

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
	liquidity_a: T,
	liquidity_b: T,
) -> Option<T> {
    let liquidity_ratio = liquidity_a.checked_div(&liquidity_b);
    match liquidity_ratio {
        Some(liquidity_ratio) => token_a_amount.checked_div(&liquidity_ratio),
        None => None,
    }
}
