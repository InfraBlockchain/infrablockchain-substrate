use super::*;
use crate::mock::*;
use frame_support::assert_ok;
use sp_core::bounded_vec;

fn make_test_delegate_contract(data_owner: u64, agency: u64) {
	let detail = DataDelegateContractDetail {
		data_owner,
		data_owner_info: bounded_vec![1],
		agency: agency.clone(),
		agency_info: bounded_vec![1],
		data_owner_minimum_fee_ratio: 0,
		deligated_data: bounded_vec![1],
		effective_at: 0,
		expired_at: 20,
	};
	assert_ok!(DataMarket::make_delegate_contract(RuntimeOrigin::signed(agency.clone()), detail));
}

fn make_test_purchase_contract_without_agency(data_buyer: u64, data_verifier: u64) {
	let is_agency_exist = false;
	let detail = DataPurchaseContractDetail {
		data_buyer: data_buyer.clone(),
		data_buyer_info: bounded_vec![1],
		effective_at: 0,
		expired_at: 20,
		data_purchase_info: DataPurchaseInfo::<AnyText>::new(bounded_vec![1], bounded_vec![1]),
		system_token_id: 1,
		agency: None,
		agency_info: None,
		data_verifier: Some(data_verifier.clone()),
		deposit: 100000,
	};
	assert_ok!(DataMarket::make_purchase_contract(
		RuntimeOrigin::signed(data_buyer.clone()),
		detail,
		is_agency_exist
	));
}

fn make_test_purchase_contract_with_agency(data_buyer: u64, agency: u64) {
	let is_agency_exist = true;
	let detail = DataPurchaseContractDetail {
		data_buyer: data_buyer.clone(),
		data_buyer_info: bounded_vec![1],
		effective_at: 0,
		expired_at: 20,
		data_purchase_info: DataPurchaseInfo::<AnyText>::new(bounded_vec![1], bounded_vec![1]),
		system_token_id: 1,
		agency: Some(agency.clone()),
		agency_info: Some(bounded_vec![1]),
		data_verifier: None,
		deposit: 100000,
	};
	assert_ok!(DataMarket::make_purchase_contract(
		RuntimeOrigin::signed(data_buyer.clone()),
		detail,
		is_agency_exist
	));
}

fn sign_test_delegate_contract(data_owner: u64, contract_id: u128) {
	assert_ok!(DataMarket::sign_delegate_contract(
		RuntimeOrigin::signed(data_owner.clone()),
		contract_id
	));
}

fn sign_test_purchase_contract(data_buyer: u64, contract_id: u128, data_verifier: u64) {
	assert_ok!(DataMarket::sign_purchase_contract(
		RuntimeOrigin::signed(data_buyer.clone()),
		contract_id,
		data_verifier
	));
}

#[test]
fn make_delegate_contract_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_owner = 10;
		let agency = 20;
		make_test_delegate_contract(data_owner.clone(), agency.clone());
		System::assert_last_event(RuntimeEvent::DataMarket(
			crate::Event::MakeDataDelegateContract { contract_id, agency },
		));
	});
}

#[test]
fn sign_delegate_contract_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_owner = 10;
		let agency = 20;
		make_test_delegate_contract(data_owner.clone(), agency.clone());
		sign_test_delegate_contract(data_owner.clone(), contract_id.clone());
		System::assert_last_event(RuntimeEvent::DataMarket(
			crate::Event::SignDateDelegateContract { contract_id, data_owner },
		));
	});
}

#[test]
fn make_purchase_contract_without_agency_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_buyer = 10;
		let data_verifier = 20;
		make_test_purchase_contract_without_agency(data_buyer.clone(), data_verifier.clone());
		System::assert_last_event(RuntimeEvent::DataMarket(
			crate::Event::MakeDataPurchaseContract { contract_id, data_buyer },
		));
	});
}

#[test]
fn make_purchase_contract_with_agency_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_buyer = 10;
		let agency = 20;
		make_test_purchase_contract_with_agency(data_buyer.clone(), agency.clone());
		System::assert_last_event(RuntimeEvent::DataMarket(
			crate::Event::MakeDataPurchaseContract { contract_id, data_buyer },
		));
	});
}

