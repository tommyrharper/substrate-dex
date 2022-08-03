use super::*;

impl<T: Config> Pallet<T>
where
	<T::Assets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
{
	// TODO: get rid of unwraps
	pub fn send_lp_tokens_to_pool_contributor(
		sender: &T::AccountId,
		pool_id: &T::AccountId,
		new_token_amount: BalanceOf<T>,
		current_token_amount: BalanceOf<T>,
	) -> Result<(AssetIdOf<T>, BalanceOf<T>), DispatchError> {
		let lp_token_id = Self::get_lp_token_id(pool_id);
		let total_lp_token_supply = T::Assets::total_issuance(lp_token_id);
		let lp_tokens_amount = get_lp_tokens_for_existing_pool(
			new_token_amount,
			current_token_amount,
			total_lp_token_supply,
		)
		.unwrap();
		let asset_id: AssetIdOf<T> = Self::get_lp_token_id(pool_id);
		T::Assets::mint_into(asset_id, sender, lp_tokens_amount)?;
		Ok((lp_token_id, lp_tokens_amount))
	}

	pub fn process_liquidity_pool_deposit(
		sender: &T::AccountId,
		asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
		asset_amounts: (BalanceOf<T>, BalanceOf<T>),
		current_token_amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		// Initialize the new pool
		let pool_id = Self::get_pool_id(asset_pair);

		// Transfer the tokens to the new pool
		Self::transfer_tokens_to_pool(&sender, &pool_id, asset_pair, asset_amounts)?;

		// Send the lp tokens in exchange to the pool creator
		let (lp_token_id, lp_tokens_amount) = Self::send_lp_tokens_to_pool_contributor(
			&sender,
			&pool_id,
			asset_amounts.0,
			current_token_amount,
		)?;

		Self::deposit_event(Event::LiquidityProvided(pool_id, lp_token_id, lp_tokens_amount));
		Ok(())
	}
}
