use crate::{mock::*, Error};
use frame_support::{
	assert_noop, assert_ok,
	traits::{tokens::fungibles::Mutate, Currency},
};

const USER: AccountId = 1u32;
const Asset1: u32 = 1u32;
const Asset2: u32 = 2u32;
const Asset1Amount: u128 = 1;
const Asset2Amount: u128 = 1;
const MintedAmount: u128 = 1_000_000_000;

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
fn provide_liquidity_without_any_tokens() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(USER);
		assert_noop!(
			TemplateModule::provide_liquidity(origin, Asset1, Asset2, Asset1Amount, Asset2Amount),
			Error::<Test>::NotEnoughTokensToStake
		);
	});
}

#[test]
fn provide_liquidity_without_second_token() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(USER);
		Balances::make_free_balance_be(&USER, MintedAmount);
		Assets::create(origin, Asset1, USER, 1).expect("Asset creation failed");
		Assets::mint_into(Asset1, &USER, MintedAmount).expect("Minting failed");

		let origin = Origin::signed(USER);
		assert_noop!(
			TemplateModule::provide_liquidity(origin, Asset1, Asset2, Asset1Amount, Asset2Amount),
			Error::<Test>::NotEnoughTokensToStake
		);
	});
}

#[test]
fn provide_liquidity_without_first_token() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(USER);
		Balances::make_free_balance_be(&USER, MintedAmount);
		Assets::create(origin, Asset2, USER, 1).expect("Asset creation failed");
		Assets::mint_into(Asset2, &USER, MintedAmount).expect("Minting failed");

		let origin = Origin::signed(USER);
		assert_noop!(
			TemplateModule::provide_liquidity(origin, Asset1, Asset2, Asset1Amount, Asset2Amount),
			Error::<Test>::NotEnoughTokensToStake
		);
	});
}

#[test]
fn provide_liquidity() {
	new_test_ext().execute_with(|| {
		let origin = Origin::signed(USER);
		Balances::make_free_balance_be(&USER, MintedAmount);
		Assets::create(origin, Asset1, USER, 1).expect("Asset creation failed");
		Assets::mint_into(Asset1, &USER, MintedAmount).expect("Minting failed");

		let origin = Origin::signed(USER);
		Assets::create(origin, Asset2, USER, 1).expect("Asset creation failed");
		Assets::mint_into(Asset2, &USER, MintedAmount).expect("Minting failed");

		let origin = Origin::signed(USER);
		assert_ok!(TemplateModule::provide_liquidity(
			origin,
			Asset1,
			Asset2,
			Asset1Amount,
			Asset2Amount
		),);
	});
}
