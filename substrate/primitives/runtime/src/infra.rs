//! InfraBlockchain primitives types

use crate::*;
use sp_std::vec::Vec;

pub use config::*;
pub use token::*;
pub use voting::*;

/// Module that defines configuration related
pub mod config {

	use super::*;

	/// System configuration for Infra-* Runtime
	#[derive(
		Encode,
		Decode,
		Clone,
		PartialEq,
		Eq,
		Default,
		RuntimeDebug,
		TypeInfo,
		MaxEncodedLen,
		serde::Serialize,
		serde::Deserialize,
	)]
	pub struct SystemConfig {
		/// Detail of base system token
		pub base_system_token_detail: BaseSystemTokenDetail,
		/// Scale of weight for calculating tx fee
		pub weight_scale: u128,
		/// Base fee rate for para_fee_rate
		pub base_para_fee_rate: u128,
	}

	/// Error type while initializing system configuration
	#[derive(RuntimeDebug)]
	pub enum InitError {
		/// Base system token is not initialized
		InvalidBaseSystemTokenDetail,
		/// Weight scale is not initialized
		InvalidWeightScale,
	}

	impl SystemConfig {
		/// Check if system configuration is valid
		pub fn check_validity(&self) -> Result<(), InitError> {
			if self.base_system_token_detail.base_weight == 0 {
				return Err(InitError::InvalidBaseSystemTokenDetail)
			}
			if self.weight_scale == 0 {
				return Err(InitError::InvalidWeightScale)
			}
			Ok(())
		}

		/// Panic if system configuration is not valid
		pub fn panic_if_not_validated(&self) {
			if let Err(err) = self.check_validity() {
				panic!("System configuration is not initalized: {:?}\nSCfg:\n{:#?}", err, self);
			}
		}
	}
	#[derive(
		Encode,
		Decode,
		Clone,
		PartialEq,
		Eq,
		Default,
		RuntimeDebug,
		TypeInfo,
		MaxEncodedLen,
		serde::Serialize,
		serde::Deserialize,
	)]
	/// Detail of base system token
	pub struct BaseSystemTokenDetail {
		/// Currency type of base system token
		pub base_currency: Fiat,
		/// Weight of base system token
		pub base_weight: u128,
		/// Decimal of base system token
		pub base_decimals: u8,
	}

	impl BaseSystemTokenDetail {
		/// Create new instance of base system token detail
		pub fn new(fiat: Fiat, base_weight: u128, decimals: u8) -> Self {
			Self { base_currency: fiat, base_weight, base_decimals: decimals }
		}
	}

	/// API for providing Infra-* Runtime configuration
	pub trait RuntimeConfigProvider<Balance> {
		/// General error type
		type Error;

		/// System configuration
		fn system_config() -> Result<SystemConfig, Self::Error>;
		/// Para fee rate of Infra-* Runtime
		fn para_fee_rate() -> Result<Balance, Self::Error>;
		/// Query for tx fee of `ext` extrinsic
		fn fee_for(ext: ExtrinsicMetadata) -> Option<Balance>;
		/// State of Infar-* Runtime
		fn runtime_state() -> Mode;
	}

	use sp_std::vec::Vec;

	#[allow(missing_docs)]
	#[derive(
		Encode, Decode, Eq, Clone, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo, Default,
	)]
	pub enum Mode {
		#[default]
		Bootstrap,
		Normal,
	}

	#[derive(
		Clone,
		Encode,
		Decode,
		Eq,
		PartialEq,
		PartialOrd,
		Ord,
		RuntimeDebug,
		Default,
		TypeInfo,
		Serialize,
		Deserialize,
	)]
	/// We used it for getting fee from fee table.
	pub struct ExtrinsicMetadata {
		pallet_name: Vec<u8>,
		call_name: Vec<u8>,
	}

	impl ExtrinsicMetadata {
		#[allow(missing_docs)]
		pub fn new<Pallet: Encode, Call: Encode>(pallet_name: Pallet, call_name: Call) -> Self {
			Self { pallet_name: pallet_name.encode(), call_name: call_name.encode() }
		}
	}
}

