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

///! Traits and default implementation for paying transaction fees in assets.
use super::*;
use crate::Config;

use frame_support::{
	traits::{
		fungibles::{Balanced, Credit, Inspect},
		tokens::{AssetId, Balance, Fortitude::Polite, Precision::Exact, Preservation::Protect},
	},
	unsigned::TransactionValidityError,
};
use pallet_system_token_conversion::SystemTokenConversion;
use sp_runtime::{
	traits::{DispatchInfoOf, One, PostDispatchInfoOf},
	transaction_validity::InvalidTransaction,
};
use sp_std::marker::PhantomData;

/// Handle withdrawing, refunding and depositing of transaction fees.
pub trait OnChargeSystemToken<T: Config> {
	/// The underlying integer type in which fees are calculated.
	type Balance: Balance;
	/// The type used to identify the assets used for transaction payment.
	type AssetId: AssetId + From<SystemTokenAssetIdOf<T>>;
	/// The type used to store the intermediate values between pre- and post-dispatch.
	type LiquidityInfo;

	/// Before the transaction is executed the payment of the transaction fees needs to be secured.
	///
	/// Note: The `fee` already includes the `tip`.
	fn withdraw_fee(
		who: &T::AccountId,
		call: &T::RuntimeCall,
		dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
		asset_id: Option<Self::AssetId>,
		fee: Self::Balance,
		tip: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError>;

	/// After the transaction was executed the actual fee can be calculated.
	/// This function should refund any overpaid fees and optionally deposit
	/// the corrected amount.
	///
	/// Note: The `fee` already includes the `tip`.
	///
	/// Returns the fee and tip in the asset used for payment as (fee, tip).
	fn correct_and_deposit_fee(
		who: &T::AccountId,
		dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
		post_info: &PostDispatchInfoOf<T::RuntimeCall>,
		corrected_fee: Self::Balance,
		tip: Self::Balance,
		already_withdrawn: Self::LiquidityInfo,
		refundable: bool,
	) -> Result<(SystemTokenBalanceOf<T>, SystemTokenBalanceOf<T>, SystemTokenBalanceOf<T>), TransactionValidityError>;
}

/// Allows specifying what to do with the withdrawn asset fees.
pub trait HandleCredit<AccountId, B: Balanced<AccountId>> {
	/// Implement to determine what to do with the withdrawn asset fees.
	/// Default for `CreditOf` from the assets pallet is to burn and
	/// decrease total issuance.
	fn handle_credit(credit: Credit<AccountId, B>);
}

/// Default implementation that just drops the credit according to the `OnDrop` in the underlying
/// imbalance type.
impl<A, B: Balanced<A>> HandleCredit<A, B> for () {
	fn handle_credit(_credit: Credit<A, B>) {}
}

/// Implements the asset transaction for a balance to asset converter (implementing
/// [`BalanceConversion`]) and a credit handler (implementing [`HandleCredit`]).
///
/// The credit handler is given the complete fee in terms of the asset used for the transaction.
pub struct TransactionFeeCharger<T, CON, HC>(PhantomData<(T, CON, HC)>);

/// Default implementation for a runtime instantiating this pallet, a balance to asset converter and
/// a credit handler.
impl<T, CON, HC> OnChargeSystemToken<T> for TransactionFeeCharger<T, CON, HC>
where
	T: Config,
	CON: SystemTokenConversion<AssetKind = SystemTokenAssetIdOf<T>, Balance = BalanceOf<T>>,
	HC: HandleCredit<T::AccountId, T::Fungibles>,
	SystemTokenAssetIdOf<T>: AssetId,
	SystemTokenBalanceOf<T>: From<BalanceOf<T>> + From<SystemTokenWeightOf<T>> + From<u128>,
{
	type Balance = BalanceOf<T>;
	type AssetId = SystemTokenAssetIdOf<T>;
	type LiquidityInfo = Credit<T::AccountId, T::Fungibles>;

	/// Withdraw the predicted fee from the transaction origin.
	///
	/// Note: The `fee` already includes the `tip`.
	fn withdraw_fee(
		who: &T::AccountId,
		_call: &T::RuntimeCall,
		_info: &DispatchInfoOf<T::RuntimeCall>,
		// which asset to pay
		asset_id: Option<Self::AssetId>,
		// actual fee
		fee: Self::Balance,
		_tip: Self::Balance,
	) -> Result<Self::LiquidityInfo, TransactionValidityError> {
		let (asset_id, _) =
			<T::Fungibles as InspectSystemToken<T::AccountId>>::balance(who, asset_id)
				.take()
				.ok_or(TransactionValidityError::from(InvalidTransaction::Payment))?;
		let min_converted_fee = if fee.is_zero() { Zero::zero() } else { One::one() };
		// CON::to_asset_balance => fee / system_token_weight
		let converted_fee = CON::to_system_token_balance(asset_id.clone(), fee)
			.map_err(|_| TransactionValidityError::from(InvalidTransaction::Payment))?
			.max(min_converted_fee);
		let can_withdraw = <T::Fungibles as Inspect<T::AccountId>>::can_withdraw(
			asset_id.clone(),
			who,
			converted_fee.into(),
		);
		if !matches!(can_withdraw, WithdrawConsequence::Success) {
			return Err(InvalidTransaction::Payment.into())
		}
		<T::Fungibles as Balanced<T::AccountId>>::withdraw(
			asset_id,
			who,
			converted_fee.into(),
			Exact,
			Protect,
			Polite,
		)
		.map_err(|_| TransactionValidityError::from(InvalidTransaction::Payment))
	}

	/// Hand the fee and the tip over to the `[HandleCredit]` implementation.
	/// Since the predicted fee might have been too high, parts of the fee may be refunded.
	///
	/// Note: The `corrected_fee` already includes the `tip`.
	///
	/// Returns the fee and tip in the asset used for payment as (fee, tip).
	fn correct_and_deposit_fee(
		who: &T::AccountId,
		_dispatch_info: &DispatchInfoOf<T::RuntimeCall>,
		_post_info: &PostDispatchInfoOf<T::RuntimeCall>,
		corrected_fee: Self::Balance,
		tip: Self::Balance,
		paid: Self::LiquidityInfo,
		refundable: bool,
	) -> Result<(SystemTokenBalanceOf<T>, SystemTokenBalanceOf<T>, SystemTokenBalanceOf<T>), TransactionValidityError> {
		let min_converted_fee = if corrected_fee.is_zero() { Zero::zero() } else { One::one() };
		// Convert the corrected fee and tip into the asset used for payment.
		let converted_fee = CON::to_system_token_balance(paid.asset(), corrected_fee)
			.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })?
			.max(min_converted_fee);
		let converted_tip = CON::to_system_token_balance(paid.asset(), tip)
			.map_err(|_| -> TransactionValidityError { InvalidTransaction::Payment.into() })?;

