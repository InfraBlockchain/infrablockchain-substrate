#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as DataMarket;
use frame_benchmarking::v1::{account, benchmarks, whitelisted_caller};
use frame_support::{assert_ok, BoundedVec};
use frame_system::RawOrigin;
use sp_core::bounded_vec;
use sp_std::prelude::*;

fn make_default_delegate_contract<T: Config>(
	detail: DataDelegateContractDetail<T::AccountId, BlockNumberFor<T>, AnyText>,
) where
	<T as pallet_assets::Config>::Balance: From<u128> + Into<u128>,
	<T as pallet_assets::Config>::AssetIdParameter: From<u32>,
{
	assert_ok!(DataMarket::<T>::make_delegate_contract(
		RawOrigin::Signed(detail.clone().agency).into(),
		detail
	));
}

fn make_default_purchase_contract<T: Config>(
	detail: DataPurchaseContractDetail<
		T::AccountId,
		BlockNumberFor<T>,
		<T as pallet_assets::Config>::Balance,
		AnyText,
	>,
	is_agency_exist: bool,
) where
	<T as pallet_assets::Config>::Balance: From<u128> + Into<u128>,
	<T as pallet_assets::Config>::AssetIdParameter: From<u32>,
{
	assert_ok!(DataMarket::<T>::make_purchase_contract(
		RawOrigin::Signed(detail.clone().data_buyer).into(),
		detail,
		is_agency_exist
	));
}

fn sign_default_purchase_contract<T: Config>(
	agency: T::AccountId,
	contract_id: ContractId,
	data_verifier: T::AccountId,
) where
	<T as pallet_assets::Config>::Balance: From<u128> + Into<u128>,
	<T as pallet_assets::Config>::AssetIdParameter: From<u32>,
{
	assert_ok!(DataMarket::<T>::sign_purchase_contract(
		RawOrigin::Signed(agency).into(),
		contract_id,
		data_verifier
	));
}

