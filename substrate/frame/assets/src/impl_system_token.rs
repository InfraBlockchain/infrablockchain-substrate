
use sp_runtime::types::RemoteAssetMetadata;
use sp_std::vec::Vec;

use super::{
    pallet::*,
    fungibles::{
        InspectSystemToken, InspectSystemTokenMetadata,
        EnumerateSystemToken, ManageSystemToken
    },
    BoundedVec, DispatchError, AssetDetails, AssetMetadata,
    SystemTokenWeight, Fiat,
};

impl<T: Config<I>, I: 'static> InspectSystemToken<T::AccountId> for Pallet<T, I> {
    fn is_system_token(asset: T::AssetId) -> bool {
        if let Some(ad) = Asset::<T, I>::get(asset) {
            return ad.is_sufficient
        } 
        false
    }

    fn balance(who: T::AccountId, maybe_asset: Option<T::AssetId>) -> T::Balance {
        maybe_asset.map_or_else(|| Self::most_system_token_balance(&who), |a| Self::balance(a, &who))
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

// impl<T: Config<I>, I: 'static> InspectSystemTokenMetadata<
//     T::AccountId, 
//     Fiat,
//     SystemTokenWeight,
//     RemoteAssetMetadata<Self::AssetId, Self::Balance>
// > for Pallet<T, I> {

//     fn system_token_metadata(asset: Self::AssetId) -> Result<RemoteAssetMetadata<Self::AssetId, Self::Balance>, DispatchError> {
// 		let asset_detail = Asset::<T, I>::get(&asset).ok_or(Error::<T, I>::Unknown)?;
// 		let AssetDetails { is_sufficient, min_balance, .. } = asset_detail;
// 		if is_sufficient {
// 			return Err(Error::<T, I>::IncorrectStatus.into())
// 		}
// 		let AssetMetadata { name, symbol, currency_type, decimals, .. } =
// 			Metadata::<T, I>::get(&asset).ok_or(Error::<T, I>::IncorrectStatus)?;
// 		let currency_type = currency_type.ok_or(Error::<T, I>::IncorrectStatus)?;
// 		Ok(RemoteAssetMetadata {
// 			asset,
// 			name: name.into(),
// 			symbol: symbol.into(),
// 			currency_type,
// 			decimals,
// 			min_balance,
// 		})
//     }
//     fn system_token_weight(asset: Self::AssetId) -> SystemTokenWeight {
//         Default::default()
//     }
//     /// Return `Self::Fiat` of System Token
//     fn fiat(asset: Self::AssetId) -> Fiat {
//         Default::default()
//     }
// }

