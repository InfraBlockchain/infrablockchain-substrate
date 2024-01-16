
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

	fn base_weight() -> Result<SystemTokenWeight, Self::Error> {
		Ok(T::BaseWeight::get())
	}

	fn fee_rate() -> Result<SystemTokenWeight, Self::Error> {
		Ok(T::BaseWeight::get())
	}
	fn fee_for(ext: ExtrinsicMetadata) -> Option<SystemTokenBalance> {
		FeeTable::<T>::get(&ext)
	}
	fn runtime_state() -> Mode {
		RuntimeState::<T>::get()
	}
}

impl<T: Config> InfraConfigInterface for Pallet<T> {
    fn set_base_weight(para_id: SystemTokenParaId) {
        let set_base_weight_call = ParachainRuntimePallets::ParachainConfig(ParachainConfigCalls::SetBaseWeight);
        let message = Xcm(vec![
            Instruction::UnpaidExecution {
                weight_limit: WeightLimit::Unlimited,
                check_origin: None,
            },
            Instruction::Transact {
                origin_kind: OriginKind::Native,
                require_weight_at_most: Weight::from_parts(1_000_000_000, 200000),
                call: set_base_weight_call.encode().into(),
            },
        ]);

        match send_xcm::<T::XcmRouter>(MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
            Ok(_) => log::info!(
                target: "runtime::parachain-config",
                "Instruction to `set fee table` sent successfully."
            ),
            Err(e) => log::error!(
                target: "runtime::parachain-config",
                "Instruction to `set fee table` failed to send: {:?}",
                e
            ),
        }
    }

    fn set_fee_table(para_id: SystemTokenParaId, pallet_name: Vec<u8>, call_name: Vec<u8>, fee: SystemTokenBalance) {
        let set_fee_table_call = ParachainRuntimePallets::ParachainConfig(ParachainConfigCalls::SetFeeTable(pallet_name, call_name, fee));
        let message = Xcm(vec![
            Instruction::UnpaidExecution {
                weight_limit: WeightLimit::Unlimited,
                check_origin: None,
            },
            Instruction::Transact {
                origin_kind: OriginKind::Native,
                require_weight_at_most: Weight::from_parts(1_000_000_000, 200000),
                call: set_fee_table_call.encode().into(),
            },
        ]);

        match send_xcm::<T::XcmRouter>(MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
            Ok(_) => log::info!(
                target: "runtime::parachain-config",
                "Instruction to `set fee table` sent successfully."
            ),
            Err(e) => log::error!(
                target: "runtime::parachain-config",
                "Instruction to `set fee table` failed to send: {:?}",
                e
            ),
        }
    }

    fn set_fee_rate(para_id: SystemTokenParaId, fee_rate: SystemTokenWeight) {
        let set_fee_rate_call = ParachainRuntimePallets::ParachainConfig(ParachainConfigCalls::SetFeeRate(fee_rate));
        let message = Xcm(vec![
            Instruction::UnpaidExecution {
                weight_limit: WeightLimit::Unlimited,
                check_origin: None,
            },
            Instruction::Transact {
                origin_kind: OriginKind::Native,
                require_weight_at_most: Weight::from_parts(1_000_000_000, 200000),
                call: set_fee_rate_call.encode().into(),
            },
        ]);

        match send_xcm::<T::XcmRouter>(MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
            Ok(_) => log::info!(
                target: "runtime::parachain-config",
                "Instruction to `set fee rate` sent successfully."
            ),
            Err(e) => log::error!(
                target: "runtime::parachain-config",
                "Instruction to `set fee rate` failed to send: {:?}",
                e
            ),
        }
    }

    fn set_runtime_state(para_id: SystemTokenParaId) {
        let set_runtime_state_call = ParachainRuntimePallets::ParachainConfig(ParachainConfigCalls::SetRuntimeState);
        let message = Xcm(vec![
            Instruction::UnpaidExecution {
                weight_limit: WeightLimit::Unlimited,
                check_origin: None,
            },
            Instruction::Transact {
                origin_kind: OriginKind::Native,
                require_weight_at_most: Weight::from_parts(1_000_000_000, 200000),
                call: set_runtime_state_call.encode().into(),
            },
        ]);

        match send_xcm::<T::XcmRouter>(MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
            Ok(_) => log::info!(
                target: "runtime::parachain-config",
                "Instruction to `set runtime state` sent successfully."
            ),
            Err(e) => log::error!(
                target: "runtime::parachain-config",
                "Instruction to `set runtime state` failed to send: {:?}",
                e
            ),
        }
    }

