// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

mod tests {
    use crate::mock::*;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok};
    use sp_runtime::traits::Zero;

	use base::*;

	const TEST_SYMBOL: AssetSymbol = AssetSymbol::from_const_string("DOT");

	#[test]
    fn test_register_asset() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			assert!(FungibleAsset::ensure_asset_exists(&DEFAULT_SYMBOL).is_err());
			assert_ok!(FungibleAsset::register_asset(
                &ALICE,
                &TEST_SYMBOL,
                &AssetName::from_string("polkadot"),
                18,
				true,
				true,
				None,
                Balance::zero(),
            ));

			assert_noop!(FungibleAsset::register_asset(
					&ALICE,
					&TEST_SYMBOL,
					&AssetName::from_string("polkadot"),
					18,
					true,
					true,
					None,
					Balance::zero(),
				),
				Error::<Runtime>::AssetAlreadyExists
			);
		});
	}

	#[test]
    fn test_asset_name() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			assert_eq!(
                AssetName::from_string("Thisislongnameexample"),//will be truncated to FIXED_STRINGZ_SIZE!
                AssetName::from_const_string("Thisislongnameexample"),
            );
		});
	}

	#[test]
    fn test_initial_supply() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			assert_ok!(FungibleAsset::register_asset(
                &ALICE,
                &TEST_SYMBOL,
                &AssetName::from_string("dot"),
                18,
				true,
				true,
				None,
                Balance::from(256u64),
            ));
			assert_eq!(
                FungibleAsset::free_balance(&TEST_SYMBOL, &ALICE).expect("Failed to query free balance."),
                Balance::from(256u64),
            );
		});
	}

	#[test]
    fn test_mint_asset() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			assert_ok!(FungibleAsset::register_asset(
                &ALICE,
                &TEST_SYMBOL,
                &AssetName::from_string("polkadot"),
                18,
				true,
				true,
				None,
                Balance::zero(),
            ));

			assert_ok!(FungibleAsset::mint_asset(
                &ALICE,
                &TEST_SYMBOL,
                Balance::from(10u64),
            ));
			assert_eq!(
                FungibleAsset::free_balance(&TEST_SYMBOL, &ALICE).expect("Failed to query free balance."),
                Balance::from(10u64),
            );

			assert_noop!(FungibleAsset::mint_asset(
					&BOB,
					&TEST_SYMBOL,
					Balance::from(10u64),
				),
				Error::<Runtime>::InvalidOwner
			);
		});
	}

	#[test]
    fn test_burn_asset() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			assert_ok!(FungibleAsset::register_asset(
                &ALICE,
                &TEST_SYMBOL,
                &AssetName::from_string("polkadot"),
                18,
				true,
				true,
				None,
                Balance::zero(),
            ));

			assert_ok!(FungibleAsset::mint_asset(
                &ALICE,
                &TEST_SYMBOL,
                Balance::from(100u64),
            ));
			assert_ok!(FungibleAsset::burn_asset(
                &ALICE,
                &TEST_SYMBOL,
                Balance::from(10u64),
            ));
			assert_eq!(
                FungibleAsset::free_balance(&TEST_SYMBOL, &ALICE).expect("Failed to query free balance."),
                Balance::from(90u64),
            );
		});
	}

	#[test]
    fn test_transfer_asset() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			assert_ok!(FungibleAsset::register_asset(
                &ALICE,
                &TEST_SYMBOL,
                &AssetName::from_string("polkadot"),
                18,
				true,
				true,
				None,
                Balance::zero(),
            ));

			assert_noop!(FungibleAsset::transfer_asset(
					&ALICE,
					&TEST_SYMBOL,
					&BOB,
					Balance::from(10u64),
				),
				Error::<Runtime>::NoEnoughBalance
			);

			assert_ok!(FungibleAsset::mint_asset(
                &ALICE,
                &TEST_SYMBOL,
                Balance::from(100u64),
            ));
			assert_ok!(FungibleAsset::transfer_asset(
					&ALICE,
					&TEST_SYMBOL,
					&BOB,
					Balance::from(10u64),
				));
			assert_eq!(
                FungibleAsset::free_balance(&TEST_SYMBOL, &ALICE).expect("Failed to query free balance."),
                Balance::from(90u64),
            );
			assert_eq!(
                FungibleAsset::free_balance(&TEST_SYMBOL, &BOB).expect("Failed to query free balance."),
                Balance::from(10u64),
            );
		});
	}

}
