#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod ssal_commods {
    use ink::storage::Mapping;
    use scale::{
        Decode,
        Encode,
    };

    pub type ContractId = u64;
    pub type Grams = u64;

    /// Used to query all contract specs at the same time.
    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CommodityContract {
        seller: Option<AccountId>,
        buyer: Option<AccountId>,
        price: Option<Balance>,
        total: Option<Balance>,
        volume: Option<Grams>,
        finality_block:Option<BlockNumber>,
        finalized: Option<bool>
    }

    #[ink(storage)]
    #[derive(Default)]
    pub struct SsalCommods {
        /// Commodity Contract Data
        /// Seller for a given contract.
        seller: Mapping<ContractId, AccountId>,
        /// Buyer for a given contract.
        buyer: Mapping<ContractId, AccountId>,
        /// Price to purchace a contract. 
        price: Mapping<ContractId, Balance>, 
        /// Amount the buyer pays on the finality date for a given contract.
        total: Mapping<ContractId, Balance>,
        /// Volume of product being sold as specified by the contract. 
        volume: Mapping<ContractId, Grams>, 
        /// Block number at which buyer's funds are locked if the seller doesn't.
        finality_block: Mapping<ContractId, BlockNumber>,
        /// Whether or not the contract has been finalized.
        finalized: Mapping<ContractId, bool>,
        /// Running count for contracts which doubles as the ContractId for each consecutive contract.
        contract_count: ContractId, 

        /// Token Data
        /// Total token supply.
        total_supply: Balance,
        /// Mapping from owner to number of owned token.
        balances: Mapping<AccountId, Balance>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: Mapping<(AccountId, AccountId), Balance>,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }
    
    /// Event emitted when a contract is created.
    #[ink(event)]
    pub struct NewContract {
        contract_id: ContractId,
        seller: AccountId,
        price: Balance,
        total: Balance,
        volume: Grams,
        finality_block: BlockNumber,
    }

    /// Event emitted when a contract is bought.
    #[ink(event)]
    pub struct ContractBought {
        contract_id: ContractId,
        buyer: AccountId,
        price: Balance,
        total: Balance,
    }

    /// Event emitted when a contract is finalized.
    #[ink(event)]
    pub struct ContractFinalized {
        contract_id: ContractId,
        buyer: AccountId,
        total: Balance,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Returned when the an account attempts to list a contract whose finality date precedes the current block. 
        InvalidBlockNumber,
        /// Returned for general cases where contract is not found (superset of SellerNotFound).
        ContractNotFound,
        /// Returned when there are no listed sellers for a given contract id.
        SellerNotFound,
        /// Returned if an account attempts to purchase a contract that already has a buyer.
        ContractAlreadyBought,
        /// Returned if not enough balance to fulfill a request is available.
        InsufficientBalance,
        /// Returned if not enough allowance to fulfill a request is available.
        InsufficientAllowance,
        /// Returned if finalize is called on a contract that has not been bought.
        ContractNotPurchased,
        /// Returned if a caller other than the buyer of a contract attempts to finalize.
        OnlyBuyerCanFinalize,
        /// Returned if a caller attempts to finalize a contract before its finality block
        CannotFinalizeBeforeFinalityBlock,
    }

    impl SsalCommods {
        /// Creates a new ssal contract.
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            // Initialize Commodity Contract Data   
            let seller = Mapping::default();
            let buyer = Mapping::default();
            let price = Mapping::default();
            let total = Mapping::default();
            let volume = Mapping::default();
            let finality_block = Mapping::default();
            let finalized = Mapping::default();
            let contract_count = 0;

            // Initialize Token Data 
            let mut balances = Mapping::default();
            let caller = Self::env().caller();
            balances.insert(caller, &total_supply);
            Self::env().emit_event(Transfer {
                from: None,
                to: Some(caller),
                value: total_supply,
            });

            // Initialize storage
            Self {
                seller,
                buyer,
                price, 
                total,
                volume, 
                finality_block,
                finalized,
                contract_count, 
                total_supply,
                balances,
                allowances: Default::default(),
            }
        }


        // READING DATA

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance_of_impl(&owner)
        }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `balance_of` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn balance_of_impl(&self, owner: &AccountId) -> Balance {
            self.balances.get(owner).unwrap_or_default()
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowance_impl(&owner, &spender)
        }

        /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set.
        ///
        /// # Note
        ///
        /// Prefer to call this method over `allowance` since this
        /// works using references which are more efficient in Wasm.
        #[inline]
        fn allowance_impl(&self, owner: &AccountId, spender: &AccountId) -> Balance {
            self.allowances.get((owner, spender)).unwrap_or_default()
        }

        /// Returns seller's AccountId at the given ContractId
        #[ink(message)]
        pub fn get_seller(&self, id: ContractId) -> Option<AccountId> {
            self.seller.get(id)
        }

        /// Returns buyer's AccountId at the given ContractId
        #[ink(message)]
        pub fn get_buyer(&self, id: ContractId) -> Option<AccountId> {
            self.buyer.get(id)
        }

        /// Returns price at the given ContractId
        #[ink(message)]
        pub fn get_price(&self, id: ContractId) -> Option<Balance> {
            self.price.get(id)
        }

        /// Returns total at the given ContractId
        #[ink(message)]
        pub fn get_total(&self, id: ContractId) -> Option<Balance> {
            self.total.get(id)
        }

        /// Returns volume at the given ContractId
        #[ink(message)]
        pub fn get_volume(&self, id: ContractId) -> Option<Grams> {
            self.volume.get(id)
        }

        /// Returns finality block at the given ContractId
        #[ink(message)]
        pub fn get_finality_block(&self, id: ContractId) -> Option<BlockNumber> {
            self.finality_block.get(id)
        }

        /// Checks whether or not a contract has been finalized
        #[ink(message)]
        pub fn is_finalized(&self, id: ContractId) -> Option<bool> {
            self.finalized.get(id)
        }

        /// Returns the contract count at the given ContractId.
        #[ink(message)]
        pub fn get_contract_count(&self) -> ContractId {
            self.contract_count
        }

        /// Returns all data at the given ContractId.
        #[ink(message)]
        pub fn get_contract(&self, id:ContractId) -> Result<CommodityContract, Error> {
            match self.seller.get(id) {
                Some(_) => {
                    Ok(CommodityContract{ 
                        seller: self.seller.get(id),
                        buyer: self.buyer.get(id),
                        price: self.price.get(id),
                        total: self.total.get(id),
                        volume: self.volume.get(id),
                        finality_block: self.finality_block.get(id),
                        finalized: self.finalized.get(id),
                    })
                }
                None => Err(Error::ContractNotFound)
            }
        }

        /// Returns the balance of the contract account 
        #[ink(message)]
        pub fn get_balance(&self) -> Balance {
            self.env().balance()
        }

        /// Returns the current block number
        #[ink(message)]
        pub fn get_block(&self) -> BlockNumber {
            self.env().block_number()
        }


        // WRITING DATA

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<(), Error> {
            let from = self.env().caller();
            self.transfer_from_to(&from, &to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with
        /// `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<(), Error> {
            let owner = self.env().caller();
            self.allowances.insert((&owner, &spender), &value);
            self.env().emit_event(Approval {
                owner,
                spender,
                value,
            });
            Ok(())
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the account balance of `from`.
        #[ink(message)]
        pub fn transfer_from(
            &mut self,
            from: AccountId,
            to: AccountId,
            value: Balance,
        ) -> Result<(), Error> {
            let caller = self.env().caller();
            let allowance = self.allowance_impl(&from, &caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance)
            }
            self.transfer_from_to(&from, &to, value)?;
            self.allowances
                .insert((&from, &caller), &(allowance - value));
            Ok(())
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn transfer_from_to(
            &mut self,
            from: &AccountId,
            to: &AccountId,
            value: Balance,
        ) -> Result<(), Error> {
            let from_balance = self.balance_of_impl(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance)
            }

            self.balances.insert(from, &(from_balance - value));
            let to_balance = self.balance_of_impl(to);
            self.balances.insert(to, &(to_balance + value));
            self.env().emit_event(Transfer {
                from: Some(*from),
                to: Some(*to),
                value,
            });
            Ok(())
        }

        /// Creates a new Ssal Contract
        /// 
        /// Adds inputted data to the relevant mappings, increments the contract 
        /// count, and adds the caller as the seller for the contract.
        /// 
        /// # Errors
        /// 
        /// Returns InvalidBlockNumber if the inputted finality_block is less than 
        /// the current block number.
        #[ink(message)]
        pub fn create_contract(
            &mut self,
            _price: Balance,
            _total: Balance,
            _volume: Grams,
            _finality_block: BlockNumber
        ) -> Result<(), Error> {
            // Check that finality block is valid
            if _finality_block < self.env().block_number(){
                return Err(Error::InvalidBlockNumber);
            }

            let caller = self.env().caller();
            // Set contract data into relevant mappings
            self.seller.insert(self.contract_count, &caller);
            self.price.insert(self.contract_count, &_price);
            self.total.insert(self.contract_count, &_total);
            self.volume.insert(self.contract_count, &_volume);
            self.finality_block.insert(self.contract_count, &_finality_block);
            self.finalized.insert(self.contract_count, &false);

            self.contract_count += 1;

            Self::env().emit_event(NewContract {
                contract_id: self.contract_count - 1,
                seller: caller,
                price: _price,
                total: _total,
                volume: _volume,
                finality_block: _finality_block,
            });

            Ok(())
        }

        /// Buy a contract given a ContractId
        /// 
        /// Transfers funds (the amount designated by price) from buyer's account to 
        /// seller's account.
        /// 
        /// Transfers funds (the amount designated by total) to the smart contract account 
        /// to lockup the buyer's assets.
        /// 
        /// Adds buyer's AccountId to the buyer mapping if all the transfers goes through.
        /// 
        /// # Errors
        /// 
        /// Returns ContractNotFound if the seller cannot be found for the given contract.
        /// 
        /// Returns ContractAlreadyBought if the contract already has a buyer listed.
        /// 
        /// Returns Insufficient Balance if the buyer does not have enough funds to
        /// cover both the contract price and total price of the product.
        #[ink(message)]
        pub fn buy_contract(&mut self, id: ContractId) -> Result<(), Error> {
            // Check wether contract exists
            let seller = match self.seller.get(id) {
                Some(p) => p,
                None => return Err(Error::ContractNotFound)
            };
            // Check whether there is already a buyer.
            match self.buyer.get(id) {
                Some(_) => return Err(Error::ContractAlreadyBought),
                None => ()
            }

            // Fetch transactional variables
            let caller = self.env().caller();
            let price = self.price.get(id).unwrap();
            let total = self.total.get(id).unwrap();
            // Check caller has enough money
            if self.balance_of(caller) < price + total {
                return Err(Error::InsufficientBalance)
            }

            // Run transfers
            self.transfer_from_to(&caller, &seller, price)?;
            self.transfer_from_to(&caller, &self.env().account_id(), total)?;
            // Add buyer to the relevant mapping
            self.buyer.insert(id, &caller);

            Self::env().emit_event(ContractBought {
                contract_id: id,
                buyer: caller,
                price: price,
                total: total,
            });

            Ok(())
        }

        /// Finalize the contract: transfer total from the contract account to the seller.
        /// 
        /// The buyer calls this function when they have received their product.
        /// 
        /// Only the buyer can call this function.
        /// 
        /// This function can only be called at or after the finality block.
        /// 
        /// # Errors
        /// 
        /// Returns ContractNotFound if there is no seller for the given contract.
        /// 
        /// Returns CannotFinalizeBeforeFinalityBlock if the caller attempts to 
        /// finalize the contract before the finality block.
        /// 
        /// Returns OnlyBuyerCanFinalize if a caller other than the buyer for the 
        /// given contract attempts to finalize.
        /// 
        /// In theory, would return InsufficientBalance if the contract account 
        /// does not have enough funds to pay the seller, but this should never
        /// occur.
        #[ink(message)]
        pub fn finalize(&mut self, id: ContractId) -> Result<(), Error> {
            // Check that contract exists.
            let seller = match self.seller.get(id) {
                Some(p) => p,
                None => return Err(Error::ContractNotFound)
            };
            // Check that current block >= to the finality block of the contract
            if self.finality_block.get(id).unwrap() >= self.env().block_number() {
                return Err(Error::CannotFinalizeBeforeFinalityBlock)
            }
            // Check that contract has been bought.
            let buyer = match self.buyer.get(id){
                Some(p) => p,
                None => { return Err(Error::ContractNotPurchased) }
            };
            // Check that buyer is caller
            if self.env().caller() != buyer {
                return Err(Error::OnlyBuyerCanFinalize)
            }

            // Transfer total from contract account to seller 
            let total = self.total.get(id).unwrap();
            self.transfer_from_to(&self.env().account_id(), &seller, total)?;

            self.finalized.insert(id, &true);

            Self::env().emit_event(ContractFinalized {
                contract_id: id,
                buyer: buyer,
                total: total,
            });

            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        // COMMOD TESTS

        /// We test if the default constructor does its job.
        #[ink::test]
        fn default_works() {
            let ssal = SsalCommods::new(100_000);
            assert_eq!(ssal.get_contract_count(), 0);
        }

        /// We test a simple use case of our contract.
        #[ink::test]
        fn it_works() {
            let mut ssal = SsalCommods::new(100_000);
            assert!(
                ssal.create_contract(10, 10000, 10, 20).is_err(), 
                "could not create contract"
            );
            assert_eq!(ssal.get_contract_count(), 1);
        }

        // TOKEN TESTS
    }


    /// This is how you'd write end-to-end (E2E) or integration tests for ink! contracts.
    ///
    /// When running these you need to make sure that you:
    /// - Compile the tests with the `e2e-tests` feature flag enabled (`--features e2e-tests`)
    /// - Are running a Substrate node which contains `pallet-contracts` in the background
    #[cfg(all(test, feature = "e2e-tests"))]
    mod e2e_tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;

        /// A helper function used for calling contract messages.
        use ink_e2e::build_message;

        /// The End-to-End test `Result` type.
        type E2EResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

        /// We test that we can upload and instantiate the contract using its default constructor.
        #[ink_e2e::test]
        async fn default_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = SsalCommodsRef::default();

            // When
            let contract_account_id = client
                .instantiate("SsalCommods", &ink_e2e::alice(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            // Then
            let get = build_message::<SsalCommodsRef>(contract_account_id.clone())
                .call(|SsalCommods| SsalCommods.get());
            let get_result = client.call_dry_run(&ink_e2e::alice(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            Ok(())
        }

        /// We test that we can read and write a value from the on-chain contract contract.
        #[ink_e2e::test]
        async fn it_works(mut client: ink_e2e::Client<C, E>) -> E2EResult<()> {
            // Given
            let constructor = SsalCommodsRef::new(false);
            let contract_account_id = client
                .instantiate("SsalCommods", &ink_e2e::bob(), constructor, 0, None)
                .await
                .expect("instantiate failed")
                .account_id;

            let get = build_message::<SsalCommodsRef>(contract_account_id.clone())
                .call(|SsalCommods| SsalCommods.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), false));

            // When
            let flip = build_message::<SsalCommodsRef>(contract_account_id.clone())
                .call(|SsalCommods| SsalCommods.flip());
            let _flip_result = client
                .call(&ink_e2e::bob(), flip, 0, None)
                .await
                .expect("flip failed");

            // Then
            let get = build_message::<SsalCommodsRef>(contract_account_id.clone())
                .call(|SsalCommods| SsalCommods.get());
            let get_result = client.call_dry_run(&ink_e2e::bob(), &get, 0, None).await;
            assert!(matches!(get_result.return_value(), true));

            Ok(())
        }
    }
}
