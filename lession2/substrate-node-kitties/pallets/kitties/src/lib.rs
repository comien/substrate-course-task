#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure,
                    traits::{Randomness, Get},
};
use frame_system::{ensure_signed};
use sp_runtime::DispatchError;
use sp_io::hashing::blake2_128;

// use sp_std::vec::Vec;

type KittyIndex = u32;

// ID
#[derive(Encode, Decode, Default)]
pub struct Kitty(pub [u8; 16]); // data

#[derive(Encode, Decode, Default)]
pub struct KittyTree{
    m :KittyIndex,
    f :KittyIndex,
    children: Vec<KittyIndex>
}

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type Randomness: Randomness<Self::Hash>;
    type KittyIndexValue: Get<u32>; // runtime 指定 Kitty Index
}

decl_storage! {
	trait Store for Module<T: Trait> as Kitties {
		pub Kitties get(fn kitties): map hasher(blake2_128_concat) KittyIndex => Option<Kitty>;
		pub KittiesCount get(fn kitties_count): KittyIndex;
		pub KittyOwners get(fn kitty_owner): map hasher(blake2_128_concat) KittyIndex => Option<T::AccountId>;
		pub AccountKitties get(fn account_kitties): double_map hasher(blake2_128_concat) T::AccountId,hasher(blake2_128_concat) KittyIndex => Kitty; // 3.所有kitties
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
		pub fn create(origin) { // 创建kitty
			let sender = ensure_signed(origin)?;
			let kitty_id = Self::next_kitty_id()?; // 取id
			let dna = Self::random_value(&sender);
            let kitty = Kitty(dna);

            Self::insert_kitty(&sender, kitty_id, kitty);
			Self::deposit_event(RawEvent::Created(sender, kitty_id));
		}

		#[weight = 0]
		pub fn transfer(origin, to: T::AccountId, kitty_id: KittyIndex){
            let sender = ensure_signed(origin)?;
            let account_id = Self::kitty_owner(kitty_id).ok_or(Error::<T>::InvalidKittyId)?; // todo course bug，没有验证所有者
            ensure!(account_id == sender.clone(), Error::<T>::NotKittyOwner);
            let kit = Self::kitties(kitty_id).ok_or(Error::<T>::InvalidKittyId)?;
            Self::remove_account_kitty(&sender, kitty_id);
            Self::insert_account_kitty(&to, kitty_id, &kit);

            <KittyOwners<T>>::insert(kitty_id, to.clone());
            Self::deposit_event(RawEvent::Transferred(sender, to, kitty_id));
		}
        /// 孕育kitty
		#[weight = 0]
		pub fn breed(origin, kitty_id_1: KittyIndex, kitty_id_2: KittyIndex){
            let sender = ensure_signed(origin)?;
            let new_kitty_id = Self::do_breed(&sender, kitty_id_1, kitty_id_2)?;
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
        Ok(kitty_id) // 返回
    }
    // 插入
    fn insert_kitty(owner: &T::AccountId, kitty_id: KittyIndex, kitty: Kitty) {
        Self::insert_account_kitty(owner, kitty_id, &kitty);
        Kitties::insert(kitty_id, kitty); // 插入kitty
        KittiesCount::put(kitty_id + 1); // 下一个index
        <KittyOwners<T>>::insert(kitty_id, owner); // kitty所有者
    }
    fn insert_account_kitty(owner: &T::AccountId, kitty_id: KittyIndex, kitty: &Kitty) {
        <AccountKitties<T>>::insert(owner, kitty_id, kitty);
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
    use frame_support::{impl_outer_origin, parameter_types, weights::Weight, assert_ok, assert_noop, debug,
                        traits::{OnFinalize, OnInitialize},
    };
    use sp_runtime::{
        traits::{BlakeTwo256, IdentityLookup}, testing::Header, Perbill,
    };
    use frame_system as system;

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
        pub const KittyIndexValue: u32 = 1;
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
        type Event = ();
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
        type AccountData = ();
        type OnNewAccount = ();
        type OnKilledAccount = ();
        type SystemWeightInfo = ();
    }

    type Randomness = pallet_randomness_collective_flip::Module<Test>;

    impl Trait for Test {
        type Event = ();
        type Randomness = Randomness;
        type KittyIndexValue = KittyIndexValue;
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
        system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
    }

    /// 创建kitty
    #[test]
    fn owned_kitties_can_append_values() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
        })
    }

    /// breed
    #[test]
    fn breed_kitties() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
            assert_noop!( // id 不存在
                Kitties::breed(Origin::signed(1), 1, 2),
                Error::<Test>::InvalidKittyId
            );
            assert_noop!( // 父级相同错误
                Kitties::breed(Origin::signed(1), 1, 1),
                Error::<Test>::RrquireDifferentParent
            );
            assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
            assert_ok!(Kitties::breed(Origin::signed(1),1 ,2)); // breed success
            assert_eq!(AccountKitties::<Test>::iter_prefix_values(1).count(), 3); // 验证该账号有3 kitty
        })
    }

    /// transfer kitty
    #[test]
    fn transfer_kitties() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_ok!(Kitties::create(Origin::signed(1)));
            let id = Kitties::kitties_count();
            assert_ok!(Kitties::transfer(Origin::signed(1), 2 , id-1)); // transfer to account 2
            // 转移
            assert_noop!(
                Kitties::transfer(Origin::signed(1), 2, id -1 ),
                Error::<Test>::NotKittyOwner //非拥有者
                );
        })
    }

    /// 账号拥有的所有kitty
    #[test]
    fn account_owned_kitties() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
            let id = Kitties::kitties_count();
            debug::info!("输出index:{}", id);
            assert_eq!(AccountKitties::<Test>::contains_key(1, id - 1), true); // 查看账号是否存在该kitty
            assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
            assert_eq!(Kitties::create(Origin::signed(1)), Ok(()));
            assert_eq!(AccountKitties::<Test>::iter_prefix_values(1).count(), 3); // 验证账号共 3 kitty
        })
    }

    /// transfer kitty后，验证账号kitty数量
    #[test]
    fn transfer_kitties_validate_account_count() {
        new_test_ext().execute_with(|| {
            run_to_block(10);
            assert_ok!(Kitties::create(Origin::signed(1)));
            let id = Kitties::kitties_count();
            assert_noop!(
                Kitties::transfer(Origin::signed(2), 1, id -1 ),
                Error::<Test>::NotKittyOwner //非拥有者
                );
            assert_ok!(Kitties::transfer(Origin::signed(1), 2 , id-1)); // 转移给2
            assert_eq!(AccountKitties::<Test>::iter_prefix_values(1).count(), 0); // 转移后没有了kitty
            assert_eq!(AccountKitties::<Test>::iter_prefix_values(2).count(), 1); // 转移过来一个
            assert_noop!(
                Kitties::transfer(Origin::signed(1), 1, id ),
                Error::<Test>::InvalidKittyId // 不存在
                );
        })
    }
}