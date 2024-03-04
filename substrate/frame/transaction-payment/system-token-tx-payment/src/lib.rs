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
		tokens::{
			fungibles::{Balanced, Credit, Inspect, InspectSystemToken},
			WithdrawConsequence,
		},
		CallMetadata, Contains, GetCallMetadata, IsType,
	},
	DefaultNoBound, PalletId,
};
use pallet_transaction_payment::OnChargeTransaction;
use scale_info::TypeInfo;
use softfloat::F64;
use sp_runtime::{
	traits::{
		AccountIdConversion, DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SignedExtension,
		Zero,
	},
	transaction_validity::{TransactionValidity, TransactionValidityError, ValidTransaction},
	types::{fee::*, infra_core::*, token::*, vote::*},
	FixedPointNumber, FixedPointOperand, FixedU128,
};

use sp_std::prelude::*;

pub use pallet::*;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use frame_support::traits::Contains;

	use super::*;

	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_transaction_payment::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Interface that is related to transaction for Infrablockchain Runtime
		type InfraTxInterface: RuntimeConfigProvider + VotingHandler;
		/// The fungibles instance used to pay for transactions in assets.
		type Fungibles: Balanced<Self::AccountId> + InspectSystemToken<Self::AccountId>;
		/// The actual transaction charging logic that charges the fees.
		type OnChargeSystemToken: OnChargeSystemToken<Self>;
		/// Filters for bootstrappring runtime.
		type BootstrapCallFilter: Contains<Self::RuntimeCall>;
		/// Id for handling fee(e.g SoverignAccount for some Runtime).
		#[pallet::constant]
		type PalletId: Get<PalletId>;
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

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
	}

	#[pallet::error]
	pub enum Error<T> {
		ErrorConvertToAssetBalance,
	}

	impl<T: Config> Pallet<T> {
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}
	}
}

impl<T: Config> Pallet<T> {
	fn check_bootstrap_and_filter(call: &T::RuntimeCall) -> Result<bool, TransactionValidityError> {
		match (T::InfraTxInterface::runtime_state(), T::BootstrapCallFilter::contains(call)) {
			(Mode::Bootstrap, false) =>
				Err(TransactionValidityError::Invalid(InvalidTransaction::InvalidBootstrappingCall)),
			(Mode::Bootstrap, true) => Ok(true),
			(Mode::Normal, _) => Ok(false),
		}
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
	Credit<T::AccountId, T::Fungibles>: IsType<ChargeAssetLiquidityOf<T>>,
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
	AssetBalanceOf<T>: Send + Sync + FixedPointOperand + IsType<SystemTokenBalance>,
	BalanceOf<T>: From<SystemTokenBalance>,
	AssetIdOf<T>: Send + Sync + IsType<ChargeSystemTokenAssetIdOf<T>>,
	BalanceOf<T>: Send
		+ Sync
		+ From<u64>
		+ FixedPointOperand
		+ IsType<ChargeAssetBalanceOf<T>>
		+ From<AssetBalanceOf<T>>,
	ChargeSystemTokenAssetIdOf<T>: Send + Sync,
	Credit<T::AccountId, T::Fungibles>: IsType<ChargeAssetLiquidityOf<T>>,
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
			(Zero::zero(), InitialPayment::Bootstrap)
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
					let ext_metadata = ExtrinsicMetadata::new(
						call_metadata.pallet_name,
						call_metadata.function_name,
					);
					let mut refundable: bool = true;

					let actual_fee: BalanceOf<T> =
						// `fee` will be calculated based on the 'fee table'.
						// The fee will be directly applied to the `final_fee` without any refunds.
						if let Some(fee) = T::InfraTxInterface::fee_for(ext_metadata) {
							refundable = false;
							fee.into()
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
							let vote_weight = F64::from_i128(converted_fee.into() as i128);
							T::InfraTxInterface::update_pot_vote(
								vote_candidate.clone().into(),
								system_token_id.clone(),
								vote_weight,
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
impl<T: Config> HandleCredit<T::AccountId, T::Fungibles> for CreditToBucket<T> {
	fn handle_credit(credit: Credit<T::AccountId, T::Fungibles>) {
		let dest = T::PalletId::get().into_account_truncating();
		let _ = <T::Fungibles as Balanced<T::AccountId>>::resolve(&dest, credit);
	}
}
