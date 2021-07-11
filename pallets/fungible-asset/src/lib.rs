// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg_attr(not(feature = "std"), no_std)]

#[allow(unused_imports)]
use sp_std::marker::PhantomData;

#[cfg(feature = "std")]
#[allow(unused)]
use std::fmt;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

use frame_support::sp_runtime::traits::{MaybeSerializeDeserialize, Member};
use frame_support::{ensure, Parameter};
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_system::{ensure_root, ensure_signed};
use sp_std::vec::Vec;
use traits::{
    MultiCurrency, MultiCurrencyExtended, MultiLockableCurrency, MultiReservableCurrency,
};

pub use base::*;

pub const MAX_PRECISION: u8 = 18;

pub type AssetIdOf<T> = <T as Config>::AssetId;
type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
type CurrencyIdOf<T> = <<T as Config>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + tokens::Config
    {
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type AssetId: Parameter
            + Member
            + Copy
            + MaybeSerializeDeserialize
            + Default
			+ From<FIXED_ARRAY>
			+ ToFixedArray
            + Into<CurrencyIdOf<Self>>
            + Into<<Self as tokens::Config>::CurrencyId>;

        /// Currency to transfer, reserve/unreserve, lock/unlock assets
        type Currency: MultiLockableCurrency<
                Self::AccountId,
                Moment = Self::BlockNumber,
                CurrencyId = Self::AssetId,
                Balance = Balance,
            > + MultiReservableCurrency<Self::AccountId, CurrencyId = Self::AssetId, Balance = Balance>
            + MultiCurrencyExtended<Self::AccountId, Amount = Amount>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::storage]
    #[pallet::getter(fn asset_permission)]
    pub type AssetPermission<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AssetId,
		(bool,),
        ValueQuery,
    >;

	#[pallet::storage]
    #[pallet::getter(fn asset_info_data)]
    pub type AssetInfoData<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AssetId,
		AssetInfo,
        ValueQuery,
    >;

	#[pallet::storage]
    #[pallet::getter(fn asset_owner_list)]
    pub type AssetOwnerList<T: Config> = StorageMap<
        _,
        Twox64Concat,
        T::AssetId,
		T::AccountId,
        ValueQuery,
    >;

    #[pallet::call]
    impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000_000 + T::DbWeight::get().writes(1))]
		pub fn set_asset_permission(
			root: OriginFor<T>,
			symbol: AssetSymbol,
			allow_to_register: bool,
		) -> DispatchResultWithPostInfo {
			ensure_root(root)?;
			ensure!(
				crate::is_valid_symbol(&symbol),
				Error::<T>::InvalidAssetSymbol
			);

			let asset_id = Self::get_asset_id(&symbol);
			AssetPermission::<T>::insert(asset_id, (allow_to_register,));
			Ok(().into())
		}

		#[pallet::weight(10_000_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn register(
            origin: OriginFor<T>,
            symbol: AssetSymbol,
			name: AssetName,
			precision: BalancePrecision,
			is_mintable: bool,
			is_burnable: bool,
			description: Option<StdString>,
			initial_supply: Balance,
        ) -> DispatchResultWithPostInfo {
			ensure_root(origin.clone())?;
			let issuer = ensure_signed(origin)?;
			Self::register_asset(
				&issuer,
				&symbol,
				&name,
				precision,
				is_mintable,
				is_burnable,
				description,
				initial_supply,
			)?;
            Ok(().into())
        }

		#[pallet::weight(5_000_000 + T::DbWeight::get().writes(1))]
        pub fn mint(
            origin: OriginFor<T>,
            symbol: AssetSymbol,
			amount: Balance,
        ) -> DispatchResultWithPostInfo {
			let issuer = ensure_signed(origin.clone())?;
			Self::mint_asset(
				&issuer,
				&symbol,
				amount
			)?;
			Ok(().into())
		}

		#[pallet::weight(2_000_000 + T::DbWeight::get().reads_writes(1, 1))]
        pub fn burn(
            origin: OriginFor<T>,
            symbol: AssetSymbol,
			amount: Balance,
        ) -> DispatchResultWithPostInfo {
			let issuer = ensure_signed(origin.clone())?;
			Self::burn_asset(
				&issuer,
				&symbol,
				amount,
			)?;
			Ok(().into())
		}

		#[pallet::weight(3_000_000 + T::DbWeight::get().reads_writes(1, 2))]
        pub fn transfer(
            origin: OriginFor<T>,
            symbol: AssetSymbol,
			to: T::AccountId,
			amount: Balance,
        ) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin.clone())?;
			Self::transfer_asset(
				&from,
				&symbol,
				&to,
				amount,
			)?;
			Ok(().into())
		}
	}

    #[pallet::event]
    #[pallet::metadata(AccountIdOf<T> = "AccountId", AssetIdOf<T> = "AssetId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
		AssetRegistered(AssetIdOf<T>, AccountIdOf<T>),
		Mint(AccountIdOf<T>, AccountIdOf<T>, AssetIdOf<T>, Balance),
		Burn(AccountIdOf<T>, AssetIdOf<T>, Balance),
		Transfer(AccountIdOf<T>, AccountIdOf<T>, AssetIdOf<T>, Balance),
    }

    #[pallet::error]
    pub enum Error<T> {
		AssetNotAllowedToRegister,
		AssetAlreadyExists,
		InvalidAssetSymbol,
		InvalidAssetName,
		InvalidPrecision,
		InvalidOwner,
		AssetNotExists,
		AssetIsNotMintable,
		AssetIsNotBurnable,
		NoEnoughBalance,
		IncRefError,
    }

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub endowed_assets: Vec<(
			T::AccountId,
            AssetSymbol,
            AssetName,
            BalancePrecision,
			bool,
			bool,
			Option<StdString>,
            Balance,
        )>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                endowed_assets: Default::default(),
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
			self.endowed_assets.iter().cloned().for_each(
                |(account_id, symbol, name, precision, is_mintable, is_burnable, description, initial_supply)| {
                    Pallet::<T>::register_asset(
                        &account_id,
                        &symbol,
                        &name,
                        precision,
						is_mintable,
						is_burnable,
						description,
                        initial_supply,
                    )
                    .expect("Failed to register asset.");
				},
			)
        }
    }
}

