use super::*;
use frame_support::traits::OnRuntimeUpgrade;

pub mod v1 {
	use super::*;
	use frame_support::pallet_prelude::Weight;

	pub struct MigrationToV1<T>(sp_std::marker::PhantomData<T>);

	impl<T: Config> OnRuntimeUpgrade for MigrationToV1<T> {
		fn on_runtime_upgrade() -> Weight {
			log::info!(
				target: LOG_TARGET,
				"Migration did not execute. This probably should be removed"
			);
			T::DbWeight::get().reads(1)
		}
	}
}