    fn set_system_token_weight(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId, weight: SystemTokenWeight) {
        let set_system_token_weight_call = ParachainRuntimePallets::ParachainConfig(ParachainConfigCalls::SetSystemTokenWeight(asset_id, weight));
        let message = Xcm(vec![
            Instruction::UnpaidExecution {
                weight_limit: WeightLimit::Unlimited,
                check_origin: None,
            },
            Instruction::Transact {
                origin_kind: OriginKind::Native,
                require_weight_at_most: Weight::from_parts(1_000_000_000, 200000),
                call: set_system_token_weight_call.encode().into(),
            },
        ]);

        match send_xcm::<T::XcmRouter>(MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
            Ok(_) => log::info!(
                target: "runtime::parachain-config",
                "Instruction to `set system token weight` sent successfully."
            ),
            Err(e) => log::error!(
                target: "runtime::parachain-config",
                "Instruction to `set system token weight` failed to send: {:?}",
                e
            ),
        }
    }

    fn register_system_token(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId, weight: SystemTokenWeight) {
        let register_call = ParachainRuntimePallets::ParachainConfig(ParachainConfigCalls::Register(asset_id, weight));
        let message = Xcm(vec![
            Instruction::UnpaidExecution {
                weight_limit: WeightLimit::Unlimited,
                check_origin: None,
            },
            Instruction::Transact {
                origin_kind: OriginKind::Native,
                require_weight_at_most: Weight::from_parts(1_000_000_000, 200000),
                call: register_call.encode().into(),
            },
        ]);

        match send_xcm::<T::XcmRouter>(MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
            Ok(_) => log::info!(
                target: "runtime::parachain-config",
                "Instruction to `register system token` sent successfully."
            ),
            Err(e) => log::error!(
                target: "runtime::parachain-config",
                "Instruction to `register system token` failed to send: {:?}",
                e
            ),
        }
    }

    fn create_system_token(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId, weight: SystemTokenWeight) {
        let create_call = ParachainRuntimePallets::ParachainConfig(ParachainConfigCalls::Create(asset_id, weight));
        let message = Xcm(vec![
            Instruction::UnpaidExecution {
                weight_limit: WeightLimit::Unlimited,
                check_origin: None,
            },
            Instruction::Transact {
                origin_kind: OriginKind::Native,
                require_weight_at_most: Weight::from_parts(1_000_000_000, 200000),
                call: create_call.encode().into(),
            },
        ]);

        match send_xcm::<T::XcmRouter>(MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
            Ok(_) => log::info!(
                target: "runtime::parachain-config",
                "Instruction to `create system token` sent successfully."
            ),
            Err(e) => log::error!(
                target: "runtime::parachain-config",
                "Instruction to `create system token` failed to send: {:?}",
                e
            ),
        }
    }

    fn deregister_system_token(para_id: SystemTokenParaId, asset_id: SystemTokenAssetId) {
        let deregister_call = ParachainRuntimePallets::ParachainConfig(ParachainConfigCalls::Deregister(asset_id));
        let message = Xcm(vec![
            Instruction::UnpaidExecution {
                weight_limit: WeightLimit::Unlimited,
                check_origin: None,
            },
            Instruction::Transact {
                origin_kind: OriginKind::Native,
                require_weight_at_most: Weight::from_parts(1_000_000_000, 200000),
                call: deregister_call.encode().into(),
            },
        ]);

        match send_xcm::<T::XcmRouter>(MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
            Ok(_) => log::info!(
                target: "runtime::parachain-config",
                "Instruction to `deregister system token` sent successfully."
            ),
            Err(e) => log::error!(
                target: "runtime::parachain-config",
                "Instruction to `deregister system token` failed to send: {:?}",
                e
            ),
        }
    }
}