#[test]
fn sign_purchase_contract_with_agency_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_buyer = 10;
		let agency = 20;
		let data_verifier = 30;
		make_test_purchase_contract_with_agency(data_buyer.clone(), agency.clone());
		sign_test_purchase_contract(agency.clone(), contract_id.clone(), data_verifier.clone());
		System::assert_last_event(RuntimeEvent::DataMarket(
			crate::Event::SignDataPurchaseContract { contract_id, agency, data_verifier },
		));
	});
}

#[test]
fn terminate_delegate_contract_before_active_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_owner = 10;
		let agency = 20;
		make_test_delegate_contract(data_owner.clone(), agency.clone());
		assert_ok!(DataMarket::terminate_delegate_contract(
			RuntimeOrigin::signed(agency.clone()),
			contract_id.clone()
		));
		System::assert_last_event(RuntimeEvent::DataMarket(crate::Event::ContractTerminated {
			contract_type: ContractType::Delegate,
			contract_id,
		}));
	});
}

#[test]
fn terminate_delegate_contract_after_active_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_owner = 10;
		let agency = 20;
		make_test_delegate_contract(data_owner.clone(), agency.clone());
		sign_test_delegate_contract(data_owner.clone(), contract_id.clone());
		assert_ok!(DataMarket::terminate_delegate_contract(
			RuntimeOrigin::signed(data_owner.clone()),
			contract_id.clone()
		));
		System::assert_last_event(RuntimeEvent::DataMarket(
			crate::Event::PendingContractTerminate {
				contract_type: ContractType::Delegate,
				contract_id: contract_id.clone(),
			},
		));
		assert_ok!(DataMarket::terminate_delegate_contract(
			RuntimeOrigin::signed(agency.clone()),
			contract_id.clone()
		));
		System::assert_last_event(RuntimeEvent::DataMarket(crate::Event::ContractTerminated {
			contract_type: ContractType::Delegate,
			contract_id,
		}));
	});
}

#[test]
fn terminate_purchase_contract_without_agency_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_buyer = 10;
		let data_verifier = 20;
		make_test_purchase_contract_without_agency(data_buyer.clone(), data_verifier.clone());
		assert_ok!(DataMarket::terminate_purchase_contract(
			RuntimeOrigin::signed(data_buyer.clone()),
			contract_id.clone()
		));
		System::assert_last_event(RuntimeEvent::DataMarket(crate::Event::ContractTerminated {
			contract_type: ContractType::Purchase,
			contract_id,
		}));
	});
}

#[test]
fn terminate_purchase_contract_with_agency_before_active_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_buyer = 10;
		let agency = 20;
		make_test_purchase_contract_with_agency(data_buyer.clone(), agency.clone());
		assert_ok!(DataMarket::terminate_purchase_contract(
			RuntimeOrigin::signed(data_buyer.clone()),
			contract_id.clone()
		));
		System::assert_last_event(RuntimeEvent::DataMarket(crate::Event::ContractTerminated {
			contract_type: ContractType::Purchase,
			contract_id,
		}));
	});
}

#[test]
fn terminate_purchase_contract_with_agency_after_active_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_buyer = 10;
		let agency = 20;
		let data_verifier = 30;
		make_test_purchase_contract_with_agency(data_buyer.clone(), agency.clone());
		sign_test_purchase_contract(agency.clone(), contract_id.clone(), data_verifier.clone());
		assert_ok!(DataMarket::terminate_purchase_contract(
			RuntimeOrigin::signed(data_buyer.clone()),
			contract_id.clone()
		));
		System::assert_last_event(RuntimeEvent::DataMarket(
			crate::Event::PendingContractTerminate {
				contract_type: ContractType::Purchase,
				contract_id: contract_id.clone(),
			},
		));
		assert_ok!(DataMarket::terminate_purchase_contract(
			RuntimeOrigin::signed(agency.clone()),
			contract_id.clone()
		));
		System::assert_last_event(RuntimeEvent::DataMarket(crate::Event::ContractTerminated {
			contract_type: ContractType::Purchase,
			contract_id,
		}));
	});
}

