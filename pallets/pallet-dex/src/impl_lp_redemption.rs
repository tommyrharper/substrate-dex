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
        ensure!(Self::has_enough_tokens(lp_token_id, lp_token_amount, sender), Error::<T>::NotEnoughLPTokens);

		Ok(())
	}

	pub fn handle_lp_token_redemption(
		sender: &T::AccountId,
        lp_token_id: AssetIdOf<T>,
		lp_token_amount: BalanceOf<T>,
		asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
	) -> Result<(), DispatchError> {
        // Send the user their assets
        

        // Burn the LP tokens

		Ok(())
	}
}
