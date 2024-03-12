use sp_runtime::types::RemoteAssetMetadata;
use sp_std::vec::Vec;

use super::{
	fungibles::{EnumerateSystemToken, InspectSystemToken, InspectSystemTokenMetadata},
	pallet::*,
	AssetDetails, AssetMetadata, BoundedVec, DispatchError, Fiat,
};

impl<T: Config<I>, I: 'static> InspectSystemToken<T::AccountId> for Pallet<T, I> {
	type SystemTokenWeight = T::SystemTokenWeight;
	type Fiat = Fiat;

	fn is_system_token(asset: &Self::AssetId) -> bool {
		if let Some(ad) = Asset::<T, I>::get(asset) {
			return ad.is_sufficient
		}
		false
	}

	fn balance(
		who: &T::AccountId,
		maybe_asset: Option<T::AssetId>,
	) -> Option<(Self::AssetId, Self::Balance)> {
		Self::system_token_balance(who, maybe_asset)
	}

	fn system_token_weight(asset: Self::AssetId) -> Result<Self::SystemTokenWeight, DispatchError> {
		let ad = Asset::<T, I>::get(asset).ok_or(Error::<T, I>::Unknown)?;
		ad.system_token_weight.ok_or(Error::<T, I>::IncorrectStatus.into())
	}

	fn fiat(asset: Self::AssetId) -> Result<Self::Fiat, DispatchError> {
		let ad = Asset::<T, I>::get(asset).ok_or(Error::<T, I>::Unknown)?;
		ad.currency_type.ok_or(Error::<T, I>::IncorrectStatus.into())
	}
}

impl<T: Config<I>, I: 'static> EnumerateSystemToken<T::AccountId> for Pallet<T, I> {
	fn system_token_ids() -> impl IntoIterator<Item = Self::AssetId> {
		Asset::<T, I>::iter()
			.filter(|(_, ad)| ad.is_sufficient)
			.map(|(id, _)| id)
			.collect::<Vec<_>>()
			.into_iter()
	}
	fn system_token_account_balances(
		who: &T::AccountId,
	) -> impl IntoIterator<Item = (Self::AssetId, Self::Balance)> {
		Self::account_balances(who)
			.iter()
			.filter(|(i, _)| <Self as InspectSystemToken<T::AccountId>>::is_system_token(i))
			.map(|(i, b)| (i.clone(), b.clone()))
			.collect::<Vec<_>>()
			.into_iter()
	}
}
