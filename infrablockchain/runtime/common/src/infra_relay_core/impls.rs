use core::f64::consts::E;

use super::{types::*, *};
use sp_std::vec;

impl<T: Config> TaaV for Pallet<T> {

	type AccountId = T::AccountId;
	type VoteWeight = T::VoteWeight;
	type Error = DispatchError;

	fn process_vote(bytes: &mut Vec<u8>) -> Result<(), Self::Error> {
		// Try decode
		let vote = PotVote::<Self::AccountId, SystemTokenAssetIdOf<T>, Self::VoteWeight>::decode(&mut bytes[..]).map_err(|_| Error::<T>::ErrorDecode)?;
		log::info!("😇😇😇 Vote: {:?}", vote);	
		// Validity Check
		// Check whether it is registered system token
		// if !T::SystemTokenInterface::is_system_token(&system_token_id) {
		// 	return Ok(())
		// }
		// let weight = T::SystemTokenInterface::adjusted_weight(&system_token_id, vote_weight);
		// T::VotingInterface::update_vote_status(who.clone(), weight);
		// Self::deposit_event(Event::<T>::Voted { who, system_token_id, vote_weight: weight });
		Ok(())
	}
}

impl<T: Config, SystemTokenBalance, SystemTokenWeight> RuntimeConfigProvider<SystemTokenBalance, SystemTokenWeight> for Pallet<T> {
	type Error = DispatchError;

	fn system_token_config() -> Result<SystemTokenConfig, Self::Error> {
		Ok(ActiveSystemConfig::<T>::get())
	}

	fn para_fee_rate() -> Result<SystemTokenWeight, Self::Error> {
		// Relay chain's fee rate is same as base weight
		Ok(ActiveSystemConfig::<T>::get().base_weight())
	}

	fn fee_for(ext: ExtrinsicMetadata) -> Option<SystemTokenBalance> {
		FeeTable::<T>::get(&ext)
	}
	fn runtime_state() -> Mode {
		RuntimeState::<T>::get()
	}
}

// TODO: Find a way to dispatch XCM locally. Then it would be clearer
impl<T: Config, Location, OriginId, Weight, Balance> UpdateInfraConfig<Location> for Pallet<T> 
where
	Location: Encode,
	OriginId: Encode,
	Weight: Encode,
	Balance: Encode
{
	fn update_fee_table(
		dest_id: Option<OriginId>,
		pallet_name: Vec<u8>,
		call_name: Vec<u8>,
		fee: Self::Balance,
	) {
		if let Some(dest_id) = dest_id {
			let set_fee_table_call = ParachainRuntimePallets::InfraParaCore(
				ParachainConfigCalls::UpdateFeeTable(pallet_name, call_name, fee),
			);
			Self::send_xcm_for(set_fee_table_call.encode(), dest_id);
		}
	}

	fn update_para_fee_rate(dest_id: Option<OriginId>, fee_rate: Weight) {
		if let Some(dest_id) = dest_id {
			let set_fee_rate_call = ParachainRuntimePallets::InfraParaCore(
				ParachainConfigCalls::UpdateParaFeeRate(fee_rate),
			);
			Self::send_xcm_for(set_fee_rate_call.encode(), dest_id);
		}
	}

	fn update_runtime_state(dest_id: Option<OriginId>) {
		if let Some(dest_id) = dest_id {
			let set_runtime_state_call =
				ParachainRuntimePallets::InfraParaCore(ParachainConfigCalls::UpdateRuntimeState);
			Self::send_xcm_for(set_runtime_state_call.encode(), dest_id)
		} else {
			Self::do_update_runtime_state();
		}
	}

	fn update_system_token_weight(
		asset_id: Location,
		system_token_weight: Self::SystemTokenWeight,
	) {
		// TODO
		unimplemented!("impl me!")
	}

	fn register_system_token(
		dest_id: Option<OriginId>,
		asset_id: Location,
		system_token_weight: Weight,
	) {
		if let Some(dest_id) = dest_id {
			let register_call = ParachainRuntimePallets::InfraParaCore(
				ParachainConfigCalls::RegisterSystemToken(asset_id, system_token_weight),
			);
			Self::send_xcm_for(register_call.encode(), dest_id);
		} else {
			// TODO: Error handling
			let _ = T::LocalAssetManager::promote(asset_id, system_token_weight);
		}
	}

	fn create_wrapped_local(
		dest_id: Option<OriginId>,
		original: Location,
		currency_type: Fiat,
		min_balance: Self::Balance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: Weight,
	) {
		if let Some(dest_id) = dest_id {
			let create_call =
			ParachainRuntimePallets::InfraParaCore(ParachainConfigCalls::CreateWrappedLocal(
				original,
				currency_type,
				min_balance,
				name,
				symbol,
				decimals,
				system_token_weight
			));
			Self::send_xcm_for(create_call.encode(), dest_id);
		} else {
			// TODO: Error handling
			// let _ = T::LocalAssetManager::create_wrapped_local(
			// 	asset_id,
			// 	currency_type,
			// 	min_balance,
			// 	name,
			// 	symbol,
			// 	decimals,
			// 	system_token_weight,
			// );
			// let _ = T::AssetLink::link(&asset_id, asset_link_parent, original);
			unimplemented!("impl me!")
		}
	}

	fn deregister_system_token(
		dest_id: Option<OriginId>,
		asset_id: Location,
		is_unlink: bool,
	) {
		if let Some(dest_id) = dest_id {
			let deregister_call = ParachainRuntimePallets::InfraParaCore(
				ParachainConfigCalls::DeregisterSystemToken(asset_id, is_unlink),
			);
			Self::send_xcm_for(deregister_call.encode(), dest_id);
		} else {
			// TODO: Error handling
			let _ = T::LocalAssetManager::demote(asset_id);
		}
	}
}

impl<T: Config, OriginId> Pallet<T> 
where
	OriginId: Encode
{
	pub(super) fn send_xcm_for(call: Vec<u8>, dest_id: OriginId) {
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
