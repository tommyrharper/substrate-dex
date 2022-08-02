use super::*;

impl<T: Config> Pallet<T>
where
	<T::MultiAssets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
{
	// TODO: check which of these functions need to be published
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
		new_token_amount: BalanceOf<T>,
		current_token_amount: BalanceOf<T>,
	) -> Result<(), DispatchError> {
		let lp_token_id = Self::get_lp_token_id(pool_id);
		let total_lp_token_supply = T::MultiAssets::total_issuance(lp_token_id);
		let lp_tokens_amount = get_lp_tokens_for_existing_pool(
			new_token_amount,
			current_token_amount,
			total_lp_token_supply,
		)
		.unwrap();
		let asset_id: AssetIdOf<T> = Self::get_lp_token_id(pool_id);
		T::MultiAssets::mint_into(asset_id, sender, lp_tokens_amount)?;
		Ok(())
	}

	pub fn check_deposit_is_valid(
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

	pub fn process_liquidity_pool_deposit(
		sender: &T::AccountId,
		asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
		asset_amounts: (BalanceOf<T>, BalanceOf<T>),
		pool_liquidity: (BalanceOf<T>, BalanceOf<T>),
	) -> Result<(), DispatchError> {
		// Initialize the new pool
		let pool_id = Self::get_pool_id(asset_pair);

		// Transfer the tokens to the new pool
		Self::transfer_tokens_to_pool(&sender, &pool_id, asset_pair, asset_amounts)?;

		// Send the lp tokens in exchange to the pool creator
		Self::send_lp_tokens_to_pool_contributor(
			&sender,
			&pool_id,
			asset_amounts.0,
			pool_liquidity.0,
		)?;

		Ok(())
	}

	pub fn derive_second_asset_amount(
		pool_liquidity: (BalanceOf<T>, BalanceOf<T>),
		asset_a_amount: BalanceOf<T>,
	) -> Result<BalanceOf<T>, DispatchError> {
		let second_asset_amount =
			get_token_b_amount(asset_a_amount, pool_liquidity).unwrap();

		Ok(second_asset_amount)
	}

	pub fn get_pool_liquidity(
		asset_pair: (AssetIdOf<T>, AssetIdOf<T>),
	) -> Result<(BalanceOf<T>, BalanceOf<T>), DispatchError> {
		let pool_id = Self::get_pool_id(asset_pair);

		let token_a_liquidity = T::MultiAssets::balance(asset_pair.0, &pool_id);
		let token_b_liquidity = T::MultiAssets::balance(asset_pair.1, &pool_id);

		Ok((token_a_liquidity, token_b_liquidity))
	}
}
