// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "std")]
#[allow(unused)]
use std::fmt;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode};
use frame_support::sp_runtime::traits::{MaybeSerializeDeserialize, Member};
use frame_support::{ensure, Parameter};
use frame_support::dispatch::{DispatchResult};
use frame_support::{RuntimeDebug};
use sp_std::collections::btree_set::BTreeSet;

pub use base::*;


#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct TokenInfo<AccountId, Data> {
    pub owner: AccountId,
    pub metadata: TokenMetadata,
    pub data: Data,
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct NftClass<AccountId, Data> {
	pub owner: AccountId,
	pub data: Data,
	pub tokens: BTreeSet<NftTokenId>,
}

pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
		type Data: Parameter + Member + MaybeSerializeDeserialize;
	}

	pub type TokenInfoOf<T> = TokenInfo<<T as frame_system::Config>::AccountId, <T as Config>::Data>;
	pub type NftClassOf<T> = NftClass<<T as frame_system::Config>::AccountId, <T as Config>::Data>;

	#[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	#[pallet::error]
	pub enum Error<T> {
		NftClassIdAlreadyExisted,
		NftClassIdNotExisted,
		NoClassPermission,
		InvalidTokenOwner,
		NoPermission,
		CannotDestroyNftClass,
		TokenIdAlreadyExisted,
		TokenIdNotExisted,
	}

	#[pallet::event]
    #[pallet::metadata(AccountIdOf<T> = "AccountId", NftClassId = "NftClassId", NftTokenId = "NftTokenId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
		NftClassCreated(AccountIdOf<T>, NftClassId),
        NftClassDestroyed(AccountIdOf<T>, NftClassId),
        NftTokenMint(AccountIdOf<T>, NftClassId, NftTokenId),
		NftTokenBurn(AccountIdOf<T>, NftClassId, NftTokenId),
		NftTokenTransfer(AccountIdOf<T>, AccountIdOf<T>, NftClassId, NftTokenId),
    }

	#[pallet::storage]
	#[pallet::getter(fn token_info_data)]
	pub type TokenInfos<T: Config> = StorageMap<_, Twox64Concat, NftTokenId, TokenInfoOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn nft_class_data)]
	pub type NftClasses<T: Config> = StorageMap<_, Twox64Concat, NftClassId, NftClassOf<T>>;

	#[pallet::storage]
	#[pallet::getter(fn accounts)]
	pub type Accounts<T: Config> = StorageDoubleMap<
		_, 
		Twox64Concat, 
		T::AccountId, 
		Twox64Concat, 
		(NftClassId, NftTokenId),
		(), 
		ValueQuery,
	>;

	pub type GenesisTokens<T> = (
		<T as frame_system::Config>::AccountId,
		NftClassId,
		TokenMetadata,
		<T as Config>::Data,
		BTreeSet<NftTokenId>,
	);

	#[pallet::genesis_config]
	pub struct GenesisConfig<T: Config> {
		pub endowed_nfts: Vec<GenesisTokens<T>>,
	}

	#[cfg(feature = "std")]
	impl<T: Config> Default for GenesisConfig<T> {
		fn default() -> Self {
			GenesisConfig { endowed_nfts: vec![] }
		}
	}

	#[pallet::genesis_build]
	impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
		fn build(&self) {
			self.endowed_nfts.iter().for_each(|nft_data| {
				for token_id in &nft_data.4 {
					Pallet::<T>::mint_token(
						&nft_data.0,
						&nft_data.1,
						token_id,
						&nft_data.2,
						&nft_data.3,
					).expect("Token mint cannot fail during genesis");
				}
			})
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(1_000)]
		pub fn create_class(
			origin: OriginFor<T>,
			class_id: NftClassId,
			data: T::Data,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
            Pallet::<T>::create_nft_class(&who, &class_id, &data)?;
            Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn destroy_class(
			origin: OriginFor<T>,
			class_id: NftClassId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
            Pallet::<T>::destroy_nft_class(&who, &class_id)?;
            Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: NftClassId,
			token_id: NftTokenId,
			metadata: TokenMetadata,
			data: T::Data,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
            Pallet::<T>::mint_token(&who, &class_id, &token_id, &metadata, &data)?;
            Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn burn(
			origin: OriginFor<T>,
			class_id: NftClassId,
			token_id: NftTokenId,
		) -> DispatchResultWithPostInfo {
			let who = ensure_signed(origin)?;
            Pallet::<T>::burn_token(&who, &class_id, &token_id)?;
            Ok(().into())
		}

		#[pallet::weight(1_000)]
		pub fn transfer(
			origin: OriginFor<T>,
			to: T::AccountId,
			class_id: NftClassId,
			token_id: NftTokenId,
		) -> DispatchResultWithPostInfo {
			let from = ensure_signed(origin)?;
            Pallet::<T>::transfer_token(&from, &to, &class_id, &token_id)?;
            Ok(().into())
		}
	}
}

pub use pallet::*;

