// Copyright (C) 2023 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod fungible_conversion;
pub mod local_and_foreign_assets;
pub mod matching;
pub mod runtime_api;

use crate::matching::{LocalLocationPattern, ParentLocation, StartsWith};
use sp_runtime::traits::Zero;

use frame_support::traits::{
	fungibles::{self},
	Contains, Equals, EverythingBut, ProcessMessageError,
};
use xcm::prelude::Location;
use xcm_builder::{
	AsPrefixedGeneralIndex, CreateMatcher, MatchXcm, MatchedConvertedConcreteId,
	V4V4LocationConverter,
};
use xcm_executor::traits::{JustTry, Properties};

use frame_support::{
	traits::{fungibles::Inspect, tokens::ConversionToAssetBalance, ContainsPair},
	weights::{Weight, WeightToFee, WeightToFeePolynomial},
};
use sp_runtime::traits::Get;
use sp_std::{cell::Cell, fmt::Debug, marker::PhantomData, ops::ControlFlow, result::Result};
use xcm::latest::prelude::*;
use xcm_executor::traits::ShouldExecute;

type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

/// Deny executing the XCM if it matches any of the Deny filter regardless of anything else.
/// If it passes the Deny, and matches one of the Allow cases then it is let through.
pub struct DenyThenTry<Deny, Allow>(PhantomData<Deny>, PhantomData<Allow>)
where
	Deny: ShouldExecute,
	Allow: ShouldExecute;

impl<Deny, Allow> ShouldExecute for DenyThenTry<Deny, Allow>
where
	Deny: ShouldExecute,
	Allow: ShouldExecute,
{
	fn should_execute<RuntimeCall>(
		origin: &Location,
		message: &mut [Instruction<RuntimeCall>],
		max_weight: Weight,
		properties: &mut Properties,
	) -> Result<(), ProcessMessageError> {
		Deny::should_execute(origin, message, max_weight, properties)?;
		Allow::should_execute(origin, message, max_weight, properties)
	}
}

// See issue <https://github.com/paritytech/polkadot/issues/5233>
pub struct DenyReserveTransferToRelayChain;
impl ShouldExecute for DenyReserveTransferToRelayChain {
	fn should_execute<RuntimeCall>(
		origin: &Location,
		message: &mut [Instruction<RuntimeCall>],
		_max_weight: Weight,
		_properties: &mut Properties,
	) -> Result<(), ProcessMessageError> {
		message.matcher().match_next_inst_while(
			|_| true,
			|inst| match inst {
				InitiateReserveWithdraw {
					reserve: Location { parents: 1, interior: Here },
					..
				} |
				DepositReserveAsset { dest: Location { parents: 1, interior: Here }, .. } |
				TransferReserveAsset { dest: Location { parents: 1, interior: Here }, .. } => {
					Err(ProcessMessageError::Unsupported) // Deny
				},

				// An unexpected reserve transfer has arrived from the Relay Chain. Generally,
				// `IsReserve` should not allow this, but we just log it here.
				ReserveAssetDeposited { .. }
					if matches!(origin, Location { parents: 1, interior: Here }) =>
				{
					log::warn!(
						target: "xcm::barrier",
						"Unexpected ReserveAssetDeposited from the Relay Chain",
					);
					Ok(ControlFlow::Continue(()))
				},

				_ => Ok(ControlFlow::Continue(())),
			},
		)?;

		// Permit everything else
		Ok(())
	}
}

/// A `ChargeFeeInFungibles` implementation that converts the output of
/// a given WeightToFee implementation an amount charged in
/// a particular assetId from pallet-assets
pub struct AssetFeeAsExistentialDepositMultiplier<
	Runtime,
	WeightToFee,
	BalanceConverter,
	AssetInstance: 'static,
>(PhantomData<(Runtime, WeightToFee, BalanceConverter, AssetInstance)>);
impl<CurrencyBalance, Runtime, WeightToFee, BalanceConverter, AssetInstance>
	cumulus_primitives_utility::ChargeWeightInFungibles<
		AccountIdOf<Runtime>,
		pallet_assets::Pallet<Runtime, AssetInstance>,
	> for AssetFeeAsExistentialDepositMultiplier<Runtime, WeightToFee, BalanceConverter, AssetInstance>
