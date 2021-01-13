#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure,
                    traits::{Randomness, Get, Currency, ExistenceRequirement::AllowDeath, ReservableCurrency},
};
use frame_system::{ensure_signed};
use sp_runtime::DispatchError;
use sp_io::hashing::blake2_128;


type KittyIndex = u32;

// ID
#[derive(Encode, Decode, Default)]
pub struct Kitty(pub [u8; 16]); // data

type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    // 2. runtime 指定 Kitty Index
    type KittyIndexValue: Get<u32>;
    type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>; // 6.质押
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		pub Kitties get(fn kitties): map hasher(blake2_128_concat) KittyIndex => Option<Kitty>;
		pub KittiesCount get(fn kitties_count): KittyIndex;
		pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) KittyIndex => Option<T::AccountId>;
		// 3. double_map记录账号所有kitty，double_map方便增删查
		pub AccountKitties get(fn account_kitties): double_map hasher(blake2_128_concat) T::AccountId,hasher(blake2_128_concat) KittyIndex => Option<KittyIndex>;
		// 4.记录其父母
		pub KittyParents get(fn kitty_parents): map hasher(blake2_128_concat) KittyIndex => (KittyIndex, KittyIndex);
		// 4.记录所有孩子
		pub KittyChidren get(fn kitty_chidren): double_map hasher(blake2_128_concat) KittyIndex,hasher(blake2_128_concat) KittyIndex => Option<KittyIndex>;
		// 4.伴侣
		pub KittyMate get(fn kitty_mate): map hasher(blake2_128_concat) (KittyIndex, KittyIndex) => Option<KittyIndex>;
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		Created(AccountId, KittyIndex),
		Transferred(AccountId, AccountId, KittyIndex),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		KittiesCountOverFlow,
		InvalidKittyId,
		RrquireDifferentParent,
		NotKittyOwner,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		#[weight = 0]
		pub fn create(origin, amount: BalanceOf<T>) { // 创建kitty
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?; // 取id
			let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);
            Self::insert_kitty(&sender, kitty_id, kitty);
            //质押
            T::Currency::reserve(&sender, amount).map_err(|_| "locker can't afford to lock the amount requested")?;
			Self::deposit_event(RawEvent::Created(sender, kitty_id));
		}

		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: KittyIndex, amount: BalanceOf<T>){
            let sender = ensure_signed(origin)?;
            let account_id = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?; // 1.bug，没有验证所有者
            ensure!(account_id == sender.clone(), Error::<T>::NotKittyOwner);
            let _kit = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
            Self::remove_account_kitty(&sender, kitty_id);
            Self::insert_account_kitty(&to, kitty_id);

            <KittyOwners<T>>::insert(kitty_id, to.clone());
            T::Currency::transfer(&sender, &to, amount, AllowDeath)?;
            Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
		}
        /// 孕育kitty
		#[weight = 0]
		pub fn breed(origin, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex, amount: BalanceOf<T>){
            let sender = ensure_signed(origin)?;
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
            //质押
            T::Currency::reserve(&sender, amount).map_err(|_| "locker can't afford to lock the amount requested")?;
			Self::deposit_event(RawEvent::Created(sender, new_kitty_id));
		}

	}
}
fn combine_dna(dna1: u8, dna2: u8, selector: u8) -> u8 {
    (selector & dna1) | (!selector & dna2)
}

