// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg(test)]

#![allow(dead_code)]

use frame_support::weights::Weight;
use frame_support::{construct_runtime, parameter_types};
use frame_system;
use sp_core::H256;
use sp_runtime::testing::Header;
use sp_runtime::traits::{IdentityLookup};
use sp_runtime::Perbill;

use base::*;
use crate as pallet_nft;


pub type AccountId = FixedString;
pub type BlockNumber = u64;

pub const ALICE: AccountId = FixedString::from_const_string("ALICE");
pub const BOB: AccountId = FixedString::from_const_string("BOB");
pub const TEST_CLASS1: NftClassId = NftClassId::from_const_string("TEST_CLASS1");

pub const TEST_TOKEN1: NftTokenId = NftTokenId::from_const_string("NftToken1");
pub const TEST_TOKEN2: NftTokenId = NftTokenId::from_const_string("NftToken2");

parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 4096;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(80);
}

construct_runtime! {
    pub enum Runtime where
        Block = Block,
        NodeBlock = Block,
        UncheckedExtrinsic = UncheckedExtrinsic,
    {
        System: frame_system::{Module, Call, Config, Storage, Event<T>},
		PalletNft: pallet_nft::{Module, Call, Storage, Config<T>, Event<T>},
    }
}

impl frame_system::Config for Runtime {
	type Origin = Origin;
	type Index = u64;
	type BlockNumber = BlockNumber;
	type Call = Call;
	type Hash = H256;
	type Hashing = ::sp_runtime::traits::BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

impl crate::Config for Runtime {
    type Event = Event;
    type Data = ();
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Runtime>;
type Block = frame_system::mocking::MockBlock<Runtime>;

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
