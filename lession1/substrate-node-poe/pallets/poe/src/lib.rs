#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage, decl_event, decl_error, ensure};
use frame_system::ensure_signed;
use sp_std::vec::Vec;
use frame_support::traits::Get;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    type MaxProofLength: Get<usize>;
}

decl_storage! {
	trait Store for Module<T: Trait> as PoeModule {
		Proofs:map hasher(blake2_128_concat) Vec<u8> => (T::AccountId, T::BlockNumber);
	}
}

decl_event!(
	pub enum Event<T> where AccountId = <T as frame_system::Trait>::AccountId {
		ClaimCreated(AccountId, Vec<u8>),
		ClaimRevoked(AccountId, Vec<u8>),
		TransferClaim(AccountId, Vec<u8>, AccountId),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		ProofAlreadyClaimed,
		NoSuchProof,
		NotProofOwner,
		ProofTooLong,
	}
}

decl_module! {
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;
        // const MaxLength: u32 = T::MaxProofLength::get() as u32;

		#[weight = 10_000]
		pub fn claim_created(origin, proof: Vec<u8>){

			let sender = ensure_signed(origin)?;
			ensure!(proof.len() <= T::MaxProofLength::get(), Error::<T>::ProofTooLong); // 限制长度

			ensure!(!Proofs::<T>::contains_key(&proof), Error::<T>::ProofAlreadyClaimed);
			let current_block = <frame_system::Module<T>>::block_number();
			Proofs::<T>::insert(&proof,(sender.clone(),current_block));
			Self::deposit_event(RawEvent::ClaimCreated(sender, proof));
		}

		#[weight = 10_000]
		pub fn claim_revoked(origin, proof: Vec<u8>){
			let sender = ensure_signed(origin)?;

			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);
			let (owner, _) = Proofs::<T>::get(&proof);
			ensure!(sender==owner,Error::<T>::NotProofOwner);
			Proofs::<T>::remove(&proof);

			Self::deposit_event(RawEvent::ClaimRevoked(sender, proof));

		}

		#[weight = 10_000]
		pub fn transfer_claim(origin, proof: Vec<u8>, dest: T::AccountId){
			let sender = ensure_signed(origin)?;

			ensure!(Proofs::<T>::contains_key(&proof), Error::<T>::NoSuchProof);
			let (owner, _block_number) = Proofs::<T>::get(&proof);
			ensure!(sender==owner,Error::<T>::NotProofOwner);
			Proofs::<T>::insert(&proof,(dest.clone(),frame_system::Module::<T>::block_number()));

			Self::deposit_event(RawEvent::TransferClaim(sender, proof, dest));

		}
	}
}