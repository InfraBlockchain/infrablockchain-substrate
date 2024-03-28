use super::*;

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
pub(crate) type SystemTokenBalanceOf<T> =
	<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::Balance;
/// Asset id type alias
pub(crate) type SystemTokenAssetIdOf<T> =
	<<T as Config>::Fungibles as Inspect<<T as frame_system::Config>::AccountId>>::AssetId;

/// SystemTokenWeight type alias
pub(crate) type SystemTokenWeightOf<T> = <<T as Config>::Fungibles as InspectSystemToken<
	<T as frame_system::Config>::AccountId,
>>::SystemTokenWeight;

// Type aliases used for interaction with `OnChargeAssetTransaction`.
// Balance type alias.
pub(crate) type ChargeSystemTokenBalanceOf<T> =
	<<T as Config>::OnChargeSystemToken as OnChargeSystemToken<T>>::Balance;

pub(crate) type ChargeSystemTokenAssetIdOf<T> =
	<<T as Config>::OnChargeSystemToken as OnChargeSystemToken<T>>::AssetId;

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
	Asset(Credit<T::AccountId, T::Fungibles>),
}

#[derive(Encode, Decode, Clone, TypeInfo, PartialEq, RuntimeDebug)]
/// Details of fee payment of which system token used and its amount.
pub struct Detail<ChargeAsset, Balance, AssetBalance> {
	pub paid_asset_id: ChargeAsset,
	pub actual_fee: Balance,
	pub converted_fee: AssetBalance,
	pub tip: Option<AssetBalance>,
}
