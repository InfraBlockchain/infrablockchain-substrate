use super::{types::*, *};
use sp_std::vec;

impl<T: Config> VotingHandler for Pallet<T> {
	fn update_pot_vote(
		who: VoteAccountId,
		system_token_id: SystemTokenId,
		vote_weight: VoteWeight,
	) {
		// Validity Check
		// Check whether it is registered system token
		if !T::SystemTokenInterface::is_system_token(&system_token_id) {
			return
		}
		let weight = T::SystemTokenInterface::adjusted_weight(&system_token_id, vote_weight);
		T::VotingInterface::update_vote_status(who.clone(), weight);
		Self::deposit_event(Event::<T>::Voted { who, system_token_id, vote_weight: weight });
	}
}

impl<T: Config> RuntimeConfigProvider for Pallet<T> {
	type Error = DispatchError;

	fn infra_system_config() -> Result<InfraSystemConfig, Self::Error> {
		Ok(SystemConfig::<T>::get())
	}

	fn para_fee_rate() -> Result<SystemTokenWeight, Self::Error> {
		// Relay chain's fee rate is same as base weight
		Ok(SystemConfig::<T>::get().base_weight())
	}

	fn fee_for(ext: ExtrinsicMetadata) -> Option<SystemTokenBalance> {
		FeeTable::<T>::get(&ext)
	}
	fn runtime_state() -> Mode {
		RuntimeState::<T>::get()
	}
}

// TODO: Find a way to dispatch XCM locally. Then it would be clearer
impl<T: Config> UpdateInfraConfig for Pallet<T> {
	fn update_infra_system_config(
		dest_id: SystemTokenParaId,
		infra_system_config: InfraSystemConfig,
	) {
		if dest_id != RELAY_CHAIN_PARA_ID {
			let set_base_config_call = ParachainRuntimePallets::InfraParaCore(
				ParachainConfigCalls::UpdateInfraSystemConfig(infra_system_config),
			);
			Self::send_xcm_for(set_base_config_call.encode(), dest_id);
		}
	}

	fn update_fee_table(
		dest_id: SystemTokenParaId,
		pallet_name: Vec<u8>,
		call_name: Vec<u8>,
		fee: SystemTokenBalance,
	) {
		if dest_id != RELAY_CHAIN_PARA_ID {
			let set_fee_table_call = ParachainRuntimePallets::InfraParaCore(
				ParachainConfigCalls::SetFeeTable(pallet_name, call_name, fee),
			);
			Self::send_xcm_for(set_fee_table_call.encode(), dest_id);
		}
	}

	fn update_para_fee_rate(dest_id: SystemTokenParaId, fee_rate: SystemTokenWeight) {
		if dest_id != RELAY_CHAIN_PARA_ID {
			let set_fee_rate_call = ParachainRuntimePallets::InfraParaCore(
				ParachainConfigCalls::SetParaFeeRate(fee_rate),
			);
			Self::send_xcm_for(set_fee_rate_call.encode(), dest_id);
		}
	}

	fn update_runtime_state(dest_id: SystemTokenParaId) {
		if dest_id == RELAY_CHAIN_PARA_ID {
			// Do something locally
		} else {
			let set_runtime_state_call =
				ParachainRuntimePallets::InfraParaCore(ParachainConfigCalls::SetRuntimeState);
			Self::send_xcm_for(set_runtime_state_call.encode(), dest_id)
		}
	}

	fn update_system_token_weight(
		dest_id: SystemTokenParaId,
		asset_id: SystemTokenAssetId,
		system_token_weight: SystemTokenWeight,
	) {
		if dest_id == RELAY_CHAIN_PARA_ID {
			// TODO: Error handling
			let _ = T::LocalAssetManager::update_system_token_weight(asset_id, system_token_weight);
		} else {
			let update_system_token_weight_call = ParachainRuntimePallets::InfraParaCore(
				ParachainConfigCalls::UpdateSystemTokenWeight(asset_id, system_token_weight),
			);
			Self::send_xcm_for(update_system_token_weight_call.encode(), dest_id);
		}
	}

	fn register_system_token(
		dest_id: SystemTokenParaId,
		asset_id: SystemTokenAssetId,
		system_token_weight: SystemTokenWeight,
	) {
		if dest_id == RELAY_CHAIN_PARA_ID {
			// TODO: Error handling
			let _ = T::LocalAssetManager::promote(asset_id, system_token_weight);
		} else {
			let register_call = ParachainRuntimePallets::InfraParaCore(
				ParachainConfigCalls::RegisterSystemToken(asset_id, system_token_weight),
			);
			Self::send_xcm_for(register_call.encode(), dest_id);
		}
	}

	fn create_wrapped_local(
		dest_id: SystemTokenParaId,
		asset_id: SystemTokenAssetId,
		currency_type: Fiat,
		min_balance: SystemTokenBalance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: SystemTokenWeight,
		asset_link_parent: u8,
		original: SystemTokenId,
	) {
		if dest_id == RELAY_CHAIN_PARA_ID {
			// TODO: Error handling
			let _ = T::LocalAssetManager::create_wrapped_local(
				asset_id,
				currency_type,
				min_balance,
				name,
				symbol,
				decimals,
				system_token_weight,
			);
			let _ = T::AssetLink::link(&asset_id, asset_link_parent, original);
		} else {
			let create_call =
				ParachainRuntimePallets::InfraParaCore(ParachainConfigCalls::CreateWrappedLocal(
					asset_id,
					currency_type,
					min_balance,
					name,
					symbol,
					decimals,
					system_token_weight,
					asset_link_parent,
					original,
				));
			Self::send_xcm_for(create_call.encode(), dest_id);
		}
	}

	fn deregister_system_token(
		dest_id: SystemTokenParaId,
		asset_id: SystemTokenAssetId,
		is_unlink: bool,
	) {
		if dest_id == RELAY_CHAIN_PARA_ID {
			// TODO: Error handling
			let _ = T::LocalAssetManager::demote(asset_id);
		} else {
			let deregister_call = ParachainRuntimePallets::InfraParaCore(
				ParachainConfigCalls::DeregisterSystemToken(asset_id, is_unlink),
			);
			Self::send_xcm_for(deregister_call.encode(), dest_id);
		}
	}
}

impl<T: Config> Pallet<T> {
	pub(super) fn send_xcm_for(call: Vec<u8>, dest_id: u32) {
		let message = Xcm(vec![
			Instruction::UnpaidExecution {
				weight_limit: WeightLimit::Unlimited,
				check_origin: None,
			},
			Instruction::Transact {
				origin_kind: OriginKind::Native,
				require_weight_at_most: Weight::from_parts(1_000_000_000, 200000),
				call: call.into(),
			},
		]);

		match send_xcm::<T::XcmRouter>(
			MultiLocation::new(0, X1(Parachain(dest_id))),
			message.clone(),
		) {
			Ok(_) => log::info!(
				target: "runtime::parachain-config",
				"Instruction sent successfully."
			),
			Err(e) => log::error!(
				target: "runtime::parachain-config",
				"Error on sending XCM to parachain {:?} => {:?}",
				dest_id, e
			),
		}
	}
}
