#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{
		// dispatch::fmt::Display,
		pallet_prelude::*,
		sp_runtime::traits::{AccountIdConversion, AtLeast32Bit},
		traits::tokens::fungibles::{Inspect, Mutate, Transfer},
		Hashable, PalletId,
	};
	use frame_system::pallet_prelude::*;
    use scale_info::prelude::vec;

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
			+ Mutate<Self::AccountId>;

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
		NotEnoughTokensToStake,
		ProvidedInvalidAssetIds,
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn sub_account_id(sub: &[u8; 16]) -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(sub)
		}

		pub fn pot(asset_id: AssetIdOf<T>) -> BalanceOf<T> {
			T::MultiAssets::balance(asset_id, &Self::account_id())
		}

		pub fn sub_pot(asset_id: AssetIdOf<T>, sub: &[u8; 16]) -> BalanceOf<T> {
			T::MultiAssets::balance(asset_id, &Self::sub_account_id(sub))
		}
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T::MultiAssets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
		<T::MultiAssets as Inspect<T::AccountId>>::AssetId: Encode,
		/* where <T::MultiAssets
		 * as Inspect<T::AccountId>>::AssetId:
		 * AtLeast32Bit,
		 * <T::MultiAssets as
		 * Inspect<T::AccountId>>::AssetId:
		 * Display */
	{
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]

		// Args AssetsPallet::Config::AssetId,
		// DispatchResult {
		pub fn provide_liquidity(
			origin: OriginFor<T>,
			asset1: AssetIdOf<T>,
			asset2: AssetIdOf<T>,
			asset1_amount: BalanceOf<T>,
			asset2_amount: BalanceOf<T>,
		) -> DispatchResult {
			// check if message is signed
			let sender = ensure_signed(origin)?;

			ensure!(asset1 != asset2, Error::<T>::ProvidedInvalidAssetIds);

			// check if the user has enough assets
			let asset1_balance = T::MultiAssets::balance(asset1, &sender);
			ensure!(asset1_balance >= asset1_amount, Error::<T>::NotEnoughTokensToStake);

			let asset2_balance = T::MultiAssets::balance(asset2, &sender);
			ensure!(asset2_balance >= asset2_amount, Error::<T>::NotEnoughTokensToStake);

            let mut assets = vec![asset1, asset2];
            assets.sort();

            let hashed_assets = assets.twox_128();

            let sub_account_id = Self::sub_account_id(&hashed_assets);

			let res1 = T::MultiAssets::transfer(
				asset1,
				&sender,
				&sub_account_id,
				asset1_amount,
				false,
			);
			// .expect("Deposit of liquidity for asset1 failed.");
			let res2 = T::MultiAssets::transfer(
				asset2,
				&sender,
				&sub_account_id,
				asset2_amount,
				false,
			);
			// .expect("Deposit of liquidity for asset2 failed.");

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
