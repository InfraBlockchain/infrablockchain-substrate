// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![cfg_attr(not(feature = "std"), no_std)]
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

mod types;

pub use types::*;

mod payment;
pub use payment::*;

use codec::{Decode, Encode};
use frame_support::{
	dispatch::{DispatchInfo, DispatchResult, PostDispatchInfo},
	pallet_prelude::*,
	traits::{
		infra_support::pot::VotingHandler,
		tokens::{
			fungibles::{Balanced, Credit, Inspect},
			WithdrawConsequence,
		},
		CallMetadata, Contains, GetCallMetadata, IsType,
	},
	DefaultNoBound, PalletId,
};
use frame_system::pallet_prelude::*;
use pallet_system_token::{ensure_system_token_origin, Origin as SystemTokenOrigin};
use pallet_transaction_payment::OnChargeTransaction;
use scale_info::TypeInfo;
use sp_runtime::{
	traits::{
		AccountIdConversion, DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension,
		Zero,
	},
	transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
	types::{
		AssetId as InfraAssetId, ExtrinsicMetadata, SystemTokenId, SystemTokenLocalAssetProvider,
		VoteAccountId, VoteWeight,
	},
	FixedPointOperand,
};

use sp_std::prelude::*;

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::traits::Contains;

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
		type RuntimeOrigin: From<SystemTokenOrigin>
			+ From<<Self as frame_system::Config>::RuntimeOrigin>
			+ Into<Result<SystemTokenOrigin, <Self as Config>::RuntimeOrigin>>;
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The fungibles instance used to pay for transactions in assets.
		type Assets: Balanced<Self::AccountId>
			+ SystemTokenLocalAssetProvider<InfraAssetId, Self::AccountId>;
		/// The actual transaction charging logic that charges the fees.
		type OnChargeSystemToken: OnChargeSystemToken<Self>;
		/// The type that handles the voting.
		type VotingHandler: VotingHandler;
		/// Filters for bootstrappring runtime.
		type BootstrapCallFilter: Contains<Self::RuntimeCall>;
		/// Id for handling fee(e.g SoverignAccount for some Runtime).
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::storage]
	#[pallet::unbounded]
	pub type FeeTable<T: Config> =
		StorageMap<_, Twox128, ExtrinsicMetadata, BalanceOf<T>, OptionQuery>;

	#[pallet::storage]
	/// The fee rate imposed to parachain. The fee rate 1_000 actually equals 1.
	/// It is initilzed as 1_000(1.0), then it SHOULD be only set by a dmp call from RELAY CHAIN.
	pub(super) type ParaFeeRate<T: Config> = StorageValue<_, BalanceOf<T>, OptionQuery>;

	#[pallet::storage]
	pub(super) type State<T: Config> = StorageValue<_, RuntimeState, ValueQuery>;

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		pub fn set_fee_table(
			origin: OriginFor<T>,
			pallet_name: Vec<u8>,
			call_name: Vec<u8>,
			fee: BalanceOf<T>,
		) -> DispatchResult {
			ensure_system_token_origin(<T as Config>::RuntimeOrigin::from(origin))?;
			let extrinsic_metadata = ExtrinsicMetadata::new(pallet_name, call_name);
			FeeTable::<T>::insert(&extrinsic_metadata, fee);
			Self::deposit_event(Event::<T>::FeeTableUpdated { metadata: extrinsic_metadata, fee });
			Ok(())
		}

		#[pallet::call_index(1)]
		pub fn set_para_fee_rate(
			origin: OriginFor<T>,
			para_fee_rate: BalanceOf<T>,
		) -> DispatchResult {
			ensure_system_token_origin(<T as Config>::RuntimeOrigin::from(origin))?;

			ParaFeeRate::<T>::set(Some(para_fee_rate));

			Self::deposit_event(Event::ParaFeeRateUpdated { para_fee_rate });
			Ok(())
		}

		#[pallet::call_index(2)]
		pub fn set_runtime_state(origin: OriginFor<T>) -> DispatchResult {
			ensure_system_token_origin(<T as Config>::RuntimeOrigin::from(origin))?;
			Self::do_set_runtime_state()?;
			Ok(())
		}
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A transaction fee `actual_fee`, of which `tip` was added to the minimum inclusion fee,
		/// has been paid by `who` in an asset `asset_id`.
		SystemTokenTxFeePaid {
			fee_payer: T::AccountId,
			detail: Detail<T>,
			vote_candidate: Option<VoteAccountId>,
		},
		/// Currently, Runtime is in bootstrap mode.
		OnBootstrapping,
		/// Fee Tabe has been updated
		FeeTableUpdated {
			metadata: ExtrinsicMetadata,
			fee: BalanceOf<T>,
		},
		/// Para fee rate has been updated
		ParaFeeRateUpdated {
			para_fee_rate: BalanceOf<T>,
		},
		BootstrapEnded,
	}

	#[pallet::error]
	pub enum Error<T> {
		ErrorConvertToAssetBalance,
		NotInBootstrap,
		NotAllowedToChangeState,
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}

