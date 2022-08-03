use super::*;

impl<T: Config> Pallet<T>
where
	<T::Assets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
{

	// TODO: get rid of unwraps
	pub fn send_lp_tokens_to_pool_creator(
		sender: &T::AccountId,
		pool_id: &T::AccountId,
		asset_amounts: (BalanceOf<T>, BalanceOf<T>),
	) -> Result<(), DispatchError> {
		let lp_tokens_amount =
			get_lp_tokens_for_new_pool(asset_amounts.0, asset_amounts.1).unwrap();
		let asset_id: AssetIdOf<T> = Self::get_lp_token_id(pool_id);
		T::Assets::create(asset_id, Self::account_id(), true, 1u32.into())?;
		T::Assets::mint_into(asset_id, sender, lp_tokens_amount)?;
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

        Self::deposit_event(Event::NewPoolCreated(pool_id));
		Ok(())
	}
}
