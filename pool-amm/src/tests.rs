// This file is part of PolkaExchange.

// Copyright (C) 2020-2021 John.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

mod tests {
    use crate::mock::*;
    use crate::Error;
    use frame_support::{assert_noop, assert_ok};
	use sp_std::collections::btree_map::BTreeMap;

	use base::*;

	pub fn symbol_data_to_string(sd: &SymbolData) -> String{
		let mut s = format!("{{");
		for (symbol, balance) in sd {
			s += &format!("[{}: {}],", symbol.to_string(), balance)
		}
		s += &format!("}}");
		s
	}

	pub fn volume_data_to_string(sd: &VolumeData) -> String{
		let mut s = format!("{{");
		for (symbol, swap_amount) in sd {
			s += &format!("[{}: {}, {}],", symbol.to_string(), swap_amount.input, swap_amount.output)
		}
		s += &format!("}}");
		s
	}

	fn test_add_liquidity(who: &crate::mock::AccountId, balance1: Balance, balance2: Balance){
		let mut amounts: SymbolData = BTreeMap::new();
		amounts.insert(TEST_SYMBOL1.clone(), balance1);
		amounts.insert(TEST_SYMBOL2.clone(), balance2);

		assert_ok!(PoolAmm::add_liquidity_to_pool(
            &who,
            &POOL_AMM,
			&amounts,
			)
		);
	}

