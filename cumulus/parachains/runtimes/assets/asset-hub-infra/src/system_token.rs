
use crate::*;
use codec::{Decode, Encode};
use pallet_system_token::{StandardUnixTime, Fiat, ExchangeRate, SystemTokenOracleInterface};
use xcm::latest::prelude::*;

/// A type containing the encoding of the system token manager pallet in the Relay chain runtime. Used to
/// construct any remote calls. The codec index must correspond to the index of `SystemTokenManager` in the
/// `construct_runtime` of the Relay Chain.
#[derive(Encode, Decode)]
enum RelayRuntimePallets {
    #[codec(index = 21)]
    SystemTokenManager(SystemTokenManagerCalls)
}

/// Call encoding for the calls needed from the relay system token manager pallet.
#[derive(Encode, Decode)]
enum SystemTokenManagerCalls {
    #[codec(index = 7)]
    SetExchangeRates(
        StandardUnixTime,
        Vec<(Fiat, ExchangeRate)>
    )
}

/// Type that implements `SystemTokenOracleInterface`.
pub struct SystemTokenOracle;
impl SystemTokenOracleInterface for SystemTokenOracle {
    fn exchange_rates_at(standard_time: StandardUnixTime, exchange_rates: Vec<(Fiat, ExchangeRate)>) {
        use crate::system_token::SystemTokenManagerCalls::SetExchangeRates;
        let set_exchange_rate_call = RelayRuntimePallets::SystemTokenManager(SetExchangeRates(standard_time, exchange_rates));
        let message = Xcm(vec![
            Instruction::UnpaidExecution {
				weight_limit: WeightLimit::Unlimited,
				check_origin: None,
			},
			Instruction::Transact {
				origin_kind: OriginKind::Native,
				require_weight_at_most: Weight::from_parts(1_000_000_000, 200000),
				call: set_exchange_rate_call.encode().into(),
			},
        ]);

        match InfraXcm::send_xcm(Here, MultiLocation::parent(), message.clone()) {
			Ok(_) => log::info!(
				target: "runtime::system-token",
				"Instruction to `exchange rate` sent successfully."
			),
			Err(e) => log::error!(
				target: "runtime::system-token",
				"Instruction to `exchange rate` failed to send: {:?}",
				e
			),
		}
    }
}

