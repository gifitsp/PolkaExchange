// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use codec::{Decode, Encode};
use frame_support::sp_runtime::traits::{One, Zero};
use frame_support::{ensure};
use frame_support::dispatch::{DispatchResult, DispatchError};
use frame_support::{RuntimeDebug};
use frame_system::ensure_signed;
use sp_std::vec::Vec;
use sp_std::ops::{Mul, Sub};
use core::convert::TryInto;

pub use base::*;

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum TokenType<AssetId> {
    Notset,
    FT(AssetId),
    NFT(NftClassId, NftTokenId),
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Farmer<AccountId, BlockNumber> {
    pub farmer_id: AccountId,
    pub stake_amount: Balance,//total staked asset amount, 1 for NFT
    pub last_reward_block: BlockNumber,
    pub rewards: Balance,
}

impl<AccountId, BlockNumber> Default for Farmer<AccountId, BlockNumber>
where
    AccountId: Default,
    BlockNumber:
        Default + Copy + TryInto<u32> + PartialOrd + Sub<Output = BlockNumber> + Mul<Output = BlockNumber>,
{
    fn default() -> Self {
        Self{
            farmer_id: AccountId::default(),
            stake_amount: Zero::zero(),
            last_reward_block: BlockNumber::default(),
            rewards: Zero::zero(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Farm<AccountId, AssetId> {
    pub owner: AccountId,
    pub stake_token_type: TokenType<AssetId>,
    pub shares_asset_id: AssetId, //liquidity reward asset
    pub shares_per_block: Balance,
    pub total_stake_amount: Balance,
    pub reward_weight: u8, //must be positive integer, should compare with each other among all farms as ref
}

type FungibleAsset<T> = pallet_fungible_asset::Pallet<T>;
type NFT<T> = pallet_nft::Pallet<T>;

pub type AssetIdOf<T> = <T as pallet_fungible_asset::Config>::AssetId;
pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;

impl<AccountId, AssetId> Default for Farm<AccountId, AssetId>
where
    AccountId: Default,
    AssetId: Default,
{
    fn default() -> Self {
        Self{
            owner: AccountId::default(),
            stake_token_type: TokenType::<AssetId>::Notset,
            shares_asset_id: AssetId::default(),
            shares_per_block: Zero::zero(),
            total_stake_amount: Zero::zero(),
            reward_weight: One::one(),
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_fungible_asset::Config + pallet_nft::Config
    {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(PhantomData<T>);

	#[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::storage]
    #[pallet::getter(fn farm_data)]
    pub type FarmData<T: Config> = StorageMap<
        _,
        Twox64Concat,
        FarmId,
		Farm<T::AccountId, T::AssetId>,
        ValueQuery,
    >;

    #[pallet::storage]
	#[pallet::getter(fn farmer_data)]
	pub type FarmerData<T: Config> = StorageDoubleMap<
		_, 
		Twox64Concat, 
		FarmId,
		Twox64Concat, 
		T::AccountId, //FarmerId
		Farmer<T::AccountId, T::BlockNumber>, 
		ValueQuery,
	>;

    #[pallet::event]
    #[pallet::metadata(AccountIdOf<T> = "AccountId", FarmId = "FarmId")]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
		FarmCreated(T::AccountId, FarmId, TokenType<T::AssetId>),
        FarmDestroyed(T::AccountId, FarmId, TokenType<T::AssetId>),
        RewardsClaimed(T::AccountId, FarmId, Balance),
    }

    #[pallet::error]
    pub enum Error<T> {
        NftTokenNotFound,
        AssetIdNotFound,
        StakeAssetNotMintable,
        StakeAssetNotOwner,
        FarmIdAlreadyExisted,
        FarmIdNotExisted,
        ZeroRewardWeight,
        NoPermission,
        FarmInStaking,
        FarmerNotExisted,
        NFTFarmerAlreadyInUse,
        AmountOverflow,
        WrongStakeType,
        InvalidLastRewardBlockNumber,
        TooLargeStakeAmount,
        ZeroStakeAmount,
        NoEnoughRewards,
        IncRefError,
    }

    pub type GenesisFarmInfo<T> = (
        AccountIdOf<T>, 
        FarmId, 
        AssetIdOf<T>, 
        Option<AssetIdOf<T>>, 
        Option<(NftClassId, NftTokenId)>, 
        Balance, 
        u8,
    );

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub endowed_farms: Vec<GenesisFarmInfo<T>>,
    }

    #[cfg(feature = "std")]
    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self{
				 endowed_farms: vec![],
			}
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> GenesisBuild<T> for GenesisConfig<T> {
        fn build(&self) {
            self.endowed_farms.iter().cloned().for_each(|(owner, farm_id, shares_asset_id, stake_asset_id, stake_nft_ids, shares_per_block, reward_weight)| {
                Pallet::<T>::create_farm(&owner, 
                                         &farm_id, 
                                         &shares_asset_id, 
                                         stake_asset_id.clone(),
                                         stake_nft_ids.clone(), 
                                         shares_per_block, 
                                         reward_weight,
                                        ).expect("creat_farm cannot fail during genesis");
            })
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::weight(0)]
        pub fn call_create_farm(
            origin: OriginFor<T>,
            farm_id: FarmId,
            shares_asset_id: T::AssetId,
            stake_asset_id: Option<T::AssetId>,
            stake_nft_ids: Option<(NftClassId, NftTokenId)>,
            shares_per_block: Balance,
            reward_weight: u8,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Pallet::<T>::create_farm(&who, &farm_id, &shares_asset_id, stake_asset_id.clone(), stake_nft_ids.clone(), shares_per_block, reward_weight)?;
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn call_destroy_farm(
            origin: OriginFor<T>,
            farm_id: FarmId,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Pallet::<T>::destroy_farm(&who, &farm_id)?;
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn call_stake_asset(
            origin: OriginFor<T>,
            farm_id: FarmId,
            stake_amount: Option<Balance>, // will be ignored if stake type is NFT
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Pallet::<T>::stake_asset(&who, &farm_id, stake_amount)?;
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn call_unstake_asset(
            origin: OriginFor<T>,
            farm_id: FarmId,
            unstake_amount: Option<Balance>, // None means unstake all amount
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Pallet::<T>::unstake_asset(&who, &farm_id, unstake_amount)?;
            Ok(().into())
        }

        #[pallet::weight(0)]
        pub fn call_claim(
            origin: OriginFor<T>,
            farm_id: FarmId,
            reward_to_claim: Balance,
        ) -> DispatchResultWithPostInfo {
            let who = ensure_signed(origin)?;
            Pallet::<T>::claim(&who, &farm_id, reward_to_claim)?;
            Ok(().into())
        }

    }

}

pub use pallet::*;

impl<T: Config> Pallet<T> {
    /// stake_asset_id or stake_nft_ids must be either one
    pub fn create_farm(
        issuer: &T::AccountId,
        farm_id: &FarmId,
        shares_asset_id: &T::AssetId,
		stake_asset_id: Option<T::AssetId>,
        stake_nft_ids: Option<(NftClassId, NftTokenId)>,
        shares_per_block: Balance,
        reward_weight: u8,
	) -> DispatchResult {
        ensure!(FarmData::<T>::contains_key(farm_id) == false, Error::<T>::FarmIdAlreadyExisted);
        ensure!(reward_weight > 0, Error::<T>::ZeroRewardWeight);

        ensure!(FungibleAsset::<T>::is_asset_id_existed(&shares_asset_id), Error::<T>::AssetIdNotFound);
        ensure!(FungibleAsset::<T>::is_asset_id_mintable(&shares_asset_id), Error::<T>::StakeAssetNotMintable);
        ensure!(FungibleAsset::<T>::is_asset_id_owner(&shares_asset_id, issuer), Error::<T>::StakeAssetNotOwner);

        let stake_token_type = match stake_asset_id.is_none() {
            true => {
                let nft_ids_value = stake_nft_ids.unwrap().clone();
                ensure!(NFT::<T>::is_token_existed(&nft_ids_value.0, &nft_ids_value.1), Error::<T>::NftTokenNotFound);
                TokenType::<T::AssetId>::NFT(nft_ids_value.0, nft_ids_value.1)
            },
            false => {
                let stake_asset_id_value = stake_asset_id.unwrap();
                ensure!(FungibleAsset::<T>::is_asset_id_existed(&stake_asset_id_value), Error::<T>::AssetIdNotFound);
                TokenType::<T::AssetId>::FT(stake_asset_id_value.clone())
            }
        };

        let farm = Farm{
            owner: issuer.clone(),
            stake_token_type: stake_token_type.clone(),
            shares_asset_id: shares_asset_id.clone(),
            shares_per_block: shares_per_block,
            total_stake_amount: Zero::zero(),
            reward_weight: reward_weight,
        };

        frame_system::Pallet::<T>::inc_consumers(&issuer)
                    .map_err(|_| Error::<T>::IncRefError)?;

        FarmData::<T>::insert(farm_id.clone(), farm);
        Self::deposit_event(Event::FarmCreated(issuer.clone(), farm_id.clone(), stake_token_type.clone()));
        Ok(())
    }

    pub fn destroy_farm(
        issuer: &T::AccountId,
        farm_id: &FarmId,
	) -> DispatchResult {
        FarmData::<T>::try_mutate_exists(farm_id, |farm_data| -> DispatchResult {
            let farm = farm_data.take().ok_or(Error::<T>::FarmIdNotExisted)?;
            ensure!(farm.owner == *issuer, Error::<T>::NoPermission);
            ensure!(farm.total_stake_amount == Zero::zero(), Error::<T>::FarmInStaking);

            FarmData::<T>::remove(farm_id);
            Self::deposit_event(Event::FarmDestroyed(issuer.clone(), farm_id.clone(), farm.stake_token_type.clone()));
            Ok(())
        })
    }

    pub fn stake_asset(
        who: &T::AccountId,
        farm_id: &FarmId,
        stake_amount: Option<Balance>, // will be ignored if stake type is NFT
	) -> DispatchResult {
        FarmData::<T>::try_mutate_exists(farm_id, |farm_data| -> DispatchResult {
            let farm = farm_data.as_mut().ok_or(Error::<T>::FarmIdNotExisted)?;

            let mut farmer = Farmer::default();
            farmer.farmer_id = who.clone();
            let is_existed = FarmerData::<T>::contains_key(farm_id, who);

            if is_existed {
                Self::update_rewards(who, farm_id)?;
            }

            match farm.stake_token_type {
                TokenType::<T::AssetId>::NFT(nft_class_id, nft_token_id) => {
                    ensure!(is_existed == false, Error::<T>::NFTFarmerAlreadyInUse);

                    NFT::<T>::transfer_token(who, &farm.owner, &nft_class_id, &nft_token_id)?;
                    farmer.stake_amount = One::one();
                    farm.total_stake_amount = farmer.stake_amount;
                },
                TokenType::<T::AssetId>::FT(asset_id) => {
                    ensure!(stake_amount.is_some(), Error::<T>::ZeroStakeAmount);
                    if is_existed {
                        farmer = FarmerData::<T>::get(farm_id, who);
                    }

                    let stake_amount_value = stake_amount.unwrap_or(Zero::zero());
                    FungibleAsset::<T>::transfer_asset(who, &FungibleAsset::<T>::get_asset_symbol(&asset_id), &farm.owner, stake_amount_value)?;
                    farmer.stake_amount = farmer.stake_amount.checked_add(stake_amount_value).ok_or(Error::<T>::AmountOverflow)?;
                    farm.total_stake_amount = farm.total_stake_amount.checked_add(stake_amount_value).ok_or(Error::<T>::AmountOverflow)?;
                },
                _ => {
                    ensure!(false, Error::<T>::WrongStakeType);
                }
            }

            farmer.last_reward_block = <frame_system::Pallet<T>>::block_number();
            FarmerData::<T>::insert(farm_id.clone(), who.clone(), farmer);
            Ok(())
        })
    }

    pub fn unstake_asset(
        who: &T::AccountId,
        farm_id: &FarmId,
        unstake_amount: Option<Balance>, // None means unstake all amount
	) -> DispatchResult {
        FarmData::<T>::try_mutate_exists(farm_id, |farm_data| -> DispatchResult {
            let farm = farm_data.as_mut().ok_or(Error::<T>::FarmIdNotExisted)?;
            ensure!(FarmerData::<T>::contains_key(farm_id, who), Error::<T>::FarmerNotExisted);

            Self::update_rewards(who, farm_id)?;
            let mut farmer = FarmerData::<T>::get(farm_id, who);

            let stake_amount_value = if unstake_amount.is_none() {farmer.stake_amount} else {unstake_amount.unwrap()};
            ensure!(stake_amount_value <= farmer.stake_amount, Error::<T>::TooLargeStakeAmount);
            ensure!(stake_amount_value <= farm.total_stake_amount, Error::<T>::TooLargeStakeAmount);

            if stake_amount_value > 0 {
                match farm.stake_token_type {
                    TokenType::<T::AssetId>::NFT(nft_class_id, nft_token_id) => {
                        NFT::<T>::transfer_token(&farm.owner, who, &nft_class_id, &nft_token_id)?;
                    },
                    TokenType::<T::AssetId>::FT(asset_id) => {
                        FungibleAsset::<T>::transfer_asset(&farm.owner, &FungibleAsset::<T>::get_asset_symbol(&asset_id), who, stake_amount_value)?;
                    },
                    _ => {
                        ensure!(false, Error::<T>::WrongStakeType);
                    }
                }
                
                farmer.stake_amount = farmer.stake_amount.checked_sub(stake_amount_value).ok_or(Error::<T>::AmountOverflow)?;
                farm.total_stake_amount = farm.total_stake_amount.checked_sub(stake_amount_value).ok_or(Error::<T>::AmountOverflow)?;
                
                FarmerData::<T>::insert(farm_id.clone(), who.clone(), farmer);
                Self::remove_farmer(who, farm_id);
            }

            Ok(())
        })
    }

    pub fn get_farmer_stake_amount(
        who: &T::AccountId,
        farm_id: &FarmId,
	) -> Result<Balance, DispatchError> {
        ensure!(FarmerData::<T>::contains_key(farm_id, who), Error::<T>::FarmerNotExisted);
        let farmer = FarmerData::<T>::get(farm_id, who);
        Ok(farmer.stake_amount)
    }

    pub fn get_farm_total_stake_amount(
        farm_id: &FarmId,
	) -> Result<Balance, DispatchError> {
        ensure!(FarmData::<T>::contains_key(farm_id), Error::<T>::FarmIdNotExisted);
        let farm = FarmData::<T>::get(farm_id);
        Ok(farm.total_stake_amount)
    }

    pub fn calculate_reward(
        who: &T::AccountId,
        farm_id: &FarmId,
        block_number: &T::BlockNumber,
	) -> Balance {
        let mut reward = Balance::zero();

        if FarmData::<T>::contains_key(farm_id) == false {
            return reward;
        }
        if FarmerData::<T>::contains_key(farm_id, who) == false {
            return reward;
        }

        let farm = FarmData::<T>::get(farm_id);
        let farmer = FarmerData::<T>::get(farm_id, who);
        if *block_number < farmer.last_reward_block {
            return reward;
        }
        if farm.total_stake_amount == Zero::zero() {
            return reward;
        }

        let n_blocks: Balance = (*block_number - farmer.last_reward_block).try_into().unwrap_or(0u64).into();
        reward = farm.shares_per_block.checked_mul(n_blocks).unwrap_or(Balance::zero());
        reward = reward.checked_mul(farmer.stake_amount).unwrap_or(Balance::zero());

        let _reward = reward;
        reward = reward.checked_div(farm.total_stake_amount).unwrap_or(Balance::zero());
        if reward == Zero::zero() { // do mul before div to check if reward is not zero
            reward = _reward.checked_mul(farm.reward_weight as Balance).unwrap_or(Balance::zero());
            reward = reward.checked_div(farm.total_stake_amount).unwrap_or(Balance::zero());
        }
        else {
            reward = reward.checked_mul(farm.reward_weight as Balance).unwrap_or(Balance::zero());
        }
        reward
    }

    pub fn update_rewards(
        who: &T::AccountId,
        farm_id: &FarmId,
	) -> Result<Balance, DispatchError> {
        FarmerData::<T>::try_mutate_exists(farm_id, who, |farmer_data| -> Result<Balance, DispatchError> {
            let farmer = farmer_data.as_mut().ok_or(Error::<T>::FarmerNotExisted)?;
            ensure!(FarmData::<T>::contains_key(farm_id), Error::<T>::FarmIdNotExisted);

            let current_block = <frame_system::Pallet<T>>::block_number();
            ensure!(current_block >= farmer.last_reward_block, Error::<T>::InvalidLastRewardBlockNumber);

            let mut rewards = farmer.rewards;
            let reward = Self::calculate_reward(who, farm_id, &current_block);
            if reward > 0 {
                rewards = rewards.checked_add(reward).ok_or(Error::<T>::AmountOverflow)?;

                let farm = FarmData::<T>::get(farm_id);
                FungibleAsset::<T>::mint_asset(&farm.owner, &FungibleAsset::<T>::get_asset_symbol(&farm.shares_asset_id), reward)?;
            
                farmer.rewards = rewards;
            }
            farmer.last_reward_block = current_block;

            Ok(rewards)
        })
        
    }

    pub fn claim(
        who: &T::AccountId,
        farm_id: &FarmId,
        reward_to_claim: Balance, // zero mean all to claim
	) -> DispatchResult {
        Self::update_rewards(who, farm_id)?;

        FarmerData::<T>::try_mutate_exists(farm_id, who, |farmer_data| -> DispatchResult {
            let mut farmer = farmer_data.as_mut().ok_or(Error::<T>::FarmerNotExisted)?;
            ensure!(FarmData::<T>::contains_key(farm_id), Error::<T>::FarmIdNotExisted);
            let farm = FarmData::<T>::get(farm_id);
            ensure!(reward_to_claim <= farmer.rewards, Error::<T>::NoEnoughRewards);

            let mut _reward_to_claim;
            if reward_to_claim == Zero::zero() {
                _reward_to_claim = farmer.rewards;
            }
            else {
                _reward_to_claim = reward_to_claim;
            }
            
            FungibleAsset::<T>::transfer_asset(&farm.owner, &FungibleAsset::<T>::get_asset_symbol(&farm.shares_asset_id), who, _reward_to_claim)?;
            farmer.rewards = farmer.rewards.checked_sub(_reward_to_claim).ok_or(Error::<T>::AmountOverflow)?;

            Self::deposit_event(Event::RewardsClaimed(who.clone(), farm_id.clone(), _reward_to_claim));
            Ok(())
        })?;

        Self::remove_farmer(who, farm_id);
        Ok(())
    }

    pub fn is_farmer_existed(
        who: &T::AccountId,
        farm_id: &FarmId,
	) -> bool {
        FarmerData::<T>::contains_key(farm_id, who)
    }

    pub fn remove_farmer(
        who: &T::AccountId,
        farm_id: &FarmId,
	) -> bool {
        let farmer = FarmerData::<T>::get(farm_id, who);
        let farm = FarmData::<T>::get(farm_id);

        if farmer.stake_amount == Zero::zero() && farmer.rewards == Zero::zero() {
            match farm.stake_token_type {
                TokenType::<T::AssetId>::NFT(_, _) => {
                    if FarmerData::<T>::contains_key(&farm_id, &who) {
                        FarmerData::<T>::remove(&farm_id, &who);
                        return true;
                    }
                },
                _ => ()
            };
        }
        false
    }

}