where
	Runtime: pallet_assets::Config<AssetInstance>,
	WeightToFee: WeightToFeePolynomial<Balance = CurrencyBalance>,
	BalanceConverter: ConversionToAssetBalance<
		CurrencyBalance,
		<Runtime as pallet_assets::Config<AssetInstance>>::AssetId,
		<Runtime as pallet_assets::Config<AssetInstance>>::Balance,
	>,
	AccountIdOf<Runtime>:
		From<polkadot_primitives::AccountId> + Into<polkadot_primitives::AccountId>,
	CurrencyBalance: Debug,
{
	fn charge_weight_in_fungibles(
		asset_id: <pallet_assets::Pallet<Runtime, AssetInstance> as Inspect<
			AccountIdOf<Runtime>,
		>>::AssetId,
		weight: Weight,
	) -> Result<
		<pallet_assets::Pallet<Runtime, AssetInstance> as Inspect<AccountIdOf<Runtime>>>::Balance,
		XcmError,
	> {
		log::trace!(target: "xcm::charge_weight_in_fungibles",
			"charge_weight_in_fungibles asset: asset_id: {:?}, weight: {:?}",
			asset_id, weight);
		let amount = WeightToFee::weight_to_fee(&weight);

		log::trace!(target: "xcm::charge_weight_in_fungibles",
			"charge_weight_in_fungibles asset: amount: {:?}", amount);

		// If the amount gotten is not at least the ED, then make it be the ED of the asset
		// This is to avoid burning assets and decreasing the supply
		let asset_amount = BalanceConverter::to_asset_balance(amount, asset_id)
			.map_err(|_| XcmError::TooExpensive)?;
		Ok(asset_amount)
	}
}

pub struct AssetFeeAsExistentialDepositMultiplierForPara<
	Runtime,
	WeightToFee,
	BalanceConverter,
	AssetInstance: 'static,
>(PhantomData<(Runtime, WeightToFee, BalanceConverter, AssetInstance)>);
impl<CurrencyBalance, Runtime, WeightToFee, BalanceConverter, AssetInstance>
	cumulus_primitives_utility::ChargeWeightInFungibles<
		AccountIdOf<Runtime>,
		pallet_assets::Pallet<Runtime, AssetInstance>,
	>
	for AssetFeeAsExistentialDepositMultiplierForPara<
		Runtime,
		WeightToFee,
		BalanceConverter,
		AssetInstance,
	> where
	Runtime: pallet_assets::Config<AssetInstance>,
	WeightToFee: frame_support::weights::WeightToFee<Balance = CurrencyBalance>,
	BalanceConverter: ConversionToAssetBalance<
		CurrencyBalance,
		<Runtime as pallet_assets::Config<AssetInstance>>::AssetId,
		<Runtime as pallet_assets::Config<AssetInstance>>::Balance,
	>,
	AccountIdOf<Runtime>:
		From<polkadot_primitives::AccountId> + Into<polkadot_primitives::AccountId>,
	CurrencyBalance: Debug,
{
	fn charge_weight_in_fungibles(
		asset_id: <pallet_assets::Pallet<Runtime, AssetInstance> as Inspect<
			AccountIdOf<Runtime>,
		>>::AssetId,
		weight: Weight,
	) -> Result<
		<pallet_assets::Pallet<Runtime, AssetInstance> as Inspect<AccountIdOf<Runtime>>>::Balance,
		XcmError,
	> {
		log::trace!(target: "xcm::charge_weight_in_fungibles",
			"charge_weight_in_fungibles asset: asset_id: {:?}, weight: {:?}",
			asset_id, weight);
		let amount = WeightToFee::weight_to_fee(&weight);

		log::trace!(target: "xcm::charge_weight_in_fungibles",
			"charge_weight_in_fungibles asset: amount: {:?}", amount);

		// If the amount gotten is not at least the ED, then make it be the ED of the asset
		// This is to avoid burning assets and decreasing the supply
		let asset_amount = BalanceConverter::to_asset_balance(amount, asset_id)
			.map_err(|_| XcmError::TooExpensive)?;
		Ok(asset_amount)
	}
}

/// Accepts an asset if it is a native asset from a particular `Location`.
pub struct ConcreteNativeAssetFrom<LocationValue>(PhantomData<LocationValue>);
impl<LocationValue: Get<Location>> ContainsPair<Asset, Location>
	for ConcreteNativeAssetFrom<LocationValue>
{
	fn contains(asset: &Asset, origin: &Location) -> bool {
		log::trace!(target: "xcm::filter_asset_location",
			"ConcreteNativeAsset asset: {:?}, origin: {:?}, location: {:?}",
			asset, origin, LocationValue::get());
		asset.id.0 == *origin && origin == &LocationValue::get()
	}
}

