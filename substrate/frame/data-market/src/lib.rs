#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
	pallet_prelude::*,
	traits::{ConstU32, Get},
	BoundedVec, PalletId,
};

use frame_system::pallet_prelude::*;
use pallet_assets::TransferFlags;
use sp_runtime::traits::AccountIdConversion;
use sp_std::vec::Vec;

pub use pallet::*;

pub mod types;
pub use types::*;

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

		// To be deleted
		#[pallet::constant]
		type MaxPurchaseQuantity: Get<Quantity>;

		/// The total fee ratio, defined as 100% and represented by the value 10,000.
		#[pallet::constant]
		type TotalFeeRatio: Get<u32>;

		/// The minimum platform fee ratio, set at a fixed 10%.
		#[pallet::constant]
		type MinPlatformFeeRatio: Get<u32>;
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

	#[pallet::storage]
	#[pallet::getter(fn data_trade_records)]
	pub type DataTradeRecords<T: Config> =
		StorageDoubleMap<_, Twox64Concat, PurchaseId, Twox64Concat, T::AccountId, ()>;

	#[pallet::storage]
	pub type TradeCountForPurchase<T: Config> =
		StorageMap<_, Twox64Concat, PurchaseId, Quantity, ValueQuery>;

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
		/// The account has already participated to trade.
		AlreadyTraded,
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

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		<T as pallet_assets::Config>::Balance: From<u128> + Into<u128>,
		<T as pallet_assets::Config>::AssetIdParameter: From<u32>,
	{
		#[pallet::call_index(0)]
		pub fn register_data_purchase(
			origin: OriginFor<T>,
			data_buyer_info: DataBuyerInfo<AnyText>,
			data_purchase_info: DataPurchaseInfo<AnyText>,
			data_verifier: T::AccountId,
			purchase_deadline: BlockNumberFor<T>,
			system_token_asset_id: u32,
			quantity: Quantity,
			price_per_data: <T as pallet_assets::Config>::Balance,
			data_issuer_fee_ratio: u32,
			data_owner_fee_ratio: u32,
		) -> DispatchResult {
			let data_buyer = ensure_signed(origin.clone())?;

			let sum_fee_ratio =
				data_issuer_fee_ratio + data_owner_fee_ratio + T::MinPlatformFeeRatio::get();
			ensure!(sum_fee_ratio <= T::TotalFeeRatio::get(), Error::<T>::InvalidFeeRatio);

			let data_purchase_id = NextPurchaseId::<T>::get();
			NextPurchaseId::<T>::try_mutate(|c| -> DispatchResult {
				*c = c.checked_add(1).ok_or(Error::<T>::Overflow)?;
				Ok(())
			})?;

			let data_purchase_register_details = DataPurchaseRegisterDetails {
				data_buyer: data_buyer.clone(),
				data_buyer_info,
				data_purchase_info,
				data_verifier,
				purchase_deadline,
				system_token_asset_id,
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
				let total_amount = quantity.saturating_mul(price_per_data.into());
				let escrow_account = Self::get_escrow_account();
				// Self::transfer_to_escrow(origin, escrow_account, system_token_asset_id,
				// total_amount)?;
				Self::transfer_escrow(
					TransferFrom::Origin(data_buyer.clone()),
					escrow_account,
					system_token_asset_id,
					total_amount,
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
		pub fn execute_data_trade(
			origin: OriginFor<T>,
			data_purchase_id: PurchaseId,
			data_owner: T::AccountId,
			data_issuer: Vec<(T::AccountId, IssuerWeight)>,
			data_verification_proof: VerificationProof<AnyText>,
		) -> DispatchResult {
			let maybe_verifier = ensure_signed(origin)?;

			// Ensure the purchase register is valid and active
			let data_purchase_register_details = DataPurchaseRegisters::<T>::get(data_purchase_id)
				.ok_or(Error::<T>::PurchaseDoesNotExist)?;

			let DataPurchaseRegisterDetails {
				data_buyer,
				data_verifier,
				data_owner_fee_ratio,
				data_issuer_fee_ratio,
				price_per_data,
				system_token_asset_id,
				purchase_status,
				quantity,
				..
			} = data_purchase_register_details.clone();

			let mut trade_count = TradeCountForPurchase::<T>::get(data_purchase_id);

			ensure!(trade_count < quantity, Error::<T>::TradeLimitReached);
			ensure!(purchase_status == PurchaseStatus::Active, Error::<T>::PurchaseNotActive);
			ensure!(data_verifier == maybe_verifier, Error::<T>::InvalidVerifier);

			if DataTradeRecords::<T>::contains_key(data_purchase_id, &data_owner) {
				return Err(Error::<T>::AlreadyTraded.into())
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
				system_token_asset_id,
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

		Ok(())
	}

	pub fn do_finish_data_purchase(
		maybe_data_buyer: T::AccountId,
		data_purchase_id: PurchaseId,
	) -> DispatchResult {
		let mut data_purchase_register_details = DataPurchaseRegisters::<T>::get(&data_purchase_id)
			.ok_or(Error::<T>::PurchaseDoesNotExist)?;

		let DataPurchaseRegisterDetails {
			data_buyer,
			price_per_data,
			system_token_asset_id,
			mut purchase_status,
			quantity,
			..
		} = data_purchase_register_details.clone();

		ensure!(maybe_data_buyer == data_buyer, Error::<T>::InvalidBuyer);
		ensure!(purchase_status == PurchaseStatus::Active, Error::<T>::PurchaseNotActive);

		// Change purchase status
		purchase_status = PurchaseStatus::Completed;
		data_purchase_register_details.purchase_status = purchase_status;
		DataPurchaseRegisters::<T>::insert(&data_purchase_id, data_purchase_register_details);

		// Refund the remaining deposit
		let remainig_quantity = quantity - TradeCountForPurchase::<T>::get(data_purchase_id);
		let remaining_deposit = price_per_data.into().saturating_mul(remainig_quantity);

		if remainig_quantity > 0 {
			Self::transfer_escrow(
				TransferFrom::Escrow,
				data_buyer.clone(),
				system_token_asset_id,
				remaining_deposit,
			)?;
		}

		// Remove storage data
		TradeCountForPurchase::<T>::remove(data_purchase_id);
		let _ = DataTradeRecords::<T>::clear_prefix(data_purchase_id, u32::MAX, None);

		Self::deposit_event(Event::<T>::DataPurchaseFinished {
			data_buyer,
			data_purchase_id,
			remaining_deposit: remaining_deposit.into(),
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
		let quantity = 1;
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
}
