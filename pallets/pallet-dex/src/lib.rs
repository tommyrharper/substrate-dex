#![cfg_attr(not(feature = "std"), no_std)]

use crate::dex_math::*;
use frame_support::{
	dispatch::{Codec, Decode},
	pallet_prelude::*,
	sp_runtime::traits::{AccountIdConversion, AtLeast32Bit},
	traits::tokens::{
		currency::Currency,
		fungibles::{Create, Inspect, Mutate, Transfer},
	},
	Hashable,
	PalletId,
};
use frame_system::pallet_prelude::*;
pub use pallet::*;
use scale_info::prelude::vec;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod dex_math;
mod impl_dex;
mod impl_create_pool;
mod impl_provide_liquidity;
mod impl_lp_redemption;
mod impl_swap;

type AssetIdOf<T: Config> = <T::Assets as Inspect<T::AccountId>>::AssetId;
type BalanceOf<T: Config> = <T::Assets as Inspect<T::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        /// Asset type for this pallet
		type Assets: Inspect<Self::AccountId>
			+ Transfer<Self::AccountId>
			+ Mutate<Self::AccountId>
			+ Create<Self::AccountId>;

        /// Balances is the Currency type for this pallet
		type Balances: Currency<Self::AccountId>;

        /// PalletId for this pallet - used to manage the liquidity pools
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
        /// This is triggered when a new liquidity pool is successfully created
		/// parameters: [the id of the new pool]
        NewPoolCreated(T::AccountId),
        /// This is triggered when a new liquidity pool is successfully created
		/// parameters: [the id of the new pool, the id of the liquidity tokens, the number of liquidity tokens earned]
        LiquidityProvided(T::AccountId, AssetIdOf<T>, BalanceOf<T>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The user tried to stake more tokens than they have
		NotEnoughTokensToStake,
		/// The user did not provide valid asset ids
		ProvidedInvalidAssetIds,
		/// The dex math has had an overflow
		MathOverflow,
		/// The user does not have enough LP tokens for the redemption request
		NotEnoughLPTokens,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T::Assets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
		<T::Assets as Inspect<T::AccountId>>::AssetId: Codec,
	{
		// TODO: see if tuples are a practical input here
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn create_pool(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			asset_a_amount: BalanceOf<T>,
			asset_b_amount: BalanceOf<T>,
		) -> DispatchResult {
			// check if message is signed
			let sender = ensure_signed(origin)?;

			// Check the user is able to make the required deposit
			Self::check_deposit_is_valid(
				&sender,
				(asset_a, asset_b),
				(asset_a_amount, asset_b_amount),
			)?;

			// Create the new liquidity pool
			Self::create_new_pool(&sender, (asset_a, asset_b), (asset_a_amount, asset_b_amount))?;

			Ok(())
		}

		#[pallet::weight(5_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn provide_liquidity(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			asset_a_amount: BalanceOf<T>,
		) -> DispatchResult {
			// check if message is signed
			let sender = ensure_signed(origin)?;

			// Get pool data
			let pool_liquidity = Self::get_pool_liquidity((asset_a, asset_b))?;
			let asset_b_amount = Self::derive_second_asset_amount(pool_liquidity, asset_a_amount)?;

			// Check the user is able to make the required deposit
			Self::check_deposit_is_valid(
				&sender,
				(asset_a, asset_b),
				(asset_a_amount, asset_b_amount),
			)?;

			// Handle the deposit to the liquidity pool
			Self::process_liquidity_pool_deposit(
				&sender,
				(asset_a, asset_b),
				(asset_a_amount, asset_b_amount),
				pool_liquidity.0,
			)?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn swap(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			asset_a_amount: BalanceOf<T>,
		) -> DispatchResult {
			// check if message is signed
			let sender = ensure_signed(origin)?;

			// Check the user is able to make the swap
			Self::check_deposit_is_valid(
				&sender,
				(asset_a, asset_b),
				(asset_a_amount, 0u32.into()),
			)?;

			// Handle the swap
			Self::process_swap(&sender, (asset_a, asset_b), asset_a_amount)?;

			Ok(())
		}

		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn redeem_lp_tokens(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			lp_token_amount: BalanceOf<T>,
		) -> DispatchResult {
			// check if message is signed
			let sender = ensure_signed(origin)?;

			// Get pool data
			let pool_id = Self::get_pool_id((asset_a, asset_b));
			let lp_token_id = Self::get_lp_token_id(&pool_id);

			// Check the user is able to make redemption
			Self::check_lp_redemption_is_valid(
				&sender,
				lp_token_id,
				lp_token_amount,
				(asset_a, asset_b),
			)?;

			// Redeem the users LP tokens
			Self::handle_lp_token_redemption(
				&sender,
				&pool_id,
				lp_token_id,
				lp_token_amount,
				(asset_a, asset_b),
			)?;

			Ok(())
		}
	}
}
