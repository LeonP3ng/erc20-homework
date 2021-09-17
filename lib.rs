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
        //授权转账的额度
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
        //授权金额溢出
        ApproveOverflow, 
        //减少授权金额时金额小于0的Error
        AllowanceBelowZero, 
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

        //所有代币数量
        #[ink(message)]
        pub fn total_supply(&self) -> Balance{
            *self.total_supply
        }

        //某个账户的代币
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            self.balance.get(&owner).copied().unwrap_or(0)
        }

        //查看授权的金额
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
        }

        //转账
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()>{
            let from  = self.env().caller();
            self.inner_transfer(from, to, value)
        }

        //设置授权转账的额度
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
        
        //增加授权转账的额度
        #[ink(message)]
        pub fn increase_approve(&mut self, spender: AccountId, value: Balance) -> Result<()>{
            let owner = self.env().caller();
            let origin_value = self.allowance(owner, spender);
            let new_value = origin_value.checked_add(value).ok_or(Error::ApproveOverflow)?;
            self.allowances.insert((owner, spender),  new_value);
            self.env().emit_event(Approval{
                owner : owner,
                spender : spender,
                value : new_value,
            });
            Ok(())
        }

        //减少授权转账的额度
        #[ink(message)]
        pub fn decrease_approve(&mut self, spender: AccountId, value: Balance) -> Result<()>{
            let owner = self.env().caller();
            let origin_value = self.allowance(owner, spender);
            if origin_value < value {
                return Err(Error::AllowanceBelowZero);
            }
            self.allowances.insert((owner, spender),  origin_value - value);
            self.env().emit_event(Approval{
                owner : owner,
                spender : spender,
                value : origin_value - value,
            });
            Ok(())
        }


        //授权转账
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()>{
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsuffientApproval)
            }
            self.inner_transfer(from, to, value)?;
            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }

        //内部转账操作
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