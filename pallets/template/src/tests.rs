use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, traits::Currency};
use frame_support::traits::tokens::fungibles::Mutate;
use frame_system::{Pallet as System, RawOrigin};

const USER: AccountId = 1u32;
const Asset1: u32 = 1u32;

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
fn provide_liquidity_without_tokens() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			TemplateModule::provide_liquidity(Origin::signed(1), 1, 2, 1, 1),
			Error::<Test>::NotEnoughTokensToStake
		);
	});
}

#[test]
fn provide_liquidity() {
	new_test_ext().execute_with(|| {
		let signed = Origin::signed(USER);
		Balances::make_free_balance_be(&USER, 1_000_000_000);
		Assets::create(signed, Asset1, USER, 1).expect("Asset creation failed");
		Assets::mint_into(1, &USER, 1_000_000_000).expect("Minting failed");

		let signed = Origin::signed(USER);
		assert_ok!(TemplateModule::provide_liquidity(signed, 1, 2, 1, 1));
	});
}
