// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Cumulus.

// Cumulus is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Cumulus is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Cumulus.  If not, see <http://www.gnu.org/licenses/>.

use crate::chain_spec::{
	get_account_id_from_seed, get_collator_keys_from_seed, Extensions, SAFE_XCM_VERSION,
};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use parachains_common::types::{AccountId, AuraId, Balance as URAuthBalance};
use sc_service::ChainType;
use sp_core::{crypto::UncheckedInto, sr25519};

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type URAuthChainSpec =
	sc_service::GenericChainSpec<urauth_runtime::RuntimeGenesisConfig, Extensions>;

const URAUTH_INFRA_RELAY_ED: URAuthBalance =
	parachains_common::infra_relay::currency::EXISTENTIAL_DEPOSIT;

const URAUTH_PARACHAIN_ID: u32 = 2000;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn urauth_session_keys(keys: AuraId) -> urauth_runtime::SessionKeys {
	urauth_runtime::SessionKeys { aura: keys }
}

pub fn urauth_development_config() -> URAuthChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("ss58Format".into(), 0.into());
	properties.insert("tokenSymbol".into(), "".into());
	properties.insert("tokenDecimals".into(), 10.into());

	URAuthChainSpec::from_genesis(
		// Name
		"InfraBlockchain URAuth Dev",
		// ID
		"urauth-infra-dev",
		ChainType::Local,
		move || {
			urauth_genesis(
				// initial collators.
				vec![(
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_collator_keys_from_seed::<AuraId>("Alice"),
				)],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
				],
				vec![],
				Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
				URAUTH_PARACHAIN_ID.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(properties),
		Extensions { relay_chain: "infra-relay-local".into(), para_id: URAUTH_PARACHAIN_ID },
	)
}

pub fn urauth_local_config() -> URAuthChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("ss58Format".into(), 0.into());
	properties.insert("tokenSymbol".into(), "".into());
	properties.insert("tokenDecimals".into(), 10.into());

	URAuthChainSpec::from_genesis(
		// Name
		"InfraBlockchain URAuth Local",
		// ID
		"urauth-infra-local",
		ChainType::Local,
		move || {
			urauth_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed::<AuraId>("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed::<AuraId>("Bob"),
					),
				],
				vec![
					get_account_id_from_seed::<sr25519::Public>("Alice"),
					get_account_id_from_seed::<sr25519::Public>("Bob"),
					get_account_id_from_seed::<sr25519::Public>("Charlie"),
					get_account_id_from_seed::<sr25519::Public>("Dave"),
					get_account_id_from_seed::<sr25519::Public>("Eve"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie"),
					get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
					get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
					get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
					get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
					get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
					get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
				],
				Default::default(),
				Some(get_account_id_from_seed::<sr25519::Public>("Alice")),
				URAUTH_PARACHAIN_ID.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		Some(properties),
		Extensions { relay_chain: "infra-relay-local".into(), para_id: URAUTH_PARACHAIN_ID },
	)
}

// Not used for syncing, but just to determine the genesis values set for the upgrade from shell.
pub fn urauth_config() -> URAuthChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("ss58Format".into(), 0.into());
	properties.insert("tokenSymbol".into(), "".into());
	properties.insert("tokenDecimals".into(), 10.into());

	URAuthChainSpec::from_genesis(
		// Name
		"InfraBlockchain URAuth Main",
		// ID
		"urauth-infra",
		ChainType::Live,
		move || {
			urauth_genesis(
				// initial collators.
				vec![
					(
						hex!("4c3d674d2a01060f0ded218e5dcc6f90c1726f43df79885eb3e22d97a20d5421")
							.into(),
						hex!("4c3d674d2a01060f0ded218e5dcc6f90c1726f43df79885eb3e22d97a20d5421")
							.unchecked_into(),
					),
					(
						hex!("c7d7d38d16bc23c6321152c50306212dc22c0efc04a2e52b5cccfc31ab3d7811")
							.into(),
						hex!("c7d7d38d16bc23c6321152c50306212dc22c0efc04a2e52b5cccfc31ab3d7811")
							.unchecked_into(),
					),
					(
						hex!("c5c07ba203d7375675f5c1ebe70f0a5bb729ae57b48bcc877fcc2ab21309b762")
							.into(),
						hex!("c5c07ba203d7375675f5c1ebe70f0a5bb729ae57b48bcc877fcc2ab21309b762")
							.unchecked_into(),
					),
					(
						hex!("0b2d0013fb974794bd7aa452465b567d48ef70373fe231a637c1fb7c547e85b3")
							.into(),
						hex!("0b2d0013fb974794bd7aa452465b567d48ef70373fe231a637c1fb7c547e85b3")
							.unchecked_into(),
					),
				],
				vec![],
				Default::default(),
				None,
				URAUTH_PARACHAIN_ID.into(),
			)
		},
		vec![],
		None,
		None,
		None,
		Some(properties),
		Extensions { relay_chain: "infra-relay".into(), para_id: URAUTH_PARACHAIN_ID },
	)
}

fn urauth_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	oracle_members: Vec<AccountId>,
	root_key: Option<AccountId>,
	id: ParaId,
) -> urauth_runtime::RuntimeGenesisConfig {
	urauth_runtime::RuntimeGenesisConfig {
		system: urauth_runtime::SystemConfig {
			code: urauth_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			..Default::default()
		},
		balances: urauth_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, URAUTH_INFRA_RELAY_ED * 4096))
				.collect(),
		},
		assets: Default::default(),
		parachain_info: urauth_runtime::ParachainInfoConfig {
			parachain_id: id,
			..Default::default()
		},
		collator_selection: urauth_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: URAUTH_INFRA_RELAY_ED * 16,
			..Default::default()
		},
		session: urauth_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),               // account id
						acc,                       // validator id
						urauth_session_keys(aura), // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		infra_xcm: urauth_runtime::InfraXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		ur_auth: urauth_runtime::URAuthConfig { oracle_members },
		sudo: urauth_runtime::SudoConfig { key: root_key },
	}
}
