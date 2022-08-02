use super::*;

impl<T: Config> Pallet<T>
where
	<T::MultiAssets as Inspect<T::AccountId>>::AssetId: AtLeast32Bit,
{
	pub fn sub_account_id(sub: &[u8; 16]) -> T::AccountId {
		T::PalletId::get().into_sub_account_truncating(sub)
	}
}
