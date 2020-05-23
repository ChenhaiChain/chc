#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    // weights::{
    //     ClassifyDispatch, DispatchClass, FunctionOf, PaysFee, SimpleDispatchInfo, WeighData, Weight,
    // },
    StorageDoubleMap,
};
use sp_std::prelude::*;
use frame_system::{self as system, ensure_signed};
// use sp_runtime::traits::SaturatedConversion;

pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Resource {
    pub id: Vec<u8>,
    pub freeze_at: u64,
    pub price: u128,
    pub min_output_kg: u16,
    pub info: Vec<u8>,
    // pub photo_url: Vec<u8>,
    // pub photo: Vec<u8>,
    // pub rtype: Vec<u8>,
    // pub year: u16,
    // pub price: u32,
    // pub harvest_season: u8,
    // pub geohash: Vec<u8>,
}

// struct LinearWeight(u32);

// impl WeighData<(&Vec<u8>, &u128, &u16, &u64, &Vec<u8>)> for LinearWeight {
//     fn weigh_data(&self, data: (&Vec<u8>, &u128, &u16, &u64, &Vec<u8>)) -> Weight {
//         let multiplier = self.0;
//         // TODO
//         multiplier.saturated_into::<Weight>()
//     }
// }

// impl ClassifyDispatch<(&Vec<u8>, &u128, &u16, &u64, &Vec<u8>)> for LinearWeight {
//     fn classify_dispatch(&self, data: (&Vec<u8>, &u128, &u16, &u64, &Vec<u8>)) -> DispatchClass {
//         DispatchClass::Normal
//     }
// }

// impl PaysFee<(&Vec<u8>, &u128, &u16, &u64, &Vec<u8>)> for LinearWeight {
//     fn pays_fee(&self, data: (&Vec<u8>, &u128, &u16, &u64, &Vec<u8>)) -> bool {
//         true
//     }
// }

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Contract {
    pub id: Vec<u8>,
    pub resource_id: Vec<u8>,
    pub start_at: u64,
    pub end_at: u64,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct ResourceEvent {
    pub id: Vec<u8>,
    pub event: Vec<u8>,
}

decl_storage! {
    trait Store for Module<T: Trait> as ResourceAdoption {
      Resources get(fn retrieve_resource): double_map
        hasher(blake2_128_concat) T::AccountId,
        hasher(blake2_128_concat) Vec<u8>
      => (T::BlockNumber, Resource);

      Contracts get(fn retrieve_contract): double_map
        hasher(blake2_128_concat) T::AccountId,
        hasher(blake2_128_concat) Vec<u8>
      => (T::BlockNumber, Contract);

      Events get(fn retrieve_events): double_map
        hasher(blake2_128_concat) T::AccountId,
        hasher(blake2_128_concat) Vec<u8>
      => (T::BlockNumber, ResourceEvent);
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        // Hash = <T as system::Trait>::Hash,
    {
        ResourceOnline(AccountId, Vec<u8>),
        ResourceAdopted(AccountId, Vec<u8>, Vec<u8>),
        ResourceStateChanged(AccountId, Vec<u8>, Vec<u8>),
        ResourceOffline(AccountId, Vec<u8>),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        ResourceAlreadyExist,
        ResourceNotExist,
        ResourceAdopted,
        ResourceFreezed,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        fn deposit_event() = default;

        #[weight = 1_000]
        pub fn publish_resource(origin, id: Vec<u8>, price: u128, min_output_kg: u16, freeze_at: u64, info: Vec<u8>)
                                -> dispatch::DispatchResult {
            let publisher = ensure_signed(origin)?;
            ensure!(!Resources::<T>::contains_key(&publisher, &id), Error::<T>::ResourceAlreadyExist);
            let res = Resource {
                id: id,
                price: price,
                min_output_kg: min_output_kg,
                freeze_at: freeze_at,
                info: info,
            };
            let current_block = <system::Module<T>>::block_number();
            let id = res.id.clone();
            Resources::<T>::insert(publisher.clone(), res.id.clone(), (current_block, res));
            Self::deposit_event(RawEvent::ResourceOnline(publisher, id));
            Ok(())
        }
    }
}
