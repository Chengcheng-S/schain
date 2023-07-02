#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

pub mod weights;

pub use weights::*;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;
use sp_std::prelude::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use core::marker::PhantomData;

    #[pallet::pallet]
    pub struct Pallet<T>(PhantomData<T>);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        type WeightInfo: WeightInfo;

        #[pallet::constant]
        type MaxMultisigNumber: Get<u32>; //5

        #[pallet::constant]
        type MaxProposalNumber: Get<u32>; // 15

        #[pallet::constant]
        type MinMultisigNumber: Get<u32>; // 2
    }

    #[pallet::storage]
    #[pallet::getter(fn members)]
    pub type MultisigMembers<T: Config> =
    StorageValue<_, BoundedVec<T::AccountId, T::MaxMultisigNumber>, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn proposals)]
    pub type Proposals<T: Config> =
    StorageMap<_, Twox64Concat, u32, BoundedVec<Proposal, T::MaxMultisigNumber>, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/main-docs/build/events-errors/
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        CreateMultisig {
            who: T::AccountId,
            dyn_threshold: u32,
        },
        CreateProposal {
            who: T::AccountId,
            proposal_id: u32,
            threshold: ProposalThreshold,
            status: ProposalStatus,
        },
        ApprovalProposal {
            proposal_id: u32,
            who: T::AccountId,
        },
        RejectProposal {
            proposal_id: u32,
            who: T::AccountId,
        },
        // add / remove members
        ChangeGroup {
            account: T::AccountId,
            dynthreshold: u32,
        },
	}

    #[derive(PartialEq, Eq, Debug, Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum ProposalStatus {
        Pending,
        Rejected,
        Approved,
    }

    #[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub struct Proposal {
        pub proposal_id: u32,
        pub threshold: ProposalThreshold,
        pub status: ProposalStatus,
    }

    #[derive(Clone, PartialEq, Eq, Debug, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum ProposalThreshold {
        All,
        // 100%
        MoreThanhalf,
        // 1/2 +
        MoreThanTwoThirds,
        //  2/3 +
        MoreThanThreeQuarters, // 3/4 +
    }

    #[derive(Clone, PartialEq, Eq, Debug, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
    pub enum DynThreshold {
        All,
        //100%
        MoreThanhalf,
        //1/2 +
        MoreThanTwoThirds, // 2/3 +
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        MaxProposalNumber,
        MinMultisigNumber,
        MaxMultisigNumber,
        NotFoundAccount,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// create multisig group
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(1_000, 0))]
        pub fn create_multisig_group(
            origin: OriginFor<T>,
            members: Vec<T::AccountId>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let mut members =
                members.iter().map(|account| account.clone()).collect::<Vec<T::AccountId>>();

            match members.is_empty() {
                true => return Err(Error::<T>::MinMultisigNumber.into()),
                false => {
                    Self::change_multisig_members(&mut members)?;
                    let dyn_threshold = Self::calculate_dyn_threshold(&members);

                    Self::deposit_event(Event::CreateMultisig { who, dyn_threshold });
                    // todo! Dynamically adjust signing thresholds
                }
            }

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        /// create proposal
        #[pallet::call_index(1)]
        #[pallet::weight(Weight::from_parts(3_000, 0))]
        pub fn create_propoasl(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;

            // does account contain the multisig group?
            match MultisigMembers::<T>::get().contains(&who) {
                true => {
                    let multisig_members = MultisigMembers::<T>::get();
                    let multisig_members_len = multisig_members.len();

                    if multisig_members_len > 5 {
                        return Err(Error::<T>::MaxProposalNumber.into());
                    } else {
                        let proposal_id = Proposals::<T>::iter().count() as u32 + 1;
                        let threshold = match threshold {
                            1 => ProposalThreshold::All,
                            2 => ProposalThreshold::MoreThanhalf,
                            3 | _ => ProposalThreshold::MoreThanTwoThirds,
                        };
                        let status = ProposalStatus::Pending;

                        let mut proposal = Proposal { proposal_id, threshold, status };

                        Self::insert_proposal(&mut proposal)?;

                        Self::approve(origin, proposal_id)?;

                        Self::deposit_event(Event::CreateProposal {
                            who,
                            proposal_id,
                            threshold,
                            status,
                        });
                    }
                }
                false => return Err(Error::<T>::NotFoundAccount.into()),
            }

            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(Weight::from_parts(3_000, 0))]
        pub fn approve(origin: OriginFor<T>, _proposal_id: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            //todo! check if proposal exists
            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(Weight::from_parts(3_000, 0))]
        pub fn reject(origin: OriginFor<T>, _proposal_id: u32) -> DispatchResult {
            let _who = ensure_signed(origin)?;
            //todo! check if proposal exists
            Ok(())
        }

        // remove member from multisig
        #[pallet::call_index(4)]
        #[pallet::weight(Weight::from_parts(3_000, 0))]
        pub fn remove_member(origin: OriginFor<T>, member: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;

            // check the sender  in multisig group
            match MultisigMembers::<T>::get().contains(&who) && MultisigMembers::<T>::get().contains(&member) {
                true => {
                    // create remove member proposal
                    Self::create_propoasl(origin.clone(), 1)?;

                    let _ = MultisigMembers::<T>::get().iter().cloned().filter(|account| account.ne(&member)).collect::<Vec<T::AccountId>>();

                    Self::do_change_members(who, &mut MultisigMembers::<T>::get().into_inner());
                }

                false => return Err(Error::<T>::NotFoundAccount.into()),
            }

            //todo ! check if member exists
            Ok(())
        }

        #[pallet::call_index(5)]
        #[pallet::weight(Weight::from_parts(3_000, 0))]
        pub fn add_member(origin: OriginFor<T>, member: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin.clone())?;

            // check the sender  in multisig group
            match MultisigMembers::<T>::get().contains(&who) && !MultisigMembers::<T>::get().contains(&member) {
                true => {
                    // create add member proposal
                    Self::create_propoasl(origin.clone(), 1)?;
                    Self::do_change_members(who, &mut MultisigMembers::<T>::get().into_inner());
                }
                false => return Err(Error::<T>::NotFoundAccount.into()),
            }


            //todo ! check if member exists
            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        pub fn do_change_members(who: T::AccountId, members: &mut Vec<T::AccountId>, ) {
            let _ = Self::change_multisig_members(members);

            let dyn_threshold = Self::calculate_dyn_threshold(&members);

            Self::deposit_event(Event::ChangeGroup { account: who, dynthreshold: dyn_threshold });
        }

        fn change_multisig_members(members: &mut Vec<T::AccountId>, ) -> DispatchResult {
            MultisigMembers::<T>::try_mutate(|accounts| -> DispatchResult {
                accounts.try_append(members).map_err(|_| Error::<T>::MaxMultisigNumber)?;
                accounts.sort();
                Ok(())
            })?;

            Ok(())
        }

        fn insert_proposal(proposal: &mut Proposal) -> DispatchResult {
            Proposals::<T>::try_mutate(proposal.proposal_id, |pro| -> DispatchResult {
                pro.try_push(proposal.clone()).map_err(|_| Error::<T>::MaxProposalNumber)?;
                Ok(())
            })?;
            Ok(())
        }

        fn calculate_dyn_threshold(members: &Vec<T::AccountId>) -> u32 {
            match members.len() {
                0..=2 => 1,
                3..=5 => 2,
                _ => 3,
                // todo! some details doesn't deal with
            }
        }
    }
}
