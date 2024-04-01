pub use super::*;

pub const LOG_TARGET: &str = "runtime::system-token-helper";
// TODO: Remove
pub const API_END_POINT: &str =
	"https://v6.exchangerate-api.com/v6/b56c27b3b77e4cb53379e1ac/pair/";

pub trait SystemTokenOracleInterface {
	/// Send exchange rates of the currencies to Relay-chain at the given standard time.
	fn submit_exchange_rates(exchange_rates: Vec<(Fiat, ExchangeRate)>);
}

impl SystemTokenOracleInterface for () {
	fn submit_exchange_rates(_exchange_rates: Vec<(Fiat, ExchangeRate)>) {}
}
