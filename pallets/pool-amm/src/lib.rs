// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
extern crate alloc;

#[cfg(test)]
mod mock;

//#[cfg(test)]
//mod tests;

use frame_support::sp_runtime::traits::{MaybeSerializeDeserialize, Member};
use frame_support::{ensure, Parameter};
use frame_support::dispatch::{DispatchResult};
use frame_system::ensure_signed;
use sp_runtime::traits::Zero;
use sp_std::vec::Vec;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::cmp::min;

use alloc::string::String;
use alloc::format;

pub use base::*;

type FungibleAsset<T> = pallet_fungible_asset::Pallet<T>;
pub type PoolIdOf<T> = <T as Config>::PoolId;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

pub const INIT_SHARES_SUPPLY: u128 = 1_000_000_000_000_000_000_000_000;
pub const FEE_DIVISOR: u32 = 10_000;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_fungible_asset::Config
    {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type PoolId: Parameter
            + Member
            + Copy
            + MaybeSerializeDeserialize
            + Default
			+ From<FIXED_ARRAY>
			+ ToFixedArray;
	}

	#[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
    #[pallet::getter(fn pool_info_data)]
    pub type PoolInfoData<T: Config> = StorageMap<
        _,
        Twox64Concat,
        PoolId,
		PoolInfo<T::AccountId>,
        ValueQuery,
    >;

	#[pallet::storage]
    #[pallet::getter(fn pool_owners)]
    pub type PoolOwners<T: Config> = StorageMap<
        _,
        Twox64Concat,
        PoolId,
		T::AccountId,
        ValueQuery,
    >;

	#[pallet::call]
    impl<T: Config> Pallet<T> {
		#[pallet::weight(10_000_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn register(
			origin: OriginFor<T>,
			pid: PoolId,
			total_fee: u32,
			exchange_fee: u32,
			symbol_data: SymbolData,
			description: Option<StdString>,
		) -> DispatchResultWithPostInfo {
			let issuer = ensure_signed(origin)?;
			Self::register_pool(
				&issuer,
				&pid,
				total_fee,
				exchange_fee,
				&symbol_data,
				description,
			)?;
            Ok(().into())
		}

		#[pallet::weight(1_000_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn unregister(
			origin: OriginFor<T>,
			pid: PoolId,
		) -> DispatchResultWithPostInfo {
			let issuer = ensure_signed(origin)?;
			Self::unregister_pool(
				&issuer,
				&pid,
			)?;
            Ok(().into())
		}

		#[pallet::weight(2_000_000 + T::DbWeight::get().reads_writes(1, 2))]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			pid: PoolId,
			amounts: SymbolData,
		) -> DispatchResultWithPostInfo {
			let issuer = ensure_signed(origin)?;
			Self::add_liquidity_to_pool(
				&issuer,
				&pid,
				&amounts,
			)?;
			Ok(().into())
		}

		#[pallet::weight(1_000_000 + T::DbWeight::get().reads_writes(1, 1))]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			pid: PoolId,
			shares: Balance,
			amounts: SymbolData,
		) -> DispatchResultWithPostInfo {
			let issuer = ensure_signed(origin)?;
			Self::remove_liquidity_from_pool(
				&issuer,
				&pid,
				shares,
				&amounts,
			)?;
			Ok(().into())
		}

		#[pallet::weight(2_000_000 + T::DbWeight::get().reads_writes(2, 3))]
		pub fn swap(
			origin: OriginFor<T>,
			pid: PoolId,
			asset_in: AssetSymbol,
			amount_in: Balance,
			asset_out: AssetSymbol,
			min_amount_out: Balance,
		) -> DispatchResultWithPostInfo {
			let issuer = ensure_signed(origin)?;
			Self::swap_asset(
				&issuer,
				&pid,
				&asset_in,
				amount_in,
				&asset_out,
				min_amount_out,
			)?;
			Ok(().into())
		}
	}

	#[pallet::event]
    #[pallet::metadata(AccountIdOf<T> = "AccountId", PoolIdOf<T> = "PoolId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
		PoolRegistered(PoolIdOf<T>, AccountIdOf<T>),
		PoolUnregistered(PoolIdOf<T>, AccountIdOf<T>),
		AddLiquidity(PoolIdOf<T>, AccountIdOf<T>, StdString, Balance),
		RemoveLiquidity(PoolIdOf<T>, AccountIdOf<T>, StdString, Balance),
		SwapAsset(PoolIdOf<T>, AccountIdOf<T>, AssetSymbol, Balance, AssetSymbol, Balance),
    }

	#[pallet::error]
    pub enum Error<T> {
		InvalidPoolId,
		PoolAlreadyExists,
		PoolNotExists,
		InvalidOwner,
		TooLargeFee,
		TooManySymbols,
		EmptySymbols,
		HasRemainingShares,
		SymbolNotExistsInAsset,
		SymbolNotExistsInPool,
		InvalidBalance,
		ZeroAmount,
		ZeroShares,
		NoEnoughShares,
		TooLessSharesAmount,
		NoEnoughSwapAmount,
		WrongInvariant,
    }

	#[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
		pub endowed_pool: Vec<(
			T::AccountId,
            PoolId,
			u32,
			u32,
			SymbolData,
			Option<StdString>,
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
			self.endowed_pool.iter().cloned().for_each(
                |(account_id, pid, total_fee, exchange_fee, symbol_data, description)| {
					let _symbol_data = symbol_data.iter().filter_map(|(symbol, value)| {
						match symbol.is_empty() == false {
							true => Some((*symbol, *value)),
							false => None,
							}
						}
					).collect::<SymbolData>();

					for (symbol, _) in &_symbol_data{
						if FungibleAsset::<T>::is_asset_existed(symbol) == false {
							FungibleAsset::<T>::register_asset(
								&account_id,
								symbol,
								&AssetName::default(),
								18,
								true,
								true,
								None,
								0,
							).expect("Failed to register asset.");
						}
					}
					
					if _symbol_data.len() > 0 {
                    	Pallet::<T>::register_pool(
							&account_id,
							&pid,
							total_fee,
							exchange_fee,
							&_symbol_data,
							description,
                    	)
                    	.expect("GenesisBuild: Failed to register pool.");
					}
				},
			)
        }
    }
}

