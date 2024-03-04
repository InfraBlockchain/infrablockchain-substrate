
use sp_runtime::types::RemoteAssetMetadata;
use sp_std::vec::Vec;

use super::{
    pallet::*,
    fungibles::{
        InspectSystemToken, InspectSystemTokenMetadata,
        EnumerateSystemToken
    },
    BoundedVec, DispatchError, AssetDetails, AssetMetadata, Fiat,
};

impl<T: Config<I>, I: 'static> InspectSystemToken<T::AccountId> for Pallet<T, I> {

    type SystemTokenWeight = T::SystemTokenWeight;
    type Fiat = Fiat;

    fn is_system_token(asset: T::AssetId) -> bool {
        if let Some(ad) = Asset::<T, I>::get(asset) {
            return ad.is_sufficient
        } 
        false
    }

    fn balance(who: T::AccountId, maybe_asset: Option<T::AssetId>) -> T::Balance {
        maybe_asset.map_or_else(|| Self::most_system_token_balance(&who), |a| Self::balance(a, &who))
    }
    
    fn system_token_weight(asset: Self::AssetId) -> Result<Self::SystemTokenWeight, DispatchError> {
        let ad = Asset::<T, I>::get(asset).ok_or(Error::<T ,I>::Unknown)?;
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
}

