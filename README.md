## Introduction
A simple decentralized exchange application based on Polka/Substrate blockchain framework. The goal of it is to exchange as many tokens as possible on a simple DeFi platform. So far it is a beta version without any UI and more features are possibly in development.

## Features
* Low exchange fees
* AMM exchange mode like Uniswap
* Exchange can be crossing multiple pools so that liquidity can be aggregated very easily

## Build Dependencies
* latest rust nightly
* gcc 5.1 or above
* CMake3 or above
* Clang9 or above and llvm
* other dependencies explained by substrate: https://substrate.dev/docs/en/

## Build
    git clone https://github.com/gifitsp/PolkaExchange.git
    cd PolkaExchange
    cargo build --release

To rebuild you may run this command first:

    cargo clean -p node-polkaexchange

## Parameters and Subcommands Helps
    ./target/release/node-polkaexchange -h

## Unit Test
    cargo test

## Run
**single node**

    cargo run --release -- --dev
    
or

    ./target/release/node-polkaexchange --dev
    
**multi-node**

    cargo run --release -- --chain local
    
or

    ./target/release/node-polkaexchange --chain local
    
To run a fresh instance, you can run the following to clean local db:
    
    cargo run --release -- purge-chain --dev
    
or

    ./target/release/node-polkaexchange purge-chain --dev

## More Running Configs and Debugging Info
Please refer https://substrate.dev/docs/en/tutorials/start-a-private-network/customchain
