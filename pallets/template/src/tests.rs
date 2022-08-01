use crate::{mock::*, Error};
use frame_support::{
	assert_noop, assert_ok,
	traits::{tokens::fungibles::Mutate, Currency},
};

#[cfg(test)]
mod tests {
	use crate::tests::*;

	const USER: AccountId = 1u32;
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

	fn give_user_two_assets(user: AccountId, asset1: u32, asset2: u32, amount: u128) {
		give_user_asset(user, asset1, amount);
		give_user_asset(user, asset2, amount);
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
				TemplateModule::create_pool(
					origin,
					ASSET1,
					ASSET2,
					ASSET1_AMOUNT,
					ASSET2_AMOUNT
				),
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
				TemplateModule::create_pool(
					origin,
					ASSET1,
					ASSET2,
					ASSET1_AMOUNT,
					ASSET2_AMOUNT
				),
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
				TemplateModule::create_pool(
					origin,
					ASSET1,
					ASSET2,
					ASSET1_AMOUNT,
					ASSET2_AMOUNT
				),
				Error::<Test>::NotEnoughTokensToStake
			);
		});
	}

	#[test]
	fn create_pool() {
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
	fn create_pool_same_asset_ids() {
		new_test_ext().execute_with(|| {
			give_user_two_assets(USER, ASSET1, ASSET2, MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_noop!(
				TemplateModule::create_pool(
					origin,
					ASSET1,
					ASSET1,
					ASSET1_AMOUNT,
					ASSET2_AMOUNT
				),
				Error::<Test>::ProvidedInvalidAssetIds
			);
		});
	}
}
