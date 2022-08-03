use crate::{dex_math::*, mock::*};
use frame_support::{
	assert_ok,
	traits::{tokens::fungibles::Mutate, Currency},
};

pub fn create_and_give_user_asset(user: AccountId, asset: u32, amount: u128) {
	let origin = Origin::signed(user);
	Balances::make_free_balance_be(&user, amount);
	Assets::create(origin, asset, user, 1).expect("Asset creation failed");
	Assets::mint_into(asset, &user, amount).expect("Minting failed");
}

pub fn give_user_asset(user: AccountId, asset: u32, amount: u128) {
	Balances::make_free_balance_be(&user, amount);
	Assets::mint_into(asset, &user, amount).expect("Minting failed");
}

pub fn create_and_give_user_two_assets(user: AccountId, asset_pair: (u32, u32), amount: u128) {
	create_and_give_user_asset(user, asset_pair.0, amount);
	create_and_give_user_asset(user, asset_pair.1, amount);
}

pub fn give_user_two_assets(user: AccountId, asset_pair: (u32, u32), amount: u128) {
	give_user_asset(user, asset_pair.0, amount);
	give_user_asset(user, asset_pair.1, amount);
}

pub fn check_users_balance(user: AccountId, asset: u32, amount: u128) {
	let asset_balance = Assets::balance(asset, &user);
	assert!(asset_balance == amount);
}

pub fn check_liquidity_taken(
	user: AccountId,
	asset_pair: (u32, u32),
	starting_balances: (u128, u128),
	asset_amounts: (u128, u128),
	starting_liquidity: (u128, u128),
) {
	check_users_balance(user, asset_pair.0, starting_balances.0 - asset_amounts.0);
	check_users_balance(user, asset_pair.1, starting_balances.1 - asset_amounts.1);
	let pool_id = DexModule::get_pool_id(asset_pair);
	check_users_balance(pool_id, asset_pair.0, starting_liquidity.0 + asset_amounts.0);
	check_users_balance(pool_id, asset_pair.1, starting_liquidity.0 + asset_amounts.1);
}

pub fn check_lp_tokens_sent_to_pool_creator(
	user: AccountId,
	asset_pair: (u32, u32),
	asset_amounts: (u128, u128),
) {
	let pool_id = DexModule::get_pool_id(asset_pair);
	let lp_token_id = DexModule::get_lp_token_id(&pool_id);
	let amount = get_lp_tokens_for_new_pool(asset_amounts.0, asset_amounts.1).unwrap();
	check_users_balance(user, lp_token_id, amount);
}

pub fn check_lp_tokens_sent_to_provider(
	user: AccountId,
	asset_pair: (u32, u32),
	new_token_amount: u128,
	current_token_amount: u128,
	total_lp_token_supply: u128,
) {
	let pool_id = DexModule::get_pool_id(asset_pair);
	let lp_token_id = DexModule::get_lp_token_id(&pool_id);
	let amount = get_lp_tokens_for_existing_pool(
		new_token_amount,
		current_token_amount,
		total_lp_token_supply,
	)
	.unwrap();
	check_users_balance(user, lp_token_id, amount);
}

pub fn create_liquidity_pool(user: AccountId, asset_pair: (u32, u32), asset_amounts: (u128, u128), user_initial_balance: u128) {
	// create_and_give_user_two_assets(user, asset_pair, asset_amounts.0);
	create_and_give_user_two_assets(user, asset_pair, user_initial_balance);

	let origin = Origin::signed(user);

	assert_ok!(DexModule::create_pool(
		origin,
		asset_pair.0,
		asset_pair.1,
		asset_amounts.0,
		asset_amounts.0,
	),);
}

pub fn check_user_swap_executed(
	user: AccountId,
	asset_pair: (u32, u32),
	asset_a_amount: u128,
	liquidity_amounts: (u128, u128),
	user_original_balance: u128,
) {
	let expected_return = get_swap_return::<u128, Test>(asset_a_amount, liquidity_amounts).unwrap();

	check_users_balance(user, asset_pair.0, user_original_balance - asset_a_amount);
	check_users_balance(user, asset_pair.1, expected_return);
	let pool_id = DexModule::get_pool_id(asset_pair);
	check_users_balance(pool_id, asset_pair.0, liquidity_amounts.0 + asset_a_amount);
	check_users_balance(pool_id, asset_pair.1, liquidity_amounts.1 - expected_return);
}

pub fn check_lp_tokens_redeemed(
	user: AccountId,
	asset_pair: (u32, u32),
	lp_tokens_amount: u128,
) {
    check_users_balance(user, asset_pair.0, lp_tokens_amount);
    check_users_balance(user, asset_pair.1, lp_tokens_amount);

    let pool_id = DexModule::get_pool_id(asset_pair);
    let lp_token_id = DexModule::get_lp_token_id(&pool_id);

    check_users_balance(user, lp_token_id, 0);
    check_users_balance(pool_id, asset_pair.0, 0);
    check_users_balance(pool_id, asset_pair.1, 0);
    check_users_balance(pool_id, lp_token_id, 0);
}
