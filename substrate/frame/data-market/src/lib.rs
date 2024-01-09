#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{ConstU32, Get},
	BoundedVec, PalletId,
};

use frame_system::pallet_prelude::*;
use sp_runtime::traits::{AccountIdConversion, StaticLookup};
use sp_std::vec::Vec;

pub use pallet::*;

pub mod types;
pub use types::*;

pub type PurchaseId = u128;
pub type Quantity = u32;
pub type IssuerWeight = u32;

// TOTAL_FEE_RATIO is 100%(indicates 10_000)
const TOTAL_FEE_RATIO: u32 = 10_000;
// MIN_PLATFORM_FEE_RATIO is fixed at a minimum of 10%
const MIN_PLATFORM_FEE_RATIO: u32 = 1_000;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

#[frame_support::pallet(dev_mode)]
pub mod pallet {

	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_assets::Config {
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		#[pallet::constant]
		type MaxPurchaseQuantity: Get<Quantity>;
	}

	#[pallet::storage]
	pub(super) type NextPurchaseId<T: Config> = StorageValue<_, PurchaseId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn data_purchase_registers)]
	pub type DataPurchaseRegisters<T: Config> = StorageMap<
		_,
		Twox64Concat,
		PurchaseId,
		DataPurchaseRegisterDetails<
			T::AccountId,
			BlockNumberFor<T>,
			<T as pallet_assets::Config>::Balance,
			AnyText,
		>,
		OptionQuery,
	>;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Hash, Debug))]
	pub struct DataPurchaseRegisterDetails<AccountId, BlockNumber, Balance, AnyText> {
		pub data_buyer: AccountId,
		pub data_buyer_info: DataBuyerInfo<AnyText>,
		pub data_purchase_info: DataPurchaseInfo<AnyText>,
		pub data_verifiers: Vec<AccountId>,
		pub purchase_deadline: BlockNumber,
		pub system_token_id: u32,
		pub quantity: Quantity,
		pub price_per_data: Balance,
		pub data_issuer_fee_ratio: u32,
		pub data_owner_fee_ratio: u32,
		pub purchase_status: PurchaseStatus,
	}

	#[pallet::storage]
	#[pallet::getter(fn data_trade_records)]
	pub type DataTradeRecords<T: Config> =
		StorageDoubleMap<_, Twox64Concat, PurchaseId, Twox64Concat, T::AccountId, ()>;

	#[pallet::storage]
	pub type TradeCountForPurchase<T: Config> =
		StorageMap<_, Twox64Concat, PurchaseId, u32, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DataPurchaseRegistered {
			data_buyer: T::AccountId,
			data_purchase_id: PurchaseId,
			data_purchase_register_details: DataPurchaseRegisterDetails<
				T::AccountId,
				BlockNumberFor<T>,
				<T as pallet_assets::Config>::Balance,
				AnyText,
			>,
		},
		DataTradeExecuted {
			data_owner: T::AccountId,
			data_purchase_id: PurchaseId,
			data_issuer: Vec<(T::AccountId, IssuerWeight)>,
			data_owner_fee: u128,
			data_issuer_fee: u128,
			platform_fee: u128,
			data_verification_proof: VerificationProof<AnyText>,
		},
		DataPurchaseFinished {
			data_buyer: T::AccountId,
			data_purchase_id: PurchaseId,
			remaining_deposit: <T as pallet_assets::Config>::Balance,
			purchase_status: PurchaseStatus,
		},
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Overflow for NextPurchaseId.
		Overflow,
		/// The account has already participated in the trade.
		AccountAlreadyExists,
		/// Error that the total trade limit has been reached.
		TradeLimitReached,
		/// Error failed to the existing purchase request.
		PurchaseDoesNotExist,
		/// Purchase has already been finished.
		PurchaseNotActive,
		/// Origin is different with data buyer.
		InvalidBuyer,
		/// Verifier of the origin is invalid.
		InvalidVerifier,
		/// Issuer weight should be greater than zero
		IssuerWeightInvalid,
		/// Max verifier members exceed
		MaxVeirifierMembersExceed,
		/// Invalid calculation of fee ratio
		InvalidFeeRatio,
	}

	#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Hash))]
	pub enum PurchaseStatus {
		Active,
		Finished,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as pallet_assets::Config>::Balance: From<u128> + Into<u128>,
		<T as pallet_assets::Config>::AssetIdParameter: From<u32>,
	{
		#[pallet::call_index(0)]
		#[pallet::weight(1_000)]
		pub fn register_data_purchase(
			origin: OriginFor<T>,
			data_buyer_info: DataBuyerInfo<AnyText>,
			data_purchase_info: DataPurchaseInfo<AnyText>,
			data_verifiers: Vec<T::AccountId>,
			purchase_deadline: BlockNumberFor<T>,
			system_token_id: u32,
			quantity: Quantity,
			price_per_data: <T as pallet_assets::Config>::Balance,
			data_issuer_fee_ratio: u32,
			data_owner_fee_ratio: u32,
		) -> DispatchResult {
			let data_buyer = ensure_signed(origin.clone())?;

			ensure!(
				data_issuer_fee_ratio + data_owner_fee_ratio <=
					TOTAL_FEE_RATIO - MIN_PLATFORM_FEE_RATIO,
				Error::<T>::InvalidFeeRatio
			);

			let data_purchase_id = NextPurchaseId::<T>::get();
			NextPurchaseId::<T>::try_mutate(|c| -> DispatchResult {
				*c = c.checked_add(1).ok_or(Error::<T>::Overflow)?;
				Ok(())
			})?;

			let data_purchase_register_details = DataPurchaseRegisterDetails {
				data_buyer: data_buyer.clone(),
				data_buyer_info,
				data_purchase_info,
				data_verifiers,
				purchase_deadline,
				system_token_id,
				quantity,
				price_per_data,
				data_issuer_fee_ratio,
				data_owner_fee_ratio,
				purchase_status: PurchaseStatus::Active,
			};

			DataPurchaseRegisters::<T>::insert(
				data_purchase_id,
				data_purchase_register_details.clone(),
			);

			{
				let total_amount = (quantity as u128).saturating_mul(price_per_data.into());
				let escrow_account = Self::get_escrow_account();

				pallet_assets::Pallet::<T>::transfer(
					origin.clone(),
					system_token_id.into(),
					T::Lookup::unlookup(escrow_account),
					<T as pallet_assets::Config>::Balance::from(total_amount),
				)?;
			}

			Self::deposit_event(Event::<T>::DataPurchaseRegistered {
				data_buyer,
				data_purchase_id,
				data_purchase_register_details,
			});

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(10_000)]
		pub fn execute_data_trade(
			origin: OriginFor<T>,
			data_purchase_id: PurchaseId,
			data_owner: T::AccountId,
			data_issuer: Vec<(T::AccountId, IssuerWeight)>,
			data_verification_proof: VerificationProof<AnyText>,
		) -> DispatchResult {
			let origin_verifier = ensure_signed(origin)?;

			// Ensure the purchase register is valid and active
			let data_purchase_register_details = DataPurchaseRegisters::<T>::get(data_purchase_id)
				.ok_or(Error::<T>::PurchaseDoesNotExist)?;

			let DataPurchaseRegisterDetails {
				data_buyer,
				data_verifiers,
				data_owner_fee_ratio,
				data_issuer_fee_ratio,
				price_per_data,
				system_token_id,
				purchase_status,
				quantity,
				..
			} = data_purchase_register_details.clone();

			let mut trade_count = TradeCountForPurchase::<T>::get(data_purchase_id);

			ensure!(trade_count < quantity, Error::<T>::TradeLimitReached);
			ensure!(purchase_status == PurchaseStatus::Active, Error::<T>::PurchaseNotActive);
			ensure!(data_verifiers.contains(&origin_verifier), Error::<T>::InvalidVerifier);

			if DataTradeRecords::<T>::contains_key(data_purchase_id, &data_owner) {
				return Err(Error::<T>::AccountAlreadyExists.into())
			} else {
				trade_count += 1;
				DataTradeRecords::<T>::insert(data_purchase_id, &data_owner, ());
				TradeCountForPurchase::<T>::insert(data_purchase_id, trade_count);
			}

			// Transfer system tokens from the escrow to owner, issuer and platform
			let (data_owner_fee, data_issuer_fee, platform_fee) = Self::calculate_data_fee(
				price_per_data.into(),
				data_owner_fee_ratio,
				data_issuer_fee_ratio,
			);

			Self::settle_data_trade(
				data_owner.clone(),
				data_owner_fee,
				data_issuer.clone(),
				data_issuer_fee,
				platform_fee,
				system_token_id,
			)?;

			Self::deposit_event(Event::<T>::DataTradeExecuted {
				data_owner,
				data_purchase_id,
				data_issuer,
				data_owner_fee,
				data_issuer_fee,
				platform_fee,
				data_verification_proof,
			});

			if trade_count == quantity {
				Self::do_finish_data_purchase(data_buyer, data_purchase_id)?;
			}

			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(1_000)]
		pub fn finish_data_purchase(
			origin: OriginFor<T>,
			data_purchase_id: PurchaseId,
		) -> DispatchResult {
			let data_buyer = ensure_signed(origin)?;

			Self::do_finish_data_purchase(data_buyer, data_purchase_id)?;

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
		system_token_id: u32,
	) -> DispatchResult {
		let escrow_account = Self::get_escrow_account();
		let platform_account = Self::get_platform_account();

		pallet_assets::Pallet::<T>::transfer(
			frame_system::RawOrigin::Signed(escrow_account.clone()).into(),
			system_token_id.into(),
			T::Lookup::unlookup(data_owner),
			<T as pallet_assets::Config>::Balance::from(data_owner_fee),
		)?;

		let total_weight: u32 = data_issuer.iter().map(|(_, weight)| weight).sum();
		ensure!(total_weight > 0u32, Error::<T>::IssuerWeightInvalid);

		for (account, weight) in data_issuer.iter() {
			let distributed_fee = data_issuer_fee
				.saturating_mul(*weight as u128)
				.saturating_div(total_weight as u128);
			pallet_assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(escrow_account.clone()).into(),
				system_token_id.into(),
				T::Lookup::unlookup(account.clone()),
				<T as pallet_assets::Config>::Balance::from(distributed_fee),
			)?;
		}

		pallet_assets::Pallet::<T>::transfer(
			frame_system::RawOrigin::Signed(escrow_account).into(),
			system_token_id.into(),
			T::Lookup::unlookup(platform_account),
			<T as pallet_assets::Config>::Balance::from(platform_fee),
		)?;

		Ok(())
	}

	pub fn do_finish_data_purchase(
		data_buyer: T::AccountId,
		data_purchase_id: PurchaseId,
	) -> DispatchResult {
		let mut data_purchase_register_details = DataPurchaseRegisters::<T>::get(&data_purchase_id)
			.ok_or(Error::<T>::PurchaseDoesNotExist)?;

		let DataPurchaseRegisterDetails {
			price_per_data,
			system_token_id,
			purchase_status,
			quantity,
			..
		} = data_purchase_register_details.clone();

		ensure!(data_buyer == data_purchase_register_details.data_buyer, Error::<T>::InvalidBuyer);
		ensure!(purchase_status == PurchaseStatus::Active, Error::<T>::PurchaseNotActive);

		// Change purchase status
		data_purchase_register_details.purchase_status = PurchaseStatus::Finished;
		DataPurchaseRegisters::<T>::insert(&data_purchase_id, data_purchase_register_details);

		// Refund the remaining deposit
		let escrow_account = Self::get_escrow_account();
		let remainig_quantity = quantity - TradeCountForPurchase::<T>::get(data_purchase_id);
		let remaining_deposit = {
			let r_d = (price_per_data.into()).saturating_mul(remainig_quantity as u128);
			<T as pallet_assets::Config>::Balance::from(r_d)
		};

		if remainig_quantity > 0 {
			pallet_assets::Pallet::<T>::transfer(
				frame_system::RawOrigin::Signed(escrow_account).into(),
				system_token_id.into(),
				T::Lookup::unlookup(data_buyer.clone()),
				remaining_deposit,
			)?;
		}

		// Remove storage data
		TradeCountForPurchase::<T>::remove(data_purchase_id);
		let _ = DataTradeRecords::<T>::clear_prefix(data_purchase_id, u32::MAX, None);

		Self::deposit_event(Event::<T>::DataPurchaseFinished {
			data_buyer,
			data_purchase_id,
			remaining_deposit,
			purchase_status,
		});

		Ok(())
	}

	fn calculate_data_fee(
		price_per_data: u128,
		data_owner_fee_ratio: u32,
		data_issuer_fee_ratio: u32,
	) -> (u128, u128, u128) {
		let platform_fee_ratio = 10_000 - data_owner_fee_ratio - data_issuer_fee_ratio;
		let quantity = 1u128;
		let total_amount = price_per_data * quantity;

		let data_owner_fee =
			total_amount.saturating_mul(data_owner_fee_ratio as u128).saturating_div(10_000);
		let data_issuer_fee = total_amount
			.saturating_mul(data_issuer_fee_ratio as u128)
			.saturating_div(10_000);
		let platform_fee =
			total_amount.saturating_mul(platform_fee_ratio as u128).saturating_div(10_000);

		(data_owner_fee, data_issuer_fee, platform_fee)
	}
}