impl<T: Config> Pallet<T> {
	#[inline]
	pub fn get_asset_id(symbol: &AssetSymbol) -> T::AssetId {
		T::AssetId::from(symbol.0)
	}

	#[inline]
	pub fn get_asset_symbol(asset_id: &T::AssetId) -> AssetSymbol {
		FixedString(asset_id.to_fixed_array())
	}

	pub fn get_asset_owner(asset_id: &T::AssetId) -> Option<T::AccountId> {
        let account_id = Self::asset_owner_list(&asset_id);
        if account_id == T::AccountId::default() {
            None
        } else {
            Some(account_id)
        }
    }

	pub fn ensure_asset_exists(asset_id: &T::AssetId) -> DispatchResult {
        if Self::get_asset_owner(asset_id).is_none() {
            return Err(Error::<T>::AssetNotExists.into());
        }
        Ok(())
    }

	pub fn register_asset(
        issuer: &T::AccountId,
        symbol: &AssetSymbol,
		name: &AssetName,
		precision: BalancePrecision,
		is_mintable: bool,
		is_burnable: bool,
		description: Option<StdString>,
		initial_supply: Balance,
        ) -> DispatchResult {
		ensure!(
			crate::is_valid_symbol(symbol),
			Error::<T>::InvalidAssetSymbol
		);
		ensure!(
			crate::is_valid_name(name),
			Error::<T>::InvalidAssetName
		);
		ensure!(
			precision <= MAX_PRECISION,
			Error::<T>::InvalidPrecision
		);

		let asset_id = Self::get_asset_id(symbol);
		ensure!(
			Self::get_asset_owner(&asset_id).is_none(),
			Error::<T>::AssetAlreadyExists,
		);
		if AssetPermission::<T>::contains_key(&asset_id){
			let permissions = AssetPermission::<T>::get(&asset_id);
			ensure!(
				permissions.0,
				Error::<T>::AssetNotAllowedToRegister
			);
		}

		frame_system::Pallet::<T>::inc_consumers(&issuer)
			.map_err(|_| Error::<T>::IncRefError)?;
		AssetOwnerList::<T>::insert(asset_id.clone(), issuer.clone());

		if initial_supply > 0 {
			T::Currency::deposit(asset_id.clone(), &issuer, initial_supply)?;
		}
		let asset_info = AssetInfo{symbol: symbol.clone()
								 , name: name.clone()
								 , precision: precision
								 , is_mintable: is_mintable
								 , is_burnable: is_burnable
								 , description: description};
		AssetInfoData::<T>::insert(asset_id.clone(), asset_info);
		frame_system::Pallet::<T>::inc_account_nonce(&issuer);
		Self::deposit_event(Event::AssetRegistered(asset_id.clone(), issuer.clone()));
        Ok(())
    }

