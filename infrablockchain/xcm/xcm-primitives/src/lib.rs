#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	sp_runtime::SaturatedConversion,
	traits::{fungibles::Inspect, Currency},
	weights::Weight,
};
use sp_runtime::{traits::MaybeEquivalence};
use sp_std::{borrow::Borrow, marker::PhantomData, prelude::*, vec::Vec};
use xcm::{
	latest::{AssetId::Concrete, Fungibility::Fungible, MultiAsset, MultiLocation},
	v3::{
		Junction::{GeneralIndex, PalletInstance},
		XcmContext,
	},
};
use xcm_executor::{
	traits::{DropAssets, Error as MatchError, MatchesFungibles},
	Assets,
};

pub trait Convert<A: Clone, B: Clone> {
	/// Convert from `value` (of type `A`) into an equivalent value of type `B`, `Err` if not
	/// possible.
	fn convert(value: A) -> Result<B, A> {
		Self::convert_ref(&value).map_err(|_| value)
	}
	fn convert_ref(value: impl Borrow<A>) -> Result<B, ()> {
		Self::convert(value.borrow().clone()).map_err(|_| ())
	}
	/// Convert from `value` (of type `B`) into an equivalent value of type `A`, `Err` if not
	/// possible.
	fn reverse(value: B) -> Result<A, B> {
		Self::reverse_ref(&value).map_err(|_| value)
	}
	fn reverse_ref(value: impl Borrow<B>) -> Result<A, ()> {
		Self::reverse(value.borrow().clone()).map_err(|_| ())
	}
}

#[impl_trait_for_tuples::impl_for_tuples(30)]
impl<A: Clone, B: Clone> Convert<A, B> for Tuple {
	fn convert(value: A) -> Result<B, A> {
		for_tuples!( #(
			let value = match Tuple::convert(value) {
				Ok(result) => return Ok(result),
				Err(v) => v,
			};
		)* );
		Err(value)
	}
	fn reverse(value: B) -> Result<A, B> {
		for_tuples!( #(
			let value = match Tuple::reverse(value) {
				Ok(result) => return Ok(result),
				Err(v) => v,
			};
		)* );
		Err(value)
	}
	fn convert_ref(value: impl Borrow<A>) -> Result<B, ()> {
		let value = value.borrow();
		for_tuples!( #(
			match Tuple::convert_ref(value) {
				Ok(result) => return Ok(result),
				Err(_) => (),
			}
		)* );
		Err(())
	}
	fn reverse_ref(value: impl Borrow<B>) -> Result<A, ()> {
		let value = value.borrow();
		for_tuples!( #(
			match Tuple::reverse_ref(value.clone()) {
				Ok(result) => return Ok(result),
				Err(_) => (),
			}
		)* );
		Err(())
	}
}

pub struct AsAssetMultiLocation<AssetId, AssetIdInfoGetter>(
	PhantomData<(AssetId, AssetIdInfoGetter)>,
);
impl<AssetId, AssetIdInfoGetter> MaybeEquivalence<MultiLocation, AssetId>
	for AsAssetMultiLocation<AssetId, AssetIdInfoGetter>
where
	AssetId: Clone,
	AssetIdInfoGetter: AssetMultiLocationGetter<AssetId>,
{
	fn convert(a: &MultiLocation) -> Option<AssetId> {
		AssetIdInfoGetter::get_asset_id(a.clone())
	}

	fn convert_back(b: &AssetId) -> Option<MultiLocation> {
		AssetIdInfoGetter::get_asset_multi_location(b.clone())
	}
}

impl<AssetId, AssetIdInfoGetter> Convert<MultiLocation, AssetId>
	for AsAssetMultiLocation<AssetId, AssetIdInfoGetter>
where
	AssetId: Clone,
	AssetIdInfoGetter: AssetMultiLocationGetter<AssetId>,
{
	fn convert_ref(asset_multi_location: impl Borrow<MultiLocation>) -> Result<AssetId, ()> {
		AssetIdInfoGetter::get_asset_id(asset_multi_location.borrow().clone()).ok_or(())
	}

	fn reverse_ref(asset_id: impl Borrow<AssetId>) -> Result<MultiLocation, ()> {
		AssetIdInfoGetter::get_asset_multi_location(asset_id.borrow().clone()).ok_or(())
	}
}