pub use pallet::*;

impl<T: Config> Pallet<T> {
	#[inline]
	pub fn is_valid_pool_id(pid: &PoolId) -> bool {
		crate::is_valid_id(&pid)
	}

	#[inline]
	pub fn get_pool_id(pid: &PoolId) -> T::PoolId{
		T::PoolId::from(pid.0)
	}

	pub fn get_pool_owner(pid: &PoolId) -> Option<T::AccountId> {
		let account_id = Self::pool_owners(&pid);
        if account_id == T::AccountId::default() {
            None
        } else {
            Some(account_id)
        }
    }

	pub fn ensure_pool_exists(pid: &PoolId) -> DispatchResult {
        if Self::get_pool_owner(pid).is_none() {
            return Err(Error::<T>::PoolNotExists.into());
        }
        Ok(())
    }

	pub fn register_pool(
		issuer: &T::AccountId,
		pid: &PoolId,
		total_fee: u32,
		exchange_fee: u32,
		symbol_data: &SymbolData,
		description: Option<StdString>,
	) -> DispatchResult {
		ensure!(
			Self::is_valid_pool_id(pid),
			Error::<T>::InvalidPoolId
		);
		ensure!(
				Self::get_pool_owner(pid).is_none(),
				Error::<T>::PoolAlreadyExists,
		);
		ensure!(
			total_fee < FEE_DIVISOR && exchange_fee <= total_fee,
			Error::<T>::TooLargeFee
		);
		ensure!(
			symbol_data.len() > 0,
			Error::<T>::EmptySymbols
		);
		ensure!(
			symbol_data.len() <= MAX_NUM_SYMBOLS,
			Error::<T>::TooManySymbols
		);

		for (symbol, _) in symbol_data{
			ensure!(
				symbol.is_empty() == false,
				pallet_fungible_asset::Error::<T>::InvalidAssetSymbol
			);
			ensure!(
				FungibleAsset::<T>::is_asset_existed(&symbol),
				pallet_fungible_asset::Error::<T>::AssetNotExists
			);
		}

		PoolOwners::<T>::insert(pid.clone(), issuer.clone());

		let pool_info = PoolInfo{pid: pid.clone()
								 , total_fee: total_fee
								 , exchange_fee: exchange_fee
								 , shares_total_supply: 0
								 , shares_data: Default::default()
								 , symbol_data: symbol_data.clone()
								 , volume_data: Default::default()
								 , description: description
								 };
		PoolInfoData::<T>::insert(pid.clone(), pool_info.clone());

		Self::deposit_event(Event::PoolRegistered(Self::get_pool_id(pid), issuer.clone()));

		let _symbol_data = symbol_data.iter().filter_map(|(key, value)| {
				match *value > 0 {
					true => Some((*key, *value)),
					false => None,
				}
			}
		).collect::<SymbolData>();
		if _symbol_data.len() > 0 {
			Self::add_liquidity_to_pool(issuer, pid, &_symbol_data)?;
		}
		Ok(())
	}
	
