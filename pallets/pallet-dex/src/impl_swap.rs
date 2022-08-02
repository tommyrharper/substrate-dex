use super::*;

impl<T: Config> Pallet<T>
where
	<T::MultiAssets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
{
    pub fn check_swap_is_valid(
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

    pub fn process_swap() {
        
    }

}
