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
pub type TradeCount = u64;
pub type IssuerWeight = u32;
pub type FeeRatio = u32;
pub type Quantity = u128;
pub const MAX_RECORDS_SIZE: u128 = 3 * 1024;

#[cfg(test)]
pub mod mock;

#[cfg(test)]
pub mod tests;

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

		/// The current members of Oracle.
		#[pallet::constant]
		type MaxPurchaseQuantity: Get<u32>;
	}

	#[pallet::storage]
	pub(super) type CurrentPurchaseId<T: Config> = StorageValue<_, PurchaseId, ValueQuery>;

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

	// To show buy and sell lists for AccountId, reverse storage should be needed.

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
	pub type DataTradeRecords<T: Config> = StorageMap<
		_,
		Twox64Concat,
		PurchaseId,
		BoundedVec<T::AccountId, T::MaxPurchaseQuantity>,
		ValueQuery,
	>;

	#[derive(Encode, Decode, Clone, PartialEq, Eq, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Hash, Debug))]
	pub struct TradeReceiptDetails<AccountId> {
		pub data_owner: AccountId,
		pub data_issuer: Vec<(AccountId, IssuerWeight)>,
		pub quantity: Quantity,
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		DataPurchaseFinished {
			data_buyer: T::AccountId,
			data_purchase_id: PurchaseId,
			data_purchase_status: PurchaseStatus,
		},
		DataTradeExecuted {
			data_owner: T::AccountId,
			data_purchase_id: PurchaseId,
			total_amount: u128,
			data_owner_fee: u128,
			data_issuer_fee: u128,
			platform_fee: u128,
			data_verification_proof: VerificationProof<AnyText>,
		},
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
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Overflow for CurrentPurchaseId
		Overflow,
		/// The account has already participated in the trade
		AccountAlreadyExists,
		/// Verifier of the origin is invalid
		InvalidVerifier,
		/// Error that the total trade limit has been reached.
		TradeLimitReached,
		/// Error that the total trade limit has been reached.
		BoundLimitReached,
		/// Error failed to the existing purchase request
		PurchaseDoesNotExist,
		/// Purchase has already been finished
		PurchaseNotActive,
		/// Origin is different with data buyer
		InvalidBuyer,
	}

	#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Hash))]
	pub enum PurchaseStatus {
		Active,
		Finished,
	}

	#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Hash))]
	pub enum DataType {
		Image,
		Json,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T>
	where
		u32: PartialEq<<T as pallet_assets::Config>::AssetId>,
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

			let data_purchase_id = CurrentPurchaseId::<T>::get();
			CurrentPurchaseId::<T>::try_mutate(|c| -> DispatchResult {
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

			// Transfer system tokens from data buyer to the escrow account
			let total_amount = quantity.saturating_mul(price_per_data.into());
			let escrow_account = Self::get_escrow_account();

			pallet_assets::Pallet::<T>::transfer(
				origin.clone(),
				system_token_id.into(),
				T::Lookup::unlookup(escrow_account.clone()),
				<T as pallet_assets::Config>::Balance::from(total_amount),
			)?;

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
			origin: OriginFor<T>, // data verifier
			data_owner: T::AccountId,
			data_purchase_id: PurchaseId,
			data_issuer: Vec<(T::AccountId, IssuerWeight)>,
			data_verification_proof: VerificationProof<AnyText>,
		) -> DispatchResult {
			let current_verifier = ensure_signed(origin)?;

			// Ensure the purchase register is valid and active
			let mut data_purchase_register_details =
				DataPurchaseRegisters::<T>::get(data_purchase_id)
					.ok_or(Error::<T>::PurchaseDoesNotExist)?;

			let DataPurchaseRegisterDetails {
				data_verifiers,
				data_owner_fee_ratio,
				data_issuer_fee_ratio,
				price_per_data,
				system_token_id,
				purchase_status,
				quantity,
				..
			} = data_purchase_register_details.clone();

			ensure!(purchase_status == PurchaseStatus::Active, Error::<T>::PurchaseNotActive);

			let mut trade_accounts = DataTradeRecords::<T>::get(data_purchase_id);

			// Ensure that it is the first trade of data_owner
			if trade_accounts.contains(&data_owner) {
				return Err(Error::<T>::AccountAlreadyExists.into())
			}

			match trade_accounts.try_push(data_owner.clone()) {
				Ok(_) => {
					let len_trade_accounts = trade_accounts.len() as u128;
					DataTradeRecords::<T>::insert(data_purchase_id, trade_accounts);

					if len_trade_accounts == quantity {
						data_purchase_register_details.purchase_status = PurchaseStatus::Finished;

						Self::do_finish_data_purchase(
							data_purchase_register_details.data_buyer.clone(),
							data_purchase_id,
						)?;

						DataPurchaseRegisters::<T>::insert(
							data_purchase_id,
							data_purchase_register_details,
						);
					} else if len_trade_accounts > quantity {
						return Err(Error::<T>::TradeLimitReached.into())
					}
				},
				Err(_) => return Err(Error::<T>::BoundLimitReached.into()),
			}

			if !data_verifiers.contains(&current_verifier) {
				return Err(Error::<T>::InvalidVerifier.into())
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

			// Record the trade details in the DataTradeRecords storage
			let _new_trade_receipt =
				TradeReceiptDetails { data_owner: data_owner.clone(), data_issuer, quantity };

			// Emit an event for successful trade execution
			Self::deposit_event(Event::<T>::DataTradeExecuted {
				data_owner,
				data_purchase_id,
				total_amount: data_owner_fee + data_issuer_fee + platform_fee,
				data_owner_fee,
				data_issuer_fee,
				platform_fee,
				data_verification_proof,
			});

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
	u32: PartialEq<<T as pallet_assets::Config>::AssetId>,
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

		let total_weight: IssuerWeight = data_issuer.iter().map(|(_, weight)| weight).sum();

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

		ensure!(data_buyer == data_purchase_register_details.data_buyer, Error::<T>::InvalidBuyer);

		ensure!(
			data_purchase_register_details.purchase_status == PurchaseStatus::Active,
			Error::<T>::PurchaseNotActive
		);

		data_purchase_register_details.purchase_status = PurchaseStatus::Finished;
		let data_purchase_status = data_purchase_register_details.purchase_status;
		DataPurchaseRegisters::<T>::insert(&data_purchase_id, data_purchase_register_details);

		Self::deposit_event(Event::<T>::DataPurchaseFinished {
			data_buyer,
			data_purchase_id,
			data_purchase_status,
		});

		Ok(())
	}

	fn calculate_data_fee(
		price_per_data: u128,
		data_owner_fee_ratio: u32,
		data_issuer_fee_ratio: u32,
	) -> (u128, u128, u128) {
		let platform_fee_ratio = 10000u32 - data_owner_fee_ratio - data_issuer_fee_ratio;
		let quantity = 1u128;
		let total_amount = price_per_data * quantity;

		let data_owner_fee =
			total_amount.saturating_mul(data_owner_fee_ratio as u128).saturating_div(10000);
		let data_issuer_fee =
			total_amount.saturating_mul(data_issuer_fee_ratio as u128).saturating_div(10000);
		let platform_fee =
			total_amount.saturating_mul(platform_fee_ratio as u128).saturating_div(10000);

		(data_owner_fee, data_issuer_fee, platform_fee)
	}
}