	pub fn unregister_pool(
		issuer: &T::AccountId,
		pid: &PoolId,
	) -> DispatchResult {
		Self::ensure_pool_exists(pid)?;
		ensure!(
			Self::get_pool_owner(pid) == Some(issuer.clone()),
			Error::<T>::InvalidOwner,
		);

		let pool_info = PoolInfoData::<T>::get(pid);
		ensure!(
			pool_info.shares_total_supply == 0,
			Error::<T>::HasRemainingShares,
		);

		PoolInfoData::<T>::remove(pid);
		PoolOwners::<T>::remove(pid);
		Self::deposit_event(Event::PoolUnregistered(Self::get_pool_id(pid), issuer.clone()));
		Ok(())
	}

	pub fn add_liquidity_to_pool(
		issuer: &T::AccountId,
		pid: &PoolId,
		amounts: &SymbolData,
	) -> DispatchResult {
		ensure!(
			amounts.len() > 0,
			Error::<T>::EmptySymbols
		);

		Self::ensure_pool_exists(pid)?;
		let mut pool_info = PoolInfoData::<T>::get(pid);

		for (symbol, balance) in amounts {
			ensure!(
				FungibleAsset::<T>::is_asset_existed(symbol),
				Error::<T>::SymbolNotExistsInAsset
			);
			ensure!(
				*balance > 0,
				Error::<T>::InvalidBalance
			);
			ensure!(
				pool_info.symbol_data.contains_key(&symbol),
				Error::<T>::SymbolNotExistsInPool
			);
		}

		let shares = if pool_info.shares_total_supply > 0 {
			let mut fair_supply = u128::max_value();
			for (symbol, balance) in amounts {
				let self_balance = pool_info.symbol_data.get(&symbol).unwrap();
                fair_supply = min(
                    fair_supply,
                    u128::from(*balance) * u128::from(pool_info.shares_total_supply) / u128::from(*self_balance),
                );
            }
			for (symbol, balance) in &mut amounts.clone() {
				let self_balance = pool_info.symbol_data.get_mut(symbol).unwrap();
				let amount = u128::from(*self_balance) * fair_supply / u128::from(pool_info.shares_total_supply);
				ensure!(
					amount > 0,
					Error::<T>::ZeroAmount
				);
				*self_balance += amount;
				*balance = amount;
			}
			fair_supply
		}
		else {
			for (symbol, balance) in amounts {
				let self_balance = pool_info.symbol_data.get_mut(symbol).unwrap();
				*self_balance += *balance;
			}
			INIT_SHARES_SUPPLY
		};

		ensure!(
			shares > 0,
			Error::<T>::ZeroShares
		);
		pool_info.shares_total_supply += shares;
		pool_info.shares_data.insert(issuer.clone(), shares);

		for (symbol, balance) in amounts {
			FungibleAsset::<T>::update_balance(symbol, issuer, *balance as Amount)?;
		}
		PoolInfoData::<T>::insert(pid, pool_info.clone());

		let symbol_list = format!("{:?}",
			amounts.iter().map(|(key, _)| format!("{}", key.to_string())).collect::<Vec<String>>());
		Self::deposit_event(Event::AddLiquidity(Self::get_pool_id(pid), issuer.clone(), StdString::from_string(&symbol_list), shares));
		Ok(())
	}

