#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;

pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
    use core::marker::PhantomData;
    use std::vec::Vec;
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// Type representing the weight of this pallet
        type WeightInfo: WeightInfo;

        #[pallet::constant]
        type MaxMultisgNumber: Get<u32>;

        #[pallet::constant]
        type MaxPropolasNumber: Get<u32>;

        #[pallet::constant]
        type MinMultisgNumber: Get<u32>;
    }

    #[pallet::storage]
    #[pallet::getter(fn multisig_members)]
    pub type MultisigMembers<T> = StorageValue<_, Vec<T::AccountId>>;


    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Propoasls<T: Config> = StorageMap<_, Twox64Concat, u32, Vec<T::Account>, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/main-docs/build/events-errors/
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        CreateMultisig { members: Vec<T::AccountId> },
        CreateProposal { who: T::AccountId, threshold: u32, proposal_id: u32 },
        ApprovalProposal { proposal_id: u32, who: T::AccountId },
        RejectProposal { proposal_id: u32, who: T::AccountId },
    }


    pub enum ProposalStatus {
        Pending,
        Rejected,
        Approved,
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub enum PropoaslThreshold {
        All,
        MoreThanhalf,
        MoreThanTwoThirds,
        MoreThanThreeQuarters,
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        MaxPropolasNumber,
        MinMultisigNumber,
        MaxMultisigNumber,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::call_index(0)]
        #[pallet::weight(50000)]
        pub fn create_multisig_group(origin: OriginFor<T>, members: Vec<T::AccountId>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            match members.len() {
                0u32..=T::MinMultisgNumber => return Err(Error::<T>::MinMultisigNumber.into()),
                T::MinMultisgNumber ..=T::MaxMultisgNumber => {
                    MultisigMembers::<T>::put(members.clone());
                    Self::deposit_event(Event::CreateMultisig { members });
                }
                _ => {
                    return Err(Error::<T>::MaxMultisigNumber.into());
                }
            }

            Self::deposit_event(Event::CreateMultisig { members });
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        /// An example dispatchable that may throw a custom error.
        #[pallet::call_index(1)]
        #[pallet::weight(10000)]
        pub fn create_propoasl(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // does account contain the multisig group?
            let multisig_members = MultisigMembers::<T>::get();
            let multisig_members_len = multisig_members.len();
            if multisig_members_len == T::MaxMultisgNumber as usize {
                return Err(Error::<T>::MaxMultisigNumber.into());
            } else {
                let proposal_id = T::MaxPropolasNumber::get() as u32;
                // let threshold: PropoaslThreshold = match threshold {
                //     1 => PropoaslThreshold::All,
                //     2 => PropoaslThreshold::MoreThanhalf,
                //     3|_ => PropoaslThreshold::MoreThanTwoThirds,
                //     4 => PropoaslThreshold::MoreThanhalf,
                // };

                Self::deposit_event(Event::CreateProposal { threshold, who, proposal_id });
            }

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }
    }
}