	#[test]
    fn test_register_pool() {
		let mut symbol_data: SymbolData = BTreeMap::new();
		symbol_data.insert(TEST_SYMBOL1.clone(), 0);
		symbol_data.insert(TEST_SYMBOL2.clone(), 0);

		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			assert_noop!(PoolAmm::register_pool(
                &ALICE,
                &POOL_AMM,
                30,
				10,
				&symbol_data,
				None,
				),
				Error::<Runtime>::PoolAlreadyExists
			);

			assert_ok!(PoolAmm::unregister_pool(
                &ALICE,
                &POOL_AMM,
				)
			);

			assert_ok!(PoolAmm::register_pool(
                &ALICE,
                &POOL_AMM,
                30,
				10,
				&symbol_data,
				None,
				)
			);
		})
	}

	#[test]
	fn test_liquidity() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			test_add_liquidity(&BOB, 100, 30);

			let shares = PoolAmm::share_balance_of(&BOB, &POOL_AMM);
			let total_shares = PoolAmm::share_total_balance(&POOL_AMM);
			let amounts_in_pool = symbol_data_to_string(&PoolAmm::get_symbol_data(&POOL_AMM));
			sp_std::if_std! {
				println!(">>>test_liquidity[1]: shares: {}, total_shares: {}, amounts_in_pool: {}", shares, total_shares, amounts_in_pool);
			}

			let mut amounts: SymbolData = BTreeMap::new();
			amounts.insert(TEST_SYMBOL1.clone(), 5);
			amounts.insert(TEST_SYMBOL2.clone(), 1);

			assert_ok!(PoolAmm::remove_liquidity_from_pool(
                &BOB,
                &POOL_AMM,
				shares / 10,
				&amounts,
				)
			);

			let shares = PoolAmm::share_balance_of(&BOB, &POOL_AMM);
			let total_shares = PoolAmm::share_total_balance(&POOL_AMM);
			let amounts_in_pool = symbol_data_to_string(&PoolAmm::get_symbol_data(&POOL_AMM));
			sp_std::if_std! {
				println!(">>>test_liquidity[2]: shares: {}, total_shares: {}, amounts_in_pool: {}", shares, total_shares, amounts_in_pool);
			}

			assert_ok!(PoolAmm::remove_liquidity_from_pool(
                &BOB,
                &POOL_AMM,
				shares,
				&amounts,
				)
			);

			let shares = PoolAmm::share_balance_of(&BOB, &POOL_AMM);
			let total_shares = PoolAmm::share_total_balance(&POOL_AMM);
			let amounts_in_pool = symbol_data_to_string(&PoolAmm::get_symbol_data(&POOL_AMM));
			sp_std::if_std! {
				println!(">>>test_liquidity[3]: shares: {}, total_shares: {}, amounts_in_pool: {}", shares, total_shares, amounts_in_pool);
			}
			//assert!(false);
		})
	}

	#[test]
	fn test_swap() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			test_add_liquidity(&BOB, 100, 30);

			assert!(FungibleAsset::free_balance(
					&TEST_SYMBOL1,
					&BOB,
				) == Ok(100)
			);
			assert!(FungibleAsset::free_balance(
					&TEST_SYMBOL2,
					&BOB,
				) == Ok(30)
			);

			let expected_asset_out1 = PoolAmm::get_swap_return_asset(
				&POOL_AMM,
				&TEST_SYMBOL2,
				10,
				&TEST_SYMBOL1,
			);
			sp_std::if_std! {
				println!(">>>test_swap: expected_asset_out1: {}", expected_asset_out1);
			}
			
			assert_ok!(PoolAmm::swap_asset(
                &BOB,
                &POOL_AMM,
				&TEST_SYMBOL2,
				10,
				&TEST_SYMBOL1,
				1,
				)
			);

			assert!(FungibleAsset::free_balance(
					&TEST_SYMBOL1,
					&BOB,
				) == Ok(100 + expected_asset_out1)
			);
			assert!(FungibleAsset::free_balance(
					&TEST_SYMBOL2,
					&BOB,
				) == Ok(30 - 10)
			);

			let expected_asset_out2 = PoolAmm::get_swap_return_asset(
				&POOL_AMM,
				&TEST_SYMBOL2,
				10,
				&TEST_SYMBOL1,
			);
			sp_std::if_std! {
				println!(">>>test_swap: expected_asset_out2: {}", expected_asset_out2);
			}
			//assert!(false);
		})
	}

	#[test]
	fn test_swap_and_shares() {
		let mut ext = ExtBuilder::default().build();
		ext.execute_with(|| {
			test_add_liquidity(&ALICE, 1000, 800);
			test_add_liquidity(&BOB, 100, 30);

			let shares = PoolAmm::share_balance_of(&ALICE, &POOL_AMM);
			let total_shares = PoolAmm::share_total_balance(&POOL_AMM);
			let amounts_in_pool = symbol_data_to_string(&PoolAmm::get_symbol_data(&POOL_AMM));
			let volumes = volume_data_to_string(&PoolAmm::get_volume_data(&POOL_AMM));
			sp_std::if_std! {
				println!(">>>test_swap_liquidity[1][ALICE]: shares: {}, total_shares: {}, amounts_in_pool: {}, volumes: {}", shares, total_shares, amounts_in_pool, volumes);
			}

			let shares = PoolAmm::share_balance_of(&BOB, &POOL_AMM);
			let total_shares = PoolAmm::share_total_balance(&POOL_AMM);
			let amounts_in_pool = symbol_data_to_string(&PoolAmm::get_symbol_data(&POOL_AMM));
			let volumes = volume_data_to_string(&PoolAmm::get_volume_data(&POOL_AMM));
			sp_std::if_std! {
				println!(">>>test_swap_liquidity[1][BOB]: shares: {}, total_shares: {}, amounts_in_pool: {}, volumes: {}", shares, total_shares, amounts_in_pool, volumes);
			}

			assert_ok!(PoolAmm::swap_asset(
                &ALICE,
                &POOL_AMM,
				&TEST_SYMBOL1,
				300,
				&TEST_SYMBOL2,
				1,
				)
			);

			let shares = PoolAmm::share_balance_of(&ALICE, &POOL_AMM);
			let total_shares = PoolAmm::share_total_balance(&POOL_AMM);
			let amounts_in_pool = symbol_data_to_string(&PoolAmm::get_symbol_data(&POOL_AMM));
			let volumes = volume_data_to_string(&PoolAmm::get_volume_data(&POOL_AMM));
			sp_std::if_std! {
				println!(">>>test_swap_liquidity[2][ALICE]: shares: {}, total_shares: {}, amounts_in_pool: {}, volumes: {}", shares, total_shares, amounts_in_pool, volumes);
			}
			let shares = PoolAmm::share_balance_of(&BOB, &POOL_AMM);
			let total_shares = PoolAmm::share_total_balance(&POOL_AMM);
			let amounts_in_pool = symbol_data_to_string(&PoolAmm::get_symbol_data(&POOL_AMM));
			let volumes = volume_data_to_string(&PoolAmm::get_volume_data(&POOL_AMM));
			sp_std::if_std! {
				println!(">>>test_swap_liquidity[2][BOB]: shares: {}, total_shares: {}, amounts_in_pool: {}, volumes: {}", shares, total_shares, amounts_in_pool, volumes);
			}

			assert_ok!(PoolAmm::swap_asset(
                &BOB,
                &POOL_AMM,
				&TEST_SYMBOL2,
				10,
				&TEST_SYMBOL1,
				1,
				)
			);

			let shares = PoolAmm::share_balance_of(&ALICE, &POOL_AMM);
			let total_shares = PoolAmm::share_total_balance(&POOL_AMM);
			let amounts_in_pool = symbol_data_to_string(&PoolAmm::get_symbol_data(&POOL_AMM));
			let volumes = volume_data_to_string(&PoolAmm::get_volume_data(&POOL_AMM));
			sp_std::if_std! {
				println!(">>>test_swap_liquidity[3][ALICE]: shares: {}, total_shares: {}, amounts_in_pool: {}, volumes: {}", shares, total_shares, amounts_in_pool, volumes);
			}
			let shares = PoolAmm::share_balance_of(&BOB, &POOL_AMM);
			let total_shares = PoolAmm::share_total_balance(&POOL_AMM);
			let amounts_in_pool = symbol_data_to_string(&PoolAmm::get_symbol_data(&POOL_AMM));
			let volumes = volume_data_to_string(&PoolAmm::get_volume_data(&POOL_AMM));
			sp_std::if_std! {
				println!(">>>test_swap_liquidity[3][BOB]: shares: {}, total_shares: {}, amounts_in_pool: {}, volumes: {}", shares, total_shares, amounts_in_pool, volumes);
			}

			//assert!(false);
		})
	}
}
