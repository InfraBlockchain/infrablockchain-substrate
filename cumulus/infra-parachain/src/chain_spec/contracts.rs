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
use parachains_common::types::{AccountId, AuraId};
use sc_service::ChainType;
use sp_core::{crypto::UncheckedInto, sr25519};

pub type ContractsInfraChainSpec =
	sc_service::GenericChainSpec<contracts_infra_runtime::RuntimeGenesisConfig, Extensions>;

/// No relay chain suffix because the id is the same over all relay chains.
const CONTRACTS_PARACHAIN_ID: u32 = 1001;

/// The existential deposit is determined by the runtime "contracts-infra".
const CONTRACTS_INFRA_ED: contracts_infra_runtime::Balance =
	parachains_common::infra_relay::currency::EXISTENTIAL_DEPOSIT;

pub fn contracts_infra_development_config() -> ContractsInfraChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "INFRA".into());
	properties.insert("tokenDecimals".into(), 12.into());

	ContractsInfraChainSpec::from_genesis(
		// Name
		"InfraBlockchain Contracts Hub Dev",
		// ID
		"contracts-hub-infra-dev",
		ChainType::Development,
		move || {
			contracts_infra_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed::<contracts_infra_runtime::AuraId>("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed::<contracts_infra_runtime::AuraId>("Bob"),
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
				CONTRACTS_PARACHAIN_ID.into(),
			)
		},
		Vec::new(),
		None,
		None,
		None,
		None,
		Extensions {
			relay_chain: "infra-relay-local".into(), // You MUST set this to the correct network!
			para_id: CONTRACTS_PARACHAIN_ID,
		},
	)
}

pub fn contracts_infra_local_config() -> ContractsInfraChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "INFRA".into());
	properties.insert("tokenDecimals".into(), 12.into());

	ContractsInfraChainSpec::from_genesis(
		// Name
		"InfraBlockchain Contracts Hub Local",
		// ID
		"contracts-hub-infra-local",
		ChainType::Local,
		move || {
			contracts_infra_genesis(
				// initial collators.
				vec![
					(
						get_account_id_from_seed::<sr25519::Public>("Alice"),
						get_collator_keys_from_seed::<contracts_infra_runtime::AuraId>("Alice"),
					),
					(
						get_account_id_from_seed::<sr25519::Public>("Bob"),
						get_collator_keys_from_seed::<contracts_infra_runtime::AuraId>("Bob"),
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
				CONTRACTS_PARACHAIN_ID.into(),
			)
		},
		// Bootnodes
		Vec::new(),
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions {
			relay_chain: "infra-relay-local".into(), // You MUST set this to the correct network!
			para_id: CONTRACTS_PARACHAIN_ID,
		},
	)
}

pub fn contracts_infra_config() -> ContractsInfraChainSpec {
	// Give your base currency a unit name and decimal places
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "INFRA".into());
	properties.insert("tokenDecimals".into(), 12.into());

	ContractsInfraChainSpec::from_genesis(
		// Name
		"InfraBlockchain Contracts Hub Main",
		// ID
		"contracts-hub-infra",
		ChainType::Live,
		move || {
			contracts_infra_genesis(
				vec![
					// 5GKFbTTgrVS4Vz1UWWHPqMZQNFWZtqo7H2KpCDyYhEL3aS26
					(
						hex!["bc09354c12c054c8f6b3da208485eacec4ac648bad348895273b37bab5a0937c"]
							.into(),
						hex!["bc09354c12c054c8f6b3da208485eacec4ac648bad348895273b37bab5a0937c"]
							.unchecked_into(),
					),
					// 5EPRJHm2GpABVWcwnAujcrhnrjFZyDGd5TwKFzkBoGgdRyv2
					(
						hex!["66be63b7bcbfb91040e5248e2d1ceb822cf219c57848c5924ffa3a1f8e67ba72"]
							.into(),
						hex!["66be63b7bcbfb91040e5248e2d1ceb822cf219c57848c5924ffa3a1f8e67ba72"]
							.unchecked_into(),
					),
					// 5GH62vrJrVZxLREcHzm2PR5uTLAT5RQMJitoztCGyaP4o3uM
					(
						hex!["ba62886472a0a9f66b5e39f1469ce1c5b3d8cad6be39078daf16f111e89d1e44"]
							.into(),
						hex!["ba62886472a0a9f66b5e39f1469ce1c5b3d8cad6be39078daf16f111e89d1e44"]
							.unchecked_into(),
					),
					// 5FHfoJDLdjRYX5KXLRqMDYBbWrwHLMtti21uK4QByUoUAbJF
					(
						hex!["8e97f65cda001976311df9bed39e8d0c956089093e94a75ef76fe9347a0eda7b"]
							.into(),
						hex!["8e97f65cda001976311df9bed39e8d0c956089093e94a75ef76fe9347a0eda7b"]
							.unchecked_into(),
					),
				],
				// Warning: The configuration for a production chain should not contain
				// any endowed accounts here, otherwise it'll be minting extra native tokens
				// from the relay chain on the parachain.
				vec![
					// NOTE: Remove endowed accounts if deployed on other relay chains.
					// Endowed accounts
					hex!["baa78c7154c7f82d6d377177e20bcab65d327eca0086513f9964f5a0f6bdad56"].into(),
					// AccountId of an account which `ink-waterfall` uses for automated testing
					hex!["0e47e2344d523c3cc5c34394b0d58b9a4200e813a038e6c5a6163cc07d70b069"].into(),
				],
				CONTRACTS_PARACHAIN_ID.into(),
			)
		},
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Fork ID
		None,
		// Properties
		Some(properties),
		// Extensions
		Extensions { relay_chain: "infra-relay".into(), para_id: CONTRACTS_PARACHAIN_ID },
	)
}

fn contracts_infra_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	id: ParaId,
) -> contracts_infra_runtime::RuntimeGenesisConfig {
	contracts_infra_runtime::RuntimeGenesisConfig {
		system: contracts_infra_runtime::SystemConfig {
			code: contracts_infra_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			..Default::default()
		},
		balances: contracts_infra_runtime::BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k| (k, 1 << 60)).collect(),
		},
		parachain_info: contracts_infra_runtime::ParachainInfoConfig {
			parachain_id: id,
			..Default::default()
		},
		collator_selection: contracts_infra_runtime::CollatorSelectionConfig {
			invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			candidacy_bond: CONTRACTS_INFRA_ED * 16,
			..Default::default()
		},
		session: contracts_infra_runtime::SessionConfig {
			keys: invulnerables
				.into_iter()
				.map(|(acc, aura)| {
					(
						acc.clone(),                                   // account id
						acc,                                           // validator id
						contracts_infra_runtime::SessionKeys { aura }, // session keys
					)
				})
				.collect(),
		},
		// no need to pass anything to aura, in fact it will panic if we do. Session will take care
		// of this.
		aura: Default::default(),
		aura_ext: Default::default(),
		parachain_system: Default::default(),
		ibs_xcm: contracts_infra_runtime::IbsXcmConfig {
			safe_xcm_version: Some(SAFE_XCM_VERSION),
			..Default::default()
		},
		assets: contracts_infra_runtime::AssetsConfig {
			assets: vec![(
				99,                                                   // asset_id
				get_account_id_from_seed::<sr25519::Public>("Alice"), // owner
				true,                                                 // is_sufficient
				1000,                                                 // min_balance
			)],
			metadata: vec![(99, "iBOOT".into(), "iBOOT".into(), 2)],
			accounts: vec![(
				99,
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				100_000_000_000, // 1_000_000_000 iTEST
			)],
			..Default::default()
		},
	}
}
