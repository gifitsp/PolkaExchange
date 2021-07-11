// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

use frame_support::{RuntimeDebug};
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::collections::btree_map::BTreeMap;

use crate::primitives::*;

pub type SymbolData = BTreeMap<AssetSymbol, Balance>;
pub type SharesData<AccountId> = BTreeMap<AccountId, Balance>;
pub type VolumeData = BTreeMap<AssetSymbol, SwapVolume>;


pub const DEFAULT_SYMBOL: AssetSymbol = FixedString::from_const_string("PEX");
pub const DEFAULT_POOL_ID: PoolId = FixedString::from_const_string("ROOT");
pub const DEFAULT_BALANCE_PRECISION: BalancePrecision = 18;

pub const MAX_NUM_SYMBOLS: usize = 100;

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct AssetInfo {
    pub symbol: AssetSymbol,
	pub name: AssetName,
	pub precision: BalancePrecision,
	pub is_mintable: bool,
	pub is_burnable: bool,
    pub description: Option<StdString>,
}

impl Default for AssetInfo {
    fn default() -> Self {
        AssetInfo {
            symbol: AssetSymbol::get_default_symbol(),
			name: AssetName::default(),
			precision: DEFAULT_BALANCE_PRECISION,
			is_mintable: false,
			is_burnable: false,
			description: None,
        }
    }
}

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
pub struct SwapVolume {
    pub input: u128,
    pub output: u128,
}

#[derive(Encode, Decode, RuntimeDebug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Hash))]
pub struct PoolInfo<AccountId: Ord> {
	pub pid: PoolId,
	pub total_fee: u32,
	pub exchange_fee: u32,
	pub shares_total_supply: Balance,
	pub shares_data: SharesData<AccountId>,
	pub symbol_data: SymbolData,
	pub volume_data: VolumeData,
	pub description: Option<StdString>,
}

impl<AccountId: Ord> Default for PoolInfo<AccountId> {
    fn default() -> Self {
        PoolInfo {
            pid: DEFAULT_POOL_ID,
			total_fee: 0,
			exchange_fee: 0,
			shares_total_supply: 0,
			shares_data: Default::default(),
			symbol_data: Default::default(),
			volume_data: Default::default(),
			description: None,
        }
    }
}