	pub fn remove_liquidity_from_pool(
		issuer: &T::AccountId,
		pid: &PoolId,
		shares: Balance,
		amounts: &SymbolData,
	) -> DispatchResult {
		Self::ensure_pool_exists(pid)?;
		let mut pool_info = PoolInfoData::<T>::get(pid);

		if pool_info.shares_data.contains_key(&issuer) == false {
			pool_info.shares_data.insert(issuer.clone(), Balance::zero());
		}
		let prev_shares_amount = pool_info.shares_data.get_mut(issuer).unwrap();
		
		ensure!(
			shares > 0 && *prev_shares_amount >= shares && pool_info.shares_total_supply >= shares,
			Error::<T>::NoEnoughShares
		);

		for (symbol, balance) in amounts {
			let self_balance = pool_info.symbol_data.get_mut(symbol).unwrap();
			let amount = u128::from(*self_balance) * u128::from(shares) / u128::from(pool_info.shares_total_supply);
			ensure!(
				amount >= *balance,
				Error::<T>::TooLessSharesAmount
			);
			*self_balance -= amount;
		}

		if *prev_shares_amount == shares {
            pool_info.shares_data.remove(issuer);
        } else {
            *prev_shares_amount -= shares;
        }
		pool_info.shares_total_supply -= shares;
		PoolInfoData::<T>::insert(pid, pool_info.clone());

		let symbol_list = format!("{:?}",
			amounts.iter().map(|(key, _)| format!("{}", key.to_string())).collect::<Vec<String>>());
		Self::deposit_event(Event::RemoveLiquidity(Self::get_pool_id(pid), issuer.clone(), StdString::from_string(&symbol_list), shares));
		Ok(())
	}

	pub fn get_swap_return_asset(
		pid: &PoolId,
		asset_in: &AssetSymbol,
		amount_in: Balance,
		asset_out: &AssetSymbol,
	) -> Balance {
		assert!(
			PoolInfoData::<T>::contains_key(pid),
			"pid not existing"
		);

		let pool_info = PoolInfoData::<T>::get(pid);

		let in_balance = pool_info.symbol_data.get(asset_in).unwrap();
        let out_balance = pool_info.symbol_data.get(asset_out).unwrap();
		assert!(
			*in_balance > 0
			&& *out_balance > 0
			&& asset_in != asset_out
			&& amount_in > 0,
			"Invalid swap data"
		);

		let amount_with_fee = u128::from(amount_in) * u128::from(FEE_DIVISOR - pool_info.total_fee);
        amount_with_fee * out_balance / (u128::from(FEE_DIVISOR) * in_balance + amount_with_fee)
	}

