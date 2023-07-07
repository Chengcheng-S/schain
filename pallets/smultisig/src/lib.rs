#![cfg_attr(not(feature = "std"), no_std)]

///multisig module
pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod test;
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
	pub type Proposals<T: Config> = StorageMap<_, Twox64Concat, u32, Proposal<T>>;

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
			vote: u32,
			who: T::AccountId,
		},

		FinshedProposal {
			proposal_id: u32,
			vote: u32,
		},

		RejectProposal {
			proposal_id: u32,
			vote: u32,
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
		Finished,
	}

	#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]
	pub struct Proposal<T: Config> {
		pub proposal_id: u32,
		pub threshold: ProposalThreshold,
		pub status: ProposalStatus,
		pub vote: u32,
		pub owner: T::AccountId,
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
		MustContainCaller,
		NotFoundProposal,
		InvalidVote,
	}

	// when begin block or endblock  we need to deal with the proposal
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
					if members.contains(&who) {
						Self::change_multisig_members(&mut members)?;
						let dyn_threshold = Self::calculate_dyn_threshold(&members);

						Self::deposit_event(Event::CreateMultisig { who, dyn_threshold });
					// todo! Dynamically adjust signing thresholds
					} else {
						return Err(Error::<T>::MinMultisigNumber.into())
					}
				},
			}

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// create proposal
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(3_000, 0))]
		pub fn create_proposal(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// does account contain the multisig group?
			match MultisigMembers::<T>::get().contains(&who) {
				true => {
					let multisig_members = MultisigMembers::<T>::get();
					let multisig_members_len = multisig_members.len();

					if multisig_members_len > 5 {
						return Err(Error::<T>::MaxProposalNumber.into())
					} else {
						let proposal_id = Proposals::<T>::iter().count() as u32 + 1;
						let threshold = match threshold {
							1 => ProposalThreshold::All,
							2 => ProposalThreshold::MoreThanhalf,
							3 | _ => ProposalThreshold::MoreThanTwoThirds,
						};
						let status = ProposalStatus::Pending;

						let proposal = Proposal::<T> {
							proposal_id,
							threshold,
							status,
							vote: 1,
							owner: who.clone(),
						};

						Proposals::<T>::insert(&proposal_id, &proposal);

						Self::approve(origin, proposal_id)?;

						Self::deposit_event(Event::CreateProposal {
							who,
							proposal_id,
							threshold,
							status,
						});
					}
				},
				false => return Err(Error::<T>::NotFoundAccount.into()),
			}

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(Weight::from_parts(3_000, 0))]
		pub fn approve(origin: OriginFor<T>, proposal_id: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match MultisigMembers::<T>::get().contains(&who) {
				true => {
					let dyn_threshold = Self::calculate_dyn_threshold(&MultisigMembers::<T>::get());
					Self::exec_proposal(who.clone(), proposal_id, true, dyn_threshold)?;
				},
				false => return Err(Error::<T>::MustContainCaller.into()),
			}

			//todo! check if proposal exists
			Ok(())
		}

		#[pallet::call_index(3)]
		#[pallet::weight(Weight::from_parts(3_000, 0))]
		pub fn reject(origin: OriginFor<T>, proposal_id: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			match MultisigMembers::<T>::get().contains(&who) {
				true => {
					let dyn_threshold = Self::calculate_dyn_threshold(&MultisigMembers::<T>::get());
					Self::exec_proposal(who.clone(), proposal_id, false, dyn_threshold)?;
				},
				false => return Err(Error::<T>::MustContainCaller.into()),
			}

			Ok(())
		}

		// remove member from multisig
		#[pallet::call_index(4)]
		#[pallet::weight(Weight::from_parts(3_000, 0))]
		pub fn remove_member(origin: OriginFor<T>, member: T::AccountId) -> DispatchResult {
			let who = ensure_signed(origin.clone())?;

			// check the sender  in multisig group
			match MultisigMembers::<T>::get().contains(&who) &&
				MultisigMembers::<T>::get().contains(&member)
			{
				true => {
					// create remove member proposal
					Self::create_proposal(origin.clone(), 1)?;

					let mut newgroup = MultisigMembers::<T>::get()
						.iter()
						.cloned()
						.filter(|account| account.ne(&member))
						.collect::<Vec<T::AccountId>>();

					Self::do_change_members(who, &mut newgroup);
				},

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
			match MultisigMembers::<T>::get().contains(&who) &&
				!MultisigMembers::<T>::get().contains(&member)
			{
				true => {
					// create add member proposal
					Self::create_proposal(origin.clone(), 1)?;
					Self::do_change_members(who, &mut MultisigMembers::<T>::get().into_inner());
				},
				false => return Err(Error::<T>::NotFoundAccount.into()),
			}

			//todo ! check if member exists
			Ok(())
		}

		#[pallet::call_index(6)]
		#[pallet::weight(Weight::from_parts(5_000, 0))]
		pub fn get_pending_proposal(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			let _ = Proposals::<T>::iter()
				.filter(|(_id, proposal)| proposal.status == ProposalStatus::Pending);

			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		pub fn exec_proposal(
			caller: T::AccountId,
			proposal_id: u32,
			signal: bool,
			dynthreshold: u32,
		) -> DispatchResult {
			// get proposal
			let mut proposal =
				Proposals::<T>::get(proposal_id).map_or(Err(Error::<T>::NotFoundProposal), Ok)?;

			match &caller.eq(&proposal.owner){
				true => {},
				false =>{
					// check if proposal is pending and approved this proposal
					if ProposalStatus::Pending == proposal.status && signal{
						match proposal.vote < dynthreshold {
							true => {
								// approve

								proposal.vote += 1;

								Self::deposit_event(Event::ApprovalProposal {
									proposal_id,
									who: caller,
									vote: proposal.vote,
								});
							},
							false => {
								proposal.status = ProposalStatus::Finished;
								Self::deposit_event(Event::FinshedProposal {
									proposal_id,
									vote: proposal.vote,
								});
							},
						}
					}
				}

			}
			
			Ok(())
		}

		pub fn do_change_members(who: T::AccountId, members: &mut Vec<T::AccountId>) {
			let _ = Self::change_multisig_members(members);

			let dyn_threshold = Self::calculate_dyn_threshold(&members);

			Self::deposit_event(Event::ChangeGroup { account: who, dynthreshold: dyn_threshold });
		}

		fn change_multisig_members(members: &mut Vec<T::AccountId>) -> DispatchResult {
			MultisigMembers::<T>::try_mutate(|accounts| -> DispatchResult {
				accounts.try_append(members).map_err(|_| Error::<T>::MaxMultisigNumber)?;
				accounts.sort();
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
