use crate::{mock::*, Error};
use frame_support::{
	assert_noop, assert_ok,
	traits::{tokens::fungibles::Mutate, Currency},
};

#[cfg(test)]
mod tests {
	use crate::tests::*;

	const USER: AccountId = 1u32;
	const USER_2: AccountId = 2u32;
	const ASSET1: u32 = 1u32;
	const ASSET2: u32 = 2u32;
	const ASSET1_AMOUNT: u128 = 1_000_000;
	const ASSET2_AMOUNT: u128 = 1_000_000;
	const MINTED_AMOUNT: u128 = 1_000_000_000;

	fn give_user_asset(user: AccountId, asset: u32, amount: u128) {
		let origin = Origin::signed(user);
		Balances::make_free_balance_be(&user, amount);
		Assets::create(origin, asset, user, 1).expect("Asset creation failed");
		Assets::mint_into(asset, &user, amount).expect("Minting failed");
	}

	// TODO: update to use tuple
	fn give_user_two_assets(user: AccountId, asset1: u32, asset2: u32, amount: u128) {
		give_user_asset(user, asset1, amount);
		give_user_asset(user, asset2, amount);
	}

	fn check_users_balance(user: AccountId, asset: u32, amount: u128) {
		let asset_balance = Assets::balance(asset, &user);
		assert!(asset_balance == amount);
	}

	fn check_liquidity_taken(
		user: AccountId,
		asset_pair: (u32, u32),
		starting_balances: (u128, u128),
		asset_amounts: (u128, u128),
	) {
		check_users_balance(user, asset_pair.0, starting_balances.0 - asset_amounts.0);
		check_users_balance(user, asset_pair.1, starting_balances.1 - asset_amounts.1);
		let pool_id = TemplateModule::get_pool_id(asset_pair);
		check_users_balance(pool_id, asset_pair.0, asset_amounts.0);
		check_users_balance(pool_id, asset_pair.1, asset_amounts.1);
	}

	fn check_lp_tokens_sent_to_provider(
		user: AccountId,
		asset_pair: (u32, u32),
		amount: u128,
	) {
        let pool_id = TemplateModule::get_pool_id(asset_pair);
        let lp_token_id = TemplateModule::get_lp_token_id(&pool_id);
        check_users_balance(user, lp_token_id, amount);
	}

	#[test]
	fn it_works_for_default_value() {
		new_test_ext().execute_with(|| {
			// Dispatch a signed extrinsic.
			assert_ok!(TemplateModule::do_something(Origin::signed(1), 42));
			// Read pallet storage and assert an expected result.
			assert_eq!(TemplateModule::something(), Some(42));
		});
	}

	#[test]
	fn correct_error_for_none_value() {
		new_test_ext().execute_with(|| {
			// Ensure the expected error is thrown when no value is present.
			assert_noop!(TemplateModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
		});
	}

	#[test]
	fn create_pool_without_any_tokens() {
		new_test_ext().execute_with(|| {
			let origin = Origin::signed(USER);
			assert_noop!(
				TemplateModule::create_pool(origin, ASSET1, ASSET2, ASSET1_AMOUNT, ASSET2_AMOUNT),
				Error::<Test>::NotEnoughTokensToStake
			);
		});
	}

	#[test]
	fn create_pool_without_first_token() {
		new_test_ext().execute_with(|| {
			give_user_asset(USER, ASSET2, MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_noop!(
				TemplateModule::create_pool(origin, ASSET1, ASSET2, ASSET1_AMOUNT, ASSET2_AMOUNT),
				Error::<Test>::NotEnoughTokensToStake
			);
		});
	}

	#[test]
	fn create_pool_without_second_token() {
		new_test_ext().execute_with(|| {
			give_user_asset(USER, ASSET1, MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_noop!(
				TemplateModule::create_pool(origin, ASSET1, ASSET2, ASSET1_AMOUNT, ASSET2_AMOUNT),
				Error::<Test>::NotEnoughTokensToStake
			);
		});
	}

	#[test]
	fn create_pool_same_asset_ids() {
		new_test_ext().execute_with(|| {
			give_user_two_assets(USER, ASSET1, ASSET2, MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_noop!(
				TemplateModule::create_pool(origin, ASSET1, ASSET1, ASSET1_AMOUNT, ASSET2_AMOUNT),
				Error::<Test>::ProvidedInvalidAssetIds
			);
		});
	}

	#[test]
	fn create_pool_with_enough_assets() {
		new_test_ext().execute_with(|| {
			give_user_two_assets(USER, ASSET1, ASSET2, MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_ok!(TemplateModule::create_pool(
				origin,
				ASSET1,
				ASSET2,
				ASSET1_AMOUNT,
				ASSET2_AMOUNT,
			),);
		});
	}

	#[test]
	fn can_transfer_assets() {
		new_test_ext().execute_with(|| {
			give_user_two_assets(USER, ASSET1, ASSET2, MINTED_AMOUNT);
			Balances::make_free_balance_be(&USER_2, ExistentialDeposit::get());
			let origin = Origin::signed(USER);
			assert_ok!(Assets::transfer(origin, ASSET1, USER_2, ASSET1_AMOUNT));
		});
	}

	#[test]
	fn create_pool_transfers_tokens() {
		new_test_ext().execute_with(|| {
			give_user_two_assets(USER, ASSET1, ASSET2, MINTED_AMOUNT);

			let origin = Origin::signed(USER);

			assert_ok!(TemplateModule::create_pool(
				origin,
				ASSET1,
				ASSET2,
				ASSET1_AMOUNT,
				ASSET2_AMOUNT,
			),);

			check_liquidity_taken(
				USER,
				(ASSET1, ASSET2),
				(MINTED_AMOUNT, MINTED_AMOUNT),
				(ASSET1_AMOUNT, ASSET2_AMOUNT),
			);

            check_lp_tokens_sent_to_provider(USER, (ASSET1, ASSET2), ASSET1_AMOUNT);
		});
	}
}