impl<T: Config> Pallet<T> {
	fn check_bootstrap_and_filter(call: &T::RuntimeCall) -> Result<bool, TransactionValidityError> {
		match (State::<T>::get(), T::BootstrapCallFilter::contains(call)) {
			(RuntimeState::Bootstrap, false) =>
				Err(TransactionValidityError::Invalid(InvalidTransaction::InvalidBootstrappingCall)),
			(RuntimeState::Bootstrap, true) => Ok(true),
			(RuntimeState::Normal, _) => Ok(false),
		}
	}

	pub fn do_set_runtime_state() -> DispatchResult {
		ensure!(State::<T>::get() == RuntimeState::Bootstrap, Error::<T>::NotInBootstrap);
		let l = T::Assets::system_token_list();
		ensure!(!l.is_empty(), Error::<T>::NotAllowedToChangeState);
		// ToDo: Check whether a parachain has enough system token to pay
		State::<T>::put(RuntimeState::Normal);
		Self::deposit_event(Event::<T>::BootstrapEnded);
		Ok(())
	}
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue. Allows paying via both `Currency` as well as `fungibles::Balanced`.
///
/// Wraps the transaction logic in [`pallet_transaction_payment`] and extends it with assets.
/// An asset id of `None` falls back to the underlying transaction payment via the native currency.
#[derive(Encode, Decode, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ChargeSystemToken<T: Config> {
	// tip to be added for the block author
	#[codec(compact)]
	tip: BalanceOf<T>,
	// Asset to pay the fee with
	system_token_id: Option<SystemTokenId>,
	// whom to vote for
	vote_candidate: Option<VoteAccountId>,
}

impl<T: Config> ChargeSystemToken<T>
where
	T::RuntimeCall:
		Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + GetCallMetadata,
	AssetBalanceOf<T>: Send + Sync + FixedPointOperand,
	BalanceOf<T>: Send + Sync + FixedPointOperand + IsType<ChargeAssetBalanceOf<T>>,
	ChargeSystemTokenAssetIdOf<T>: Send + Sync,
	Credit<T::AccountId, T::Assets>: IsType<ChargeAssetLiquidityOf<T>>,
{
	// For benchmarking only
	pub fn new() -> Self {
		Self { tip: Default::default(), system_token_id: None, vote_candidate: None }
	}

	/// Utility constructor. Used only in client/factory code.
	pub fn from(
		tip: BalanceOf<T>,
		system_token_id: Option<SystemTokenId>,
		vote_candidate: Option<VoteAccountId>,
	) -> Self {
		Self { tip, system_token_id, vote_candidate }
	}

	/// Taking fee **before dispatching transactions.**
	/// If system token has been provided, system token will be charged.
	/// Otherwise, Runtime will take the largest amount of system token.
	// ToDo: Need to consider the weight of the system token when the largest amount of system token
	// is taken!
	fn withdraw_fee(
		&self,
		who: &T::AccountId,
		call: &T::RuntimeCall,
		info: &DispatchInfoOf<T::RuntimeCall>,
		len: usize,
	) -> Result<(BalanceOf<T>, InitialPayment<T>), TransactionValidityError> {
		let fee = pallet_transaction_payment::Pallet::<T>::compute_fee(len as u32, info, self.tip);
		debug_assert!(self.tip <= fee, "tip should be included in the computed fee");

		if fee.is_zero() {
			Ok((fee, InitialPayment::Nothing))
		} else {
			if let Some(system_token_id) = self.system_token_id {
				T::OnChargeSystemToken::withdraw_fee(
					who,
					call,
					info,
					Some(system_token_id.asset_id.into()),
					fee.into(),
					self.tip.into(),
				)
				.map(|i| (fee, InitialPayment::Asset(i.into())))
			} else {
				T::OnChargeSystemToken::withdraw_fee(
					who,
					call,
					info,
					None,
					fee.into(),
					self.tip.into(),
				)
				.map(|i| (fee, InitialPayment::Asset(i.into())))
			}
		}
	}
}

