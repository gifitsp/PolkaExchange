// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg(test)]

use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;
use crate::Error;

fn init_test_env(shares_init_supply: Balance, stake_init_supply: Balance) {
    assert_ok!(PalletFungibleAsset::register_asset(&ALICE, &TEST_SHARES_FT_TOKEN1, &TEST_SHARES_FT_TOKEN1, 8, true, true, None, shares_init_supply));
    assert_ok!(PalletFungibleAsset::register_asset(&ALICE, &TEST_STAKE_TOKEN1, &TEST_STAKE_TOKEN1, 8, true, true, None, stake_init_supply));
    // 2 nfts mint by ALICE
    assert_ok!(PalletNFT::create_nft_class(&ALICE, &TEST_STAKE_NFT_CLASS1, &()));
    assert_ok!(PalletNFT::mint_token(&ALICE, &TEST_STAKE_NFT_CLASS1, &TEST_STAKE_NFT_TOKEN1, &TokenMetadata::default(), &()));
    assert_ok!(PalletNFT::mint_token(&ALICE, &TEST_STAKE_NFT_CLASS1, &TEST_STAKE_NFT_TOKEN2, &TokenMetadata::default(), &()));

    // transfer to BOB and EVE who are the users in the tests
    assert_ok!(PalletFungibleAsset::transfer_asset(&ALICE, &TEST_STAKE_TOKEN1, &BOB, stake_init_supply / 2));
    assert_ok!(PalletFungibleAsset::transfer_asset(&ALICE, &TEST_STAKE_TOKEN1, &EVE, stake_init_supply / 2));
    assert_ok!(PalletNFT::transfer_token(&ALICE, &BOB, &TEST_STAKE_NFT_CLASS1, &TEST_STAKE_NFT_TOKEN1));
    assert_ok!(PalletNFT::transfer_token(&ALICE, &EVE, &TEST_STAKE_NFT_CLASS1, &TEST_STAKE_NFT_TOKEN2));

    // make sure the transfer succeeds
    assert!(PalletFungibleAsset::free_balance(&TEST_STAKE_TOKEN1, &BOB) == Ok(stake_init_supply / 2));
    assert!(PalletFungibleAsset::free_balance(&TEST_STAKE_TOKEN1, &EVE) == Ok(stake_init_supply / 2));
    assert!(PalletNFT::is_owner(&BOB, &TEST_STAKE_NFT_CLASS1, &TEST_STAKE_NFT_TOKEN1));
    assert!(PalletNFT::is_owner(&EVE, &TEST_STAKE_NFT_CLASS1, &TEST_STAKE_NFT_TOKEN2));


    assert_ok!(Farming::create_farm(&ALICE, &TEST_FARM1, &TEST_SHARES_FT_TOKEN1, Some(TEST_STAKE_TOKEN1.clone()), None, 10, 2));
    assert_noop!(Farming::create_farm(&BOB, &TEST_FARM1, &TEST_SHARES_FT_TOKEN1, Some(TEST_STAKE_TOKEN1.clone()), None, 10, 2), 
                 Error::<Runtime>::FarmIdAlreadyExisted);

    assert_ok!(Farming::create_farm(&ALICE, &TEST_FARM2, &TEST_SHARES_FT_TOKEN1, None, Some((TEST_STAKE_NFT_CLASS1.clone(), TEST_STAKE_NFT_TOKEN1.clone())), 10, 2));
    assert_noop!(Farming::create_farm(&BOB, &TEST_FARM2, &TEST_SHARES_FT_TOKEN1, None, Some((TEST_STAKE_NFT_CLASS1.clone(), TEST_STAKE_NFT_TOKEN1.clone())), 10, 2), 
                 Error::<Runtime>::FarmIdAlreadyExisted);
}

#[test]
fn test_farm() {
	ExtBuilder::default().build().execute_with(|| {
        init_test_env(Balance::from(100u128), Balance::from(100u128));

        assert_ok!(Farming::destroy_farm(&ALICE, &TEST_FARM1));
        assert_noop!(Farming::destroy_farm(&BOB, &TEST_FARM2), Error::<Runtime>::NoPermission);
        assert_ok!(Farming::destroy_farm(&ALICE, &TEST_FARM2));
        assert_noop!(Farming::destroy_farm(&ALICE, &TEST_FARM1), Error::<Runtime>::FarmIdNotExisted);
	});
}