/// Allow checking in assets that have issuance > 0.
pub struct NonZeroIssuance<AccountId, Assets>(PhantomData<(AccountId, Assets)>);
impl<AccountId, Assets> Contains<<Assets as fungibles::Inspect<AccountId>>::AssetId>
	for NonZeroIssuance<AccountId, Assets>
where
	Assets: fungibles::Inspect<AccountId>,
{
	fn contains(id: &<Assets as fungibles::Inspect<AccountId>>::AssetId) -> bool {
		!Assets::total_issuance(id.clone()).is_zero()
	}
}

/// Allow not checking in assets that have issuance > 0.
pub struct AnyIssuance<AccountId, Assets>(PhantomData<(AccountId, Assets)>);
impl<AccountId, Assets> Contains<<Assets as fungibles::Inspect<AccountId>>::AssetId>
	for AnyIssuance<AccountId, Assets>
where
	Assets: fungibles::Inspect<AccountId>,
{
	fn contains(_id: &<Assets as fungibles::Inspect<AccountId>>::AssetId) -> bool {
		true
	}
}

pub type AssetIdForNativeAssets = u32;

/// `Location` vs `AssetIdForNativeAssets` converter for `TrustBackedAssets`
pub type AssetIdForNativeAssetsConvert<NativeAssetsPalletLocation> =
	AsPrefixedGeneralIndex<NativeAssetsPalletLocation, AssetIdForNativeAssets, JustTry, Location>;

/// [`MatchedConvertedConcreteId`] converter dedicated for `TrustBackedAssets`
pub type NativeAssetsConvertedConcreteId<NativeAssetsPalletLocation, Balance> =
	MatchedConvertedConcreteId<
		AssetIdForNativeAssets,
		Balance,
		StartsWith<NativeAssetsPalletLocation>,
		AssetIdForNativeAssetsConvert<NativeAssetsPalletLocation>,
		JustTry,
	>;

/// AssetId used for identifying assets by MultiLocation.
pub type MultiLocationForAssetId = Location;

/// [`MatchedConvertedConcreteId`] converter dedicated for storing `AssetId` as `Location`.
pub type LocationConvertedConcreteId<LocationFilter, Balance> =
	MatchedConvertedConcreteId<Location, Balance, LocationFilter, V4V4LocationConverter, JustTry>;

/// [`MatchedConvertedConcreteId`] converter dedicated for storing `ForeignAssets` with `AssetId` as
/// `Location`.
///
/// Excludes by default:
/// - parent as relay chain
/// - all local Locations
///
/// `AdditionalLocationExclusionFilter` can customize additional excluded Locations
pub type ForeignAssetsConvertedConcreteId<AdditionalLocationExclusionFilter, Balance> =
	LocationConvertedConcreteId<
		EverythingBut<(
			// Excludes relay/parent chain currency
			Equals<ParentLocation>,
			// Here we rely on fact that something like this works:
			// assert!(Location::new(1,
			// [Parachain(100)]).starts_with(&Location::parent()));
			// assert!([Parachain(100)].into().starts_with(&Here));
			StartsWith<LocalLocationPattern>,
			// Here we can exclude more stuff or leave it as `()`
			AdditionalLocationExclusionFilter,
		)>,
		Balance,
	>;

/// For Relay
/// `AdditionalLocationExclusionFilter` can customize additional excluded Locations
pub type ForeignAssetsConvertedConcreteIdForParent<AdditionalLocationExclusionFilter, Balance> =
	LocationConvertedConcreteId<EverythingBut<AdditionalLocationExclusionFilter>, Balance>;

#[cfg(test)]
mod tests {

	use super::*;
	use xcm::latest::prelude::*;
	use xcm_executor::traits::Convert;

	frame_support::parameter_types! {
		pub TrustBackedAssetsPalletLocation: MultiLocation = MultiLocation::new(5, X1(PalletInstance(13)));
	}

	#[test]
	fn asset_id_for_trust_backed_assets_convert_works() {
		let local_asset_id = 123456789 as AssetIdForNativeAssets;
		let expected_reverse_ref =
			MultiLocation::new(5, X2(PalletInstance(13), GeneralIndex(local_asset_id.into())));

		assert_eq!(
			AssetIdForOriginalAssetsConvert::<TrustBackedAssetsPalletLocation>::reverse_ref(
				local_asset_id
			)
			.unwrap(),
			expected_reverse_ref
		);
		assert_eq!(
			AssetIdForOriginalAssetsConvert::<TrustBackedAssetsPalletLocation>::convert_ref(
				expected_reverse_ref
			)
			.unwrap(),
			local_asset_id
		);
	}
}
