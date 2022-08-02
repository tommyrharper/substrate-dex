use crate::{mock::*, Error};
use frame_support::{
	assert_noop, assert_ok,
	traits::{tokens::fungibles::Mutate, Currency},
};

#[cfg(test)]
mod tests {
	use crate::{dex_math::*, tests::*};

	const USER: AccountId = 1u32;
	const USER_2: AccountId = 2u32;
	const ASSET_A: u32 = 1u32;
	const ASSET_B: u32 = 2u32;
	const ASSET_A_AMOUNT: u128 = 1_000_000;
	const ASSET_B_AMOUNT: u128 = 1_000_000;
	const MINTED_AMOUNT: u128 = 1_000_000_000;

	fn give_user_asset(user: AccountId, asset: u32, amount: u128) {
		let origin = Origin::signed(user);
		Balances::make_free_balance_be(&user, amount);
		Assets::create(origin, asset, user, 1).expect("Asset creation failed");
		Assets::mint_into(asset, &user, amount).expect("Minting failed");
	}

	fn give_user_two_assets(user: AccountId, asset_pair: (u32, u32), amount: u128) {
		give_user_asset(user, asset_pair.0, amount);
		give_user_asset(user, asset_pair.1, amount);
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

	fn check_lp_tokens_sent_to_pool_creator(
		user: AccountId,
		asset_pair: (u32, u32),
		asset_amounts: (u128, u128),
	) {
		let pool_id = TemplateModule::get_pool_id(asset_pair);
		let lp_token_id = TemplateModule::get_lp_token_id(&pool_id);
		let amount = get_lp_tokens_for_new_pool(asset_amounts.0, asset_amounts.1).unwrap();
		check_users_balance(user, lp_token_id, amount);
	}

	fn check_lp_tokens_sent_to_provider(
		user: AccountId,
		asset_pair: (u32, u32),
		new_token_amount: u128,
		current_token_amount: u128,
		total_lp_token_supply: u128,
	) {
		let pool_id = TemplateModule::get_pool_id(asset_pair);
		let lp_token_id = TemplateModule::get_lp_token_id(&pool_id);
		let amount = get_lp_tokens_for_existing_pool(
			new_token_amount,
			current_token_amount,
			total_lp_token_supply,
		)
		.unwrap();
		check_users_balance(user, lp_token_id, amount);
	}

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
			give_user_asset(USER, ASSET_B, MINTED_AMOUNT);

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
			give_user_asset(USER, ASSET_A, MINTED_AMOUNT);

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
			give_user_two_assets(USER, (ASSET_A, ASSET_B), MINTED_AMOUNT);

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
			give_user_two_assets(USER, (ASSET_A, ASSET_B), MINTED_AMOUNT);

			let origin = Origin::signed(USER);
			assert_ok!(TemplateModule::create_pool(
				origin,
				ASSET_A,
				ASSET_B,
				ASSET_A_AMOUNT,
				ASSET_B_AMOUNT,
			),);
		});
	}

	#[test]
	fn can_transfer_assets() {
		new_test_ext().execute_with(|| {
			give_user_two_assets(USER, (ASSET_A, ASSET_B), MINTED_AMOUNT);
			Balances::make_free_balance_be(&USER_2, ExistentialDeposit::get());
			let origin = Origin::signed(USER);
			assert_ok!(Assets::transfer(origin, ASSET_A, USER_2, ASSET_A_AMOUNT));
		});
	}

	#[test]
	fn create_pool_transfers_tokens() {
		new_test_ext().execute_with(|| {
			give_user_two_assets(USER, (ASSET_A, ASSET_B), MINTED_AMOUNT);

			let origin = Origin::signed(USER);

			assert_ok!(TemplateModule::create_pool(
				origin,
				ASSET_A,
				ASSET_B,
				ASSET_A_AMOUNT,
				ASSET_B_AMOUNT,
			),);

			check_liquidity_taken(
				USER,
				(ASSET_A, ASSET_B),
				(MINTED_AMOUNT, MINTED_AMOUNT),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
			);

			check_lp_tokens_sent_to_pool_creator(
				USER,
				(ASSET_A, ASSET_B),
				(ASSET_A_AMOUNT, ASSET_B_AMOUNT),
			);
		});
	}

	#[test]
	fn provide_liquidity() {
		new_test_ext().execute_with(|| {
			give_user_two_assets(USER, (ASSET_A, ASSET_B), MINTED_AMOUNT);

			let origin = Origin::signed(USER);

			assert_ok!(TemplateModule::create_pool(
				origin,
				ASSET_A,
				ASSET_B,
				ASSET_A_AMOUNT,
				ASSET_B_AMOUNT,
			),);

			let origin = Origin::signed(USER_2);

			assert_ok!(
				TemplateModule::provide_liquidity(origin, ASSET_A, ASSET_B, ASSET_A_AMOUNT,),
			);

			check_liquidity_taken(
				USER_2,
				(ASSET_A, ASSET_B),
				(MINTED_AMOUNT, MINTED_AMOUNT),
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
