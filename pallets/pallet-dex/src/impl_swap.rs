use super::*;

impl<T: Config> Pallet<T>
where
	<T::MultiAssets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
{
	pub fn process_swap(
		sender: &T::AccountId,
		asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
		asset_a_amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
        // Get swap data
		let pool_id = Self::get_pool_id(asset_pair);
        let pool_liquidity = Self::get_pool_liquidity(asset_pair)?;
        let swap_return =
				get_swap_return::<BalanceOf<T>, T>(asset_a_amount, pool_liquidity)?;

		// Send tokens into pool
		T::MultiAssets::transfer(asset_pair.0, &sender, &pool_id, asset_a_amount, true)?;

		// Send tokens to users
		T::MultiAssets::transfer(asset_pair.1, &pool_id, &sender, swap_return, true)?;

        Ok(())
	}
}
