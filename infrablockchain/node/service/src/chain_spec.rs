// Copyright (C) Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

//! Polkadot chain configurations.

use beefy_primitives::ecdsa_crypto::AuthorityId as BeefyId;
use grandpa::AuthorityId as GrandpaId;
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use primitives::{AccountId, AccountPublic, AssignmentId, ValidatorId};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;

#[cfg(feature = "infra-relay-native")]
use infra_relay_runtime as infra_relay;
#[cfg(feature = "infra-relay-native")]
use infra_relay_runtime_constants::currency::UNITS as UNIT;
use sc_chain_spec::ChainSpecExtension;
#[cfg(any(feature = "infra-relay-native"))]
use sc_chain_spec::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_runtime::traits::IdentifyAccount;
#[cfg(any(feature = "infra-relay-native"))]
use telemetry::TelemetryEndpoints;
// ToDo: Should change
#[cfg(feature = "infra-relay-native")]
const INFRA_RELAY_STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";
#[cfg(feature = "infra-relay-native")]
const DEFAULT_INFRA_PROTOCOL_ID: &str = "infra";

/// Node `ChainSpec` extensions.
///
/// Additional parameters for some Substrate core modules,
/// customizable from the chain spec.
#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<primitives::Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<primitives::Block>,
	/// The light sync state.
	///
	/// This value will be set by the `sync-state rpc` implementation.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

// Generic chain spec, in case when we don't have the native runtime.
pub type GenericChainSpec = service::GenericChainSpec<(), Extensions>;

/// The 'ChainSpec' parameterized for the infra-relay runtime
#[cfg(feature = "infra-relay-native")]
pub type InfraRelayChainSpec = service::GenericChainSpec<InfraRelayGenesisExt, Extensions>;

#[derive(serde::Serialize, serde::Deserialize)]
#[cfg(feature = "infra-relay-native")]
pub struct InfraRelayGenesisExt {
	/// The runtime genesis config.
	runtime_genesis_config: infra_relay::RuntimeGenesisConfig,
	/// The session length in blocks.
	///
	/// If `None` is supplied, the default value is used.
	session_length_in_blocks: Option<u32>,
}

#[cfg(feature = "infra-relay-native")]
impl sp_runtime::BuildStorage for InfraRelayGenesisExt {
	fn assimilate_storage(&self, storage: &mut sp_core::storage::Storage) -> Result<(), String> {
		sp_state_machine::BasicExternalities::execute_with_storage(storage, || {
			if let Some(length) = self.session_length_in_blocks.as_ref() {
				infra_relay_runtime_constants::time::EpochDurationInSlots::set(length);
			}
		});
		self.runtime_genesis_config.assimilate_storage(storage)
	}
}

/// The `ChainSpec` parameterized for the westend runtime.
// Dummy chain spec, but that is fine when we don't have the native runtime.
#[cfg(not(feature = "infra-relay-native"))]
pub type InfraRelayChainSpec = GenericChainSpec;

pub fn infra_relay_config() -> Result<InfraRelayChainSpec, String> {
	// ToDo: Should change
	InfraRelayChainSpec::from_json_bytes(&include_bytes!("../chain-specs/polkadot.json")[..])
}

pub fn rococo_config() -> Result<RococoChainSpec, String> {
	RococoChainSpec::from_json_bytes(&include_bytes!("../chain-specs/rococo.json")[..])
}

/// The default parachains host configuration.
#[cfg(any(feature = "infra-relay-native"))]
fn default_parachains_host_configuration(
) -> runtime_parachains::configuration::HostConfiguration<primitives::BlockNumber> {
	use primitives::{MAX_CODE_SIZE, MAX_POV_SIZE};

	runtime_parachains::configuration::HostConfiguration {
		validation_upgrade_cooldown: 2u32,
		validation_upgrade_delay: 2,
		code_retention_period: 1200,
		max_code_size: MAX_CODE_SIZE,
		max_pov_size: MAX_POV_SIZE,
		max_head_data_size: 32 * 1024,
		group_rotation_frequency: 20,
		paras_availability_period: 4,
		max_upward_queue_count: 8,
		max_upward_queue_size: 1024 * 1024,
		max_downward_message_size: 1024 * 1024,
		max_upward_message_size: 50 * 1024,
		max_upward_message_num_per_candidate: 5,
		hrmp_sender_deposit: 0,
		hrmp_recipient_deposit: 0,
		hrmp_channel_max_capacity: 8,
		hrmp_channel_max_total_size: 8 * 1024,
		hrmp_max_parachain_inbound_channels: 4,
		hrmp_channel_max_message_size: 1024 * 1024,
		hrmp_max_parachain_outbound_channels: 4,
		hrmp_max_message_num_per_candidate: 5,
		dispute_period: 6,
		no_show_slots: 2,
		n_delay_tranches: 25,
		needed_approvals: 2,
		relay_vrf_modulo_samples: 2,
		zeroth_delay_tranche_width: 0,
		minimum_validation_upgrade_delay: 5,
		..Default::default()
	}
}