/// Module that defines voting related
pub mod voting {

	use super::*;

	/// Transaction-as-a-Vote
	pub trait TaaV {
		/// Error type while processing vote
		type Error;

		/// Try to decode for given opaque `PoT` and process
		fn process(bytes: &mut Vec<u8>) -> Result<(), Self::Error>;
	}

	/// `Proof-of-Transaction` which may contain `Vote` and **must** contain `Fee` amount
	#[derive(Encode, Decode, Clone, PartialEq, Eq, sp_core::RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Default, Hash))]
	pub struct PoT<Account, AssetId, Amount, Weight> {
		/// Reward amount for each transaction
		pub reward: Reward<AssetId, Amount>,
		/// Amount of vote for `Account` based on `fee_amount`
		pub maybe_vote: Option<Vote<Account, Weight>>,
	}

	/// Reward amount for each transaction
	#[derive(Encode, Decode, Clone, PartialEq, Eq, sp_core::RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Default, Hash))]
	pub struct Reward<AssetId, Amount> {
		/// Which asset is used for fee
		pub asset: AssetId,
		/// Amount of fee
		pub amount: Amount,
	}

	#[derive(Encode, Decode, Clone, PartialEq, Eq, sp_core::RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Default, Hash))]
	/// Single Pot vote type
	pub struct Vote<Account, Weight> {
		/// Subject of the vote
		pub candidate: Account,
		/// Absolute amount of vote based on tx-fee
		pub amount: Weight,
	}

	impl<Account, Weight> Vote<Account, Weight> {
		/// Create new instance of vote
		pub fn new(candidate: Account, amount: Weight) -> Self {
			Self { candidate, amount }
		}
	}
}

/// Module that defines System Token related
pub mod token {
	use super::*;

	// TODO: SystemTokenInterface

	/// General type of unix time
	pub type StandardUnixTime = u64;
	/// General type of exchange rate
	pub type ExchangeRate = u64;
	/// Generale weight type for System Token
	pub type SystemTokenWeight = u128;
	/// General balance type for System Token
	pub type SystemTokenBalance = u128;
	/// General decimal type for System Token
	pub type SystemTokenDecimal = u8;

	/// Reanchor system token
	pub trait ReanchorSystemToken<Location> {
		/// Type of error that should be handled when reanchoring system token
		type Error;
		/// Reanchor `SystemToken` in vote
		fn reanchor_system_token(l: &mut Location) -> Result<(), Self::Error>;
	}

	/// Remote asset metadata for registering
	#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
	#[cfg_attr(feature = "std", derive(Default, Hash))]
	pub struct RemoteAssetMetadata<AssetId, Balance> {
		/// General asset id on Runtime(e.g `MultiLocation`)
		pub asset_id: AssetId,
		/// Human readable name of System Token. Accept unbounded 'Vec<u8>' because it would be
		/// checked `bounded` when initiated
		pub name: Vec<u8>,
		/// Human readable symbol of System Token. Accept unbounded 'Vec<u8>' because it would be
		/// checked `bounded` when initiated
		pub symbol: Vec<u8>,
		/// Currency type of base system token
		pub currency_type: Fiat,
		/// Decimal of base system token
		pub decimals: u8,
		/// Minimum balance of system token
		#[codec(compact)]
		pub min_balance: Balance,
	}

	impl<AssetId, Balance> RemoteAssetMetadata<AssetId, Balance> {
		/// Set asset id of system token
		pub fn set_asset_id(&mut self, asset_id: AssetId) {
			self.asset_id = asset_id;
		}
	}