impl<T: Config> sp_std::fmt::Debug for ChargeSystemToken<T> {
	#[cfg(feature = "std")]
	fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		write!(f, "ChargeSystemToken<{:?}, {:?}>", self.tip, self.system_token_id.encode())
	}
	#[cfg(not(feature = "std"))]
	fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
		Ok(())
	}
}

impl<T: Config> SignedExtension for ChargeSystemToken<T>
where
	T::RuntimeCall:
		Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo> + GetCallMetadata,
	AssetBalanceOf<T>: Send + Sync + FixedPointOperand + IsType<VoteWeight>,
	AssetIdOf<T>: Send + Sync + IsType<ChargeSystemTokenAssetIdOf<T>>,
	BalanceOf<T>: Send
		+ Sync
		+ From<u64>
		+ FixedPointOperand
		+ IsType<ChargeAssetBalanceOf<T>>
		+ From<AssetBalanceOf<T>>,
	ChargeSystemTokenAssetIdOf<T>: Send + Sync,
	Credit<T::AccountId, T::Assets>: IsType<ChargeAssetLiquidityOf<T>>,
{
	const IDENTIFIER: &'static str = "ChargeSystemToken";
	type AccountId = T::AccountId;
	type Call = T::RuntimeCall;
	type AdditionalSigned = ();
	type Pre = (
		// tip
		BalanceOf<T>,
		// who paid the fee. could be 'fee_payer' or 'user(signer)'
		Self::AccountId,
		// Metadata of the call. (Pallet Name, Call Name)
		CallMetadata,
		// imbalance resulting from withdrawing the fee
		InitialPayment<T>,
		// asset_id for the transaction payment
		Option<SystemTokenId>,
		// vote info included in the transaction. Should be same as Relay Chain's AccountId type
		Option<VoteAccountId>,
	);

	fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
		Ok(())
	}

	fn validate(
		&self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> TransactionValidity {
		use pallet_transaction_payment::ChargeTransactionPayment;
		let payer = who.clone();
		let is_bootstrap = Pallet::<T>::check_bootstrap_and_filter(call)?;
		let (fee, _) = if is_bootstrap {
			(Zero::zero(), InitialPayment::Nothing)
		} else {
			let (fee, _paid) = self.withdraw_fee(&payer, call, info, len)?;
			(fee, _paid)
		};
		let priority = ChargeTransactionPayment::<T>::get_priority(info, len, self.tip, fee);
		Ok(ValidTransaction { priority, ..Default::default() })
	}

	fn pre_dispatch(
		self,
		who: &Self::AccountId,
		call: &Self::Call,
		info: &DispatchInfoOf<Self::Call>,
		len: usize,
	) -> Result<Self::Pre, TransactionValidityError> {
		let is_bootstrap = Pallet::<T>::check_bootstrap_and_filter(call)?;
		let (_, initial_payment) = if is_bootstrap {
			(Zero::zero(), InitialPayment::Nothing)
		} else {
			self.withdraw_fee(who, call, info, len)?
		};
		let call_metadata = call.get_call_metadata();
		Ok((
			self.tip,
			who.clone(),
			call_metadata,
			initial_payment,
			self.system_token_id,
			self.vote_candidate,
		))
	}

	fn post_dispatch(
		pre: Option<Self::Pre>,
		info: &DispatchInfoOf<Self::Call>,
		post_info: &PostDispatchInfoOf<Self::Call>,
		len: usize,
		_result: &DispatchResult,
	) -> Result<(), TransactionValidityError> {
		if let Some((tip, who, call_metadata, initial_payment, system_token_id, vote_candidate)) =
			pre
		{
			match initial_payment {
				// Ibs only pay with some asset
				InitialPayment::Asset(already_withdrawn) => {
					let metadata = ExtrinsicMetadata::new(
						call_metadata.pallet_name,
						call_metadata.function_name,
					);
					let mut refundable: bool = true;

					let actual_fee: BalanceOf<T> =
						// `fee` will be calculated based on the 'fee table'.
						// The fee will be directly applied to the `final_fee` without any refunds.
						if let Some(fee) = FeeTable::<T>::get(metadata) {
							refundable = false;
							fee
						} else {
							// The `fee` will be calculated according to the original fee calculation logic.
							pallet_transaction_payment::Pallet::<T>::compute_actual_fee(
								len as u32, info, post_info, tip,
								)
						};
					let paid_asset_id = already_withdrawn.asset();
					let (converted_fee, converted_tip) =
						T::OnChargeSystemToken::correct_and_deposit_fee(
							&who,
							info,
							post_info,
							actual_fee.into(),
							tip.into(),
							already_withdrawn.into(),
							refundable,
						)?;

					let tip: Option<AssetBalanceOf<T>> =
						if converted_tip.is_zero() { None } else { Some(converted_tip) };
					// update_vote_info is only excuted when vote_info has some data
					match (&vote_candidate, &system_token_id) {
						// Case: Voting and system token has clarified
						(Some(vote_candidate), Some(system_token_id)) => {
							Pallet::<T>::deposit_event(Event::<T>::SystemTokenTxFeePaid {
								fee_payer: who,
								detail: Detail::<T> {
									paid_asset_id: paid_asset_id.into(),
									actual_fee,
									converted_fee,
									tip,
								},
								vote_candidate: Some(vote_candidate.clone()),
							});
							// Update vote
							T::VotingHandler::update_pot_vote(
								vote_candidate.clone().into(),
								system_token_id.clone(),
								converted_fee.into(),
							);
						},
						_ => {
							Pallet::<T>::deposit_event(Event::<T>::SystemTokenTxFeePaid {
								fee_payer: who,
								detail: Detail::<T> {
									paid_asset_id: paid_asset_id.into(),
									actual_fee,
									converted_fee,
									tip,
								},
								vote_candidate: None,
							});
						},
					}
				},
				InitialPayment::Nothing => {
					// `actual_fee` should be zero here for any signed extrinsic. It would be
					// non-zero here in case of unsigned extrinsics as they don't pay fees but
					// `compute_actual_fee` is not aware of them. In both cases it's fine to just
					// move ahead without adjusting the fee, though, so we do nothing.
					debug_assert!(tip.is_zero(), "tip should be zero if initial fee was zero.");
				},
				InitialPayment::Bootstrap => {
					Pallet::<T>::deposit_event(Event::<T>::OnBootstrapping);
				},
				_ => return Err(TransactionValidityError::Invalid(InvalidTransaction::Payment)),
			}
		}

		Ok(())
	}
}

pub struct CreditToBucket<T>(PhantomData<T>);
impl<T: Config> HandleCredit<T::AccountId, T::Assets> for CreditToBucket<T> {
	fn handle_credit(credit: Credit<T::AccountId, T::Assets>) {
		let dest = T::PalletId::get().into_account_truncating();
		let _ = <T::Assets as Balanced<T::AccountId>>::resolve(&dest, credit);
	}
}
