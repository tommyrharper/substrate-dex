use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, traits::Currency};

use crate::test_utils::*;

#[cfg(test)]
mod tests {
	use super::*;

	const USER: AccountId = 1u32;
	const USER_2: AccountId = 2u32;
	const ASSET_A: u32 = 1u32;
	const ASSET_B: u32 = 2u32;
	const ASSET_A_AMOUNT: u128 = 1_000_000;
	const ASSET_B_AMOUNT: u128 = 1_000_000;
	const MINTED_AMOUNT: u128 = 1_000_000_000;

	#[test]
	fn create_pool_without_any_tokens() {
		new_test_ext().execute_with(|| {
			let origin = Origin::signed(USER);
			assert_noop!(
				TemplateModule::create_pool(
					origin,
					ASSET_A,
					ASSET_B,
					ASSET_A_AMOUNT,
					ASSET_B_AMOUNT
				),
				Error::<Test>::NotEnoughTokensToStake
			);
		});
	}

	#[test]
	fn create_pool_without_first_token() {
		new_test_ext().execute_with(|| {
			create_and_give_user_asset(USER, ASSET_B, MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_noop!(
				TemplateModule::create_pool(
					origin,
					ASSET_A,
					ASSET_B,
					ASSET_A_AMOUNT,
					ASSET_B_AMOUNT
				),
				Error::<Test>::NotEnoughTokensToStake
			);
		});
	}

	#[test]
	fn create_pool_without_second_token() {
		new_test_ext().execute_with(|| {
			create_and_give_user_asset(USER, ASSET_A, MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_noop!(
				TemplateModule::create_pool(
					origin,
					ASSET_A,
					ASSET_B,
					ASSET_A_AMOUNT,
					ASSET_B_AMOUNT
				),
				Error::<Test>::NotEnoughTokensToStake
			);
		});
	}

	#[test]
	fn create_pool_same_asset_ids() {
		new_test_ext().execute_with(|| {
			create_and_give_user_two_assets(USER, (ASSET_A, ASSET_B), MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_noop!(
				TemplateModule::create_pool(
					origin,
					ASSET_A,
					ASSET_A,
					ASSET_A_AMOUNT,
					ASSET_B_AMOUNT
				),
				Error::<Test>::ProvidedInvalidAssetIds
			);
		});
	}

	#[test]
	fn create_pool_with_enough_assets() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(USER, (ASSET_A, ASSET_B), (ASSET_A_AMOUNT, ASSET_B_AMOUNT));
		});
	}

	#[test]
	fn can_transfer_assets() {
		new_test_ext().execute_with(|| {
			create_and_give_user_two_assets(USER, (ASSET_A, ASSET_B), MINTED_AMOUNT);
			Balances::make_free_balance_be(&USER_2, ExistentialDeposit::get());
			let origin = Origin::signed(USER);
			assert_ok!(Assets::transfer(origin, ASSET_A, USER_2, ASSET_A_AMOUNT));
		});
	}

	#[test]
	fn create_pool_transfers_tokens() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(USER, (ASSET_A, ASSET_B), (ASSET_A_AMOUNT, ASSET_B_AMOUNT));

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

	#[test]
	fn provide_liquidity_without_any_token() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(USER, (ASSET_A, ASSET_B), (ASSET_A_AMOUNT, ASSET_B_AMOUNT));

			let origin = Origin::signed(USER_2);

			assert_noop!(
				TemplateModule::provide_liquidity(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT,),
				Error::<Test>::NotEnoughTokensToStake
			);
		});
	}

	#[test]
	fn provide_liquidity_without_first_token() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(USER, (ASSET_A, ASSET_B), (ASSET_A_AMOUNT, ASSET_B_AMOUNT));
			give_user_asset(USER_2, ASSET_B, MINTED_AMOUNT);

			let origin = Origin::signed(USER_2);

			assert_noop!(
				TemplateModule::provide_liquidity(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT,),
				Error::<Test>::NotEnoughTokensToStake
			);
		});
	}

	#[test]
	fn provide_liquidity_without_second_token() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(USER, (ASSET_A, ASSET_B), (ASSET_A_AMOUNT, ASSET_B_AMOUNT));
			give_user_asset(USER_2, ASSET_A, MINTED_AMOUNT);

			let origin = Origin::signed(USER_2);

			assert_noop!(
				TemplateModule::provide_liquidity(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT,),
				Error::<Test>::NotEnoughTokensToStake
			);
		});
	}

	#[test]
	fn provide_liquidity_same_asset_ids() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(USER, (ASSET_A, ASSET_B), (ASSET_A_AMOUNT, ASSET_B_AMOUNT));
			give_user_two_assets(USER_2, (ASSET_A, ASSET_B), MINTED_AMOUNT);

			let origin = Origin::signed(USER_2);

			assert_noop!(
				TemplateModule::provide_liquidity(origin, ASSET_A, ASSET_A, ASSET_A_AMOUNT,),
				Error::<Test>::ProvidedInvalidAssetIds
			);
		});
	}

	#[test]
	fn provide_liquidity() {
		new_test_ext().execute_with(|| {
			create_liquidity_pool(USER, (ASSET_A, ASSET_B), (ASSET_A_AMOUNT, ASSET_B_AMOUNT));
			give_user_two_assets(USER_2, (ASSET_A, ASSET_B), MINTED_AMOUNT);

			let origin = Origin::signed(USER_2);

			assert_ok!(
				TemplateModule::provide_liquidity(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT,),
			);

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