		// Calculate how much refund we should return.
		let (final_fee, refund) = if refundable {
			// Split the paid amount into final fee and refund when refundable.
			paid.split(converted_fee.into())
		} else {
			// When not refundable, split without any refund.
			paid.split_no_refund(converted_fee.into())
		};

		// Refund to the account that paid the fees. If this fails, the account might have dropped
		// below the existential balance. In that case we don't refund anything.
		let final_fee_amount = final_fee.peek();
		let _ = <T::Fungibles as Balanced<T::AccountId>>::resolve(who, refund);
		// Handle final_fee 
		let reward = T::RewardFraction::get() * final_fee_amount;
		let (block_author_reward, remain) = final_fee.split(reward);
		// 1. To Bucket
		let bucket_id: T::AccountId = T::PalletId::get().into_account_truncating();
		let bucket_amount = remain.peek();
		let _ = <T::Fungibles as Balanced<T::AccountId>>::resolve(&bucket_id, remain);
		Pallet::<T>::deposit_event(
			Event::<T>::FeeHandled {
				bucket_amount,
				remainder: reward,
			}
		);
		// 2. Handle the final fee, e.g. by transferring to the block author or burning.
		HC::handle_credit(block_author_reward);
		Ok((final_fee_amount, bucket_amount, converted_tip.into()))
	}
}
