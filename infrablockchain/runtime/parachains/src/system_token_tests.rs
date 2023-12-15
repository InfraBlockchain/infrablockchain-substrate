use super::*;
use crate::{
	mock::*,
	system_token_manager::{Error as SystemTokenManagerError, Event as SystemTokenManagerEvent, *},
};

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	let _ = pallet_balances::GenesisConfig::<Test> { balances: vec![(0, 100), (1, 98), (2, 1)] }
		.assimilate_storage(&mut storage);

	let mut ext: sp_io::TestExternalities = storage.into();
	ext.execute_with(|| System::set_block_number(1)); // For 'Event'
	ext
}

fn sys_token(
	para_id: sp_runtime::types::ParaId,
	pallet_id: sp_runtime::types::PalletId,
	asset_id: sp_runtime::types::AssetId,
) -> SystemTokenId {
	SystemTokenId { para_id, pallet_id, asset_id }
}

#[test]
fn genesis_works() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balances::free_balance(0), 100u128);
		assert_eq!(Balances::free_balance(1), 98u128);
		assert_eq!(Balances::free_balance(2), 1u128);
	})
}

#[test]
fn only_root_can_call_works() {
	new_test_ext().execute_with(|| {
		// Scenario 1: 'Root(Privileged) origin is required
		assert_noop!(
			SystemTokenManager::register_system_token(
				RuntimeOrigin::signed(1u64),
				SystemTokenId::new(1000, 50, 1),
				SystemTokenId::new(0, 50, 1),
				1_000,
				"BCLABS".into(),
				"BCLABS".into(),
				"BCLABS".into(),
				"BCLABS".into(),
				"IUSD".into(),
				4,
				1_000
			),
			BadOrigin,
		);
	})
}

#[test]
fn register_system_token_works() {
	new_test_ext().execute_with(|| {
		assert_ok!(SystemTokenManager::register_system_token(
			RuntimeOrigin::root(),
			SystemTokenId::new(1000, 50, 1),
			SystemTokenId::new(0, 50, 1),
			1_000,
			"BCLABS".into(),
			"BCLABS".into(),
			"BCLABS".into(),
			"BCLABS".into(),
			"IUSD".into(),
			4,
			1_000
		));

		// Scenario 1: Check `try-register_original`. Check only for possible error case
		let original_1000_50_1 = SystemTokenId::new(1000, 50, 1);
		assert_eq!(
			ParaIdSystemTokens::<Test>::get(&1000).unwrap().to_vec(),
			vec![original_1000_50_1,]
		);

		assert_eq!(
			SystemTokenUsedParaIds::<Test>::get(&original_1000_50_1).unwrap().to_vec(),
			vec![1000u32, 0u32]
		);

		// Scenario 2: Try register 'same' system token id. Should be failed
		assert_noop!(
			SystemTokenManager::register_system_token(
				RuntimeOrigin::root(),
				SystemTokenId::new(1000, 50, 1),
				SystemTokenId::new(0, 50, 1),
				1_000,
				"BCLABS".into(),
				"BCLABS".into(),
				"BCLABS".into(),
				"BCLABS".into(),
				"IUSD".into(),
				4,
				1_000
			),
			SystemTokenManagerError::<Test>::OriginalAlreadyRegistered
		);

		System::assert_has_event(
			SystemTokenManagerEvent::OriginalSystemTokenRegistered { original: original_1000_50_1 }
				.into(),
		)
	})
}

#[test]
fn register_wrapped_system_token_works() {
	new_test_ext().execute_with(|| {
		let original_1000_50_1 = sys_token(1000, 50, 1);
		let wrapped_2000_50_1 = sys_token(2000, 50, 99);
		let wrapped_3000_50_1 = sys_token(3000, 50, 99);
		let wrapped_4000_50_1 = sys_token(4000, 50, 99);
		let wrapped_5000_50_1 = sys_token(5000, 50, 99);
		let wrapped_0_50_1 = sys_token(0, 50, 99);
		// Error case: Try register wrapped token before registering original tokenc
		assert_noop!(
			SystemTokenManager::register_wrapped_system_token(
				RuntimeOrigin::root(),
				original_1000_50_1,
				wrapped_2000_50_1
			),
			SystemTokenManagerError::<Test>::OriginalNotRegistered
		);

		assert_ok!(SystemTokenManager::register_system_token(
			RuntimeOrigin::root(),
			original_1000_50_1,
			wrapped_0_50_1,
			1_000,
			"BCLABS".into(),
			"BCLABS".into(),
			"BCLABS".into(),
			"BCLABS".into(),
			"IUSD".into(),
			4,
			1_000
		));

		System::assert_has_event(
			SystemTokenManagerEvent::OriginalSystemTokenRegistered { original: original_1000_50_1 }
				.into(),
		);

		assert_ok!(SystemTokenManager::register_wrapped_system_token(
			RuntimeOrigin::root(),
			original_1000_50_1,
			wrapped_2000_50_1
		));

		assert_eq!(
			ParaIdSystemTokens::<Test>::get(&2000).unwrap().to_vec(),
			vec![wrapped_2000_50_1,]
		);

		assert_eq!(
			SystemTokenUsedParaIds::<Test>::get(&original_1000_50_1).unwrap().to_vec(),
			vec![1000u32, 0u32, 2000u32]
		);

		// Error case: Try register 'same' wrapped token
		assert_noop!(
			SystemTokenManager::register_wrapped_system_token(
				RuntimeOrigin::root(),
				original_1000_50_1,
				wrapped_2000_50_1
			),
			SystemTokenManagerError::<Test>::WrappedAlreadyRegistered
		);

		System::assert_has_event(
			SystemTokenManagerEvent::WrappedSystemTokenRegistered {
				original: original_1000_50_1,
				wrapped: wrapped_2000_50_1,
			}
			.into(),
		);

		for wrapped in vec![wrapped_3000_50_1, wrapped_4000_50_1] {
			assert_ok!(SystemTokenManager::register_wrapped_system_token(
				RuntimeOrigin::root(),
				original_1000_50_1,
				wrapped
			));
		}
		assert_noop!(
			SystemTokenManager::register_wrapped_system_token(
				RuntimeOrigin::root(),
				original_1000_50_1,
				wrapped_5000_50_1
			),
			SystemTokenManagerError::<Test>::TooManyUsed
		);
	})
}

