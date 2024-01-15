
use xcm::latest::prelude::*;

use crate::*;

/// A type containing the information of Configuration pallet in parachain runtime. 
/// Used to construct remote calls. Index must correspond to the index of the pallet in the runtime.
/// Otherwise, it would fail.
#[derive(Encode, Decode)]
pub enum ParachainRuntimePallets {
    #[codec(index = 2)]
    ParachainConfig(ParachainConfigCalls)
}

#[derive(Encode, Decode)]
pub enum ParachainConfigCalls {
    #[codec(index = 0)]
    SetBaseWeight,
    #[codec(index = 1)]
    SetFeeTable(Vec<u8>, Vec<u8>, SystemTokenBalance),
    #[codec(index = 2)]
    SetFeeRate(SystemTokenWeight),
    #[codec(index = 3)]
    SetRuntimeState,
    #[codec(index = 4)]
    SetSystemTokenWeight(SystemTokenAssetId, SystemTokenWeight),
    #[codec(index = 5)]
    Register(SystemTokenAssetId, SystemTokenWeight),
    #[codec(index = 6)]
    Create(SystemTokenAssetId, SystemTokenWeight),
    #[codec(index = 7)]
    Deregister(SystemTokenAssetId),
}

impl InfraConfigInterface for infra_core::pallet::Pallet<Runtime> {
    fn set_base_weight(para_id: SystemTokenParaId) {
        use crate::para_config::ParachainConfigCalls::SetBaseWeight;
        let set_base_weight_call = ParachainRuntimePallets::ParachainConfig(SetBaseWeight);
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

        match XcmPallet::send_xcm(Here, MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
            Ok(_) => log::info!(
                target: "runtime::parachain-config",
                "Instruction to `set base weight` sent successfully."
            ),
            Err(e) => log::error!(
                target: "runtime::parachain-config",
                "Instruction to `set base weight` failed to send: {:?}",
                e
            ),
        }
    }

    fn set_fee_table(para_id: SystemTokenParaId, pallet_name: Vec<u8>, call_name: Vec<u8>, fee: SystemTokenBalance) {
        use crate::para_config::ParachainConfigCalls::SetFeeTable;
        let set_fee_table_call = ParachainRuntimePallets::ParachainConfig(SetFeeTable(pallet_name, call_name, fee));
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

        match XcmPallet::send_xcm(Here, MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
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
        use crate::para_config::ParachainConfigCalls::SetFeeRate;
        let set_fee_rate_call = ParachainRuntimePallets::ParachainConfig(SetFeeRate(fee_rate));
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

        match XcmPallet::send_xcm(Here, MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
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
        use crate::para_config::ParachainConfigCalls::SetRuntimeState;
        let set_runtime_state_call = ParachainRuntimePallets::ParachainConfig(SetRuntimeState);
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

        match XcmPallet::send_xcm(Here, MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
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
        use crate::para_config::ParachainConfigCalls::SetSystemTokenWeight;
        let set_system_token_weight_call = ParachainRuntimePallets::ParachainConfig(SetSystemTokenWeight(asset_id, weight));
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

        match XcmPallet::send_xcm(Here, MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
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
        use crate::para_config::ParachainConfigCalls::Register;
        let register_call = ParachainRuntimePallets::ParachainConfig(Register(asset_id, weight));
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

        match XcmPallet::send_xcm(Here, MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
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
        use crate::para_config::ParachainConfigCalls::Create;
        let create_call = ParachainRuntimePallets::ParachainConfig(Create(asset_id, weight));
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

        match XcmPallet::send_xcm(Here, MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
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
        use crate::para_config::ParachainConfigCalls::Deregister;
        let deregister_call = ParachainRuntimePallets::ParachainConfig(Deregister(asset_id));
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

        match XcmPallet::send_xcm(Here, MultiLocation::new(1, X1(Parachain(para_id))), message.clone()) {
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
