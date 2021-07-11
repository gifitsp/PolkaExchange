// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
extern crate alloc;

use frame_support::{ensure};
use frame_support::dispatch::{DispatchResult};
use sp_std::vec::Vec;

pub use base::*;

type PoolAmm<T> = pallet_pool_amm::Pallet<T>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

	#[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_pool_amm::Config
    {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
	}
	
	#[pallet::storage]
    #[pallet::getter(fn pool_instances)]
    pub type PoolInstances<T: Config> = StorageMap<
        _,
        Twox64Concat,
        PoolId,
		PoolType,
        ValueQuery,
    >;

	#[pallet::error]
    pub enum Error<T> {
		PoolNotExists,
		PoolAlreadyExists,
	}

	#[pallet::event]
    #[pallet::metadata(AccountIdOf<T> = "AccountId", PoolIdOf<T> = "PoolId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
		AmmPoolRegistered(T::PoolId),
	}

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub endowed_pool: Vec<(
			T::PoolId,
			PoolType,
        )>,
	}

	#[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self{
				endowed_pool: Default::default(),
			}
        }
	}

	#[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
		}
	}
}

pub use pallet::*;

impl<T: Config> Pallet<T> {
	pub fn create_amm_pool(
		issuer: &T::AccountId,
		pid: &PoolId,
		total_fee: u32,
		exchange_fee: u32,
		symbol_data: &SymbolData,
		description: Option<StdString>,
	) -> DispatchResult {
		ensure!(
			PoolInstances::<T>::get(pid) == PoolType::None,
			Error::<T>::PoolAlreadyExists
		);

		PoolAmm::<T>::register_pool(
			issuer,
			pid,
			total_fee,
			exchange_fee,
			symbol_data,
			description,
		)?;

		PoolInstances::<T>::insert(pid.clone(), PoolType::AmmPool);
		Self::deposit_event(Event::AmmPoolRegistered(PoolAmm::<T>::get_pool_id(pid)));
        Ok(())
	}

	pub fn destroy_pool(
		issuer: &T::AccountId,
		pid: &PoolId,
	) -> DispatchResult {
		match PoolInstances::<T>::get(pid){
			PoolType::AmmPool => {
				PoolAmm::<T>::unregister_pool(issuer, pid)?;
				Ok(())
			}
			PoolType::None => Err(Error::<T>::PoolNotExists.into())
		}
	}

	pub fn add_liquidity_to_pool(
		issuer: &T::AccountId,
		pid: &PoolId,
		amounts: &SymbolData,
	) -> DispatchResult {
		match PoolInstances::<T>::get(pid){
			PoolType::AmmPool => {
				PoolAmm::<T>::add_liquidity_to_pool(issuer, pid, amounts)?;
				Ok(())
			}
			PoolType::None => Err(Error::<T>::PoolNotExists.into())
		}
	}

	pub fn remove_liquidity_from_pool(
		issuer: &T::AccountId,
		pid: &PoolId,
		shares: Balance,
		amounts: &SymbolData,
	) -> DispatchResult {
		match PoolInstances::<T>::get(pid){
			PoolType::AmmPool => {
				PoolAmm::<T>::remove_liquidity_from_pool(issuer, pid, shares, amounts)?;
				Ok(())
			}
			PoolType::None => Err(Error::<T>::PoolNotExists.into())
		}
	}

	pub fn get_swap_return_asset_from_pool(
		pid: &PoolId,
		asset_in: &AssetSymbol,
		amount_in: Balance,
		asset_out: &AssetSymbol,
	) -> Balance {
		match PoolInstances::<T>::get(pid){
			PoolType::AmmPool => {
				PoolAmm::<T>::get_swap_return_asset(pid, asset_in, amount_in, asset_out)
			}
			PoolType::None => 0
		}
	}

	pub fn swap_asset_in_pool(
		who: &T::AccountId,
		pid: &PoolId,
		asset_in: &AssetSymbol,
		amount_in: Balance,
		asset_out: &AssetSymbol,
		min_amount_out: Balance,
	) -> DispatchResult {
		match PoolInstances::<T>::get(pid){
			PoolType::AmmPool => {
				PoolAmm::<T>::swap_asset(who, pid, asset_in, amount_in, asset_out, min_amount_out)?;
				Ok(())
			}
			PoolType::None => Err(Error::<T>::PoolNotExists.into())
		}
	}

	pub fn share_balance_of_pool(
		who: &T::AccountId,
		pid: &PoolId,
	) -> Balance {
		match PoolInstances::<T>::get(pid){
			PoolType::AmmPool => {
				PoolAmm::<T>::share_balance_of(who, pid)
			}
			PoolType::None => 0
		}
	}

	pub fn share_total_balance_from_pool(
		pid: &PoolId,
	) -> Balance {
		match PoolInstances::<T>::get(pid){
			PoolType::AmmPool => {
				PoolAmm::<T>::share_total_balance(pid)
			}
			PoolType::None => 0
		}
	}

	pub fn get_total_fee_from_pool(
		pid: &PoolId,
	) -> u32 {
		match PoolInstances::<T>::get(pid){
			PoolType::AmmPool => {
				PoolAmm::<T>::get_total_fee(pid)
			}
			PoolType::None => 0
		}
	}

	pub fn get_volume_data_from_pool(
		pid: &PoolId,
	) -> VolumeData {
		match PoolInstances::<T>::get(pid){
			PoolType::AmmPool => {
				PoolAmm::<T>::get_volume_data(pid)
			}
			PoolType::None => VolumeData::new()
		}
	}
}