#[test]
fn deregister_wrapped_works() {
	new_test_ext().execute_with(|| {
		let original_1000_50_1 = sys_token(1000, 50, 1);
		let wrapped_2000_50_1 = sys_token(2000, 50, 1);
		let wrapped_2001_50_1 = sys_token(2001, 50, 99);
		let wrapped_0_50_1 = sys_token(0, 50, 99);

		assert_noop!(
			SystemTokenManager::deregister_wrapped_system_token(
				RuntimeOrigin::root(),
				wrapped_2000_50_1
			),
			SystemTokenManagerError::<Test>::WrappedNotRegistered
		);

		// Register 'original' & 'wrapped'
		assert_ok!(SystemTokenManager::register_system_token(
			RuntimeOrigin::root(),
			original_1000_50_1,
			wrapped_0_50_1,
			1_000,
			"BCLABS".into(),
			"BCLABS".into(),
			"BCLABS".into(),
			"BCLABS".into(),
			"IUSD".into(),
			4,
			1_000
		));
		assert_ok!(SystemTokenManager::register_wrapped_system_token(
			RuntimeOrigin::root(),
			original_1000_50_1,
			wrapped_2000_50_1
		));
		assert_ok!(SystemTokenManager::register_wrapped_system_token(
			RuntimeOrigin::root(),
			original_1000_50_1,
			wrapped_2001_50_1
		));

		assert_eq!(
			ParaIdSystemTokens::<Test>::get(2000).unwrap().to_vec(),
			vec![wrapped_2000_50_1]
		);

		assert_eq!(
			SystemTokenUsedParaIds::<Test>::get(original_1000_50_1).unwrap().to_vec(),
			vec![1000u32, 0u32, 2000u32, 2001u32]
		);

		// Try Deregister 'wrapped(2000)' system token
		assert_ok!(SystemTokenManager::deregister_wrapped_system_token(
			RuntimeOrigin::root(),
			wrapped_2000_50_1
		));
		assert_eq!(
			SystemTokenUsedParaIds::<Test>::get(original_1000_50_1).unwrap().to_vec(),
			vec![1000u32, 0u32, 2001u32]
		);
		assert_eq!(ParaIdSystemTokens::<Test>::get(2000), None);

		// Let's try to remove all 'wrapped'
		assert_ok!(SystemTokenManager::deregister_wrapped_system_token(
			RuntimeOrigin::root(),
			wrapped_2001_50_1
		));
		// Check: Is it possible to deregister 'original's wrapped?
		// Should be fail if try to deregiter original's wrapped inside this extrinsic
		// Maybe it should be removed when `deregister_system_token` is called
		assert_noop!(
			SystemTokenManager::deregister_wrapped_system_token(
				RuntimeOrigin::root(),
				original_1000_50_1
			),
			SystemTokenManagerError::<Test>::BadAccess
		);
		assert_eq!(
			SystemTokenUsedParaIds::<Test>::get(original_1000_50_1).unwrap().to_vec(),
			vec![1000u32, 0u32]
		);
	})
}

#[test]
fn deregister_system_token_works() {
	new_test_ext().execute_with(|| {
		let original_1000_50_1 = sys_token(1000, 50, 1);
		let wrapped_2000_50_1 = sys_token(2000, 50, 1);
		let wrapped_2001_50_1 = sys_token(2001, 50, 99);
		let wrapped_0_50_1 = sys_token(0, 50, 99);

		// Register 'original' & 'wrapped'
		assert_ok!(SystemTokenManager::register_system_token(
			RuntimeOrigin::root(),
			original_1000_50_1,
			wrapped_0_50_1,
			1_000,
			"BCLABS".into(),
			"BCLABS".into(),
			"BCLABS".into(),
			"BCLABS".into(),
			"IUSD".into(),
			4,
			1_000
		));
		assert_ok!(SystemTokenManager::register_wrapped_system_token(
			RuntimeOrigin::root(),
			original_1000_50_1,
			wrapped_2000_50_1
		));
		assert_ok!(SystemTokenManager::register_wrapped_system_token(
			RuntimeOrigin::root(),
			original_1000_50_1,
			wrapped_2001_50_1
		));

		// Try deregister 'original' system token
		assert_ok!(SystemTokenManager::deregister_system_token(
			RuntimeOrigin::root(),
			original_1000_50_1
		));

		assert_eq!(SystemTokenUsedParaIds::<Test>::get(original_1000_50_1), None);

		System::assert_has_event(
			SystemTokenManagerEvent::<Test>::OriginalSystemTokenDeregistered {
				original: original_1000_50_1,
			}
			.into(),
		);
	})
}
