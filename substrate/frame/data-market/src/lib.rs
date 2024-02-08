#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod types;
pub use types::*;

use frame_support::{
	pallet_prelude::*,
	traits::{ConstU32, Get},
	BoundedVec, PalletId,
};

use frame_system::pallet_prelude::*;
pub use pallet::*;
use pallet_assets::TransferFlags;
use sp_runtime::traits::AccountIdConversion;
use sp_std::{vec, vec::Vec};

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		// The maximum quantity of data that can be purchased
		#[pallet::constant]
		type MaxPurchaseQuantity: Get<Quantity>;

		/// The total fee ratio, defined as 100% and represented by the value 10,000.
		#[pallet::constant]
		type TotalFeeRatio: Get<u32>;

		/// The minimum platform fee ratio, set at a fixed 10%.
		#[pallet::constant]
		type MinPlatformFeeRatio: Get<u32>;
	}

	// The Next value of contract id
	#[pallet::storage]
	#[pallet::getter(fn get_next_contract_id)]
	pub(super) type NextContractId<T: Config> = StorageValue<_, ContractId, ValueQuery>;

	// The Data Trade Records
	#[pallet::storage]
	#[pallet::getter(fn get_data_trade_records)]
	pub type DataTradeRecords<T: Config> =
		StorageDoubleMap<_, Twox64Concat, ContractId, Twox64Concat, T::AccountId, ()>;

	// The Trade Count for Contract
	#[pallet::storage]
	#[pallet::getter(fn get_trade_count_for_purchase)]
	pub type TradeCountForContract<T: Config> =
		StorageMap<_, Twox64Concat, ContractId, Quantity, ValueQuery>;

	// The Data Delegate Contracts
	#[pallet::storage]
	#[pallet::getter(fn get_data_delegate_contracts)]
	pub(super) type DataDelegateContracts<T: Config> = StorageMap<
		_,
		Twox64Concat,
		ContractId,
		DataDelegateContractDetail<T::AccountId, BlockNumberFor<T>, AnyText>,
		OptionQuery,
	>;

	// The Data Purchase Contracts
	#[pallet::storage]
	#[pallet::getter(fn get_data_purchase_contracts)]
	pub(super) type DataPurchaseContracts<T: Config> = StorageMap<
		_,
		Twox64Concat,
		ContractId,
		DataPurchaseContractDetail<
			T::AccountId,
			BlockNumberFor<T>,
			<T as pallet_assets::Config>::Balance,
			AnyText,
		>,
		OptionQuery,
	>;

	// The Data Delegate Contract List
	#[pallet::storage]
	#[pallet::getter(fn get_data_delegate_contract_list)]
	pub(super) type DataDelegateContractList<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<ContractId>, ValueQuery>;

	// The Data Purchase Contract List
	#[pallet::storage]
	#[pallet::getter(fn get_data_purchase_contract_list)]
	pub(super) type DataPurchaseContractList<T: Config> =
		StorageMap<_, Twox64Concat, T::AccountId, Vec<ContractId>, ValueQuery>;

	// The Contract Status
	#[pallet::storage]
	#[pallet::getter(fn get_contract_status)]
	pub(super) type ContractStatus<T: Config> =
		StorageMap<_, Twox64Concat, ContractId, Vec<(T::AccountId, SignStatus)>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		// Make Data Delegate Contract
		MakeDataDelegateContract {
			contract_id: ContractId,
			agency: T::AccountId,
		},
		// Sign Data Delegate Contract
		SignDateDelegateContract {
			contract_id: ContractId,
			data_owner: T::AccountId,
		},
		// Make Data Purchase Contract
		MakeDataPurchaseContract {
			contract_id: ContractId,
			data_buyer: T::AccountId,
		},
		// Sign Data Purchase Contract
		SignDataPurchaseContract {
			contract_id: ContractId,
			agency: T::AccountId,
			data_verifier: T::AccountId,
		},
		// Pending Contract Terminate
		PendingContractTerminate {
			contract_type: ContractType,
			contract_id: ContractId,
		},
		// Contract Terminated
		ContractTerminated {
			contract_type: ContractType,
			contract_id: ContractId,
		},
		// Data Trade Executed
		DataTradeExecuted {
			contract_id: ContractId,
			data_owner: T::AccountId,
			data_issuer: Vec<(T::AccountId, IssuerWeight)>,
			data_owner_fee: u128,
			data_issuer_fee: u128,
			platform_fee: u128,
			data_verification_proof: VerificationProof<AnyText>,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Overflow for NextPurchaseId.
		Overflow,
		/// Error that the total trade limit has been reached.
		TradeLimitReached,
		/// Contract period is invalid.
		InvalidPeriod,
		/// Error failed to the existing contract.
		ContractNotExist,
		/// Error failed to the existing contract status.
		ContractStatusNotExist,
		/// Purchase has already been finished.
		ContractNotActive,
		/// Origin is different with data Owner.
		InvalidOwner,
		/// Origin is different with data buyer.
		InvalidBuyer,
		/// Verifier of the origin is invalid.
		InvalidVerifier,
		/// Agency of the origin is invalid.
		InvalidAgency,
		/// Cannot sign the contract.
		NotSigned,
		/// Issuer weight should be greater than zero
		IssuerWeightInvalid,
		/// Max verifier members exceed
		MaxVeirifierMembersExceed,
		/// Invalid calculation of fee ratio
		InvalidFeeRatio,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as pallet_assets::Config>::Balance: From<u128> + Into<u128>,
		<T as pallet_assets::Config>::AssetIdParameter: From<u32>,
	{
		/// Make a delegate contract
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `detail`: The detail of the contract.
		#[pallet::call_index(0)]
		pub fn make_delegate_contract(
			origin: OriginFor<T>,
			detail: DataDelegateContractDetail<T::AccountId, BlockNumberFor<T>, AnyText>,
		) -> DispatchResult {
			let maybe_agency = ensure_signed(origin)?;
			Self::do_make_delegate_contract(maybe_agency, detail)?;
			Ok(())
		}

		/// Sign a delegate contract
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `contract_id`: The id of the contract.
		#[pallet::call_index(1)]
		pub fn sign_delegate_contract(
			origin: OriginFor<T>,
			contract_id: ContractId,
		) -> DispatchResult {
			let maybe_owner = ensure_signed(origin)?;
			Self::do_sign_delegate_contract(maybe_owner, contract_id)?;
			Ok(())
		}

		/// Make a purchase contract
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `detail`: The detail of the contract.
		#[pallet::call_index(2)]
		pub fn make_purchase_contract(
			origin: OriginFor<T>,
			detail: DataPurchaseContractDetail<
				T::AccountId,
				BlockNumberFor<T>,
				<T as pallet_assets::Config>::Balance,
				AnyText,
			>,
			is_agency_exist: bool,
		) -> DispatchResult {
			let maybe_buyer = ensure_signed(origin)?;
			Self::do_make_purchase_contract(maybe_buyer, detail, is_agency_exist)?;
			Ok(())
		}

		/// Sign a purchase contract
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `contract_id`: The id of the contract.
		/// - `data_verifier`: The verifier of the contract.
		#[pallet::call_index(3)]
		pub fn sign_purchase_contract(
			origin: OriginFor<T>,
			contract_id: ContractId,
			data_verifier: T::AccountId,
		) -> DispatchResult {
			let maybe_agency = ensure_signed(origin)?;
			Self::do_sign_purchase_contract(maybe_agency, contract_id, data_verifier)?;
			Ok(())
		}

		/// Terminate a delegate contract
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `contract_id`: The id of the contract.
		#[pallet::call_index(4)]
		pub fn terminate_delegate_contract(
			origin: OriginFor<T>,
			contract_id: ContractId,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::do_terminate_delegate_contract(signer, contract_id)?;
			Ok(())
		}

		/// Terminate a purchase contract
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `contract_id`: The id of the contract.
		#[pallet::call_index(5)]
		pub fn terminate_purchase_contract(
			origin: OriginFor<T>,
			contract_id: ContractId,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::do_terminate_purchase_contract(signer, contract_id)?;
			Ok(())
		}

		/// Execute a data trade
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `contract_id`: The id of the contract.
		/// - `data_owner`: The owner of the data.
		/// - `data_issuer`: The issuer of the data.
		/// - `data_owner_fee_ratio`: The fee ratio of the data owner.
		/// - `data_issuer_fee_ratio`: The fee ratio of the data issuer.
		/// - `agency`: The agency of the data.
		/// - `agency_fee_ratio`: The fee ratio of the agency.
		/// - `price_per_data`: The price per data.
		/// - `data_verification_proof`: The verification proof of the data.
		#[pallet::call_index(6)]
		pub fn execute_data_trade(
			origin: OriginFor<T>,
			contract_id: ContractId,
			data_owner: T::AccountId,
			data_issuer: Vec<(T::AccountId, IssuerWeight)>,
			data_owner_fee_ratio: u32,
			data_issuer_fee_ratio: u32,
			agency: Option<T::AccountId>,
			agency_fee_ratio: Option<u32>,
			price_per_data: T::Balance,
			data_verification_proof: VerificationProof<AnyText>,
		) -> DispatchResult {
			let maybe_verifier = ensure_signed(origin)?;
			Self::do_execute_data_trade(
				maybe_verifier,
				contract_id,
				data_owner,
				data_issuer,
				data_owner_fee_ratio,
				data_issuer_fee_ratio,
				agency,
				agency_fee_ratio,
				price_per_data,
				data_verification_proof,
			)?;
			Ok(())
		}
	}
}