benchmarks! {
	where_clause { where
		<T as pallet_assets::Config>::Balance: From<u128> + Into<u128>,
		<T as pallet_assets::Config>::AssetIdParameter: From<u32>,
	}

	make_delegate_contract {
		let contract_id = 0;
		let data_owner: T::AccountId = account("data_owner", 0, 0);
		let agency: T::AccountId = account("agency", 0, 0);
		let effective_at: BlockNumberFor<T> = 0u32.into();
		let expired_at: BlockNumberFor<T> = 20u32.into();
		let detail = DataDelegateContractDetail::<T::AccountId, BlockNumberFor<T>, AnyText> {
			data_owner: data_owner.clone(),
			data_owner_info: bounded_vec![1],
			agency: agency.clone(),
			agency_info: bounded_vec![1],
			data_owner_minimum_fee_ratio: 0,
			deligated_data: bounded_vec![1],
			effective_at,
			expired_at
		};
	}: _(RawOrigin::Signed(agency.clone()), detail.clone())
	verify {
		let contract_status =
		vec![(agency.clone(), SignStatus::Signed), (data_owner.clone(), SignStatus::Unsigned)];

		assert_eq!(DataDelegateContracts::<T>::get(contract_id), Some(detail.clone()));
		assert_eq!(ContractStatus::<T>::get(contract_id), Some(contract_status.clone()));
		assert_eq!(NextContractId::<T>::get(), contract_id + 1);
	}

	sign_delegate_contract {
		let contract_id = 0;
		let data_owner: T::AccountId = account("data_owner", 0, 0);
		let agency: T::AccountId = account("agency", 0, 0);
		let effective_at: BlockNumberFor<T> = 0u32.into();
		let expired_at: BlockNumberFor<T> = 20u32.into();
		let detail = DataDelegateContractDetail::<T::AccountId, BlockNumberFor<T>, AnyText> {
			data_owner: data_owner.clone(),
			data_owner_info: bounded_vec![1],
			agency: agency.clone(),
			agency_info: bounded_vec![1],
			data_owner_minimum_fee_ratio: 0,
			deligated_data: bounded_vec![1],
			effective_at,
			expired_at,
		};
		make_default_delegate_contract::<T>(detail);

	}: _(RawOrigin::Signed(data_owner.clone()), contract_id)
	verify {
		let contract_status =
		vec![(agency.clone(), SignStatus::Signed), (data_owner.clone(), SignStatus::Signed)];

		assert_eq!(ContractStatus::<T>::get(contract_id), Some(contract_status.clone()));
	}

	make_purchase_contract {
		let contract_id = 0;
		let data_buyer: T::AccountId = account("data_buyer", 0, 0);
		let agency: T::AccountId = account("agency", 0, 0);
		let is_agency_exist = true;
		let effective_at: BlockNumberFor<T> = 0u32.into();
		let expired_at: BlockNumberFor<T> = 20u32.into();
		let detail = DataPurchaseContractDetail::<
			T::AccountId,
			BlockNumberFor<T>,
			<T as pallet_assets::Config>::Balance,
			AnyText,
		> {
			data_buyer: data_buyer.clone(),
			data_buyer_info: bounded_vec![1],
			effective_at,
			expired_at,
			data_purchase_info: DataPurchaseInfo::<AnyText>::new(bounded_vec![1], bounded_vec![1]),
			system_token_id: 1,
			agency: Some(agency.clone()),
			agency_info: Some(bounded_vec![1]),
			data_verifier: None,
			deposit: 100000.into(),
		};
	}: _(RawOrigin::Signed(data_buyer.clone()), detail.clone(), is_agency_exist)
	verify {
		let contract_status =
		vec![(data_buyer.clone(), SignStatus::Signed), (agency.clone(), SignStatus::Unsigned)];

		assert_eq!(DataPurchaseContracts::<T>::get(contract_id), Some(detail.clone()));
		assert_eq!(ContractStatus::<T>::get(contract_id), Some(contract_status.clone()));
		assert_eq!(NextContractId::<T>::get(), contract_id + 1);
	}

	sign_purchase_contract {
		let contract_id = 0;
		let data_buyer: T::AccountId = account("data_buyer", 0, 0);
		let agency: T::AccountId = account("agency", 0, 0);
		let data_verifier: T::AccountId = account("data_verifier", 0, 0);
		let is_agency_exist = true;
		let effective_at: BlockNumberFor<T> = 0u32.into();
		let expired_at: BlockNumberFor<T> = 20u32.into();
		let detail = DataPurchaseContractDetail::<
			T::AccountId,
			BlockNumberFor<T>,
			<T as pallet_assets::Config>::Balance,
			AnyText,
		> {
			data_buyer: data_buyer.clone(),
			data_buyer_info: bounded_vec![1],
			effective_at,
			expired_at,
			data_purchase_info: DataPurchaseInfo::<AnyText>::new(bounded_vec![1], bounded_vec![1]),
			system_token_id: 1,
			agency: Some(agency.clone()),
			agency_info: Some(bounded_vec![1]),
			data_verifier: None,
			deposit: 100000.into(),
		};
		make_default_purchase_contract::<T>(detail, is_agency_exist);

	}: _(RawOrigin::Signed(agency.clone()), contract_id, data_verifier)
	verify {
		let contract_status =
		vec![(data_buyer.clone(), SignStatus::Signed), (agency.clone(), SignStatus::Signed)];

		assert_eq!(ContractStatus::<T>::get(contract_id), Some(contract_status.clone()));
	}

	terminate_delegate_contract {
		let contract_id = 0;
		let data_owner: T::AccountId = account("data_owner", 0, 0);
		let agency: T::AccountId = account("agency", 0, 0);
		let effective_at: BlockNumberFor<T> = 0u32.into();
		let expired_at: BlockNumberFor<T> = 20u32.into();
		let detail = DataDelegateContractDetail::<T::AccountId, BlockNumberFor<T>, AnyText> {
			data_owner: data_owner.clone(),
			data_owner_info: bounded_vec![1],
			agency: agency.clone(),
			agency_info: bounded_vec![1],
			data_owner_minimum_fee_ratio: 0,
			deligated_data: bounded_vec![1],
			effective_at,
			expired_at,
		};
		make_default_delegate_contract::<T>(detail);

	}: _(RawOrigin::Signed(agency), contract_id)
	verify {
		assert!(!DataDelegateContracts::<T>::contains_key(contract_id));
		assert!(!ContractStatus::<T>::contains_key(contract_id));
	}

	terminate_purchase_contract {
		let contract_id = 0;
		let data_buyer: T::AccountId = account("data_buyer", 0, 0);
		let agency: T::AccountId = account("agency", 0, 0);
		let is_agency_exist = true;
		let effective_at: BlockNumberFor<T> = 0u32.into();
		let expired_at: BlockNumberFor<T> = 20u32.into();
		let detail = DataPurchaseContractDetail::<
			T::AccountId,
			BlockNumberFor<T>,
			<T as pallet_assets::Config>::Balance,
			AnyText,
		> {
			data_buyer: data_buyer.clone(),
			data_buyer_info: bounded_vec![1],
			effective_at,
			expired_at,
			data_purchase_info: DataPurchaseInfo::<AnyText>::new(bounded_vec![1], bounded_vec![1]),
			system_token_id: 1,
			agency: Some(agency.clone()),
			agency_info: Some(bounded_vec![1]),
			data_verifier: None,
			deposit: 100000.into(),
		};
		make_default_purchase_contract::<T>(detail, is_agency_exist);

	}: _(RawOrigin::Signed(data_buyer), contract_id)
	verify {
		assert!(!DataPurchaseContracts::<T>::contains_key(contract_id));
		assert!(!ContractStatus::<T>::contains_key(contract_id));
	}

	execute_data_trade {
		let contract_id = 0;
		let data_buyer: T::AccountId = account("data_buyer", 0, 0);
		let agency: T::AccountId = account("agency", 0, 0);
		let data_verifier: T::AccountId = account("data_verifier", 0, 0);
		let is_agency_exist = true;
		let effective_at: BlockNumberFor<T> = 0u32.into();
		let expired_at: BlockNumberFor<T> = 20u32.into();
		let detail = DataPurchaseContractDetail::<
			T::AccountId,
			BlockNumberFor<T>,
			<T as pallet_assets::Config>::Balance,
			AnyText,
		> {
			data_buyer: data_buyer.clone(),
			data_buyer_info: bounded_vec![1],
			effective_at,
			expired_at,
			data_purchase_info: DataPurchaseInfo::<AnyText>::new(bounded_vec![1], bounded_vec![1]),
			system_token_id: 1,
			agency: Some(agency.clone()),
			agency_info: Some(bounded_vec![1]),
			data_verifier: None,
			deposit: 100000.into(),
		};
		make_default_purchase_contract::<T>(detail, is_agency_exist);
		sign_default_purchase_contract::<T>(agency.clone(), contract_id, data_verifier.clone());

		let data_owner: T::AccountId = account("data_owner", 0, 0);
		let data_issuer = vec![(data_owner.clone(), 100)];
		let data_owner_fee_ratio: u32 = 1000;
		let data_issuer_fee_ratio: u32 = 1000;
		let agency_fee_ratio: u32 = 1000;
		let price_per_data: u128 = 1000;
		let data_verification_proof = VerificationProof::<AnyText>::new(bounded_vec![1]);

	}: _(RawOrigin::Signed(data_verifier),
		contract_id.clone(),
		data_owner.clone(),
		data_issuer.clone(),
		data_owner_fee_ratio.clone(),
		data_issuer_fee_ratio.clone(),
		Some(agency.clone()),
		Some(agency_fee_ratio.clone()),
		price_per_data.into(),
		data_verification_proof.clone())
	verify {
		assert_eq!(TradeCountForContract::<T>::get(contract_id), 1);
		assert!(DataTradeRecords::<T>::contains_key(contract_id, &data_owner))
	}

	impl_benchmark_test_suite!(DataMarket, crate::mock::new_test_ext(), crate::mock::Test);
}
