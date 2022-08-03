use super::*;

impl<T: Config> Pallet<T>
where
	<T::MultiAssets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
{
	pub fn check_lp_redemption_is_valid(
		sender: &T::AccountId,
		lp_token_id: AssetIdOf<T>,
		lp_token_amount: BalanceOf<T>,
		asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
	) -> Result<(), DispatchError> {
		// Ensure that the assets are valid.
		ensure!(asset_pair.0 != asset_pair.1, Error::<T>::ProvidedInvalidAssetIds);

		// check if sender has enough lp tokens
		ensure!(
			Self::has_enough_tokens(lp_token_id, lp_token_amount, sender),
			Error::<T>::NotEnoughLPTokens
		);

		Ok(())
	}

	pub fn handle_lp_token_redemption(
		sender: &T::AccountId,
		pool_id: &T::AccountId,
		lp_token_id: AssetIdOf<T>,
		lp_token_amount: BalanceOf<T>,
		asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
	) -> Result<(), DispatchError> {
        // Get pool data
		let pool_liquidity = Self::get_pool_liquidity(asset_pair)?;
		let total_lp_token_supply = T::MultiAssets::total_issuance(lp_token_id);
		let redeemed_token_amounts =
			get_redeemed_token_balance(lp_token_amount, total_lp_token_supply, pool_liquidity).unwrap();

		// Send the user their assets
		T::MultiAssets::transfer(asset_pair.0, &pool_id, &sender, redeemed_token_amounts.0, true)?;
		T::MultiAssets::transfer(asset_pair.1, &pool_id, &sender, redeemed_token_amounts.1, true)?;

		// Burn the LP tokens
		T::MultiAssets::burn_from(lp_token_id, &sender, lp_token_amount)?;

		Ok(())
	}
}
