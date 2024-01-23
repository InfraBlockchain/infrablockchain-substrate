//! Module that defines System Token type

use crate::{
	codec::{Decode, Encode, MaxEncodedLen},
	scale_info::TypeInfo,
	types::vote::*,
	RuntimeDebug,
};
use bounded_collections::{ConstU32, BoundedVec};
use sp_std::prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// ParaId of Relay Chain
pub const RELAY_CHAIN_PARA_ID: SystemTokenParaId = 0;

/// General type of unix time
pub type StandardUnixTime = u64;
/// General type of exchange rate
pub type ExchangeRate = u64;
/// General para id type for System Token
pub type SystemTokenParaId = u32;
/// General pallet id type for System Token
pub type SystemTokenPalletId = u8;
/// General asset id type for System Token
pub type SystemTokenAssetId = u32;
/// Generale weight type for System Token
pub type SystemTokenWeight = u128;
/// General balance type for System Token
pub type SystemTokenBalance = u128;
/// General decimal type for System Token
pub type SystemTokenDecimal = u8;
/// Bounded name for System Token
pub type BoundedSystemTokenName = BoundedVec<u8, ConstU32<20>>;
/// Bounded symbol for System Token
pub type BoundedSystemTokenSymbol = BoundedVec<u8, ConstU32<5>>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
/// Detail of base system token
pub struct BaseSystemTokenDetail {
	/// Currency type of base system token
	pub currency: Fiat,
	/// Weight of base system token
	pub weight: SystemTokenWeight,
	/// Decimal of base system token
	pub decimal: u8,
}

impl Default for BaseSystemTokenDetail {
	fn default() -> Self {
		Self {
			currency: Fiat::USD,
			weight: 1_000_000,
			decimal: 4,
		}
	}
}

/// Data structure for Original system tokens
#[derive(
	Clone,
	Encode,
	Decode,
	Copy,
	Eq,
	PartialEq,
	PartialOrd,
	Ord,
	RuntimeDebug,
	Default,
	TypeInfo,
	MaxEncodedLen,
)]
#[cfg_attr(feature = "std", derive(Hash, Serialize, Deserialize))]
pub struct SystemTokenId {
	/// ParaId where to use the system token. Especially, we assigned the relaychain as ParaID = 0
	#[codec(compact)]
	pub para_id: SystemTokenParaId,
	/// PalletId on the parachain where to use the system token
	#[codec(compact)]
	pub pallet_id: SystemTokenPalletId,
	/// AssetId on the parachain where to use the system token
	#[codec(compact)]
	pub asset_id: SystemTokenAssetId,
}

impl SystemTokenId {
	/// Create new instance of `SystemTokenId`
	pub fn new(para_id: u32, pallet_id: u8, asset_id: SystemTokenAssetId) -> Self {
		Self { para_id, pallet_id, asset_id }
	}

	/// Clone `self` and return new instance of `SystemTokenId`
	pub fn asset_id(&self) -> SystemTokenAssetId {
		self.clone().asset_id
	}
}

pub const MAX_REQUESTED_ASSETS: u32 = 1;
/// Upper limit of number of assets to be requested
pub type BoundedRequestedAssets = BoundedVec<RemoteAssetMetadata, ConstU32<MAX_REQUESTED_ASSETS>>;

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Default, Hash))]
pub struct RemoteAssetMetadata {
	/// General Assets pallet index on Runtime
	pub pallet_id: SystemTokenPalletId,
	/// General asset id on Runtime
	#[codec(compact)]
	pub asset_id: SystemTokenAssetId,
	/// Human readable name of System Token, which should be bounded
	pub name: BoundedSystemTokenName,
	/// Human readable symbol of System Token, which should be bounded
	pub symbol: BoundedSystemTokenSymbol,
	/// Currency type of base system token
	pub currency_type: Fiat,
	/// Decimal of base system token
	pub decimals: u8,
	/// Weight of base system token
	#[codec(compact)]
	pub min_balance: SystemTokenBalance,
}

impl MaxEncodedLen for RemoteAssetMetadata {
	fn max_encoded_len() -> usize {
		SystemTokenPalletId::max_encoded_len() 
		+ SystemTokenAssetId::max_encoded_len()
		+ BoundedSystemTokenSymbol::max_encoded_len()
		+ BoundedSystemTokenName::max_encoded_len()
		+ Fiat::max_encoded_len() 
		+ u8::max_encoded_len()
		+ SystemTokenBalance::max_encoded_len()
	}
}

/// API for interacting with local assets on Runtime
pub trait LocalAssetProvider<Asset, Account> {
	/// Get a list of local assets created on local chain
	fn system_token_list() -> Vec<Asset>;
	/// Get the most account balance of given `asset_id`
	fn get_most_account_system_token_balance(
		asset_ids: impl IntoIterator<Item = Asset>,
		account: Account,
	) -> Asset;
}