impl<T: Trait> Module<T> {
    /// 孕育
    fn do_breed(sender: &T::AccountId, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex) -> sp_std::result::Result<KittyIndex, DispatchError> {
        // 查询两个Kitty存在
        let kitty1 = Self::kitties(kitty_id_1).ok_or(Error::<T>::InvalidKittyId)?;
        let kitty2 = Self::kitties(kitty_id_2).ok_or(Error::<T>::InvalidKittyId)?;
        // 验证非同一kitty
        ensure!(kitty_id_1 != kitty_id_2, Error::<T>::RrquireDifferentParent);
        // 下个id
        let kitty_id = Self::next_kitty_id()?;
        let kitty_1_dna = kitty1.0;
        let kitty_2_dna = kitty2.0;
        // dna
        let selector = Self::random_value(&sender); // dna
        let mut new_dna = [0u8; 16];
        for i in 0..kitty_1_dna.len() {
            new_dna[i] = combine_dna(kitty_1_dna[i], kitty_2_dna[i], selector[i]);
        }
        Self::insert_kitty(sender, kitty_id, Kitty(new_dna)); // 插入
        // 记录其父母
        KittyParents::insert(kitty_id, (kitty_id_1, kitty_id_2));
        // 记录孩子
        KittyChidren::insert(kitty_id_1, kitty_id, kitty_id);
        KittyChidren::insert(kitty_id_2, kitty_id, kitty_id);
        // 互为伴侣
        KittyMate::insert((kitty_id_1, kitty_id_2), kitty_id_1);
        Ok(kitty_id) // 返回
    }
    // 插入
    fn insert_kitty(owner: &T::AccountId, kitty_id: KittyIndex, kitty: Kitty) {
        Self::insert_account_kitty(owner, kitty_id);
        Kitties::insert(kitty_id, kitty); // 插入kitty
        KittiesCount::put(kitty_id + 1); // 下一个index
        <KittyOwners<T>>::insert(kitty_id, owner); // kitty所有者
    }
    fn insert_account_kitty(owner: &T::AccountId, kitty_id: KittyIndex) {
        <AccountKitties<T>>::insert(owner, kitty_id, kitty_id);
    }
    fn remove_account_kitty(owner: &T::AccountId, kitty_id: KittyIndex) {
        <AccountKitties<T>>::remove(owner, kitty_id);
    }
    fn next_kitty_id() -> sp_std::result::Result<KittyIndex, DispatchError> {
        let mut kitty_id = Self::kitties_count(); // 获取
        if kitty_id == 0 {
            kitty_id = T::KittyIndexValue::get();
        }
        if kitty_id == KittyIndex::max_value() {
            return Err(Error::<T>::KittiesCountOverFlow.into());
        }
        Ok(kitty_id)
    }