impl<T: Config> Pallet<T> {
	pub fn create_nft_class(
		owner: &T::AccountId,
		class_id: &NftClassId,
		data: &T::Data,
	) -> DispatchResult {
		ensure!(NftClasses::<T>::contains_key(class_id) == false, Error::<T>::NftClassIdAlreadyExisted);

		let nft = NftClass{
							owner: owner.clone(),
							data: data.clone(),
							tokens: Default::default(),
						   };
		NftClasses::<T>::insert(class_id.clone(), nft);

		Self::deposit_event(Event::NftClassCreated(owner.clone(), class_id.clone()));
		Ok(())
	}

	pub fn destroy_nft_class(
		owner: &T::AccountId,
		class_id: &NftClassId,
	) -> DispatchResult {
		NftClasses::<T>::try_mutate_exists(class_id, |nft_class_data| -> DispatchResult {
			let nft_class = nft_class_data.take().ok_or(Error::<T>::NftClassIdNotExisted)?;
			ensure!(nft_class.owner == *owner, Error::<T>::NoClassPermission);
			ensure!(nft_class.tokens.is_empty(), Error::<T>::CannotDestroyNftClass);

			NftClasses::<T>::remove(class_id);
			Self::deposit_event(Event::NftClassDestroyed(owner.clone(), class_id.clone()));
			Ok(())
		})
	}

	pub fn mint_token(
		owner: &T::AccountId,
		class_id: &NftClassId,
		token_id: &NftTokenId,
		metadata: &TokenMetadata,
		data: &T::Data,
	) -> DispatchResult {
		NftClasses::<T>::try_mutate_exists(class_id, |nft_class_data| -> DispatchResult {
			let nft_class = nft_class_data.as_mut().ok_or(Error::<T>::NftClassIdNotExisted)?;
			ensure!(nft_class.tokens.contains(token_id) == false, Error::<T>::TokenIdAlreadyExisted);
			nft_class.tokens.insert(token_id.clone());
			Ok(())
		})?;

		let token_info = TokenInfo{
			owner: owner.clone(),
			metadata: metadata.clone(),
			data: data.clone(),
		};
		
		TokenInfos::<T>::insert(token_id.clone(), token_info.clone());
		Accounts::<T>::insert(owner.clone(), (class_id.clone(), token_id.clone()), ());

		Self::deposit_event(Event::NftTokenMint(owner.clone(), class_id.clone(), token_id.clone()));
		Ok(())
	}

	pub fn burn_token(
		owner: &T::AccountId,
		class_id: &NftClassId,
		token_id: &NftTokenId,
	) -> DispatchResult {
		NftClasses::<T>::try_mutate_exists(class_id, |nft_class_data| -> DispatchResult {
			let token_info = TokenInfos::<T>::get(token_id).take().ok_or(Error::<T>::TokenIdNotExisted)?;
			ensure!(token_info.owner == *owner, Error::<T>::InvalidTokenOwner);

			let nft_class = &mut nft_class_data.take().ok_or(Error::<T>::NftClassIdNotExisted)?;
			ensure!(nft_class.owner == *owner, Error::<T>::NoClassPermission);
			nft_class.tokens.remove(token_id);

			TokenInfos::<T>::remove(token_id);
			Accounts::<T>::remove(owner.clone(), (class_id.clone(), token_id.clone()));

			Self::deposit_event(Event::NftTokenBurn(owner.clone(), class_id.clone(), token_id.clone()));
			Ok(())
		})
	}

	pub fn transfer_token(
		from: &T::AccountId,
		to: &T::AccountId,
		class_id: &NftClassId,
		token_id: &NftTokenId,
	) -> DispatchResult {
		TokenInfos::<T>::try_mutate_exists(token_id, |token_info_data| -> DispatchResult {
			ensure!(NftClasses::<T>::contains_key(class_id), Error::<T>::NftClassIdNotExisted);

			let token_info = token_info_data.as_mut().ok_or(Error::<T>::TokenIdNotExisted)?;
			ensure!(token_info.owner == *from, Error::<T>::InvalidTokenOwner);

			let nft_class = NftClasses::<T>::get(class_id).unwrap();
			ensure!(Self::is_owner(from, class_id, token_id), Error::<T>::NoPermission);
			ensure!(nft_class.tokens.contains(token_id) == true, Error::<T>::TokenIdNotExisted);

			if from == to {
				return Ok(());
			}
			
			token_info.owner = to.clone();
			let token = (class_id.clone(), token_id.clone());
			Accounts::<T>::remove(from.clone(), token.clone());
			Accounts::<T>::insert(to.clone(), token.clone(), ());

			Self::deposit_event(Event::NftTokenTransfer(from.clone(), to.clone(), class_id.clone(), token_id.clone()));
			Ok(())
		})
	}

	pub fn is_owner(
		account: &T::AccountId,
		class_id: &NftClassId,
		token_id: &NftTokenId,
	) -> bool {
		Accounts::<T>::contains_key(account, (class_id, token_id))
	}

	pub fn is_token_existed(
		class_id: &NftClassId,
		token_id: &NftTokenId,
	) -> bool {
		NftClasses::<T>::contains_key(class_id) && TokenInfos::<T>::contains_key(token_id)
	}

}