/// API to handle local assets which refers to System Token
pub trait LocalAssetManager {
	
	type AccountId: MaxEncodedLen;
	type Error;

	/// Create local asset with metadata which refers to `wrapped` System Token
	fn create_wrapped_local(
		asset_id: SystemTokenAssetId,
		currency_type: Fiat,
		min_balance: SystemTokenBalance,
		name: Vec<u8>,
		symbol: Vec<u8>,
		decimals: u8,
		system_token_weight: SystemTokenWeight,
	) -> Result<(), Self::Error>;
	/// Promote local asset to System Token when registered(e.g `is_sufficient` to `true`)
	fn promote(
		asset_id: SystemTokenAssetId,
		system_token_weight: SystemTokenWeight,
	) -> Result<(), Self::Error>;
	/// Demote System Token to local asset(e.g `is_sufficient` to `false`)
	fn demote(asset_id: SystemTokenAssetId) -> Result<(), Self::Error>;
	/// Update weight of System Token(e.g Exhange rate has been changed)
	fn update_system_token_weight(
		asset_id: SystemTokenAssetId,
		system_token_weight: SystemTokenWeight,
	) -> Result<(), Self::Error>;
	/// Request register System Token 
	fn request_register(asset_id: SystemTokenAssetId) -> Result<(), Self::Error>;
	/// Get a list of System Token's local asset id
	fn system_token_list() -> Vec<SystemTokenAssetId> { 
		Vec::new()
	}
	/// Return most system token balance of given 'asset_id' and 'account'
	fn get_most_system_token_balance_of(
		asset_ids: impl IntoIterator<Item = SystemTokenAssetId>,
		account: Self::AccountId,
	) -> SystemTokenAssetId;

	/// Retrieve metadata of given `asset_id` and return `RemoteAssetMetadata`, which is for Relay-chain
	fn get_metadata(
		asset_id: SystemTokenAssetId,
	) -> Result<RemoteAssetMetadata, Self::Error>;
}

pub trait AssetMetadataProvider {
	fn requested(assets: Vec<RemoteAssetMetadata>);
}

/// API for interacting with registered System Token
pub trait SystemTokenInterface {
	/// Check the system token is registered.
	fn is_system_token(system_token: &SystemTokenId) -> bool;
	/// Convert para system token to original system token.
	fn convert_to_original_system_token(wrapped_token: &SystemTokenId) -> Option<SystemTokenId>;
	/// Adjust the vote weight calculating exchange rate.
	fn adjusted_weight(system_token: &SystemTokenId, vote_weight: VoteWeight) -> VoteWeight;
	/// Update the metadata for requested asset received from enshirned chain
	fn requested_asset_metadata(para_id: SystemTokenParaId, maybe_requested_assets: Option<BoundedRequestedAssets>);
}

impl SystemTokenInterface for () {
	fn is_system_token(_system_token: &SystemTokenId) -> bool {
		false
	}
	fn convert_to_original_system_token(_wrapped_token: &SystemTokenId) -> Option<SystemTokenId> {
		None
	}
	fn adjusted_weight(_system_token: &SystemTokenId, _vote_weight: VoteWeight) -> VoteWeight {
		Default::default()
	}
	fn requested_asset_metadata(_para_id: SystemTokenParaId, _maybe_requested_assets: Option<BoundedRequestedAssets>) { }
}

pub trait AssetLinkInterface<AssetId> {
	type Error;

	fn link(asset_id: &AssetId, parents: u8, original: SystemTokenId) -> Result<(), Self::Error>;
	fn unlink(asset_id: &AssetId) -> Result<(), Self::Error>;
}

impl<AssetId> AssetLinkInterface<AssetId> for () {
	type Error = ();

	fn link(
		_asset_id: &AssetId,
		_parents: u8,
		_original: SystemTokenId,
	) -> Result<(), Self::Error> {
		Ok(())
	}
	fn unlink(_asset_id: &AssetId) -> Result<(), Self::Error> {
		Ok(())
	}
}


