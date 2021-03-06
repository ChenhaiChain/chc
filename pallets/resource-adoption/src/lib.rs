#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    storage::StorageDoubleMap,
    traits::{Currency, ExistenceRequirement::AllowDeath},
};
use frame_system::{self as system, ensure_signed};
use sp_std::{
    convert::{From, TryInto},
    prelude::*,
};

pub trait Trait: system::Trait + timestamp::Trait + balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    // type Currency: Currency<Self::AccountId>;
}
// type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;
type ResourceId = Vec<u8>;

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Resource<B, M> {
    pub id: ResourceId,
    pub freeze_at: M,
    pub harvest_before: M,
    pub price: B,
    pub min_output_kg: u16,
    pub info: Vec<u8>,
    // pub photo_url: Vec<u8>,
    // pub photo: Vec<u8>,
    // pub rtype: Vec<u8>,
    // pub year: u16,
    // pub price: u32,
    // pub geohash: Vec<u8>,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Contract<K, M> {
    pub adopter: K,
    pub start_at: M,
    pub end_at: M,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct ResourceEvent {
    pub event: Vec<u8>,
}

decl_storage! {
    trait Store for Module<T: Trait> as ResourceAdoption {
      Resources get(fn retrieve_resource): double_map
        hasher(blake2_128_concat) T::AccountId,
        hasher(blake2_128_concat) ResourceId
      => (T::BlockNumber, Resource<T::Balance, T::Moment>);

      Contracts get(fn retrieve_contract): double_map
        hasher(blake2_128_concat) T::AccountId,
        hasher(blake2_128_concat) ResourceId
      => (T::BlockNumber, Contract<T::AccountId, T::Moment>);

      Events get(fn retrieve_events): double_map
        hasher(blake2_128_concat) T::AccountId,
        hasher(blake2_128_concat) ResourceId
      => (T::BlockNumber, ResourceEvent);
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
    {
        ResourceOnline(AccountId, ResourceId),
        ResourceAdopted(AccountId, ResourceId, AccountId),
        ResourceStateChanged(AccountId, ResourceId, Vec<u8>),
        ResourceOffline(AccountId, ResourceId),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        ResourceAlreadyExist,
        ResourceNotExist,
        ResourceAdopted,
        ResourceFreezed,
        IllegalAdopter,
        IllegalTimestamp,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        #[weight = 100]
        pub fn publish_resource(origin,
                                id: ResourceId,
                                price: T::Balance,
                                min_output_kg: u16,
                                freeze_at: T::Moment,
                                harvest_before: T::Moment,
                                info: Vec<u8>)
                                -> dispatch::DispatchResult {
            let publisher = ensure_signed(origin)?;
            ensure!(!Resources::<T>::contains_key(&publisher, &id), Error::<T>::ResourceAlreadyExist);
            let now = <timestamp::Module<T>>::get();
            ensure!(now < freeze_at && now < harvest_before && freeze_at < harvest_before, Error::<T>::IllegalTimestamp);
            let res = Resource {
                id: id,
                price: price,
                min_output_kg: min_output_kg,
                freeze_at: freeze_at,
                harvest_before: harvest_before,
                info: info,
            };
            let current_block = <system::Module<T>>::block_number();
            let id = res.id.clone();
            Resources::<T>::insert(publisher.clone(), res.id.clone(), (current_block, res));
            Self::deposit_event(RawEvent::ResourceOnline(publisher, id));
            Ok(())
        }

        #[weight = 0]
        pub fn revoke_resource(origin, id: ResourceId) -> dispatch::DispatchResult {
            let owner = ensure_signed(origin)?;
            ensure!(Resources::<T>::contains_key(&owner, &id), Error::<T>::ResourceNotExist);
            ensure!(!Contracts::<T>::contains_key(&owner, &id), Error::<T>::ResourceAdopted);
            Resources::<T>::remove(&owner, &id);
            Self::deposit_event(RawEvent::ResourceOffline(owner, id));
            Ok(())
        }

        // TODO weight to fee
        #[weight = 100]
        pub fn adopt(origin, owner: T::AccountId, resource_id: ResourceId) -> dispatch::DispatchResult {
            let adopter = ensure_signed(origin)?;
            ensure!(Resources::<T>::contains_key(&owner, &resource_id), Error::<T>::ResourceNotExist);
            ensure!(!Contracts::<T>::contains_key(&owner, &resource_id), Error::<T>::ResourceAdopted);
            ensure!(owner != adopter, Error::<T>::IllegalAdopter);
            let (_, res) = Resources::<T>::get(&owner, &resource_id);
            let now = <timestamp::Module<T>>::get();
            ensure!(now < res.freeze_at, Error::<T>::ResourceFreezed);
            <balances::Module<T> as Currency<_>>::transfer(&adopter, &owner, res.price, AllowDeath)?;
            let contract = Contract::<T::AccountId, T::Moment> {
                adopter: adopter.clone(),
                start_at: now,
                end_at: res.harvest_before,
            };
            let current_block = <system::Module<T>>::block_number();
            Contracts::<T>::insert(owner.clone(), resource_id.clone(), (current_block, contract));
            Self::deposit_event(RawEvent::ResourceAdopted(owner, resource_id, adopter));
            Ok(())
        }

        // TODO contract expire
    }
}
