#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::traits::{fungibles::*, tokens::Balance};
pub use pallet::*;
use sp_arithmetic::{FixedPointNumber, FixedU128};
use sp_runtime::{
	types::{infra_core::SystemConfig, RuntimeConfigProvider},
	DispatchError, Saturating,
};

pub type SystemTokenWeightOf<T> = <<T as Config>::Fungibles as InspectSystemToken<
	<T as frame_system::Config>::AccountId,
>>::SystemTokenWeight;

#[frame_support::pallet(dev_mode)]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The type in which the assets for converting are measured.
		type Balance: Balance + From<u128> + From<SystemTokenWeightOf<Self>>;
		/// Type of asset class, sourced from [`Config::Assets`], utilized to offer liquidity to a
		/// pool.
		type AssetKind: Parameter + MaxEncodedLen;
		/// Type that handles the fungibles.
		type Fungibles: InspectSystemToken<Self::AccountId, AssetId = Self::AssetKind, Balance = Self::Balance>
			+ Mutate<Self::AccountId>
			+ Balanced<Self::AccountId>;
		/// Type that provides the runtime configuration.
		type SystemConfig: RuntimeConfigProvider<Self::Balance>;
	}

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		SomethingHappened,
	}

	#[pallet::error]
	pub enum Error<T> {
		NotSystemToken,
		SystemConfigMissing,
	}
}

pub trait SystemTokenConversion {
	/// Measure units of the asset classes for converting.
	type Balance: Balance;
	/// Kind of assets that are going to be converted.
	type AssetKind;

	/// Convert System Token balance for given `asset` based on base System Token
	///
	/// ### Formula
	///
	/// - fee: weight_scale * para_fee_rate * balance / system_token_weight *
	///   base_system_token_weight
	/// - vote: fee * system_token_weight
	fn to_system_token_balance(
		asset: Self::AssetKind,
		balance: Self::Balance,
	) -> Result<Self::Balance, DispatchError>;
}

impl<T: Config> SystemTokenConversion for Pallet<T> {
	type Balance = T::Balance;

	type AssetKind = T::AssetKind;

	fn to_system_token_balance(
		asset: Self::AssetKind,
		balance: Self::Balance,
	) -> Result<Self::Balance, DispatchError> {
		frame_support::ensure!(T::Fungibles::is_system_token(&asset), Error::<T>::NotSystemToken);
		let system_token_weight =
			T::Fungibles::system_token_weight(asset).map_err(|_| Error::<T>::NotSystemToken)?;
		let SystemConfig { base_system_token_detail, weight_scale } =
			T::SystemConfig::system_config().map_err(|_| Error::<T>::SystemConfigMissing)?;
		let para_fee_rate =
			T::SystemConfig::para_fee_rate().map_err(|_| Error::<T>::SystemConfigMissing)?;
		let base_weight: Self::Balance = base_system_token_detail.base_weight.into();
		let n = balance.saturating_mul(para_fee_rate);
		let d = base_weight.saturating_mul(system_token_weight.into());
		let converted_fee =
			FixedU128::saturating_from_rational(n, d).saturating_mul_int(weight_scale.into());
		Ok(converted_fee)
	}
}
