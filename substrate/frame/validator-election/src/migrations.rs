
use super::*;
use frame_support::traits::{GetStorageVersion, OnRuntimeUpgrade};

pub mod v1 {
    use super::*;
    use frame_support::pallet_prelude::Weight;

    pub struct MigrationToV1<T>(sp_std::marker::PhantomData<T>);

    impl<T: Config> OnRuntimeUpgrade for MigrationToV1<T> {

        fn on_runtime_upgrade() -> Weight {
            let expected_next_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();
            if onchain_version == 0 && expected_next_version == 1 {
                let seed_trust_num = NumberOfSeedTrustValidators::<T>::get();
                let total_num = TotalNumberOfValidators::<T>::get();
                SeedTrustSlots::<T>::put(seed_trust_num);
                TotalValidatorSlots::<T>::put(total_num);
                NumberOfSeedTrustValidators::<T>::kill();
                TotalNumberOfValidators::<T>::kill();
                // Put next version to Pallet storage.
                expected_next_version.put::<Pallet<T>>();
                log::info!(
					target: LOG_TARGET,
					"Upgraded: storage to version {:?}",
					expected_next_version
				);
                T::DbWeight::get().reads_writes(2,2)
            } else {
                log::info!(
					target: LOG_TARGET,
					"Migration did not execute. This probably should be removed"
				);
				T::DbWeight::get().reads(1)
            }
        }
    }
}