#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo, Default, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "std", derive(Hash))]
pub enum Fiat {
	#[default]
	USD,
	AED,
	AFN,
	ALL,
	AMD,
	ANG,
	AOA,
	ARS,
	AUD,
	AWG,
	AZN,
	BAM,
	BBD,
	BDT,
	BGN,
	BHD,
	BIF,
	BMD,
	BND,
	BOB,
	BRL,
	BSD,
	BTN,
	BWP,
	BYN,
	BZD,
	CAD,
	CDF,
	CHF,
	CLP,
	CNY,
	COP,
	CRC,
	CUP,
	CVE,
	CZK,
	DJF,
	DKK,
	DOP,
	DZD,
	EGP,
	ERN,
	ETB,
	EUR,
	FJD,
	FKP,
	FOK,
	GBP,
	GEL,
	GGP,
	GHS,
	GIP,
	GMD,
	GNF,
	GTQ,
	GYD,
	HKD,
	HNL,
	HRK,
	HTG,
	HUF,
	IDR,
	ILS,
	IMP,
	INR,
	IQD,
	IRR,
	ISK,
	JEP,
	JMD,
	JOD,
	JPY,
	KES,
	KGS,
	KHR,
	KID,
	KMF,
	KRW,
	KWD,
	KYD,
	KZT,
	LAK,
	LBP,
	LKR,
	LRD,
	LSL,
	LYD,
	MAD,
	MDL,
	MGA,
	MKD,
	MMK,
	MNT,
	MOP,
	MRU,
	MUR,
	MVR,
	MWK,
	MXN,
	MYR,
	MZN,
	NAD,
	NGN,
	NIO,
	NOK,
	NPR,
	NZD,
	OMR,
	PAB,
	PEN,
	PGK,
	PHP,
	PKR,
	PLN,
	PYG,
	QAR,
	RON,
	RSD,
	RUB,
	RWF,
	SAR,
	SBD,
	SCR,
	SDG,
	SEK,
	SGD,
	SHP,
	SLE,
	SLL,
	SOS,
	SRD,
	SSP,
	STN,
	SYP,
	SZL,
	THB,
	TJS,
	TMT,
	TND,
	TOP,
	TRY,
	TTD,
	TVD,
	TWD,
	TZS,
	UAH,
	UGX,
	UYU,
	UZS,
	VES,
	VND,
	VUV,
	WST,
	XAF,
	XCD,
	XDR,
	XOF,
	XPF,
	YER,
	ZAR,
	ZMW,
	ZWL,
}

