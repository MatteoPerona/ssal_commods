# Ssal Commods
This repository contains a decentralized [futures contract]('https://www.investopedia.com/terms/f/futurescontract.asp') implementation built with [Ink!]('https://github.com/paritytech/ink') smart contracts.

Before toying with this code I would encourage you to to first go through the [getting started]('https://use.ink/getting-started/setup') section of Ink's website.

## Preparation

To build smart contract found in this repo you need to have [`cargo-contract`](https://github.com/paritytech/cargo-contract) installed.

```sh
cargo install cargo-contract --force
```

`--force` updates to the most recent `cargo-contract` version.

## Build contract and generate metadata

First, clone the repo. Next, navigate to the root of the directory (e.g. `cd path_to_your_directory/ssal_commods`) and run the following command:

`cargo contract build`

You should now have an optimized `<contract-name>.wasm` file, a `metadata.json` file and a `<contract-name>.contract` file in the `target` folder of your contract.
The `.contract` file combines the Wasm and metadata into one file and can be used for instantiation.

## Deploying and Calling the Contract 

To play with the contract you need to first install the [substrate-contracts-node]('https://github.com/paritytech/substrate-contracts-node/releases') by following [these]('https://use.ink/getting-started/setup#installing-the-substrate-smart-contracts-node') instructions. 

Next, you'll want to spin up the node running `substrate-contracts-node`. Once that's running you can access the [Contracts UI]('https://contracts-ui.substrate.io/'). Follow [these]('https://use.ink/getting-started/running-substrate') instructions.

Finally, deploy your contract to the UI by following [this link]('https://use.ink/getting-started/deploy-your-contract').

## Functions

Note: All of the token functionality came from the [erc20 example]('https://github.com/paritytech/ink-examples/blob/main/erc20') from the ink-examples repo by paritytech.

### Read Only



### Write 

## Testing Guide