#[cfg(any(feature = "infra-relay-native"))]
fn default_infra_relay_system_configuration() -> primitives::InfraSystemConfig {
	use sp_runtime::types::{BaseSystemTokenDetail, Fiat};

	primitives::InfraSystemConfig {
		base_system_token_detail: BaseSystemTokenDetail {
			base_currency: Fiat::USD,
			base_weight: 1_000_000,
			base_decimals: 4,
		},
		weight_scale: 25,
	}
}

#[cfg(any(feature = "infra-relay-native"))]
#[test]
fn default_parachains_host_configuration_is_consistent() {
	default_parachains_host_configuration().panic_if_not_consistent();
}

#[cfg(feature = "infra-relay-native")]
fn infra_relay_session_keys(
	babe: BabeId,
	grandpa: GrandpaId,
	im_online: ImOnlineId,
	para_validator: ValidatorId,
	para_assignment: AssignmentId,
	authority_discovery: AuthorityDiscoveryId,
) -> infra_relay::SessionKeys {
	infra_relay::SessionKeys {
		babe,
		grandpa,
		im_online,
		para_validator,
		para_assignment,
		authority_discovery,
	}
}

#[cfg(feature = "infra-relay-native")]
fn infra_relay_staging_testnet_config_genesis(
	wasm_binary: &[u8],
) -> infra_relay::RuntimeGenesisConfig {
	// subkey inspect "$SECRET"
	let endowed_accounts = vec![];

	let initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId,
	)> = vec![];

	const ENDOWMENT: u128 = 1_000_000 * UNIT;
	const STASH: u128 = 100 * UNIT;

	#[cfg(feature = "fast-runtime")]
	let root_key = get_account_id_from_seed::<sr25519::Public>("Alice");

	infra_relay::RuntimeGenesisConfig {
		system: infra_relay::SystemConfig { code: wasm_binary.to_vec(), ..Default::default() },
		balances: infra_relay::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.map(|k: &AccountId| (k.clone(), ENDOWMENT))
				.chain(initial_authorities.iter().map(|x| (x.0.clone(), STASH)))
				.collect(),
		},
		assets: Default::default(),
		indices: infra_relay::IndicesConfig { indices: vec![] },
		session: infra_relay::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						infra_relay_session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
							x.7.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		#[cfg(feature = "fast-runtime")]
		sudo: infra_relay::SudoConfig { key: Some(root_key) },
		phragmen_election: Default::default(),
		democracy: Default::default(),
		council: infra_relay::CouncilConfig {
			members: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
			phantom: Default::default(),
		},
		technical_committee: infra_relay::TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		},
		technical_membership: Default::default(),
		babe: infra_relay::BabeConfig {
			authorities: Default::default(),
			epoch_config: Some(infra_relay::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		grandpa: Default::default(),
		im_online: Default::default(),
		authority_discovery: infra_relay::AuthorityDiscoveryConfig {
			keys: vec![],
			..Default::default()
		},
		treasury: Default::default(),
		hrmp: Default::default(),
		configuration: infra_relay::ConfigurationConfig {
			config: default_parachains_host_configuration(),
		},
		paras: Default::default(),
		xcm_pallet: Default::default(),
		validator_election: infra_relay::ValidatorElectionConfig {
			seed_trust_validators: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
			total_validator_slots: 2,
			seed_trust_slots: 2,
			..Default::default()
		},
		infra_relay_core: infra_relay::InfraRelayCoreConfig {
			system_config: default_infra_relay_system_configuration(),
			_phantom: Default::default(),
		},
		..Default::default()
	}
}

#[cfg(feature = "infra-relay-native")]
pub fn infra_relay_staging_testnet_config() -> Result<InfraRelayChainSpec, String> {
	let wasm_binary =
		infra_relay::WASM_BINARY.ok_or("InfraRelayChain-staging wasm not available")?;
	let boot_nodes = vec![];

	Ok(InfraRelayChainSpec::from_genesis(
		"InfraRelayChain-Staging",
		"infra_relay_chain_staging",
		ChainType::Live,
		move || InfraRelayGenesisExt {
			runtime_genesis_config: infra_relay_staging_testnet_config_genesis(wasm_binary),
			session_length_in_blocks: None,
		},
		boot_nodes,
		Some(
			TelemetryEndpoints::new(vec![(INFRA_RELAY_STAGING_TELEMETRY_URL.to_string(), 0)])
				.expect("InfraRelayChain-Staging telemetry url is valid; qed"),
		),
		Some(DEFAULT_INFRA_PROTOCOL_ID),
		None,
		None,
		Default::default(),
	))
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed(
	seed: &str,
) -> (
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	ValidatorId,
	AssignmentId,
	AuthorityDiscoveryId,
	BeefyId,
) {
	let keys = get_authority_keys_from_seed_no_beefy(seed);
	(keys.0, keys.1, keys.2, keys.3, keys.4, keys.5, keys.6, keys.7, get_from_seed::<BeefyId>(seed))
}

/// Helper function to generate stash, controller and session key from seed
pub fn get_authority_keys_from_seed_no_beefy(
	seed: &str,
) -> (
	AccountId,
	AccountId,
	BabeId,
	GrandpaId,
	ImOnlineId,
	ValidatorId,
	AssignmentId,
	AuthorityDiscoveryId,
) {
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<ValidatorId>(seed),
		get_from_seed::<AssignmentId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
	)
}

