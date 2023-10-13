use sp_runtime::types::ExtrinsicMetadata;

/// Fee API.
/// Getting fee from fee table
pub trait FeeTableProvider<Balance> {
	fn get_fee_from_fee_table(key: ExtrinsicMetadata) -> Option<Balance>;
}

impl<Balance> FeeTableProvider<Balance> for () {
	fn get_fee_from_fee_table(_key: ExtrinsicMetadata) -> Option<Balance> {
		None
	}
}
