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

#[frame_support::pallet]
pub mod pallet {
    use super::*;
	type AssetIdOf<T: Config> = <T::MultiAssets as Inspect<T::AccountId>>::AssetId;
	type BalanceOf<T: Config> = <T::MultiAssets as Inspect<T::AccountId>>::Balance;

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
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		// The user tried to stake more tokens than they have
		NotEnoughTokensToStake,
		// The user did not provide valid asset ids
		ProvidedInvalidAssetIds,
	}

	impl<T: Config> Pallet<T>
	where
		<T::MultiAssets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
		// <T::MultiAssets as Inspect<T::AccountId>>::AssetId: From<T::AccountId>,
		// <T::MultiAssets as Inspect<T::AccountId>>::Balance: AtLeast32Bit,
		// T::AccountId: AtLeast32Bit,
	{
		// TODO: check which of these functions need to be published
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		// pub fn sub_account_id(sub: &[u8; 16]) -> T::AccountId {
		// 	T::PalletId::get().into_sub_account_truncating(sub)
		// }

		pub fn pot(asset_id: AssetIdOf<T>) -> BalanceOf<T> {
			T::MultiAssets::balance(asset_id, &Self::account_id())
		}

		pub fn sub_pot(asset_id: AssetIdOf<T>, sub: &[u8; 16]) -> BalanceOf<T> {
			T::MultiAssets::balance(asset_id, &Self::sub_account_id(sub))
		}

		pub fn has_enough_tokens(
			asset: AssetIdOf<T>,
			amount: BalanceOf<T>,
			sender: &T::AccountId,
		) -> bool {
			let asset_balance = T::MultiAssets::balance(asset, &sender);
			asset_balance >= amount
		}

		pub fn has_enough_of_both_tokens(
			sender: &T::AccountId,
			asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
			asset_amounts: (BalanceOf<T>, BalanceOf<T>),
		) -> DispatchResult {
			if Self::has_enough_tokens(asset_pair.0, asset_amounts.0, sender) &&
				Self::has_enough_tokens(asset_pair.1, asset_amounts.1, sender)
			{
				return Ok(())
			}
			Err(Error::<T>::NotEnoughTokensToStake.into())
		}

		pub fn get_pool_id(asset_pair: (AssetIdOf<T>, AssetIdOf<T>)) -> T::AccountId {
			let mut assets = vec![asset_pair.0, asset_pair.1];
			assets.sort();
			let hashed_assets = assets.twox_128();
			Self::sub_account_id(&hashed_assets)
		}

		pub fn initialize_pool(asset_pair: (AssetIdOf<T>, AssetIdOf<T>)) -> T::AccountId {
			let pool_id = Self::get_pool_id(asset_pair);
			T::Balances::make_free_balance_be(&pool_id, T::Balances::minimum_balance());
			pool_id
		}

		pub fn transfer_tokens_to_pool(
			sender: &T::AccountId,
			pool_id: &T::AccountId,
			asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
			asset_amounts: (BalanceOf<T>, BalanceOf<T>),
		) -> Result<(), DispatchError> {
			T::MultiAssets::transfer(asset_pair.0, &sender, &pool_id, asset_amounts.0, false)?;
			T::MultiAssets::transfer(asset_pair.1, &sender, &pool_id, asset_amounts.1, false)?;
			Ok(())
		}

		pub fn get_lp_token_id(pool_id: &T::AccountId) -> AssetIdOf<T> {
			let pool_id_hash = pool_id.twox_128();
			// TODO: remove unwrap
			let pool_id_hash_number: u32 = Decode::decode(&mut &pool_id_hash[..]).unwrap();
			let asset_id: AssetIdOf<T> = pool_id_hash_number.into();
			asset_id
		}

		// TODO: get rid of unwraps
		pub fn send_lp_tokens_to_pool_creator(
			sender: &T::AccountId,
			pool_id: &T::AccountId,
			asset_amounts: (BalanceOf<T>, BalanceOf<T>),
		) -> Result<(), DispatchError> {
			let lp_tokens_amount =
				get_lp_tokens_for_new_pool(asset_amounts.0, asset_amounts.1).unwrap();
			let asset_id: AssetIdOf<T> = Self::get_lp_token_id(pool_id);
			T::MultiAssets::create(asset_id, Self::account_id(), true, 1u32.into())?;
			T::MultiAssets::mint_into(asset_id, sender, lp_tokens_amount)?;
			Ok(())
		}

		// TODO: get rid of unwraps
		pub fn send_lp_tokens_to_pool_contributor(
			sender: &T::AccountId,
			pool_id: &T::AccountId,
			asset_amounts: (BalanceOf<T>, BalanceOf<T>),
		) -> Result<(), DispatchError> {
			// let lp_tokens_amount =
			// get_lp_tokens_for_existing_pool(asset_amounts.0, asset_amounts.1).unwrap();
			// let asset_id: AssetIdOf<T> = Self::get_lp_token_id(pool_id);
			// T::MultiAssets::create(asset_id, Self::account_id(), true, 1u32.into())?;
			// T::MultiAssets::mint_into(asset_id, sender, lp_tokens_amount)?;
			Ok(())
		}

		pub fn check_create_pool_input_is_valid(
			sender: &T::AccountId,
			asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
			asset_amounts: (BalanceOf<T>, BalanceOf<T>),
		) -> Result<(), DispatchError> {
			// Ensure that the assets are valid.
			// TODO: refactor into method
			ensure!(asset_pair.0 != asset_pair.1, Error::<T>::ProvidedInvalidAssetIds);

			// check if sender has enough tokens to stake
			Self::has_enough_of_both_tokens(&sender, asset_pair, asset_amounts)?;

			Ok(())
		}

		pub fn check_provide_liquidity_input_is_valid(
			sender: &T::AccountId,
			asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
			asset_amounts: (BalanceOf<T>, BalanceOf<T>),
		) -> Result<(), DispatchError> {
			// Ensure that the assets are valid.
			ensure!(asset_pair.0 != asset_pair.1, Error::<T>::ProvidedInvalidAssetIds);

			// check if sender has enough tokens to stake
			Self::has_enough_of_both_tokens(&sender, asset_pair, asset_amounts)?;

			Ok(())
		}

		pub fn create_new_pool(
			sender: &T::AccountId,
			asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
			asset_amounts: (BalanceOf<T>, BalanceOf<T>),
		) -> Result<(), DispatchError> {
			// Initialize the new pool
			let pool_id = Self::initialize_pool(asset_pair);

			// Transfer the tokens to the new pool
			Self::transfer_tokens_to_pool(&sender, &pool_id, asset_pair, asset_amounts)?;

			// Send the lp tokens in exchange to the pool creator
			Self::send_lp_tokens_to_pool_creator(&sender, &pool_id, asset_amounts)?;

			Ok(())
		}

		pub fn top_up_liquidity_pool(
			sender: &T::AccountId,
			asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
			asset_amounts: (BalanceOf<T>, BalanceOf<T>),
		) -> Result<(), DispatchError> {
			// Initialize the new pool
			let pool_id = Self::get_pool_id(asset_pair);

			// Transfer the tokens to the new pool
			Self::transfer_tokens_to_pool(&sender, &pool_id, asset_pair, asset_amounts)?;

			// Send the lp tokens in exchange to the pool creator
			Self::send_lp_tokens_to_pool_contributor(&sender, &pool_id, asset_amounts)?;

			Ok(())
		}

		pub fn derive_second_asset_amount(
			asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
			asset1_amount: BalanceOf<T>,
		) -> Result<BalanceOf<T>, DispatchError> {
			// Initialize the new pool
			let pool_id = Self::get_pool_id(asset_pair);
			let token_1_liquidity = T::MultiAssets::balance(asset_pair.0, &pool_id);
			let token_2_liquidity = T::MultiAssets::balance(asset_pair.1, &pool_id);

			let second_asset_amount =
				get_token_b_amount(asset1_amount, token_1_liquidity, token_2_liquidity).unwrap();

			Ok(second_asset_amount)
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T::MultiAssets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
		<T::MultiAssets as Inspect<T::AccountId>>::AssetId: Codec,
		/* where <T::MultiAssets
		 * as Inspect<T::AccountId>>::AssetId:
		 * AtLeast32Bit,
		 * <T::MultiAssets as
		 * Inspect<T::AccountId>>::AssetId:
		 * Display */
	{
		// Args AssetsPallet::Config::AssetId,
		// DispatchResult {
		// TODO: see if tuples are a practical input here
		// TODO: update asset1 and 2 to a and b
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn create_pool(
			origin: OriginFor<T>,
			asset1: AssetIdOf<T>,
			asset2: AssetIdOf<T>,
			asset1_amount: BalanceOf<T>,
			asset2_amount: BalanceOf<T>,
		) -> DispatchResult {
			// check if message is signed
			let sender = ensure_signed(origin)?;

			// Check the user input is valid
			Self::check_create_pool_input_is_valid(
				&sender,
				(asset1, asset2),
				(asset1_amount, asset2_amount),
			)?;

			// Create the new liquidity pool
			Self::create_new_pool(&sender, (asset1, asset2), (asset1_amount, asset2_amount))?;

			Ok(())
		}

		#[pallet::weight(5_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn provide_liquidity(
			origin: OriginFor<T>,
			asset1: AssetIdOf<T>,
			asset2: AssetIdOf<T>,
			asset1_amount: BalanceOf<T>,
		) -> DispatchResult {
			// check if message is signed
			let sender = ensure_signed(origin)?;

			let asset2_amount = Self::derive_second_asset_amount((asset1, asset2), asset1_amount)?;

			// Check the user input is valid
			// TODO: update to check ratios are correct
			// Self::check_create_pool_input_is_valid(
			//     &sender,
			// 	(asset1, asset2),
			// 	(asset1_amount, asset2_amount),
			// )?;

			// Top up the liquidity pool
			Self::top_up_liquidity_pool(&sender, (asset1, asset2), (asset1_amount, asset2_amount))?;

			Ok(())
		}

		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://docs.substrate.io/v3/runtime/origins
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => return Err(Error::<T>::NoneValue.into()),
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}
	}
}
