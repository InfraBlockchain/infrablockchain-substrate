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
	get_account_id_from_seed, get_collator_keys_from_seed, Extensions, GenericChainSpec,
	SAFE_XCM_VERSION,
};
use cumulus_primitives_core::ParaId;
use hex_literal::hex;
use parachains_common::{AccountId, AuraId, Balance as AssetHubBalance};
use sc_service::ChainType;
use sp_core::{crypto::UncheckedInto, sr25519};

const ASSET_HUB_INFRA_RELAY_ED: AssetHubBalance = asset_hub_runtime::ExistentialDeposit::get();
/// Specialized `ChainSpec` for the normal parachain runtime.
pub type AssetHubChainSpec = GenericChainSpec;

/// Generate the session keys from individual elements.
///
/// The input must be a tuple of individual keys (a single arg for now since we have just one key).
pub fn asset_hub_session_keys(keys: AuraId) -> asset_hub_runtime::SessionKeys {
	asset_hub_runtime::SessionKeys { aura: keys }
}

pub fn asset_hub_local_config() -> AssetHubChainSpec {
	let mut properties = sc_chain_spec::Properties::new();
	properties.insert("tokenSymbol".into(), "".into());
	properties.insert("tokenDecimals".into(), 10.into());

	AssetHubChainSpec::builder(
		asset_hub_runtime::WASM_BINARY.expect("WASM binary was not built for `InfraAssetHub`"),
		Extensions { relay_chain: "infra-relay".into(), para_id: 1000 },
	)
	.with_name("Infra Asset Hub Development")
	.with_id("asset-hub-infra-dev")
	.with_chain_type(ChainType::Local)
	.with_genesis_config_patch(asset_hub_genesis(
		// initial collators
		vec![(
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_collator_keys_from_seed::<AuraId>("Alice"),
		)], // invulnerables
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		], // endowed_accounts
		testnet_parachains_constants::infra_relay::currency::UNITS * 1_000_000, // endowment
		Some(get_account_id_from_seed::<sr25519::Public>("Alice")),             // root_key
		1000.into(),                                                            // para_id
	))
	.with_properties(properties)
	.build()
}

fn asset_hub_genesis(
	invulnerables: Vec<(AccountId, AuraId)>,
	endowed_accounts: Vec<AccountId>,
	endowment: AssetHubBalance,
	root_key: Option<AccountId>,
	id: ParaId,
) -> serde_json::Value {
	serde_json::json!({
		  "balances": asset_hub_runtime::BalancesConfig {
			  balances: endowed_accounts
				  .iter()
				  .cloned()
				  .map(|k| (k, endowment))
				  .collect(),
		  },
		  "parachainInfo": asset_hub_runtime::ParachainInfoConfig {
			  parachain_id: id,
			  ..Default::default()
		  },
		  "collatorSelection": asset_hub_runtime::CollatorSelectionConfig {
			  invulnerables: invulnerables.iter().cloned().map(|(acc, _)| acc).collect(),
			  candidacy_bond: ASSET_HUB_INFRA_RELAY_ED * 16,
			  ..Default::default()
		  },
		  "session": asset_hub_runtime::SessionConfig {
			  keys: invulnerables
				  .into_iter()
				  .map(|(acc, aura)| {
					  (
						  acc.clone(),                         // account id
						  acc,                                 // validator id
						  asset_hub_session_keys(aura), // session keys
					  )
				  })
				  .collect(),
		  },
		  "infraXcm": asset_hub_runtime::InfraXcmConfig {
	safe_xcm_version: Some(SAFE_XCM_VERSION),
		  ..Default::default()
		  },
		  "sudo": asset_hub_runtime::SudoConfig { key: root_key }
	  })
}
