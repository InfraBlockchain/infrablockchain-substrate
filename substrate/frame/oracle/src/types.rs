pub use super::*;

pub const LOG_TARGET: &str = "runtime::system-token-helper";
pub const API_END_POINT: &str =
	"https://v6.exchangerate-api.com/v6/b17c41b872d0b8a2efd77e08/latest/USD";

pub trait SystemTokenOracleInterface {
	/// Send exchange rates of the currencies to Relay-chain at the given standard time.
	fn exchange_rates_at(
		standard_time: StandardUnixTime,
		exchange_rates: Vec<(Fiat, ExchangeRate)>,
	);
}

impl SystemTokenOracleInterface for () {
	fn exchange_rates_at(
		_standard_time: StandardUnixTime,
		_exchange_rates: Vec<(Fiat, ExchangeRate)>,
	) {
	}
}