#[test]
fn test_ft_stake() {
    ExtBuilder::default().build().execute_with(|| {
        init_test_env(100u128, 100u128);

        assert_ok!(Farming::stake_asset(&BOB, &TEST_FARM1, Some(20u128)));
        assert_ok!(Farming::stake_asset(&EVE, &TEST_FARM1, Some(20u128)));
        System::set_block_number(System::block_number() + 20);
        let mut total_rewards1 = Farming::update_rewards(&BOB, &TEST_FARM1).unwrap();
        let mut total_rewards2 = Farming::update_rewards(&EVE, &TEST_FARM1).unwrap();
        sp_std::if_std! {
            println!(">>>test_ft_stake[1]: BOB total_rewards: {}, EVE total_rewards: {}", total_rewards1, total_rewards2);
        }

        assert_ok!(Farming::stake_asset(&BOB, &TEST_FARM1, Some(10u128)));
        assert!(PalletFungibleAsset::free_balance(&TEST_STAKE_TOKEN1, &BOB) == Ok(50u128 - 30u128));
        System::set_block_number(System::block_number() + 30);
        total_rewards1 = Farming::update_rewards(&BOB, &TEST_FARM1).unwrap();
        total_rewards2 = Farming::update_rewards(&EVE, &TEST_FARM1).unwrap();
        sp_std::if_std! {
            println!(">>>test_ft_stake[2]: BOB total_rewards: {}, EVE total_rewards: {}", total_rewards1, total_rewards2);
        }

        // make sure total_rewards > 3
        assert_ok!(Farming::claim(&BOB, &TEST_FARM1, total_rewards1 - 3u128));
        // shares should increase for BOB's account
        assert!(PalletFungibleAsset::free_balance(&TEST_SHARES_FT_TOKEN1, &BOB) == Ok(total_rewards1 - 3u128));
        let remaining_total_rewards = Farming::update_rewards(&BOB, &TEST_FARM1).unwrap();
        assert!(remaining_total_rewards == 3u128);
        assert_ok!(Farming::claim(&BOB, &TEST_FARM1, remaining_total_rewards));
        assert!(PalletFungibleAsset::free_balance(&TEST_SHARES_FT_TOKEN1, &BOB) == Ok(total_rewards1));
        total_rewards1 = Farming::update_rewards(&BOB, &TEST_FARM1).unwrap();
        assert!(total_rewards1 == 0u128);

        //assert!(false); //for printlin
    });
}

#[test]
fn test_ft_unstake() {
    ExtBuilder::default().build().execute_with(|| {
        init_test_env(100u128, 100u128);

        assert_ok!(Farming::stake_asset(&BOB, &TEST_FARM1, Some(20u128)));
        assert_ok!(Farming::stake_asset(&EVE, &TEST_FARM1, Some(20u128)));
        System::set_block_number(System::block_number() + 10);
        assert_ok!(Farming::unstake_asset(&BOB, &TEST_FARM1, Some(10u128)));
        System::set_block_number(System::block_number() + 10);
        let mut total_rewards1 = Farming::update_rewards(&BOB, &TEST_FARM1).unwrap();
        let mut total_rewards2 = Farming::update_rewards(&EVE, &TEST_FARM1).unwrap();
        assert!(Farming::get_farmer_stake_amount(&BOB, &TEST_FARM1) == Ok(10u128));
        assert!(Farming::get_farm_total_stake_amount(&TEST_FARM1) == Ok(30u128));
        sp_std::if_std! {
            println!(">>>test_ft_unstake[1]: BOB total_rewards: {}, EVE total_rewards: {}", total_rewards1, total_rewards2);
        }

        assert_ok!(Farming::stake_asset(&BOB, &TEST_FARM1, Some(10u128)));
        System::set_block_number(System::block_number() + 20);
        total_rewards1 = Farming::update_rewards(&BOB, &TEST_FARM1).unwrap();
        total_rewards2 = Farming::update_rewards(&EVE, &TEST_FARM1).unwrap();
        sp_std::if_std! {
            println!(">>>test_ft_unstake[1]: BOB total_rewards: {}, EVE total_rewards: {}", total_rewards1, total_rewards2);
        }

        assert_ok!(Farming::unstake_asset(&BOB, &TEST_FARM1, Some(5u128)));
        System::set_block_number(System::block_number() + 10);
        total_rewards1 = Farming::update_rewards(&BOB, &TEST_FARM1).unwrap();
        total_rewards2 = Farming::update_rewards(&EVE, &TEST_FARM1).unwrap();
        sp_std::if_std! {
            println!(">>>test_ft_unstake[1]: BOB total_rewards: {}, EVE total_rewards: {}", total_rewards1, total_rewards2);
        }

        // false cases test
        assert_noop!(Farming::unstake_asset(&ALICE, &TEST_FARM1, Some(10u128)), Error::<Runtime>::FarmerNotExisted);
        assert_noop!(Farming::update_rewards(&ALICE, &TEST_FARM1), Error::<Runtime>::FarmerNotExisted);

        //assert!(false);
    });
}