	pub fn swap_asset(
		who: &T::AccountId,
		pid: &PoolId,
		asset_in: &AssetSymbol,
		amount_in: Balance,
		asset_out: &AssetSymbol,
		min_amount_out: Balance,
	) -> DispatchResult {
		let amount_out = Self::get_swap_return_asset(pid, asset_in, amount_in, asset_out);
		ensure!(
			amount_out >= min_amount_out,
			Error::<T>::NoEnoughSwapAmount
		);

		FungibleAsset::<T>::ensure_can_withdraw(asset_in, who, amount_in)?;

		let mut pool_info = PoolInfoData::<T>::get(&pid);
		let prev_invariant: u128;
		{
			let in_balance = pool_info.symbol_data.get(asset_in).unwrap();
			let out_balance = pool_info.symbol_data.get(asset_out).unwrap();
			prev_invariant = integer_sqrt(*in_balance) * integer_sqrt(*out_balance);
		}

		let in_balance: u128;
		let out_balance: u128;
		{
			let in_balance_mut = pool_info.symbol_data.get_mut(asset_in).unwrap();
			*in_balance_mut += amount_in;
			in_balance = *in_balance_mut;
		}
		{
			let out_balance_mut = pool_info.symbol_data.get_mut(asset_out).unwrap();
			*out_balance_mut += amount_out;
			out_balance = *out_balance_mut;
		}
		let new_invariant = integer_sqrt(in_balance) * integer_sqrt(out_balance);
		ensure!(
			new_invariant >= prev_invariant,
			Error::<T>::WrongInvariant
		);

		let numerator = (new_invariant - prev_invariant) * u128::from(pool_info.shares_total_supply);
		if pool_info.exchange_fee > 0 && numerator > 0 {
            let denominator = new_invariant * u128::from(pool_info.total_fee) / u128::from(pool_info.exchange_fee);
			let shares = numerator / denominator;
			pool_info.shares_total_supply += shares;
            pool_info.shares_data.insert(who.clone(), shares);
        }

		FungibleAsset::<T>::update_balance(asset_in, who, -(amount_in as Amount))?;
		FungibleAsset::<T>::update_balance(asset_out, who, amount_out as Amount)?;

		let sv_in: SwapVolume;
		{
			let mut default_sv = SwapVolume::default();
			let sv_in_mut = pool_info.volume_data.get_mut(asset_in).unwrap_or(&mut default_sv);
			sv_in_mut.input += amount_in;
			sv_in = sv_in_mut.clone();
		}
		pool_info.volume_data.insert(asset_in.clone(), sv_in);
		let sv_out: SwapVolume;
		{
			let mut default_sv = SwapVolume::default();
			let sv_out_mut = pool_info.volume_data.get_mut(&asset_out).unwrap_or(&mut default_sv);
			sv_out_mut.output += amount_out;
			sv_out = sv_out_mut.clone();
		}
		pool_info.volume_data.insert(asset_out.clone(), sv_out);
		PoolInfoData::<T>::insert(pid, pool_info.clone());

		Self::deposit_event(Event::SwapAsset(Self::get_pool_id(pid), who.clone(), asset_in.clone(), amount_in, asset_out.clone(), amount_out));
		Ok(())
	}

	pub fn share_balance_of(
		who: &T::AccountId,
		pid: &PoolId,
	) -> Balance{
		if Self::get_pool_owner(pid).is_none() {
			return 0;
		}
		let pool_info = PoolInfoData::<T>::get(pid);
		*pool_info.shares_data.get(who).unwrap_or(&0)
	}

	pub fn share_total_balance(
		pid: &PoolId,
	) -> Balance{
		if Self::get_pool_owner(pid).is_none() {
			return 0;
		}
		let pool_info = PoolInfoData::<T>::get(pid);
		pool_info.shares_total_supply
	}

	pub fn get_symbol_data(
		pid: &PoolId,
	) -> SymbolData {
		if Self::get_pool_owner(pid).is_none() {
			return BTreeMap::new();
		}
		let pool_info = PoolInfoData::<T>::get(pid);
		pool_info.symbol_data.clone()
	}

	pub fn get_all_symbols(
		pid: &PoolId,
	) -> Vec<AssetSymbol>{
		let mut v = Vec::<AssetSymbol>::default();
		if Self::get_pool_owner(pid).is_none() {
			return v;
		}
		let pool_info = PoolInfoData::<T>::get(pid);
		for (symbol, _) in &pool_info.symbol_data{
			v.push(symbol.clone());
		}
		v
	}

	pub fn get_total_fee(
		pid: &PoolId,
	) -> u32 {
		if Self::get_pool_owner(pid).is_none() {
			return 0;
		}
		let pool_info = PoolInfoData::<T>::get(pid);
		pool_info.total_fee
	}

	pub fn get_volume_data(
		pid: &PoolId,
	) -> VolumeData {
		if Self::get_pool_owner(pid).is_none() {
			return VolumeData::default();
		}
		let pool_info = PoolInfoData::<T>::get(pid);
		pool_info.volume_data.clone()
	}

}