pub trait AssetMultiLocationGetter<AssetId> {
	fn get_asset_multi_location(asset_id: AssetId) -> Option<MultiLocation>;
	fn get_asset_id(asset_multi_location: MultiLocation) -> Option<AssetId>;
}

pub struct ConvertedRegisteredAssetId<AssetId, Balance, ConvertAssetId, ConvertBalance>(
	PhantomData<(AssetId, Balance, ConvertAssetId, ConvertBalance)>,
);
impl<
		AssetId: Clone,
		Balance: Clone,
		ConvertAssetId: Convert<MultiLocation, AssetId>,
		ConvertBalance: Convert<u128, Balance>,
	> MatchesFungibles<AssetId, Balance>
	for ConvertedRegisteredAssetId<AssetId, Balance, ConvertAssetId, ConvertBalance>
{
	fn matches_fungibles(a: &MultiAsset) -> Result<(AssetId, Balance), MatchError> {
		let (amount, id) = match (&a.fun, &a.id) {
			(Fungible(ref amount), Concrete(ref id)) => (amount, id),
			_ => return Err(MatchError::AssetNotHandled),
		};
		let what = ConvertAssetId::convert_ref(id).map_err(|_| MatchError::AssetNotHandled)?;
		let amount = ConvertBalance::convert_ref(amount)
			.map_err(|_| MatchError::AmountToBalanceConversionFailed)?;
		Ok((what, amount))
	}
}

pub struct TrappistDropAssets<
	AssetId,
	AssetIdInfoGetter,
	AssetsPallet,
	BalancesPallet,
	XcmPallet,
	AccoundId,
>(PhantomData<(AssetId, AssetIdInfoGetter, AssetsPallet, BalancesPallet, XcmPallet, AccoundId)>);
impl<AssetId, AssetIdInfoGetter, AssetsPallet, BalancesPallet, XcmPallet, AccountId> DropAssets
	for TrappistDropAssets<
		AssetId,
		AssetIdInfoGetter,
		AssetsPallet,
		BalancesPallet,
		XcmPallet,
		AccountId,
	> where
	AssetId: Clone + From<u128>,
	AssetIdInfoGetter: AssetMultiLocationGetter<AssetId>,
	AssetsPallet: Inspect<AccountId, AssetId = AssetId>,
	BalancesPallet: Currency<AccountId>,
	XcmPallet: DropAssets,
{
	// assets are whatever the Holding Register had when XCVM halts
	fn drop_assets(origin: &MultiLocation, assets: Assets, xcm_context: &XcmContext) -> Weight {
		let multi_assets: Vec<MultiAsset> = assets.into();
		let mut trap: Vec<MultiAsset> = Vec::new();

		for asset in multi_assets {
			if let MultiAsset { id: Concrete(location), fun: Fungible(amount) } = asset.clone() {
				// is location a fungible on AssetLink?
				if let Some(asset_id) = AssetIdInfoGetter::get_asset_id(location.clone()) {
					let min_balance = AssetsPallet::minimum_balance(asset_id);

					// only trap if amount ≥ min_balance
					// do nothing otherwise (asset is lost)
					if min_balance <= amount.saturated_into::<AssetsPallet::Balance>() {
						trap.push(asset);
					}

				// is location the native token?
				} else if matches!(
					location,
					MultiLocation {
						parents: 0,
						interior: xcm::v3::Junctions::X2(PalletInstance(_), GeneralIndex(_))
					}
				) {
					let asset_id = match location.interior {
						xcm::v3::Junctions::X2(PalletInstance(_), GeneralIndex(asset_id)) => {
							let asset_id: AssetId = asset_id.into();
							Some(asset_id)
						},
						_ => None,
					};
					if let Some(id) = asset_id {
						let min_balance = AssetsPallet::minimum_balance(id.into());

						// only trap if amount ≥ min_balance
						// do nothing otherwise (asset is lost)
						if min_balance <= amount.saturated_into::<AssetsPallet::Balance>() {
							trap.push(asset);
						}
					}
				} else {
					// When asset link has not set
					// ToDo: Check min balance?
					trap.push(asset);
				}
			}
		}

		// TODO: put real weight of execution up until this point here
		let mut weight = Weight::from(Weight::default());

		if !trap.is_empty() {
			// we have filtered out non-compliant assets
			// insert valid assets into the asset trap implemented by XcmPallet
			weight += XcmPallet::drop_assets(origin, trap.into(), xcm_context);
		}

		weight
	}
}
