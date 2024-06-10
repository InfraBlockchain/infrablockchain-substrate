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

use frame_support::{
	pallet_prelude::Get,
	traits::{Contains, ContainsPair},
};
use polkadot_primitives::Id as ParaId;
use xcm::{
	latest::prelude::{Asset, Location},
	prelude::*,
};

/// An implementation of `Contains` that checks for `Location` or
/// `InteriorLocation` if starts with the provided type `T`.
pub struct StartsWith<T, L = Location>(sp_std::marker::PhantomData<(T, L)>);
impl<T: Get<L>, L: TryInto<Location> + Clone> Contains<L> for StartsWith<T, L> {
	fn contains(location: &L) -> bool {
		let latest_location: Location =
			if let Ok(location) = (*location).clone().try_into() { location } else { return false };
		let latest_t = if let Ok(location) = T::get().try_into() { location } else { return false };
		latest_location.starts_with(&latest_t)
	}
}
impl<T: Get<InteriorLocation>> Contains<InteriorLocation> for StartsWith<T> {
	fn contains(t: &InteriorLocation) -> bool {
		t.starts_with(&T::get())
	}
}

/// An implementation of `Contains` that checks for `Location` or
/// `InteriorLocation` if starts with expected `GlobalConsensus(NetworkId)` provided as type
/// `T`.
pub struct StartsWithExplicitGlobalConsensus<T>(sp_std::marker::PhantomData<T>);
impl<T: Get<NetworkId>> Contains<Location> for StartsWithExplicitGlobalConsensus<T> {
	fn contains(location: &Location) -> bool {
		matches!(location.interior().global_consensus(), Ok(requested_network) if requested_network.eq(&T::get()))
	}
}
impl<T: Get<NetworkId>> Contains<InteriorLocation> for StartsWithExplicitGlobalConsensus<T> {
	fn contains(location: &InteriorLocation) -> bool {
		matches!(location.global_consensus(), Ok(requested_network) if requested_network.eq(&T::get()))
	}
}

frame_support::parameter_types! {
	pub LocalLocationPattern: Location = Location::new(0, Here);
	pub ParentLocation: Location = Location::parent();
}

/// Accepts an asset if it is from the origin.
pub struct IsForeignConcreteAsset<IsForeign>(sp_std::marker::PhantomData<IsForeign>);
impl<IsForeign: ContainsPair<Location, Location>> ContainsPair<Asset, Location>
	for IsForeignConcreteAsset<IsForeign>
{
	fn contains(asset: &Asset, origin: &Location) -> bool {
		log::trace!(target: "xcm::contains", "IsForeignConcreteAsset asset: {:?}, origin: {:?}", asset, origin);
		matches!(asset.id, AssetId(ref id) if IsForeign::contains(id, origin))
	}
}

// Checks if `a` is from sibling location `b`. Checks that `Location-a` starts with
/// `Location-b`, and that the `ParaId` of `b` is not equal to `a`.
pub struct FromSiblingParachain<SelfParaId, L = Location>(
	sp_std::marker::PhantomData<(SelfParaId, L)>,
);
impl<SelfParaId: Get<ParaId>, L: TryFrom<Location> + TryInto<Location> + Clone> ContainsPair<L, L>
	for FromSiblingParachain<SelfParaId, L>
{
	fn contains(a: &L, b: &L) -> bool {
		// We convert locations to latest
		let a = match ((*a).clone().try_into(), (*b).clone().try_into()) {
			(Ok(a), Ok(b)) if a.starts_with(&b) => a, // `a` needs to be from `b` at least
			_ => return false,
		};

		// here we check if sibling
		match a.unpack() {
			(1, interior) =>
				matches!(interior.first(), Some(Parachain(sibling_para_id)) if sibling_para_id.ne(&u32::from(SelfParaId::get()))),
			_ => false,
		}
	}
}
