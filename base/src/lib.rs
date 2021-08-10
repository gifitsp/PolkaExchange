// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

mod primitives;
mod metadata;

pub use primitives::*;
pub use metadata::*;

pub const ID_ANNOTATIONS: [u8; 4] = [b'.', b'-', b'_', b'\0'];

pub fn is_valid_id(id: &FixedString) -> bool {
	if ID_ANNOTATIONS.contains(&id.0[0]){
		return false;
	}
	id.0.iter().all(|byte|
		(b'A'..=b'Z').contains(&byte)
		|| (b'a'..=b'z').contains(&byte)
		|| ID_ANNOTATIONS.contains(&byte)
	)
	
}

#[inline]
pub fn is_valid_symbol(name: &AssetName) -> bool {
	is_valid_id(name)
}

#[inline]
pub fn is_valid_name(name: &AssetName) -> bool {
	name.is_empty() || is_valid_id(name)
}

pub fn integer_sqrt(value: u128) -> u128 {
    let mut guess: u128 = (value + 1) >> 1;
    let mut res = value;
    while guess < res {
        res = guess;
        guess = (value / guess + guess) >> 1;
    }
    res
}

