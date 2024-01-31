# Ssal Commods
This repository contains a decentralized [futures contract](https://www.investopedia.com/terms/f/futurescontract.asp) implementation built with [Ink!](https://github.com/paritytech/ink) smart contracts.

Before toying with this code I would encourage you to to first go through the [getting started](https://use.ink/getting-started/setup) section of Ink's website.

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

To play with the contract you need to first install the [substrate-contracts-node](https://github.com/paritytech/substrate-contracts-node/releases) by following [these](https://use.ink/getting-started/setup#installing-the-substrate-smart-contracts-node) instructions. 

Next, you'll want to spin up the node running `substrate-contracts-node`. Once that's running you can access the [Contracts UI](https://contracts-ui.substrate.io/). Follow [these](https://use.ink/getting-started/running-substrate) instructions.

Finally, deploy your contract to the UI by following [this link](https://use.ink/getting-started/deploy-your-contract).

## Documentation
Note: All of the token functionality came from the [erc20 example](https://github.com/paritytech/ink-examples/blob/main/erc20) from the ink-examples repo by paritytech. Here we'll only cover functions realted to commodity contract logic. I also wont cover all the read-only functions since they are self-explanatory.

### Creating a contract.
`create_contract` takes as input:
* `_price` of type `Balance`: the price of the contract itself.
* `_total` of type `Balance`: the total price of the product being sold.
* `_volume` of type `Grams`: the weight, in grams, of the product being sold.
* `_finality_block` of type `BlockNumber`: the block after which the contract can be finalized. 

The function then adds input data to the relevant mappings, increments the contract count, and adds the caller as the seller for the contract.
#### Errors: 
Returns `InvalidBlockNumber` if `_finality_block` is less than the current block number 

### Buying a contract.
`buy_contract` takes as input `id`, a `ContractId` specifying which contract the caller wants to buy. 

The function transfers the amount designated by `price` from the caller's account to the seller's account and the amount designated by `total` from the caller's accont to the contract account. Then, it adds the caller's account to the `buyer` mapping using `id` as its key. 

#### Errors:
Returns `ContractNotFound` if the seller cannot be found for the given contract.

Returns `ContractAlreadyBought` if the contract already has a buyer listed.

Returns `InsufficientBalance` if the buyer does not have enough funds to cover both the contract price and total price of the product.

### Finalizing a contract.

`finalize` takes as input `id`, a `ContractId` specifying which contract the caller wants to finalize. This function transfers the funds locked in the contract account to the seller's account. 

The buyer calls this function when they have received their product. Only the buyer can call this function. This function can only be called at or after the finality block.

#### Errors:
Returns `ContractNotFound` if there is no seller for the given contract.

Returns `CannotFinalizeBeforeFinalityBlock` if the caller attempts to finalize the contract before the finality block.

Returns `OnlyBuyerCanFinalize` if a caller other than the buyer for the given contract attempts to finalize. In theory, would return `InsufficientBalance` if the contract account does not have enough funds to pay the seller, but this should neveroccur.

Returns `ContractAlreadyFinalized` if the caller tries to finalize a contract that has already been finalized. 

Returns `ContractNotPurchased` if the caller tries to finalize a contract that has not been purchased. 

## Testing Guide
Run `cargo test --features e2e-tests` to run both unit and end-to-end tests. Otherwise, run `cargo test` for only unit tests. 