impl<T: Config> Pallet<T>
where
	<T as pallet_assets::Config>::Balance: From<u128> + Into<u128>,
	<T as pallet_assets::Config>::AssetIdParameter: From<u32>,
{
	pub fn get_escrow_account() -> T::AccountId {
		const ID: PalletId = PalletId(*b"marketid");
		AccountIdConversion::<T::AccountId>::into_account_truncating(&ID)
	}

	pub fn get_platform_account() -> T::AccountId {
		const ID: PalletId = PalletId(*b"platform");
		AccountIdConversion::<T::AccountId>::into_account_truncating(&ID)
	}

	pub fn settle_data_trade(
		data_owner: T::AccountId,
		data_owner_fee: u128,
		data_issuer: Vec<(T::AccountId, IssuerWeight)>,
		data_issuer_fee: u128,
		platform_fee: u128,
		maybe_agency: Option<T::AccountId>,
		agency_fee: u128,
		system_token_asset_id: u32,
	) -> DispatchResult {
		let platform_account = Self::get_platform_account();

		Self::transfer_escrow(
			TransferFrom::Escrow,
			data_owner,
			system_token_asset_id,
			data_owner_fee,
		)?;

		let total_weight: u32 = data_issuer.iter().map(|(_, weight)| weight).sum();
		ensure!(total_weight > 0u32, Error::<T>::IssuerWeightInvalid);

		for (issuer, weight) in data_issuer.iter() {
			let distributed_fee = data_issuer_fee
				.saturating_mul(*weight as u128)
				.saturating_div(total_weight as u128);
			Self::transfer_escrow(
				TransferFrom::Escrow,
				issuer.clone(),
				system_token_asset_id,
				distributed_fee,
			)?;
		}

		Self::transfer_escrow(
			TransferFrom::Escrow,
			platform_account,
			system_token_asset_id,
			platform_fee,
		)?;

		if let Some(agency) = maybe_agency {
			if agency_fee > 0 {
				Self::transfer_escrow(
					TransferFrom::Escrow,
					agency,
					system_token_asset_id,
					agency_fee,
				)?;
			}
		}

		Ok(())
	}

	fn calculate_data_fee(
		price_per_data: u128,
		data_owner_fee_ratio: u32,
		data_issuer_fee_ratio: u32,
		agency_fee_ratio: u32,
	) -> (u128, u128, u128, u128) {
		let platform_fee_ratio =
			10_000 - data_owner_fee_ratio - data_issuer_fee_ratio - agency_fee_ratio;
		let quantity = 1;
		let total_amount = price_per_data * quantity;

		let data_owner_fee =
			total_amount.saturating_mul(data_owner_fee_ratio as u128).saturating_div(10_000);
		let data_issuer_fee = total_amount
			.saturating_mul(data_issuer_fee_ratio as u128)
			.saturating_div(10_000);
		let platform_fee =
			total_amount.saturating_mul(platform_fee_ratio as u128).saturating_div(10_000);
		let agency_fee =
			total_amount.saturating_mul(agency_fee_ratio as u128).saturating_div(10_000);

		(data_owner_fee, data_issuer_fee, platform_fee, agency_fee)
	}

	fn transfer_escrow(
		from: TransferFrom<T>,
		to: T::AccountId,
		system_token_asset_id: u32,
		amount: u128,
	) -> DispatchResult {
		let balance = <T as pallet_assets::Config>::Balance::from(amount);
		let f = TransferFlags { keep_alive: true, best_effort: false, burn_dust: false };

		match from {
			TransferFrom::Origin(origin) => {
				pallet_assets::Pallet::<T>::do_transfer(
					system_token_asset_id.into(),
					&origin,
					&to,
					balance,
					None,
					f,
				)
				.map(|_| ())?;
			},
			TransferFrom::Escrow => {
				let escrow = Self::get_escrow_account();
				pallet_assets::Pallet::<T>::do_transfer(
					system_token_asset_id.into(),
					&escrow,
					&to,
					balance,
					None,
					f,
				)
				.map(|_| ())?;
			},
		}

		Ok(())
	}

	fn do_make_delegate_contract(
		maybe_agency: T::AccountId,
		detail: DataDelegateContractDetail<T::AccountId, BlockNumberFor<T>, AnyText>,
	) -> DispatchResult {
		let agency = detail.clone().agency;
		ensure!(maybe_agency == agency, Error::<T>::InvalidAgency);
		ensure!(detail.clone().effective_at < detail.clone().expired_at, Error::<T>::InvalidPeriod);

		let contract_id = NextContractId::<T>::get();
		NextContractId::<T>::try_mutate(|c| -> DispatchResult {
			*c = c.checked_add(1).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;

		let data_owner = detail.clone().data_owner;
		let agency = maybe_agency;

		DataDelegateContracts::<T>::insert(contract_id, detail.clone());

		let mut data_owner_list = DataDelegateContractList::<T>::get(&data_owner);
		data_owner_list.push(contract_id);
		DataDelegateContractList::<T>::insert(&data_owner, data_owner_list);

		let mut agency_list = DataDelegateContractList::<T>::get(&agency);
		agency_list.push(contract_id);
		DataDelegateContractList::<T>::insert(&agency, agency_list);

		let contract_status =
			vec![(agency.clone(), SignStatus::Signed), (data_owner, SignStatus::Unsigned)];
		ContractStatus::<T>::insert(contract_id, contract_status.clone());

		Self::deposit_event(Event::<T>::MakeDataDelegateContract { contract_id, agency });

		Ok(())
	}

	fn do_sign_delegate_contract(
		maybe_owner: T::AccountId,
		contract_id: ContractId,
	) -> DispatchResult {
		let detail =
			DataDelegateContracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExist)?;

		ensure!(maybe_owner == detail.data_owner, Error::<T>::InvalidOwner);

		let mut status =
			ContractStatus::<T>::get(contract_id).ok_or(Error::<T>::ContractStatusNotExist)?;

		let mut is_signed = false;
		for (owner, signed) in status.iter_mut() {
			if owner == &maybe_owner {
				*signed = SignStatus::Signed;
				is_signed = true;
				break;
			}
		}

		ensure!(is_signed, Error::<T>::NotSigned);
		ContractStatus::<T>::insert(contract_id, status);
		Self::deposit_event(Event::<T>::SignDateDelegateContract {
			contract_id,
			data_owner: maybe_owner,
		});
		Ok(())
	}

	fn do_make_purchase_contract(
		maybe_buyer: T::AccountId,
		detail: DataPurchaseContractDetail<
			T::AccountId,
			BlockNumberFor<T>,
			<T as pallet_assets::Config>::Balance,
			AnyText,
		>,
		is_agency_exist: bool,
	) -> DispatchResult {
		ensure!(maybe_buyer == detail.data_buyer, Error::<T>::InvalidBuyer);
		ensure!(detail.clone().effective_at < detail.clone().expired_at, Error::<T>::InvalidPeriod);

		let contract_id = NextContractId::<T>::get();
		NextContractId::<T>::try_mutate(|c| -> DispatchResult {
			*c = c.checked_add(1).ok_or(Error::<T>::Overflow)?;
			Ok(())
		})?;

		let data_buyer = maybe_buyer;

		DataPurchaseContracts::<T>::insert(contract_id, detail.clone());

		let mut contract_status = vec![(data_buyer.clone(), SignStatus::Signed)];

		if is_agency_exist {
			ensure!(detail.agency.is_some(), Error::<T>::InvalidAgency);
			ensure!(detail.data_verifier.is_none(), Error::<T>::InvalidVerifier);
			let agency = detail.agency.clone().unwrap();
			let mut agency_list = DataPurchaseContractList::<T>::get(&agency);
			agency_list.push(contract_id);
			DataPurchaseContractList::<T>::insert(&agency, agency_list);

			contract_status.push((agency.clone(), SignStatus::Unsigned));
		} else {
			ensure!(detail.agency.is_none(), Error::<T>::InvalidAgency);
			ensure!(detail.data_verifier.is_some(), Error::<T>::InvalidVerifier);
		}

		let mut data_buyer_list = DataPurchaseContractList::<T>::get(&data_buyer);
		data_buyer_list.push(contract_id);
		DataPurchaseContractList::<T>::insert(&data_buyer, data_buyer_list);

		ContractStatus::<T>::insert(contract_id, contract_status);

		{
			let escrow_account = Self::get_escrow_account();
			Self::transfer_escrow(
				TransferFrom::Origin(data_buyer.clone()),
				escrow_account,
				detail.system_token_id,
				detail.deposit.into(),
			)?;
		}

		Self::deposit_event(Event::<T>::MakeDataPurchaseContract { contract_id, data_buyer });

		Ok(())
	}

	fn do_sign_purchase_contract(
		maybe_agency: T::AccountId,
		contract_id: ContractId,
		data_verifier: T::AccountId,
	) -> DispatchResult {
		let mut detail =
			DataPurchaseContracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExist)?;
		let agency = detail.clone().agency.ok_or(Error::<T>::InvalidAgency)?;
		ensure!(maybe_agency == agency, Error::<T>::InvalidAgency);

		let mut status =
			ContractStatus::<T>::get(contract_id).ok_or(Error::<T>::ContractStatusNotExist)?;

		let mut is_signed = false;
		for (agency, signed) in status.iter_mut() {
			if agency == &maybe_agency {
				*signed = SignStatus::Signed;
				is_signed = true;
				break;
			}
		}

		ensure!(is_signed, Error::<T>::NotSigned);
		ContractStatus::<T>::insert(contract_id, status);
		detail.data_verifier = Some(data_verifier.clone());
		DataPurchaseContracts::<T>::insert(contract_id, detail);

		Self::deposit_event(Event::<T>::SignDataPurchaseContract {
			contract_id,
			agency: maybe_agency,
			data_verifier,
		});

		Ok(())
	}

	fn do_terminate_delegate_contract(
		maybe_signer: T::AccountId,
		contract_id: ContractId,
	) -> DispatchResult {
		let detail =
			DataDelegateContracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExist)?;

		let mut status =
			ContractStatus::<T>::get(contract_id).ok_or(Error::<T>::ContractStatusNotExist)?;

		let mut is_signed = false;
		for (signer, signed) in status.iter_mut() {
			if signer == &maybe_signer {
				*signed = SignStatus::WantToTerminate;
				is_signed = true;
				break;
			}
		}

		ensure!(is_signed, Error::<T>::NotSigned);

		let current_block_number = <frame_system::Pallet<T>>::block_number();

		if status.iter().all(|(_, signed)| *signed != SignStatus::Signed) ||
			current_block_number > detail.expired_at
		{
			DataDelegateContracts::<T>::remove(contract_id);
			ContractStatus::<T>::remove(contract_id);
			Self::deposit_event(Event::<T>::ContractTerminated {
				contract_type: ContractType::Delegate,
				contract_id,
			});
		} else {
			// Storage update when pending terminate only
			ContractStatus::<T>::insert(contract_id, status);

			Self::deposit_event(Event::<T>::PendingContractTerminate {
				contract_type: ContractType::Delegate,
				contract_id,
			});
		}

		Ok(())
	}

	fn do_terminate_purchase_contract(
		maybe_signer: T::AccountId,
		contract_id: ContractId,
	) -> DispatchResult {
		let detail =
			DataPurchaseContracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExist)?;
		let mut status =
			ContractStatus::<T>::get(contract_id).ok_or(Error::<T>::ContractStatusNotExist)?;

		let mut is_signed = false;
		for (signer, signed) in status.iter_mut() {
			if signer == &maybe_signer {
				*signed = SignStatus::WantToTerminate;
				is_signed = true;
				break;
			}
		}

		ensure!(is_signed, Error::<T>::NotSigned);

		let current_block_number = <frame_system::Pallet<T>>::block_number();

		if status.iter().all(|(_, signed)| *signed != SignStatus::Signed) ||
			current_block_number > detail.expired_at
		{
			DataPurchaseContracts::<T>::remove(contract_id);
			ContractStatus::<T>::remove(contract_id);

			if detail.deposit > 0.into() {
				Self::transfer_escrow(
					TransferFrom::Escrow,
					detail.data_buyer,
					detail.system_token_id,
					detail.deposit.into(),
				)?;
			}

			TradeCountForContract::<T>::remove(contract_id);
			let _ = DataTradeRecords::<T>::clear_prefix(contract_id, u32::MAX, None);

			Self::deposit_event(Event::<T>::ContractTerminated {
				contract_type: ContractType::Purchase,
				contract_id,
			});
		} else {
			// Storage update when pending terminate only
			ContractStatus::<T>::insert(contract_id, status);

			Self::deposit_event(Event::<T>::PendingContractTerminate {
				contract_type: ContractType::Purchase,
				contract_id,
			});
		}

		Ok(())
	}

	fn do_execute_data_trade(
		maybe_verifier: T::AccountId,
		contract_id: ContractId,
		data_owner: T::AccountId,
		data_issuer: Vec<(T::AccountId, IssuerWeight)>,
		data_owner_fee_ratio: u32,
		data_issuer_fee_ratio: u32,
		maybe_agency: Option<T::AccountId>,
		maybe_agency_fee_ratio: Option<u32>,
		price_per_data: T::Balance,
		data_verification_proof: VerificationProof<AnyText>,
	) -> DispatchResult {
		let detail =
			DataPurchaseContracts::<T>::get(contract_id).ok_or(Error::<T>::ContractNotExist)?;
		let status =
			ContractStatus::<T>::get(contract_id).ok_or(Error::<T>::ContractStatusNotExist)?;
		let current_block_number = <frame_system::Pallet<T>>::block_number();

		let data_verifier = detail.clone().data_verifier.ok_or(Error::<T>::InvalidVerifier)?;
		ensure!(maybe_verifier == data_verifier, Error::<T>::InvalidVerifier);
		ensure!(
			status.iter().all(|(_, signed)| *signed == SignStatus::Signed),
			Error::<T>::ContractNotActive
		);
		ensure!(detail.clone().expired_at > current_block_number, Error::<T>::ContractNotActive);
		ensure!(detail.clone().effective_at <= current_block_number, Error::<T>::ContractNotActive);
		if let Some(agency) = maybe_agency.clone() {
			let agency_from_detail = detail.clone().agency.ok_or(Error::<T>::InvalidAgency)?;
			ensure!(agency == agency_from_detail, Error::<T>::InvalidAgency);
		}

		let mut trade_count = TradeCountForContract::<T>::get(contract_id);
		ensure!(trade_count < T::MaxPurchaseQuantity::get(), Error::<T>::TradeLimitReached);
		trade_count += 1;
		TradeCountForContract::<T>::insert(contract_id, trade_count);
		if !DataTradeRecords::<T>::contains_key(contract_id, &data_owner) {
			DataTradeRecords::<T>::insert(contract_id, &data_owner, ());
		}

		let agency_fee_ratio = maybe_agency_fee_ratio.unwrap_or(0);
		let sum_fee_ratio = agency_fee_ratio +
			data_issuer_fee_ratio +
			data_owner_fee_ratio +
			T::MinPlatformFeeRatio::get();
		ensure!(sum_fee_ratio <= T::TotalFeeRatio::get(), Error::<T>::InvalidFeeRatio);

		let (data_owner_fee, data_issuer_fee, platform_fee, agency_fee) = Self::calculate_data_fee(
			price_per_data.into(),
			data_owner_fee_ratio,
			data_issuer_fee_ratio,
			agency_fee_ratio,
		);

		Self::settle_data_trade(
			data_owner.clone(),
			data_owner_fee,
			data_issuer.clone(),
			data_issuer_fee,
			platform_fee,
			maybe_agency,
			agency_fee,
			detail.system_token_id,
		)?;

		Self::deposit_event(Event::<T>::DataTradeExecuted {
			contract_id,
			data_owner,
			data_issuer,
			data_owner_fee,
			data_issuer_fee,
			platform_fee,
			data_verification_proof,
		});

		Ok(())
	}
}