#[test]
fn test_nft_stake() {
    ExtBuilder::default().build().execute_with(|| {
        init_test_env(100u128, 100u128);

        assert_noop!(Farming::stake_asset(&EVE, &TEST_FARM2, None), pallet_nft::Error::<Runtime>::InvalidTokenOwner);
        
        assert!(PalletNFT::is_owner(&ALICE, &TEST_STAKE_NFT_CLASS1, &TEST_STAKE_NFT_TOKEN1) == false);
        assert_ok!(Farming::stake_asset(&BOB, &TEST_FARM2, None));
        assert!(PalletNFT::is_owner(&ALICE, &TEST_STAKE_NFT_CLASS1, &TEST_STAKE_NFT_TOKEN1) == true);

        System::set_block_number(System::block_number() + 10);
        let mut total_rewards = Farming::update_rewards(&BOB, &TEST_FARM2).unwrap();
        sp_std::if_std! {
            println!(">>>test_nft_stake[1]: BOB total_rewards: {}", total_rewards);
        }

        System::set_block_number(System::block_number() + 10);
        total_rewards = Farming::update_rewards(&BOB, &TEST_FARM2).unwrap();
        sp_std::if_std! {
            println!(">>>test_nft_stake[2]: BOB total_rewards: {}", total_rewards);
        }
        assert_ok!(Farming::claim(&BOB, &TEST_FARM2, 0));
        let free_balance = PalletFungibleAsset::free_balance(&TEST_SHARES_FT_TOKEN1, &BOB).unwrap();
        sp_std::if_std! {
            println!(">>>test_nft_stake[3]: BOB shares free_balance: {}", free_balance);
        }
        assert!(free_balance == total_rewards);
        assert!(Farming::update_rewards(&BOB, &TEST_FARM2) == Ok(0u128));

        System::set_block_number(System::block_number() + 10);
        total_rewards = Farming::update_rewards(&BOB, &TEST_FARM2).unwrap();
        sp_std::if_std! {
            println!(">>>test_nft_stake[4]: BOB total_rewards: {}", total_rewards);
        }

        //assert!(false);
    });
}

#[test]
fn test_nft_unstake() {
    ExtBuilder::default().build().execute_with(|| {
        init_test_env(100u128, 100u128);

        assert_ok!(Farming::stake_asset(&BOB, &TEST_FARM2, None));
        System::set_block_number(System::block_number() + 10);
        let total_rewards = Farming::update_rewards(&BOB, &TEST_FARM2).unwrap();
        sp_std::if_std! {
            println!(">>>test_nft_unstake[1]: BOB total_rewards: {}", total_rewards);
        }

        assert_ok!(Farming::unstake_asset(&BOB, &TEST_FARM2, None));
        assert!(PalletNFT::is_owner(&BOB, &TEST_STAKE_NFT_CLASS1, &TEST_STAKE_NFT_TOKEN1) == true);        
        assert!(Farming::get_farmer_stake_amount(&BOB, &TEST_FARM2) == Ok(0u128));
        assert!(Farming::get_farm_total_stake_amount(&TEST_FARM2) == Ok(0u128));
        assert!(Farming::is_farmer_existed(&BOB, &TEST_FARM2) == true);
        
        assert_ok!(Farming::claim(&BOB, &TEST_FARM2, 0));
        assert!(Farming::is_farmer_existed(&BOB, &TEST_FARM2) == false);
        let free_balance = PalletFungibleAsset::free_balance(&TEST_SHARES_FT_TOKEN1, &BOB).unwrap();
        sp_std::if_std! {
            println!(">>>test_unnft_stake[2]: BOB shares free_balance: {}", free_balance);
        }
        assert!(free_balance == total_rewards);

        System::set_block_number(System::block_number() + 10);
        assert_noop!(Farming::update_rewards(&BOB, &TEST_FARM2), Error::<Runtime>::FarmerNotExisted);

        //assert!(false);
    });
}