#[test]
fn execute_data_trade_without_agency_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_buyer = 10;
		let data_verifier = 20;
		make_test_purchase_contract_without_agency(data_buyer.clone(), data_verifier.clone());
		System::assert_last_event(RuntimeEvent::DataMarket(
			crate::Event::MakeDataPurchaseContract {
				contract_id: contract_id.clone(),
				data_buyer: data_buyer.clone(),
			},
		));

		let data_owner = 11;
		let data_issuer = vec![(data_owner.clone(), 100)];
		let data_owner_fee_ratio: u32 = 1000;
		let data_issuer_fee_ratio: u32 = 1000;
		let price_per_data: u128 = 1000;
		let data_verification_proof = VerificationProof::<AnyText>::new(bounded_vec![1]);

		let data_owner_fee: u128 = price_per_data * (data_owner_fee_ratio as u128) / 10000;
		let data_issuer_fee: u128 = price_per_data * (data_issuer_fee_ratio as u128) / 10000;
		let platform_fee: u128 = price_per_data - data_owner_fee - data_issuer_fee;

		assert_ok!(DataMarket::execute_data_trade(
			RuntimeOrigin::signed(data_verifier.clone()),
			contract_id.clone(),
			data_owner.clone(),
			data_issuer.clone(),
			data_owner_fee_ratio.clone(),
			data_issuer_fee_ratio.clone(),
			None,
			None,
			price_per_data,
			data_verification_proof.clone()
		));
		System::assert_last_event(RuntimeEvent::DataMarket(crate::Event::DataTradeExecuted {
			contract_id,
			data_owner,
			data_issuer,
			data_owner_fee,
			data_issuer_fee,
			platform_fee,
			data_verification_proof,
		}));
	});
}

#[test]
fn execute_data_trade_with_agency_works() {
	new_test_ext().execute_with(|| {
		let contract_id = 0;
		let data_buyer = 10;
		let agency = 20;
		let data_verifier = 30;
		make_test_purchase_contract_with_agency(data_buyer.clone(), agency.clone());
		sign_test_purchase_contract(agency.clone(), contract_id.clone(), data_verifier.clone());
		System::assert_last_event(RuntimeEvent::DataMarket(
			crate::Event::SignDataPurchaseContract { contract_id, agency, data_verifier },
		));

		let data_owner = 11;
		let data_issuer = vec![(data_owner.clone(), 100)];
		let data_owner_fee_ratio: u32 = 1000;
		let data_issuer_fee_ratio: u32 = 1000;
		let agency_fee_ratio: u32 = 1000;
		let price_per_data: u128 = 1000;
		let data_verification_proof = VerificationProof::<AnyText>::new(bounded_vec![1]);

		let data_owner_fee: u128 = price_per_data * (data_owner_fee_ratio as u128) / 10000;
		let data_issuer_fee: u128 = price_per_data * (data_issuer_fee_ratio as u128) / 10000;
		let agency_fee: u128 = price_per_data * (agency_fee_ratio as u128) / 10000;
		let platform_fee: u128 = price_per_data - data_owner_fee - data_issuer_fee - agency_fee;

		assert_ok!(DataMarket::execute_data_trade(
			RuntimeOrigin::signed(data_verifier.clone()),
			contract_id.clone(),
			data_owner.clone(),
			data_issuer.clone(),
			data_owner_fee_ratio.clone(),
			data_issuer_fee_ratio.clone(),
			Some(agency),
			Some(agency_fee_ratio.clone()),
			price_per_data,
			data_verification_proof.clone()
		));
		System::assert_last_event(RuntimeEvent::DataMarket(crate::Event::DataTradeExecuted {
			contract_id,
			data_owner,
			data_issuer,
			data_owner_fee,
			data_issuer_fee,
			platform_fee,
			data_verification_proof,
		}));
	});
}
