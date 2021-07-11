// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

use frame_support::{RuntimeDebug};
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_std::vec::Vec;
use alloc::string::String;

use sp_std::str::FromStr;
use sp_std::fmt::Display;
use static_assertions::_core::fmt::Formatter;

#[derive(Encode, Decode, Eq, PartialEq, Clone, Ord, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct StdString(pub Vec<u8>);

pub const FIXED_STRINGZ_SIZE: usize = 16;
#[allow(non_camel_case_types)]
pub type FIXED_ARRAY = [u8; FIXED_STRINGZ_SIZE];

#[derive(Encode, Decode, Eq, PartialEq, Clone, Copy, Ord, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Hash))]
pub struct FixedString(pub FIXED_ARRAY);


#[cfg(feature = "std")]
impl Serialize for StdString {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for StdString {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|str_err| serde::de::Error::custom(str_err))
    }
}

#[cfg(feature = "std")]
impl FromStr for StdString {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars: Vec<u8> = s.chars().map(|un| un as u8).collect();
        Ok(StdString(chars))
    }
}

impl StdString {
    pub fn from_string(s: &String) -> Self {
        StdString(s.as_bytes().to_vec())
    }
}

#[cfg(feature = "std")]
impl Display for StdString {
    fn fmt(&self, f: &mut Formatter<'_>) -> sp_std::fmt::Result {
        let s: String = self.0.iter().map(|un| *un as char).collect();
        write!(f, "{}", s)
    }
}

impl Default for StdString {
    fn default() -> Self {
        Self(Vec::new())
    }
}

impl From<FIXED_ARRAY> for FixedString{
	fn from(value: FIXED_ARRAY) -> Self{
		Self(value)
	}
}

impl FixedString{

    pub fn to_string(&self) -> String {
        String::from_utf8(self.0.to_vec()).unwrap()
    }

	pub fn is_empty(&self) -> bool {
		self.0[0] == b'\0'
	}

	pub fn size(&self) -> usize {
		FIXED_STRINGZ_SIZE
	}

	pub const fn from_const_string(s: &str) -> FixedString{
		macro_rules! assign_byte{
			($index:expr, $left:expr, $right:expr) => {
				if $index < ($right).len(){
					($left)[$index] = ($right)[$index];
				}
				$index += 1;
			};
		}

		let mut a = [b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0', b'\0'];
		let _s = s.as_bytes();

		let mut index: usize = 0;
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);
		assign_byte!(index, a, _s);

		if index != FIXED_STRINGZ_SIZE{}
		FixedString(a)
	}

	pub fn from_string(s: &str) -> FixedString{
		let mut a = FIXED_ARRAY::default();
		for i in 0..FIXED_STRINGZ_SIZE{
			if i < s.len(){
				a[i] = s.as_bytes()[i];
			}
			else{
				break;
			}
		}
		FixedString(a)
	}
}

#[cfg(feature = "std")]
impl Serialize for FixedString {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{}", self))
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for FixedString {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::from_str(&s).map_err(|str_err| serde::de::Error::custom(str_err))
    }
}

#[cfg(feature = "std")]
impl FromStr for FixedString {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
		let a = FixedString::from_string(s);
        Ok(a)
    }
}

#[cfg(feature = "std")]
impl Display for FixedString {
    fn fmt(&self, f: &mut Formatter<'_>) -> sp_std::fmt::Result {
        let s: String = self.0.iter().map(|un| *un as char).collect();
        write!(f, "{}", s)
    }
}

impl Default for FixedString {
    fn default() -> Self {
        Self(FIXED_ARRAY::default())
    }
}

#[derive(Encode, Decode, Eq, PartialEq, Clone, Ord, PartialOrd, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Hash, Serialize, Deserialize))]
pub enum PoolType{
	None,
	AmmPool,
}

impl Default for PoolType{
	fn default() -> Self {
        PoolType::None
    }
}

pub type AccountId = u128;
pub type AssetSymbol = FixedString;
pub type AssetId = AssetSymbol;
pub type AssetName = FixedString;
pub type Balance = u128;
pub type Amount = i128;
pub type BalancePrecision = u8;
pub type PoolId = FixedString;


pub const DEFAULT_ID: FixedString = FixedString::from_const_string("DEFAULT");

impl AssetId{
    pub const fn get_default_asset_id() -> AssetId {
		DEFAULT_ID
    }
}

impl AssetSymbol{
    pub const fn get_default_symbol() -> AssetSymbol {
		DEFAULT_ID
    }
}

pub trait ToFixedArray{
	fn to_fixed_array(self) -> FIXED_ARRAY;
}

impl ToFixedArray for FixedString{
	fn to_fixed_array(self) -> FIXED_ARRAY{
		self.0
	}
}
