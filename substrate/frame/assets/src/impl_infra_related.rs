use super::*;

impl<T: Config<I>, I: 'static> LocalAssetManager for Pallet<T, I> {
	type Error = DispatchResult;

	fn create_wrapped_local(
		asset_id: SystemTokenAssetId,
		min_balance: SystemTokenBalance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: SystemTokenWeight,
	) -> Result<(), Self::Error> {
		let owner: T::AccountId = frame_support::PalletId(*b"infra/rt").into_account_truncating();
		Self::try_create_wrapped_local(
			asset_id,
			owner,
			min_balance,
			name,
			symbol,
			decimals,
			Some(system_token_weight),
		)?;
		Ok(())
	}

	fn promote(
		asset_id: SystemTokenAssetId,
		system_token_weight: SystemTokenWeight,
	) -> Result<(), Self::Error> {
		Self::try_promote(asset_id, system_token_weight)?;
		Ok(())
	}

	fn demote(asset_id: SystemTokenAssetId) -> Result<(), Self::Error> {
		Self::try_demote(asset_id)?;
		Ok(())
	}

	fn update_system_token_weight(
		asset_id: SystemTokenAssetId,
		system_token_weight: SystemTokenWeight,
	) -> Result<(), Self::Error> {
		Self::try_update_system_token_weight(asset_id, system_token_weight)?;
		Ok(())
	}
}
