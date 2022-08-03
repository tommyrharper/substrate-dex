use crate::{dex_math::*, mock::*, test_utils::*, Error};
use frame_support::{assert_noop, assert_ok, traits::Currency};

const USER: AccountId = 1u32;
const USER_2: AccountId = 2u32;
const ASSET_A: u32 = 1u32;
const ASSET_B: u32 = 2u32;
const ASSET_A_AMOUNT: u128 = 1_000_000;
const ASSET_B_AMOUNT: u128 = 1_000_000;
const MINTED_AMOUNT: u128 = 1_000_000_000;

#[test]
fn can_transfer_assets() {
	new_test_ext().execute_with(|| {
		create_and_give_user_two_assets(USER, (ASSET_A, ASSET_B), MINTED_AMOUNT);
		Balances::make_free_balance_be(&USER_2, ExistentialDeposit::get());
		let origin = Origin::signed(USER);
		assert_ok!(Assets::transfer(origin, ASSET_A, USER_2, ASSET_A_AMOUNT));
	});
}

#[cfg(test)]
mod dex_math_tests {
	use super::*;

	#[test]
	fn test_get_lp_tokens_for_new_pool() {
		new_test_ext().execute_with(|| {
			let expected_return = get_lp_tokens_for_new_pool(50u32, 50u32).unwrap();
			assert_eq!(expected_return, 50);

			let expected_return = get_lp_tokens_for_new_pool(25u32, 4u32).unwrap();
			assert_eq!(expected_return, 10);
		});
	}

	#[test]
	fn test_get_lp_tokens_for_existing_pool() {
		new_test_ext().execute_with(|| {
			let expected_return = get_lp_tokens_for_existing_pool(50u32, 50u32, 50u32).unwrap();
			assert_eq!(expected_return, 50);

			let expected_return = get_lp_tokens_for_existing_pool(50u32, 100u32, 50u32).unwrap();
			assert_eq!(expected_return, 25);
		});
	}

	#[test]
	fn test_get_token_b_amount() {
		new_test_ext().execute_with(|| {
			let expected_return = get_token_b_amount(50u32, (100u32, 50u32)).unwrap();
			assert_eq!(expected_return, 25);

			let expected_return = get_token_b_amount(50u32, (50u32, 100u32)).unwrap();
			assert_eq!(expected_return, 100);
		});
	}

	#[test]
	fn test_get_swap_return() {
		new_test_ext().execute_with(|| {
			let expected_return = get_swap_return::<u128, Test>(50u128, (50u128, 100u128)).unwrap();
			assert_eq!(expected_return, 45);

			let expected_return = get_swap_return::<u128, Test>(50u128, (100u128, 50u128)).unwrap();
			assert_eq!(expected_return, 15);
		});
	}

	#[test]
	fn test_get_redeemed_token_balance() {
		new_test_ext().execute_with(|| {
			let expected_return = get_redeemed_token_balance(50u128, 100u128, (100, 50)).unwrap();
			assert_eq!(expected_return, (50, 25));

			let expected_return = get_redeemed_token_balance(50u128, 50u128, (100, 50)).unwrap();
			assert_eq!(expected_return, (100, 50));

			let expected_return = get_redeemed_token_balance(0u128, 50u128, (100, 50)).unwrap();
			assert_eq!(expected_return, (0, 0));
		});
	}
}

#[cfg(test)]
mod create_pool_tests {
	use super::*;

	#[test]
	fn create_pool_without_any_tokens() {
		new_test_ext().execute_with(|| {
			let origin = Origin::signed(USER);
			assert_noop!(
				DexModule::create_pool(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				Error::<Test>::NotEnoughTokensForTransaction
			);
		});
	}

	#[test]
	fn create_pool_without_first_token() {
		new_test_ext().execute_with(|| {
			create_and_give_user_asset(USER, ASSET_B, MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_noop!(
				DexModule::create_pool(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				Error::<Test>::NotEnoughTokensForTransaction
			);
		});
	}

	#[test]
	fn create_pool_without_second_token() {
		new_test_ext().execute_with(|| {
			create_and_give_user_asset(USER, ASSET_A, MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_noop!(
				DexModule::create_pool(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				Error::<Test>::NotEnoughTokensForTransaction
			);
		});
	}

	#[test]
	fn create_pool_same_asset_ids() {
		new_test_ext().execute_with(|| {
			create_and_give_user_two_assets(USER, (ASSET_A, ASSET_B), MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_noop!(
				DexModule::create_pool(origin, ASSET_A, ASSET_A, ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				Error::<Test>::ProvidedInvalidAssetIds
			);
		});
	}

	#[test]
	fn create_pool_with_enough_assets() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				MINTED_AMOUNT,
			);
		});
	}

	#[test]
	fn create_pool_transfers_tokens() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				MINTED_AMOUNT,
			);

			check_liquidity_taken(
				USER,
				(ASSET_A, ASSET_B),
				(MINTED_AMOUNT, MINTED_AMOUNT),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				(0, 0),
			);

			check_lp_tokens_sent_to_pool_creator(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
			);
		});
	}
}

