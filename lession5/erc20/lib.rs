#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

#[ink::contract]
mod erc20 {
    use ink_storage::collections::HashMap as StorageHashMap;

    #[ink(storage)]
    pub struct Erc20 {
        total_supply: Balance,
        // 总发行量
        balances: StorageHashMap<AccountId, Balance>,
        // 余额
        allowance: StorageHashMap<(AccountId, AccountId), Balance>, // 允许别人使用的额度
    }

    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        value: Balance,
    }

    #[ink(event)]
    pub struct SetAllowance {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        value: Balance,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        InSufficientBalance,
        // 余额不足
        InSufficientAllowance, // 余额不足
    }

    pub type Result<T> = core::result::Result<T, Error>;

    impl Erc20 {
        #[ink(constructor)]
        pub fn new(total_supply: Balance) -> Self {
            let caller = Self::env().caller(); // 合约发行人
            let mut balances = StorageHashMap::new(); // 初始化
            balances.insert(caller, total_supply);
            let instance = Self {
                total_supply: total_supply,
                balances: balances,
                allowance: StorageHashMap::new(),
            };
            instance
        }

        /// 总发行量
        #[ink(message)]
        pub fn total_supply(&self) -> Balance {
            self.total_supply
        }

        /// owner 有多少钱
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance {
            *self.balances.get(&owner).unwrap_or(&0)
        }

        /// spender 可以花 owner 多少钱
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            // todo
            *self.allowance.get(&(owner, spender)).unwrap_or(&0)
        }

        /// 设置spender 可以花 owner 多少钱
        #[ink(message)]
        pub fn set_allowance(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller(); // 调用者
            self.allowance.insert((caller, spender), value);
            self.env().emit_event(SetAllowance {
                owner: caller,
                spender: spender,
                value: value,
            });
            Ok(())
        }

        /// 从from 转给 to value钱
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            // todo
            let caller = self.env().caller(); // 调用者
            let allowance = self.allowance(from, caller); // caller 可以花费from多少钱
            if allowance < value {
                return Err(Error::InSufficientAllowance);
            }
            match self.transfer_help(from, to, value) {
                Ok(_) => {
                    self.allowance.insert((from, caller), allowance - value);
                    Ok(())
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }

        // /// 销毁
        // #[ink(message)]
        // pub fn burn(&mut self) -> Balance {
        //     // todo
        //
        // }

        // #[ink(message)]
        // pub fn issue(&mut self) -> Balance {
        //     // todo 分发
        // }

        /// 转账
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            let who = self.env().caller(); // 调用者
            self.transfer_help(who, to, value)
        }

        fn transfer_help(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let from_balance = self.balance_of(from); // 转出人的额度
            if from_balance < value {
                return Err(Error::InSufficientBalance);
            }
            self.balances.insert(from, from_balance - value); // 减
            let to_balance = self.balance_of(to);
            self.balances.insert(to, to_balance + value); // 增

            self.env().emit_event(Transfer {
                from: from,
                to: to,
                value: value,
            });
            Ok(())
        }
    }

    /// Unit tests in Rust are normally defined within such a `#[cfg(test)]`
    /// module and test functions are marked with a `#[test]` attribute.
    /// The below code is technically just normal Rust code.
    #[cfg(test)]
    mod tests {
        use super::*;

        #[ink_lang::test]
        fn create_contract_works() {
            let erc20 = Erc20::new(1000);
            assert_eq!(erc20.total_supply(), 1000);
        }
    }
}
