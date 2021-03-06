// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![allow(dead_code)]

use crate::{self as pallet_farming};
use currencies::BasicCurrencyAdapter;
use frame_support::weights::Weight;
use frame_support::{construct_runtime, parameter_types};
use frame_system;
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::{BlakeTwo256, IdentityLookup};
use sp_runtime::Perbill;

use traits::parameter_type_with_key;
use base::*;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime! {
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
        PalletFungibleAsset: pallet_fungible_asset::{Module, Call, Config<T>, Storage, Event<T>},
        Tokens: tokens::{Module, Call, Config<T>, Storage, Event<T>},
        Currencies: currencies::{Module, Call, Storage, Event<T>},
        Balances: pallet_balances::{Module, Call, Storage, Event<T>},
        PalletNFT: pallet_nft::{Module, Call, Config<T>, Storage, Event<T>},
		Farming: pallet_farming::{Module, Call, Config<T>, Storage, Event<T>},
    }
}

pub type AccountId = FixedString;
pub type BlockNumber = u64;
pub type Amount = i128;

pub const ALICE: AccountId = FixedString::from_const_string("ALICE");
pub const BOB: AccountId = FixedString::from_const_string("BOB");
pub const EVE: AccountId = FixedString::from_const_string("EVE");

pub const TEST_FARM1: FarmId = FarmId::from_const_string("TEST_FARM1");
pub const TEST_FARM2: FarmId = FarmId::from_const_string("TEST_FARM2");
pub const TEST_SHARES_FT_TOKEN1: AssetSymbol = AssetSymbol::from_const_string("KSM");

pub const TEST_STAKE_TOKEN1: AssetSymbol = AssetSymbol::from_const_string("DOT");
pub const TEST_STAKE_NFT_CLASS1: NftClassId = NftClassId::from_const_string("TEST_CLASS1");
pub const TEST_STAKE_NFT_TOKEN1: NftTokenId = NftTokenId::from_const_string("NftToken1");
pub const TEST_STAKE_NFT_TOKEN2: NftTokenId = NftTokenId::from_const_string("NftToken2");


parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 4096;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(80);

	pub const GetBaseAssetId: AssetId = DEFAULT_SYMBOL;
	pub const ExistentialDeposit: u128 = 0;
}

impl frame_system::Config for Runtime {
    type BaseCallFilter = ();
    type BlockWeights = ();
    type BlockLength = ();
    type Origin = Origin;
    type Call = Call;
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = Event;
    type BlockHashCount = BlockHashCount;
    type DbWeight = ();
    type Version = ();
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
    type PalletInfo = PalletInfo;
    type SS58Prefix = ();
}

parameter_type_with_key! {
    pub ExistentialDeposits: |_currency_id: AssetId| -> Balance {
        0
    };
}

impl tokens::Config for Runtime {
    type Event = Event;
    type Balance = Balance;
    type Amount = Amount;
    type CurrencyId = AssetId;
    type WeightInfo = ();
    type ExistentialDeposits = ExistentialDeposits;
    type OnDust = ();
}

impl currencies::Config for Runtime {
    type Event = Event;
    type MultiCurrency = Tokens;
    type NativeCurrency = BasicCurrencyAdapter<Runtime, Balances, Amount, BlockNumber>;
    type GetNativeCurrencyId = GetBaseAssetId;
    type WeightInfo = ();
}

impl pallet_balances::Config for Runtime {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxLocks = ();
}

impl pallet_fungible_asset::Config for Runtime {
    type Event = Event;
    type AssetId = AssetId;
    type Currency = currencies::Module<Runtime>;
}

impl pallet_nft::Config for Runtime {
    type Event = Event;
    type Data = ();
}

impl crate::Config for Runtime {
	type Event = Event;
}

pub struct ExtBuilder;

impl Default for ExtBuilder {
    fn default() -> Self {
		ExtBuilder
    }
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let t = frame_system::GenesisConfig::default()
			.build_storage::<Runtime>()
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
