use sp_std::vec::Vec;

use super::{
	fungibles::{
		EnumerateSystemToken, InspectSystemToken, InspectSystemTokenMetadata, ManageSystemToken,
	},
	pallet::*,
	DispatchError, Fiat,
};

impl<T: Config<I>, I: 'static> InspectSystemToken<T::AccountId> for Pallet<T, I> {
	type SystemTokenWeight = T::SystemTokenWeight;

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

	fn system_token_weight(asset: &Self::AssetId) -> Result<Self::SystemTokenWeight, DispatchError> {
		let ad = Asset::<T, I>::get(asset).ok_or(Error::<T, I>::Unknown)?;
		ad.system_token_weight.ok_or(Error::<T, I>::IncorrectStatus.into())
	}

	fn fiat(asset: &Self::AssetId) -> Result<Fiat, DispatchError> {
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

impl<T: Config<I>, I: 'static> ManageSystemToken<T::AccountId> for Pallet<T, I> {
	fn register(
		asset: &Self::AssetId,
		system_token_weight: Self::SystemTokenWeight,
	) -> Result<(), DispatchError> {
		Self::do_register(asset, system_token_weight)
	}

	fn deregister(asset: &Self::AssetId) -> Result<(), DispatchError> {
		Self::do_deregister(asset)
	}

	fn update_system_token_weight(
		asset: &Self::AssetId,
		system_token_weight: Self::SystemTokenWeight,
	) -> Result<(), DispatchError> {
		Self::do_update_system_token_weight(asset, system_token_weight)
	}

	fn request_register(asset: &Self::AssetId, currency_type: Fiat) -> Result<(), DispatchError> {
		Self::do_request_register(asset, currency_type)
	}

	fn touch(
		owner: T::AccountId,
		asset: Self::AssetId,
		currency_type: Fiat,
		min_balance: Self::Balance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: Self::SystemTokenWeight,
	) -> Result<(), DispatchError> {
		Self::do_create_wrapped_local(
			owner,
			asset,
			currency_type,
			min_balance,
			name,
			symbol,
			decimals,
			system_token_weight,
		)
	}

	fn suspend(asset: &Self::AssetId) -> Result<(), DispatchError> {
		Self::do_suspend(asset)
	}

	fn unsuspend(asset: &Self::AssetId) -> Result<(), DispatchError> {
		Self::do_unsuspend(asset)
	}
}

impl<T: Config<I>, I: 'static> InspectSystemTokenMetadata<T::AccountId> for Pallet<T, I> {
	fn inner(asset: &Self::AssetId) -> Result<(Self::Balance, Fiat), DispatchError> {
		let mut asset_detail = Asset::<T, I>::get(asset).ok_or(Error::<T, I>::InvalidRequest)?;
		let min_balance = asset_detail.min_balance;
		let currency_type = asset_detail.currency_type.take().ok_or(Error::<T,I>::InvalidRequest)?;
		Ok((min_balance, currency_type))
	}
}