#[cfg(any(feature = "infra-relay-native"))]
fn testnet_accounts() -> Vec<AccountId> {
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
	]
}

/// Helper function to create infra-relay `RuntimeGenesisConfig` for testing
#[cfg(feature = "infra-relay-native")]
pub fn infra_relay_testnet_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		BabeId,
		GrandpaId,
		ImOnlineId,
		ValidatorId,
		AssignmentId,
		AuthorityDiscoveryId,
	)>,
	#[allow(unused_variables)] root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
) -> infra_relay::RuntimeGenesisConfig {
	let endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(testnet_accounts);

	const ENDOWMENT: u128 = 1_000_000 * UNIT;

	infra_relay::RuntimeGenesisConfig {
		system: infra_relay::SystemConfig { code: wasm_binary.to_vec(), ..Default::default() },
		indices: infra_relay::IndicesConfig { indices: vec![] },
		balances: infra_relay::BalancesConfig {
			balances: endowed_accounts.iter().map(|k| (k.clone(), ENDOWMENT)).collect(),
		},
		assets: Default::default(),
		session: infra_relay::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						infra_relay_session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
							x.7.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		#[cfg(feature = "fast-runtime")]
		sudo: infra_relay::SudoConfig { key: Some(root_key) },
		phragmen_election: Default::default(),
		democracy: infra_relay::DemocracyConfig::default(),
		council: infra_relay::CouncilConfig { members: vec![], phantom: Default::default() },
		technical_committee: infra_relay::TechnicalCommitteeConfig {
			members: vec![],
			phantom: Default::default(),
		},
		technical_membership: Default::default(),
		babe: infra_relay::BabeConfig {
			authorities: Default::default(),
			epoch_config: Some(infra_relay::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		grandpa: Default::default(),
		im_online: Default::default(),
		authority_discovery: infra_relay::AuthorityDiscoveryConfig {
			keys: vec![],
			..Default::default()
		},
		treasury: Default::default(),
		hrmp: Default::default(),
		configuration: infra_relay::ConfigurationConfig {
			config: default_parachains_host_configuration(),
		},
		paras: Default::default(),
		xcm_pallet: Default::default(),
		validator_election: infra_relay::ValidatorElectionConfig {
			seed_trust_validators: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
			total_validator_slots: 2,
			seed_trust_slots: 2,
			..Default::default()
		},
		infra_relay_core: infra_relay::InfraRelayCoreConfig {
			system_config: default_infra_relay_system_configuration(),
			_phantom: Default::default(),
		},
		..Default::default()
	}
}

#[cfg(feature = "infra-relay-native")]
fn infra_relay_development_config_genesis(wasm_binary: &[u8]) -> infra_relay::RuntimeGenesisConfig {
	infra_relay_testnet_genesis(
		wasm_binary,
		vec![
			get_authority_keys_from_seed_no_beefy("Alice"),
			get_authority_keys_from_seed_no_beefy("Bob"),
			get_authority_keys_from_seed_no_beefy("Charlie"),
			get_authority_keys_from_seed_no_beefy("Dave"),
			get_authority_keys_from_seed_no_beefy("Eve"),
			get_authority_keys_from_seed_no_beefy("Ferdie"),
		],
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		None,
	)
}

/// Returns the properties for the [`InfraBlockspaceChainSpec`].
pub fn infra_relay_chain_spec_properties() -> serde_json::map::Map<String, serde_json::Value> {
	serde_json::json!({
		"tokenDecimals": 4,
	})
	.as_object()
	.expect("Map given; qed")
	.clone()
}

/// Infra Relay development config
#[cfg(feature = "infra-relay-native")]
pub fn infra_relay_development_config() -> Result<InfraRelayChainSpec, String> {
	let wasm_binary = infra_relay::WASM_BINARY.ok_or("InfraRelayChain-Dev wasm not available")?;

	Ok(InfraRelayChainSpec::from_genesis(
		"InfraRelayChain-Dev",
		"infra_relay_chain-dev",
		ChainType::Development,
		move || InfraRelayGenesisExt {
			runtime_genesis_config: infra_relay_development_config_genesis(wasm_binary),
			session_length_in_blocks: Some(10),
		},
		vec![],
		None,
		Some(DEFAULT_INFRA_PROTOCOL_ID),
		None,
		None,
		Default::default(),
	))
}
