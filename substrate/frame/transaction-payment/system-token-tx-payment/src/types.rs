use super::*;

pub(super) const CORRECTION_PARA_FEE_RATE: u128 = 1_000_000;

// Type aliases used for interaction with `OnChargeTransaction`.
pub(crate) type OnChargeTransactionOf<T> =
	<T as pallet_transaction_payment::Config>::OnChargeTransaction;
// Balance type alias.
pub(crate) type BalanceOf<T> = <OnChargeTransactionOf<T> as OnChargeTransaction<T>>::Balance;
// Liquity info type alias.
pub(crate) type LiquidityInfoOf<T> =
	<OnChargeTransactionOf<T> as OnChargeTransaction<T>>::LiquidityInfo;

// Type alias used for interaction with fungibles (assets).
// Balance type alias.
pub(crate) type AssetBalanceOf<T> =
	<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
/// Asset id type alias.
pub(crate) type AssetIdOf<T> =
	<<T as Config>::Assets as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

// Type aliases used for interaction with `OnChargeAssetTransaction`.
// Balance type alias.
pub(crate) type ChargeAssetBalanceOf<T> =
	<<T as Config>::OnChargeSystemToken as OnChargeSystemToken<T>>::Balance;

pub(crate) type ChargeSystemTokenAssetIdOf<T> =
	<<T as Config>::OnChargeSystemToken as OnChargeSystemToken<T>>::SystemTokenAssetId;

// Liquity info type alias.
pub(crate) type ChargeAssetLiquidityOf<T> =
	<<T as Config>::OnChargeSystemToken as OnChargeSystemToken<T>>::LiquidityInfo;

/// Used to pass the initial payment info from pre- to post-dispatch.
#[derive(Encode, Decode, DefaultNoBound, TypeInfo)]
pub enum InitialPayment<T: Config> {
	/// No initial fee was payed.
	#[default]
	Nothing,
	/// Runtime is in bootstrap mode.
	Bootstrap,
	/// The initial fee was payed in the native currency.
	Native(LiquidityInfoOf<T>),
	/// The initial fee was payed in an asset.
	Asset(Credit<T::AccountId, T::Assets>),
}

#[derive(Encode, Decode, Clone, TypeInfo, PartialEq, RuntimeDebugNoBound)]
/// Details of fee payment of which system token used and its amount.
pub struct Detail<T: Config> {
	pub paid_asset_id: ChargeSystemTokenAssetIdOf<T>,
	pub actual_fee: BalanceOf<T>,
	pub converted_fee: AssetBalanceOf<T>,
	pub tip: Option<AssetBalanceOf<T>>,
}
