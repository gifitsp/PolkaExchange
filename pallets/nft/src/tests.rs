// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;


#[test]
fn test_create_nft_class() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(PalletNft::create_nft_class(&ALICE, &TEST_CLASS1, &()));
        assert_noop!(PalletNft::create_nft_class(&ALICE, &TEST_CLASS1, &()), Error::<Runtime>::NftClassIdAlreadyExisted);
	});
}

#[test]
fn test_destroy_nft_class() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(PalletNft::create_nft_class(&ALICE, &TEST_CLASS1, &()));
        assert_ok!(PalletNft::destroy_nft_class(&ALICE, &TEST_CLASS1));
        assert_noop!(PalletNft::destroy_nft_class(&ALICE, &TEST_CLASS1), Error::<Runtime>::NftClassIdNotExisted);
	});
}

#[test]
fn test_token() {
	ExtBuilder::default().build().execute_with(|| {
        let metadata = TokenMetadata::default();

		assert_ok!(PalletNft::create_nft_class(&ALICE, &TEST_CLASS1, &()));
        assert_ok!(PalletNft::mint_token(&ALICE, &TEST_CLASS1, &TEST_TOKEN1, &metadata, &()));
        assert_noop!(PalletNft::mint_token(&ALICE, &TEST_CLASS1, &TEST_TOKEN1, &metadata, &()), Error::<Runtime>::TokenIdAlreadyExisted);

        assert_noop!(PalletNft::destroy_nft_class(&ALICE, &TEST_CLASS1), Error::<Runtime>::CannotDestroyNftClass);

        assert_noop!(PalletNft::burn_token(&BOB, &TEST_CLASS1, &TEST_TOKEN1), Error::<Runtime>::InvalidTokenOwner);
        assert_ok!(PalletNft::burn_token(&ALICE, &TEST_CLASS1, &TEST_TOKEN1));
        assert_noop!(PalletNft::burn_token(&ALICE, &TEST_CLASS1, &TEST_TOKEN1), Error::<Runtime>::TokenIdNotExisted);
	});
}

#[test]
fn test_transfer_nft() {
	ExtBuilder::default().build().execute_with(|| {
        let metadata = TokenMetadata::default();

		assert_ok!(PalletNft::create_nft_class(&ALICE, &TEST_CLASS1, &()));
        assert_ok!(PalletNft::mint_token(&ALICE, &TEST_CLASS1, &TEST_TOKEN1, &metadata, &()));
        assert_ok!(PalletNft::mint_token(&ALICE, &TEST_CLASS1, &TEST_TOKEN2, &metadata, &()));

        assert_ok!(PalletNft::transfer_token(&ALICE, &BOB, &TEST_CLASS1, &TEST_TOKEN1));
        assert_noop!(PalletNft::transfer_token(&ALICE, &BOB, &TEST_CLASS1, &TEST_TOKEN1), Error::<Runtime>::InvalidTokenOwner);

        assert!(PalletNft::is_owner(&ALICE, &TEST_CLASS1, &TEST_TOKEN2));
        assert!(PalletNft::is_owner(&ALICE, &TEST_CLASS1, &TEST_TOKEN1) == false);
        assert!(PalletNft::is_owner(&BOB, &TEST_CLASS1, &TEST_TOKEN1));

        assert_ok!(PalletNft::transfer_token(&BOB, &ALICE, &TEST_CLASS1, &TEST_TOKEN1));
        assert!(PalletNft::is_owner(&ALICE, &TEST_CLASS1, &TEST_TOKEN1));
        assert!(PalletNft::is_owner(&BOB, &TEST_CLASS1, &TEST_TOKEN1) == false);
	});
}