#[cfg(test)]
mod provide_liquidity_tests {
	use super::*;

	#[test]
	fn provide_liquidity_without_any_token() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				MINTED_AMOUNT,
			);

			let origin = Origin::signed(USER_2);

			assert_noop!(
				DexModule::provide_liquidity(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT,),
				Error::<Test>::NotEnoughTokensForTransaction
			);
		});
	}

	#[test]
	fn provide_liquidity_without_first_token() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				MINTED_AMOUNT,
			);
			give_user_asset(USER_2, ASSET_B, MINTED_AMOUNT);

			let origin = Origin::signed(USER_2);

			assert_noop!(
				DexModule::provide_liquidity(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT,),
				Error::<Test>::NotEnoughTokensForTransaction
			);
		});
	}

	#[test]
	fn provide_liquidity_without_second_token() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				MINTED_AMOUNT,
			);
			give_user_asset(USER_2, ASSET_A, MINTED_AMOUNT);

			let origin = Origin::signed(USER_2);

			assert_noop!(
				DexModule::provide_liquidity(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT,),
				Error::<Test>::NotEnoughTokensForTransaction
			);
		});
	}

	#[test]
	fn provide_liquidity_same_asset_ids() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				MINTED_AMOUNT,
			);
			give_user_two_assets(USER_2, (ASSET_A, ASSET_B), MINTED_AMOUNT);

			let origin = Origin::signed(USER_2);

			assert_noop!(
				DexModule::provide_liquidity(origin, ASSET_A, ASSET_A, ASSET_A_AMOUNT,),
				Error::<Test>::ProvidedInvalidAssetIds
			);
		});
	}

	#[test]
	fn provide_liquidity() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				MINTED_AMOUNT,
			);
			give_user_two_assets(USER_2, (ASSET_A, ASSET_B), MINTED_AMOUNT);

			let origin = Origin::signed(USER_2);

			assert_ok!(DexModule::provide_liquidity(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT,),);

			check_liquidity_taken(
				USER_2,
				(ASSET_A, ASSET_B),
				(MINTED_AMOUNT, MINTED_AMOUNT),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
			);

			check_lp_tokens_sent_to_provider(
				USER_2,
				(ASSET_A, ASSET_B),
				ASSET_A_AMOUNT,
				ASSET_A_AMOUNT,
				ASSET_A_AMOUNT,
			);
		});
	}
}

#[cfg(test)]
mod swap_tests {
	use super::*;

	// #[test]
	// fn test_swap_without_tokens() {
	// 	new_test_ext().execute_with(|| {
	// 		create_liquidity_pool(
	// 			USER,
	// 			(ASSET_A, ASSET_B),
	// 			(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
	// 			MINTED_AMOUNT,
	// 		);

	// 		let origin = Origin::signed(USER_2);

	// 		assert_ok!(DexModule::swap(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT),);

	// 		assert_noop!(
    //             DexModule::swap(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT),
	// 			Error::<Test>::NotEnoughTokensForTransaction
	// 		);
	// 	});
    // }

	// #[test]
	// fn test_swap_invalid_assets() {
	// 	new_test_ext().execute_with(|| {
	// 		create_liquidity_pool(
	// 			USER,
	// 			(ASSET_A, ASSET_B),
	// 			(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
	// 			MINTED_AMOUNT,
	// 		);

	// 		let origin = Origin::signed(USER_2);

	// 		assert_noop!(
    //             DexModule::swap(origin, ASSET_A, ASSET_A, ASSET_A_AMOUNT),
	// 			Error::<Test>::ProvidedInvalidAssetIds
	// 		);
	// 	});
    // }

	#[test]
	fn test_swap() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				MINTED_AMOUNT,
			);
			give_user_asset(USER_2, ASSET_A, MINTED_AMOUNT);

			let origin = Origin::signed(USER_2);

			assert_ok!(DexModule::swap(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT),);

			check_user_swap_executed(
				USER_2,
				(ASSET_A, ASSET_B),
				ASSET_A_AMOUNT,
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				MINTED_AMOUNT,
			);
		});
	}
}

#[cfg(test)]
mod redeem_lp_tokens_tests {
	use super::*;

	#[test]
	fn test_redeem_lp_tokens_with_no_tokens() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				MINTED_AMOUNT,
			);

			let origin = Origin::signed(USER_2);

			assert_noop!(
				DexModule::redeem_lp_tokens(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT),
				Error::<Test>::NotEnoughLPTokens,
			);
		});
	}

	#[test]
	fn test_redeem_lp_tokens() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
				ASSET_A_AMOUNT,
			);
			check_users_balance(USER, ASSET_A, 0u32.into());

			let origin = Origin::signed(USER);

			assert_ok!(DexModule::redeem_lp_tokens(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT));

			check_lp_tokens_redeemed(USER, (ASSET_A, ASSET_B), ASSET_A_AMOUNT);
		});
	}
}