    fn random_value(sender: &T::AccountId) -> [u8; 16] {
        let payload = ( // hash data
                        T::Randomness::random_seed(),
                        &sender,
                        <frame_system::Module<T>>::extrinsic_index(),
        );
        payload.using_encoded(blake2_128) // 128 bit
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::H256;
    use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight, assert_ok, assert_noop,
                        traits::{OnFinalize, OnInitialize},
    };
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
    };
    use frame_system::{self as system};
    use pallet_balances;

    impl_outer_origin! {
	    pub enum Origin for Test {}
    }
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
        pub const ExistentialDeposit: u64 = 1;
        pub const KittyIndexValue: u32 = 0;
    }

    impl system::Trait for Test {
        type BaseCallFilter = ();
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = TestEvent;
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type DbWeight = ();
        type BlockExecutionWeight = ();
        type ExtrinsicBaseWeight = ();
        type MaximumExtrinsicWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type PalletInfo = ();
        type AccountData = pallet_balances::AccountData<u64>;
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
    }

    mod kitties_event {
        pub use crate::Event;
    }
    impl_outer_event! {
        pub enum TestEvent for Test {
            kitties_event<T>,
		    frame_system<T>,
		    pallet_balances<T>,
        }
    }

    impl pallet_balances::Trait for Test {
        type Balance = u64;
        type MaxLocks = ();
        type Event = TestEvent;
        type DustRemoval = ();
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = System;
        type WeightInfo = ();
    }

    type Randomness = pallet_randomness_collective_flip::Module<Test>;

    impl Trait for Test {
        type Event = TestEvent;
        type Randomness = Randomness;
        type KittyIndexValue = KittyIndexValue;
        type Currency = pallet_balances::Module<Self>;
    }

    pub type Kitties = Module<Test>;
    pub type System = frame_system::Module<Test>;

    fn run_to_block(n: u64) {
        while System::block_number() < n {
            Kitties::on_finalize(System::block_number());
            System::on_finalize(System::block_number());
            System::set_block_number(System::block_number() + 1);
            System::on_initialize(System::block_number());
            Kitties::on_initialize(System::block_number());
        }
    }

    pub fn new_test_ext() -> sp_io::TestExternalities {
        let mut t = system::GenesisConfig::default().build_storage::<Test>().unwrap();
        pallet_balances::GenesisConfig::<Test> {
            balances: vec![(1, 10000), (2, 11000), (3, 12000), (4, 13000), (5, 14000)],
        }.assimilate_storage(&mut t)
            .unwrap();
        let mut ext: sp_io::TestExternalities = t.into();
        ext.execute_with(|| System::set_block_number(1));
        ext
    }

    /// 创建kitty
    #[test]
    fn owned_kitties_can_append_values() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_ok!(Kitties::create(Origin::signed(1), 10));
            let create_event = TestEvent::kitties_event(Event::<Test>::Created(1u64, 0));
            assert_eq!( // 检查event
                        System::events()[1].event, create_event
            );
        })
    }

    /// 繁育kitty
    #[test]
    fn breed_kitties() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1), 10), Ok(())); // ID=0
            assert_eq!(Kitties::create(Origin::signed(1), 10), Ok(()));
            // ID=1
            assert_ok!(Kitties::breed(Origin::signed(1),0 ,1, 10)); // breed success
            let create_event = TestEvent::kitties_event(Event::<Test>::Created(1u64, 2));
            assert_eq!( // 检查event
                        System::events()[5].event, create_event
            );
        })
    }

    /// 繁育kitty，不存在的ID
    #[test]
    fn breed_kitties_not_found() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1), 10), Ok(()));
            // id=0
            assert_noop!( // id 不存在
                Kitties::breed(Origin::signed(1), 0, 1, 10),
                Error::<Test>::InvalidKittyId
            );
        })
    }

    /// 繁育kitty，父母相同ID
    #[test]
    fn breed_kitties_id_eq() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1), 10), Ok(()));
            //ID =0
            assert_noop!( // 父母id相同错误
                Kitties::breed(Origin::signed(1), 0, 0,10),
                Error::<Test>::RrquireDifferentParent
            );
        })
    }

    /// 繁育kitty，账号拥有所有的kitty
    #[test]
    fn breed_kitties_account_owned_kitties() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1), 10), Ok(())); //ID =0
            assert_eq!(Kitties::create(Origin::signed(1), 10), Ok(()));
            // id=1
            assert_ok!(Kitties::breed(Origin::signed(1),0 ,1, 10)); // breed success
            assert_eq!(AccountKitties::<Test>::iter_prefix_values(1).count(), 3); // 验证,该账号有3 kitty
        })
    }

    /// transfer kitty，检查event
    #[test]
    fn transfer_kitties() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_ok!(Kitties::create(Origin::signed(1), 10));
            // let id = Kitties::kitties_count();
            assert_ok!(Kitties::transfer(Origin::signed(1), 2 , 0, 10));
            let create_event = TestEvent::kitties_event(Event::<Test>::Transferred(1, 2, 0));
            assert_eq!( // 检查event
                        System::events()[3].event, create_event
            );
        })
    }

    /// transfer kitty，非kitty拥有者
    #[test]
    fn transfer_kitties_not_owner() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_ok!(Kitties::create(Origin::signed(1), 10));
            // let id = Kitties::kitties_count();
            assert_noop!(
                Kitties::transfer(Origin::signed(2), 1, 0, 10),
                Error::<Test>::NotKittyOwner //非拥有者
                );
        })
    }

    /// 账号拥有的所有kitty
    #[test]
    fn account_owned_kitties() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1), 10), Ok(()));
            assert_eq!(AccountKitties::<Test>::contains_key(1, 0), true); // 查看账号是否存在该kitty
            assert_eq!(Kitties::create(Origin::signed(1), 10), Ok(()));
            assert_eq!(Kitties::create(Origin::signed(1), 10), Ok(()));
            assert_eq!(Kitties::breed(Origin::signed(1), 1, 2, 10), Ok(()));
            assert_eq!(AccountKitties::<Test>::iter_prefix_values(1).count(), 4); // 验证账号共 3 kitty
        })
    }

    /// transfer kitty后，验证账号kitty数量
    #[test]
    fn transfer_kitties_validate_account_count() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_ok!(Kitties::create(Origin::signed(1), 10));
            assert_ok!(Kitties::transfer(Origin::signed(1), 2 , 0, 10)); // 转移给2
            assert_eq!(AccountKitties::<Test>::iter_prefix_values(1).count(), 0); // 转移后没有了kitty
            assert_eq!(AccountKitties::<Test>::iter_prefix_values(2).count(), 1); // 一个kitty
        })
    }
    /// parents
    #[test]
    fn kitty_parent_children_count() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_ok!(Kitties::create(Origin::signed(1), 10)); //0
            assert_ok!(Kitties::create(Origin::signed(1), 10)); //1
            assert_ok!(Kitties::breed(Origin::signed(1), 0 , 1, 10)); //2
            assert_ok!(Kitties::breed(Origin::signed(1), 0 , 1, 10)); //3
            assert_ok!(Kitties::create(Origin::signed(1), 10)); //4
            assert_ok!(Kitties::breed(Origin::signed(1), 2 , 4, 10)); //5
            assert_eq!(KittyParents::get(2), (0, 1)); // 验证其父母
            assert_eq!(KittyChidren::iter_prefix_values(0).count(), 2); // 两个孩子
            assert_eq!(KittyMate::get((0, 1)), Some(0)); // 0，1互为伴侣
            assert_eq!(KittyMate::get((2, 4)), Some(2)); // 2，4互为伴侣
        })
    }

}