	/// Currency type of system token
	#[derive(
		Clone,
		Encode,
		Decode,
		Eq,
		PartialEq,
		RuntimeDebug,
		MaxEncodedLen,
		TypeInfo,
		Default,
		serde::Serialize,
		serde::Deserialize,
	)]
	#[cfg_attr(feature = "std", derive(Hash))]
	#[allow(missing_docs)]
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

	impl TryFrom<Fiat> for Vec<u8> {
		type Error = ();

		fn try_from(value: Fiat) -> Result<Self, Self::Error> {
			Ok(match value {
				Fiat::USD => Vec::from(b"USD".as_ref()),
				Fiat::AED => Vec::from(b"AED".as_ref()),
				Fiat::AFN => Vec::from(b"AFN".as_ref()),
				Fiat::ALL => Vec::from(b"ALL".as_ref()),
				Fiat::AMD => Vec::from(b"AMD".as_ref()),
				Fiat::ANG => Vec::from(b"ANG".as_ref()),
				Fiat::AOA => Vec::from(b"AOA".as_ref()),
				Fiat::ARS => Vec::from(b"ARS".as_ref()),
				Fiat::AUD => Vec::from(b"AUD".as_ref()),
				Fiat::AWG => Vec::from(b"AWG".as_ref()),
				Fiat::AZN => Vec::from(b"AZN".as_ref()),
				Fiat::BAM => Vec::from(b"BAM".as_ref()),
				Fiat::BBD => Vec::from(b"BBD".as_ref()),
				Fiat::BDT => Vec::from(b"BDT".as_ref()),
				Fiat::BGN => Vec::from(b"BGN".as_ref()),
				Fiat::BHD => Vec::from(b"BHD".as_ref()),
				Fiat::BIF => Vec::from(b"BIF".as_ref()),
				Fiat::BMD => Vec::from(b"BMD".as_ref()),
				Fiat::BND => Vec::from(b"BND".as_ref()),
				Fiat::BOB => Vec::from(b"BOB".as_ref()),
				Fiat::BRL => Vec::from(b"BRL".as_ref()),
				Fiat::BSD => Vec::from(b"BSD".as_ref()),
				Fiat::BTN => Vec::from(b"BTN".as_ref()),
				Fiat::BWP => Vec::from(b"BWP".as_ref()),
				Fiat::BYN => Vec::from(b"BYN".as_ref()),
				Fiat::BZD => Vec::from(b"BZD".as_ref()),
				Fiat::CAD => Vec::from(b"CAD".as_ref()),
				Fiat::CDF => Vec::from(b"CDF".as_ref()),
				Fiat::CHF => Vec::from(b"CHF".as_ref()),
				Fiat::CLP => Vec::from(b"CLP".as_ref()),
				Fiat::CNY => Vec::from(b"CNY".as_ref()),
				Fiat::COP => Vec::from(b"COP".as_ref()),
				Fiat::CRC => Vec::from(b"CRC".as_ref()),
				Fiat::CUP => Vec::from(b"CUP".as_ref()),
				Fiat::CVE => Vec::from(b"CVE".as_ref()),
				Fiat::CZK => Vec::from(b"CZK".as_ref()),
				Fiat::DJF => Vec::from(b"DJF".as_ref()),
				Fiat::DKK => Vec::from(b"DKK".as_ref()),
				Fiat::DOP => Vec::from(b"DOP".as_ref()),
				Fiat::DZD => Vec::from(b"DZD".as_ref()),
				Fiat::EGP => Vec::from(b"EGP".as_ref()),
				Fiat::ERN => Vec::from(b"ERN".as_ref()),
				Fiat::ETB => Vec::from(b"ETB".as_ref()),
				Fiat::EUR => Vec::from(b"EUR".as_ref()),
				Fiat::FJD => Vec::from(b"FJD".as_ref()),
				Fiat::FKP => Vec::from(b"FKP".as_ref()),
				Fiat::FOK => Vec::from(b"FOK".as_ref()),
				Fiat::GBP => Vec::from(b"GBP".as_ref()),
				Fiat::GEL => Vec::from(b"GEL".as_ref()),
				Fiat::GGP => Vec::from(b"GGP".as_ref()),
				Fiat::GHS => Vec::from(b"GHS".as_ref()),
				Fiat::GIP => Vec::from(b"GIP".as_ref()),
				Fiat::GMD => Vec::from(b"GMD".as_ref()),
				Fiat::GNF => Vec::from(b"GNF".as_ref()),
				Fiat::GTQ => Vec::from(b"GTQ".as_ref()),
				Fiat::GYD => Vec::from(b"GYD".as_ref()),
				Fiat::HKD => Vec::from(b"HKD".as_ref()),
				Fiat::HNL => Vec::from(b"HNL".as_ref()),
				Fiat::HRK => Vec::from(b"HRK".as_ref()),
				Fiat::HTG => Vec::from(b"HTG".as_ref()),
				Fiat::HUF => Vec::from(b"HUF".as_ref()),
				Fiat::IDR => Vec::from(b"IDR".as_ref()),
				Fiat::ILS => Vec::from(b"ILS".as_ref()),
				Fiat::IMP => Vec::from(b"IMP".as_ref()),
				Fiat::INR => Vec::from(b"INR".as_ref()),
				Fiat::IQD => Vec::from(b"IQD".as_ref()),
				Fiat::IRR => Vec::from(b"IRR".as_ref()),
				Fiat::ISK => Vec::from(b"ISK".as_ref()),
				Fiat::JEP => Vec::from(b"JEP".as_ref()),
				Fiat::JMD => Vec::from(b"JMD".as_ref()),
				Fiat::JOD => Vec::from(b"JOD".as_ref()),
				Fiat::JPY => Vec::from(b"JPY".as_ref()),
				Fiat::KES => Vec::from(b"KES".as_ref()),
				Fiat::KGS => Vec::from(b"KGS".as_ref()),
				Fiat::KHR => Vec::from(b"KHR".as_ref()),
				Fiat::KID => Vec::from(b"KID".as_ref()),
				Fiat::KMF => Vec::from(b"KMF".as_ref()),
				Fiat::KRW => Vec::from(b"KRW".as_ref()),
				Fiat::KWD => Vec::from(b"KWD".as_ref()),
				Fiat::KYD => Vec::from(b"KYD".as_ref()),
				Fiat::KZT => Vec::from(b"KZT".as_ref()),
				Fiat::LAK => Vec::from(b"LAK".as_ref()),
				Fiat::LBP => Vec::from(b"LBP".as_ref()),
				Fiat::LKR => Vec::from(b"LKR".as_ref()),
				Fiat::LRD => Vec::from(b"LRD".as_ref()),
				Fiat::LSL => Vec::from(b"LSL".as_ref()),
				Fiat::LYD => Vec::from(b"LYD".as_ref()),
				Fiat::MAD => Vec::from(b"MAD".as_ref()),
				Fiat::MDL => Vec::from(b"MDL".as_ref()),
				Fiat::MGA => Vec::from(b"MGA".as_ref()),
				Fiat::MKD => Vec::from(b"MKD".as_ref()),
				Fiat::MMK => Vec::from(b"MMK".as_ref()),
				Fiat::MNT => Vec::from(b"MNT".as_ref()),
				Fiat::MOP => Vec::from(b"MOP".as_ref()),
				Fiat::MRU => Vec::from(b"MRU".as_ref()),
				Fiat::MUR => Vec::from(b"MUR".as_ref()),
				Fiat::MVR => Vec::from(b"MVR".as_ref()),
				Fiat::MWK => Vec::from(b"MWK".as_ref()),
				Fiat::MXN => Vec::from(b"MXN".as_ref()),
				Fiat::MYR => Vec::from(b"MYR".as_ref()),
				Fiat::MZN => Vec::from(b"MZN".as_ref()),
				Fiat::NAD => Vec::from(b"NAD".as_ref()),
				Fiat::NGN => Vec::from(b"NGN".as_ref()),
				Fiat::NIO => Vec::from(b"NIO".as_ref()),
				Fiat::NOK => Vec::from(b"NOK".as_ref()),
				Fiat::NPR => Vec::from(b"NPR".as_ref()),
				Fiat::NZD => Vec::from(b"NZD".as_ref()),
				Fiat::OMR => Vec::from(b"OMR".as_ref()),
				Fiat::PAB => Vec::from(b"PAB".as_ref()),
				Fiat::PEN => Vec::from(b"PEN".as_ref()),
				Fiat::PGK => Vec::from(b"PGK".as_ref()),
				Fiat::PHP => Vec::from(b"PHP".as_ref()),
				Fiat::PKR => Vec::from(b"PKR".as_ref()),
				Fiat::PLN => Vec::from(b"PLN".as_ref()),
				Fiat::PYG => Vec::from(b"PYG".as_ref()),
				Fiat::QAR => Vec::from(b"QAR".as_ref()),
				Fiat::RON => Vec::from(b"RON".as_ref()),
				Fiat::RSD => Vec::from(b"RSD".as_ref()),
				Fiat::RUB => Vec::from(b"RUB".as_ref()),
				Fiat::RWF => Vec::from(b"RWF".as_ref()),
				Fiat::SAR => Vec::from(b"SAR".as_ref()),
				Fiat::SBD => Vec::from(b"SBD".as_ref()),
				Fiat::SCR => Vec::from(b"SCR".as_ref()),
				Fiat::SDG => Vec::from(b"SDG".as_ref()),
				Fiat::SEK => Vec::from(b"SEK".as_ref()),
				Fiat::SGD => Vec::from(b"SGD".as_ref()),
				Fiat::SHP => Vec::from(b"SHP".as_ref()),
				Fiat::SLE => Vec::from(b"SLE".as_ref()),
				Fiat::SLL => Vec::from(b"SLL".as_ref()),
				Fiat::SOS => Vec::from(b"SOS".as_ref()),
				Fiat::SRD => Vec::from(b"SRD".as_ref()),
				Fiat::SSP => Vec::from(b"SSP".as_ref()),
				Fiat::STN => Vec::from(b"STN".as_ref()),
				Fiat::SYP => Vec::from(b"SYP".as_ref()),
				Fiat::SZL => Vec::from(b"SZL".as_ref()),
				Fiat::THB => Vec::from(b"THB".as_ref()),
				Fiat::TJS => Vec::from(b"TJS".as_ref()),
				Fiat::TMT => Vec::from(b"TMT".as_ref()),
				Fiat::TND => Vec::from(b"TND".as_ref()),
				Fiat::TOP => Vec::from(b"TOP".as_ref()),
				Fiat::TRY => Vec::from(b"TRY".as_ref()),
				Fiat::TTD => Vec::from(b"TTD".as_ref()),
				Fiat::TVD => Vec::from(b"TVD".as_ref()),
				Fiat::TWD => Vec::from(b"TWD".as_ref()),
				Fiat::TZS => Vec::from(b"TZS".as_ref()),
				Fiat::UAH => Vec::from(b"UAH".as_ref()),
				Fiat::UGX => Vec::from(b"UGX".as_ref()),
				Fiat::UYU => Vec::from(b"UYU".as_ref()),
				Fiat::UZS => Vec::from(b"UZS".as_ref()),
				Fiat::VES => Vec::from(b"VES".as_ref()),
				Fiat::VND => Vec::from(b"VND".as_ref()),
				Fiat::VUV => Vec::from(b"VUV".as_ref()),
				Fiat::WST => Vec::from(b"WST".as_ref()),
				Fiat::XAF => Vec::from(b"XAF".as_ref()),
				Fiat::XCD => Vec::from(b"XCD".as_ref()),
				Fiat::XDR => Vec::from(b"XDR".as_ref()),
				Fiat::XOF => Vec::from(b"XOF".as_ref()),
				Fiat::XPF => Vec::from(b"XPF".as_ref()),
				Fiat::YER => Vec::from(b"YER".as_ref()),
				Fiat::ZAR => Vec::from(b"ZAR".as_ref()),
				Fiat::ZMW => Vec::from(b"ZMW".as_ref()),
				Fiat::ZWL => Vec::from(b"ZWL".as_ref()),
			})
		}
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
}
