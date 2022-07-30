use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};
// use pallet_assets::pallet::Pallet as AssetsPallet;
use frame_support::traits::tokens::fungibles::Mutate;

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
		assert_noop!(TemplateModule::provide_liquidity(Origin::signed(1), 1, 2, 1, 1), Error::<Test>::NotEnoughTokensToStake);
    });
}

#[test]
fn provide_liquidity() {
    new_test_ext().execute_with(|| {
        <Assets as Mutate<AccountId>>::mint_into(1, &1, 1000_000);
        assert_ok!(TemplateModule::provide_liquidity(Origin::signed(1), 1, 2, 1, 1));
    });
}
