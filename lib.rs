#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use ink_storage::{
        collections::HashMap,
        lazy::Lazy,
    };

    #[ink(storage)]
    pub struct Erc20{
        //total supply of contract
        total_supply: Lazy<Balance>,
        //the balance of each user
        balance: HashMap<AccountId, Balance>,
        allowances: HashMap<(AccountId, AccountId), Balance>,
    }

    #[ink(event)]
    pub struct Transfer{
        #[ink(topic)]
        from : Option<AccountId>,
        #[ink(topic)]
        to : Option<AccountId>,
        value : Balance
    }

    #[ink(event)]
    pub struct Approval{
        #[ink(topic)]
        owner : AccountId,
        #[ink(topic)]
        spender : AccountId,
        value : Balance
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InsuffientBalance,
        InsuffientApproval,
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(supply : Balance) -> Self {
            let caller = Self::env().caller();
            let mut balance = HashMap::new();
            balance.insert(caller, supply);

            Self::env().emit_event(Transfer{
                from : None,
                to : Some(caller),
                value : supply,
            });

            Self {
                total_supply : Lazy::new(supply),
                balance : balance,
                allowances : HashMap::new(),
            }
        }

        #[ink(message)]
        pub fn total_supply(&self) -> Balance{
            *self.total_supply
        }

        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance.get(&owner).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
        }

        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()>{
            let from  = self.env().caller();
            self.inner_transfer(from, to, value)
        }

        #[ink(message)]
        pub fn approve(&mut self, spender : AccountId, value : Balance) -> Result<()>{
            let owner  = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval{
                owner : owner,
                spender : spender,
                value : value
            });
            Ok(())
        }
        
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()>{
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsuffientApproval);
            }

            self.inner_transfer(from, to, value)?;
            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }

        pub fn inner_transfer(
            &mut self, 
            from : AccountId,
            to : AccountId,
            value : Balance
        ) -> Result<()> {
            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsuffientBalance)
            }
            let to_balance = self.balance_of(to);
            self.balance.insert(from, from_balance - value);
            self.balance.insert(to, to_balance + value);
            self.env().emit_event(Transfer{
                from : Some(from),
                to : Some(to),
                value : value
            });
            Ok(())
        }

    }


}
