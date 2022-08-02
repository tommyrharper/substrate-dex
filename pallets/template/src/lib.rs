#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;
use crate::dex_math::*;
use frame_support::{
    dispatch::{fmt::Debug, Codec, Decode, Encode},
    // dispatch::fmt::Display,
    pallet_prelude::*,
    sp_runtime::traits::{AccountIdConversion, AtLeast32Bit, AtLeast32BitUnsigned},
    traits::tokens::{
        currency::Currency,
        fungibles::{Create, Inspect, Mutate, Transfer},
    },
    Hashable,
    PalletId,
};
use frame_system::pallet_prelude::*;
use scale_info::prelude::vec;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod impl_liquidity;
mod dex_math;

type AssetIdOf<T: Config> = <T::MultiAssets as Inspect<T::AccountId>>::AssetId;
type BalanceOf<T: Config> = <T::MultiAssets as Inspect<T::AccountId>>::Balance;

#[frame_support::pallet]
pub mod pallet {
    use super::*;


	// How to do tight coupling:
	// pub trait Config: frame_system::Config + pallet_assets::Config {

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type MultiAssets: Inspect<Self::AccountId>
			+ Transfer<Self::AccountId>
			+ Mutate<Self::AccountId>
			+ Create<Self::AccountId>;

		type Balances: Currency<Self::AccountId>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://docs.substrate.io/v3/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	// #[pallet::storage]
	// #[pallet::unbounded]
	// pub(super) type Proofs<T: Config> = StorageMap<_, Blake2_128Concat, Vec<u8>, (T::AccountId,
	// T::BlockNumber), OptionQuery>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		// The user tried to stake more tokens than they have
		NotEnoughTokensToStake,
		// The user did not provide valid asset ids
		ProvidedInvalidAssetIds,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T::MultiAssets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
		<T::MultiAssets as Inspect<T::AccountId>>::AssetId: Codec,
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

			// Check the user input is valid
			Self::check_create_pool_input_is_valid(
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

			let asset_b_amount = Self::derive_second_asset_amount((asset_a, asset_b), asset_a_amount)?;

			// Check the user input is valid
			// TODO: update to check ratios are correct
			// Self::check_create_pool_input_is_valid(
			//     &sender,
			// 	(asset_a, asset_b),
			// 	(asset_a_amount, asset_b_amount),
			// )?;

			// Top up the liquidity pool
			Self::top_up_liquidity_pool(&sender, (asset_a, asset_b), (asset_a_amount, asset_b_amount))?;

			Ok(())
		}
	}
}