	pub fn mint_asset(
        issuer: &T::AccountId,
        symbol: &AssetSymbol,
		amount: Balance,
        ) -> DispatchResult {
		let asset_id = Self::get_asset_id(&symbol);
		Self::ensure_asset_exists(&asset_id)?;
		let assetinfo = AssetInfoData::<T>::get(asset_id);
		ensure!(assetinfo.is_mintable, Error::<T>::AssetIsNotMintable);

		ensure!(
			Self::is_asset_owner(symbol, issuer),
			Error::<T>::InvalidOwner,
		);

		T::Currency::deposit(asset_id.clone(), issuer, amount)?;
		Self::deposit_event(Event::Mint(issuer.clone(), issuer.clone(), asset_id, amount));
		Ok(())
	}

	pub fn burn_asset(
        issuer: &T::AccountId,
        symbol: &AssetSymbol,
		amount: Balance,
        ) -> DispatchResult {
		let asset_id = Self::get_asset_id(&symbol);
		Self::ensure_asset_exists(&asset_id)?;
		let assetinfo = AssetInfoData::<T>::get(asset_id);
		ensure!(assetinfo.is_burnable, Error::<T>::AssetIsNotBurnable);

		ensure!(
			Self::is_asset_owner(symbol, issuer),
			Error::<T>::InvalidOwner,
		);

		T::Currency::withdraw(asset_id.clone(), &issuer, amount)?;
		Self::deposit_event(Event::Burn(issuer.clone(), asset_id, amount));
		Ok(())
	}

	pub fn transfer_asset(
        from: &T::AccountId,
        symbol: &AssetSymbol,
		to: &T::AccountId,
		amount: Balance,
        ) -> DispatchResult {
		let asset_id = Self::get_asset_id(&symbol);
		Self::ensure_asset_exists(&asset_id)?;
		ensure!(Self::free_balance(&symbol, &from).unwrap_or(0) >= amount, Error::<T>::NoEnoughBalance);

		T::Currency::transfer(asset_id.clone(), from, to, amount)?;
		Self::deposit_event(Event::Transfer(from.clone(), to.clone(), asset_id, amount));
		Ok(())
	}

	pub fn is_asset_existed(symbol: &AssetSymbol) -> bool {
		let asset_id = Self::get_asset_id(&symbol);
        Self::get_asset_owner(&asset_id).is_some()
    }

	pub fn is_asset_owner(symbol: &AssetSymbol, account_id: &T::AccountId) -> bool {
		let asset_id = Self::get_asset_id(&symbol);
        Self::get_asset_owner(&asset_id)
            .map(|x| &x == account_id)
            .unwrap_or(false)
    }

	pub fn total_issuance(symbol: &AssetSymbol) -> Result<Balance, DispatchError> {
		let asset_id = Self::get_asset_id(&symbol);
        Self::ensure_asset_exists(&asset_id)?;
        Ok(T::Currency::total_issuance(asset_id.clone()))
    }

	pub fn total_balance(
        symbol: &AssetSymbol,
        who: &T::AccountId,
		) -> Result<Balance, DispatchError> {
		let asset_id = Self::get_asset_id(symbol);
        Self::ensure_asset_exists(&asset_id)?;
        Ok(T::Currency::total_balance(asset_id.clone(), &who))
    }

    pub fn free_balance(
        symbol: &AssetSymbol,
        who: &T::AccountId,
		) -> Result<Balance, DispatchError> {
		let asset_id = Self::get_asset_id(&symbol);
        Self::ensure_asset_exists(&asset_id)?;
        Ok(T::Currency::free_balance(asset_id.clone(), who))
    }

	pub fn update_balance(
        symbol: &AssetSymbol,
        who: &T::AccountId,
		amount: Amount,
		) -> DispatchResult {
		let asset_id = Self::get_asset_id(symbol);
        Self::ensure_asset_exists(&asset_id)?;
        T::Currency::update_balance(asset_id.clone(), who, amount)
    }

    pub fn ensure_can_withdraw(
        symbol: &AssetSymbol,
        who: &T::AccountId,
        amount: Balance,
		) -> DispatchResult {
		let asset_id = Self::get_asset_id(&symbol);
        Self::ensure_asset_exists(&asset_id)?;
        T::Currency::ensure_can_withdraw(asset_id.clone(), who, amount)
    }

	pub fn list_registered_asset_symbols() -> Vec<AssetSymbol> {
        AssetInfoData::<T>::iter().map(|(key, _)| Self::get_asset_symbol(&key)).collect()
    }

    pub fn list_registered_asset_info_data(
		) -> Vec<AssetInfo> {
        AssetInfoData::<T>::iter()
            .map(|(_, asset_info)| {
                asset_info
            })
            .collect()
    }

    pub fn get_asset_info(
        symbol: AssetSymbol,
		) -> AssetInfo {
		let asset_id = Self::get_asset_id(&symbol);
        AssetInfoData::<T>::get(asset_id)
    }
}

