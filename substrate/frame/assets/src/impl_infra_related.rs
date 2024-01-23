use super::*;
use frame_support::traits::PalletInfoAccess;

impl<T: Config<I>, I: 'static> LocalAssetManager for Pallet<T, I>
where
	T::AssetId: From<SystemTokenAssetId>,
	T::Balance: IsType<SystemTokenBalance>,
{
	type AccountId = T::AccountId;
	type Error = DispatchError;

	fn create_wrapped_local(
		asset_id: SystemTokenAssetId,
		currency_type: Fiat,
		min_balance: SystemTokenBalance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: SystemTokenWeight,
	) -> Result<(), Self::Error> {
		let owner: T::AccountId = frame_support::PalletId(*b"infra/rt").into_account_truncating();
		Self::try_create_wrapped_local(
			asset_id,
			currency_type,
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

	fn request_register(asset_id: SystemTokenAssetId) -> Result<(), Self::Error> {
		Self::try_request_register(asset_id)?;
		Ok(())
	}

	fn system_token_list() -> Vec<SystemTokenAssetId> {
		let assets = Asset::<T, I>::iter_keys();
		let token_list = assets
			.into_iter()
			.filter_map(|asset| {
				Asset::<T, I>::get(&asset)
					.filter(|detail| detail.is_sufficient)
					.map(|_| asset.into())
			})
			.collect::<Vec<SystemTokenAssetId>>();
		token_list
	}

	fn get_most_system_token_balance_of(
		asset_ids: impl IntoIterator<Item = SystemTokenAssetId>,
		account: T::AccountId,
	) -> SystemTokenAssetId {
		let mut most_balance: (SystemTokenAssetId, T::Balance) = Default::default();
		for asset_id in asset_ids {
			if let Some(balance) = Self::maybe_balance(asset_id.into(), account.clone()) {
				if most_balance.1 < balance {
					most_balance = (asset_id, balance);
				}
			}
		}
		most_balance.0
	}

	// TODO: Check owner of the token
	fn get_metadata(asset_id: SystemTokenAssetId) -> Result<RemoteAssetMetadata, Self::Error> {
		let id: T::AssetId = asset_id.clone().into();
		let asset_detail = Asset::<T, I>::get(&id).ok_or(Error::<T, I>::Unknown)?;
		let AssetDetails { is_sufficient, min_balance, .. } = asset_detail;
		if is_sufficient {
			return Err(Error::<T, I>::IncorrectStatus.into());
		}
		let AssetMetadata { name, symbol, currency_type, decimals, .. } =
			Metadata::<T, I>::get(&id).ok_or(Error::<T, I>::IncorrectStatus)?;
		let currency_type = currency_type.ok_or(Error::<T, I>::IncorrectStatus)?;
		let min_balance: SystemTokenBalance = min_balance.into();
		let pallet_id = <Self as PalletInfoAccess>::index() as u8;
		Ok(RemoteAssetMetadata {
			pallet_id,
			asset_id,
			name,
			symbol,
			currency_type,
			decimals,
			min_balance,
		})
	}
}