impl TryFrom<Vec<u8>> for Fiat {
	type Error = ();

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		Ok(match value.as_slice() {
			b"USD" => Fiat::USD,
			b"AED" => Fiat::AED,
			b"AFN" => Fiat::AFN,
			b"ALL" => Fiat::ALL,
			b"AMD" => Fiat::AMD,
			b"ANG" => Fiat::ANG,
			b"AOA" => Fiat::AOA,
			b"ARS" => Fiat::ARS,
			b"AUD" => Fiat::AUD,
			b"AWG" => Fiat::AWG,
			b"AZN" => Fiat::AZN,
			b"BAM" => Fiat::BAM,
			b"BBD" => Fiat::BBD,
			b"BDT" => Fiat::BDT,
			b"BGN" => Fiat::BGN,
			b"BHD" => Fiat::BHD,
			b"BIF" => Fiat::BIF,
			b"BMD" => Fiat::BMD,
			b"BND" => Fiat::BND,
			b"BOB" => Fiat::BOB,
			b"BRL" => Fiat::BRL,
			b"BSD" => Fiat::BSD,
			b"BTN" => Fiat::BTN,
			b"BWP" => Fiat::BWP,
			b"BYN" => Fiat::BYN,
			b"BZD" => Fiat::BZD,
			b"CAD" => Fiat::CAD,
			b"CDF" => Fiat::CDF,
			b"CHF" => Fiat::CHF,
			b"CLP" => Fiat::CLP,
			b"CNY" => Fiat::CNY,
			b"COP" => Fiat::COP,
			b"CRC" => Fiat::CRC,
			b"CUP" => Fiat::CUP,
			b"CVE" => Fiat::CVE,
			b"CZK" => Fiat::CZK,
			b"DJF" => Fiat::DJF,
			b"DKK" => Fiat::DKK,
			b"DOP" => Fiat::DOP,
			b"DZD" => Fiat::DZD,
			b"EGP" => Fiat::EGP,
			b"ERN" => Fiat::ERN,
			b"ETB" => Fiat::ETB,
			b"EUR" => Fiat::EUR,
			b"FJD" => Fiat::FJD,
			b"FKP" => Fiat::FKP,
			b"FOK" => Fiat::FOK,
			b"GBP" => Fiat::GBP,
			b"GEL" => Fiat::GEL,
			b"GGP" => Fiat::GGP,
			b"GHS" => Fiat::GHS,
			b"GIP" => Fiat::GIP,
			b"GMD" => Fiat::GMD,
			b"GNF" => Fiat::GNF,
			b"GTQ" => Fiat::GTQ,
			b"GYD" => Fiat::GYD,
			b"HKD" => Fiat::HKD,
			b"HNL" => Fiat::HNL,
			b"HRK" => Fiat::HRK,
			b"HTG" => Fiat::HTG,
			b"HUF" => Fiat::HUF,
			b"IDR" => Fiat::IDR,
			b"ILS" => Fiat::ILS,
			b"IMP" => Fiat::IMP,
			b"INR" => Fiat::INR,
			b"IQD" => Fiat::IQD,
			b"IRR" => Fiat::IRR,
			b"ISK" => Fiat::ISK,
			b"JEP" => Fiat::JEP,
			b"JMD" => Fiat::JMD,
			b"JOD" => Fiat::JOD,
			b"JPY" => Fiat::JPY,
			b"KES" => Fiat::KES,
			b"KGS" => Fiat::KGS,
			b"KHR" => Fiat::KHR,
			b"KID" => Fiat::KID,
			b"KMF" => Fiat::KMF,
			b"KRW" => Fiat::KRW,
			b"KWD" => Fiat::KWD,
			b"KYD" => Fiat::KYD,
			b"KZT" => Fiat::KZT,
			b"LAK" => Fiat::LAK,
			b"LBP" => Fiat::LBP,
			b"LKR" => Fiat::LKR,
			b"LRD" => Fiat::LRD,
			b"LSL" => Fiat::LSL,
			b"LYD" => Fiat::LYD,
			b"MAD" => Fiat::MAD,
			b"MDL" => Fiat::MDL,
			b"MGA" => Fiat::MGA,
			b"MKD" => Fiat::MKD,
			b"MMK" => Fiat::MMK,
			b"MNT" => Fiat::MNT,
			b"MOP" => Fiat::MOP,
			b"MRU" => Fiat::MRU,
			b"MUR" => Fiat::MUR,
			b"MVR" => Fiat::MVR,
			b"MWK" => Fiat::MWK,
			b"MXN" => Fiat::MXN,
			b"MYR" => Fiat::MYR,
			b"MZN" => Fiat::MZN,
			b"NAD" => Fiat::NAD,
			b"NGN" => Fiat::NGN,
			b"NIO" => Fiat::NIO,
			b"NOK" => Fiat::NOK,
			b"NPR" => Fiat::NPR,
			b"NZD" => Fiat::NZD,
			b"OMR" => Fiat::OMR,
			b"PAB" => Fiat::PAB,
			b"PEN" => Fiat::PEN,
			b"PGK" => Fiat::PGK,
			b"PHP" => Fiat::PHP,
			b"PKR" => Fiat::PKR,
			b"PLN" => Fiat::PLN,
			b"PYG" => Fiat::PYG,
			b"QAR" => Fiat::QAR,
			b"RON" => Fiat::RON,
			b"RSD" => Fiat::RSD,
			b"RUB" => Fiat::RUB,
			b"RWF" => Fiat::RWF,
			b"SAR" => Fiat::SAR,
			b"SBD" => Fiat::SBD,
			b"SCR" => Fiat::SCR,
			b"SDG" => Fiat::SDG,
			b"SEK" => Fiat::SEK,
			b"SGD" => Fiat::SGD,
			b"SHP" => Fiat::SHP,
			b"SLE" => Fiat::SLE,
			b"SLL" => Fiat::SLL,
			b"SOS" => Fiat::SOS,
			b"SRD" => Fiat::SRD,
			b"SSP" => Fiat::SSP,
			b"STN" => Fiat::STN,
			b"SYP" => Fiat::SYP,
			b"SZL" => Fiat::SZL,
			b"THB" => Fiat::THB,
			b"TJS" => Fiat::TJS,
			b"TMT" => Fiat::TMT,
			b"TND" => Fiat::TND,
			b"TOP" => Fiat::TOP,
			b"TRY" => Fiat::TRY,
			b"TTD" => Fiat::TTD,
			b"TVD" => Fiat::TVD,
			b"TWD" => Fiat::TWD,
			b"TZS" => Fiat::TZS,
			b"UAH" => Fiat::UAH,
			b"UGX" => Fiat::UGX,
			b"UYU" => Fiat::UYU,
			b"UZS" => Fiat::UZS,
			b"VES" => Fiat::VES,
			b"VND" => Fiat::VND,
			b"VUV" => Fiat::VUV,
			b"WST" => Fiat::WST,
			b"XAF" => Fiat::XAF,
			b"XCD" => Fiat::XCD,
			b"XDR" => Fiat::XDR,
			b"XOF" => Fiat::XOF,
			b"XPF" => Fiat::XPF,
			b"YER" => Fiat::YER,
			b"ZAR" => Fiat::ZAR,
			b"ZMW" => Fiat::ZMW,
			b"ZWL" => Fiat::ZWL,
			_ => return Err(()),
		})
	}
}

