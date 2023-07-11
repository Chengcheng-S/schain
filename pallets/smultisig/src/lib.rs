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
use sp_io::hashing::blake2_256;
use sp_runtime::traits::TrailingZeroInput;
use sp_std::prelude::*;

pub type ProposalIndex = u32;
pub type Threshold = u32;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use core::marker::PhantomData;

	#[pallet::pallet]
	#[pallet::without_storage_info]
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
	pub type Proposals<T: Config> = StorageMap<_, Twox64Concat, u32, Proposal>;

	#[pallet::storage]
	#[pallet::getter(fn votings)]
	pub type Voting<T: Config> = StorageMap<_, Identity, u32, Votes<T>, OptionQuery>;

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
	pub struct Proposal {
		pub proposal_id: u32,
		pub threshold: ProposalThreshold,
		pub status: ProposalStatus,
		pub vote: u32,
	}

	/// Info for keeping track of a motion being voted on.
	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Votes<T: Config> {
		/// The proposal's unique index.
		index: ProposalIndex,
		/// The number of approval votes that are needed to pass the motion.
		threshold: Threshold,
		/// The current set of voters that approved it.
		ayes: Vec<T::AccountId>,
		/// The current set of voters that rejected it.
		nays: Vec<T::AccountId>,
		/// The hard end time of this vote.
		end: T::BlockNumber,
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

			//generate a multisig account address

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

						let proposal = Proposal { proposal_id, threshold, status, vote: 1 };

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
					Self::do_vote(who.clone(), proposal_id, true, dyn_threshold);
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
					Self::do_vote(who.clone(), proposal_id, false, dyn_threshold);
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

					if Self::do_vote(who.clone(), Proposals::<T>::iter().count() as u32, false, 1) {
						Self::do_change_members(who, &mut MultisigMembers::<T>::get().into_inner());
					}
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
		// generate multisig account
		pub fn multi_account_id(who: &[T::AccountId], threshold: u16) -> T::AccountId {
			let entropy = (b"modlpy/utilisuba", who, threshold).using_encoded(blake2_256);
			Decode::decode(&mut TrailingZeroInput::new(entropy.as_ref()))
				.expect("infinite length input; no invalid inputs for type; qed")
		}

		pub fn do_vote(
			caller: T::AccountId,
			proposal_id: u32,
			signal: bool,
			dynthreshold: u32,
		) -> bool {
			// should be execute the proposal
			let mut result: bool = false;

			// get proposal
			let mut proposal = Proposals::<T>::get(&proposal_id)
				.map_or(Err(Error::<T>::NotFoundProposal), |proposal| Ok(proposal.clone()))
				.unwrap();

			let mut vote = Voting::<T>::get(&proposal_id).unwrap();

			// check if proposal is pending
			match !vote.ayes.contains(&caller) && !vote.nays.contains(&caller) {
				true => {},
				false => {
					// check if proposal is pending and approved this proposal
					if ProposalStatus::Pending == proposal.status && signal {
						match proposal.vote < dynthreshold {
							true => {
								// approve
								proposal.vote += 1;

								vote.ayes.push(caller.clone());

								if vote.ayes.len() as u32 > dynthreshold {
									result = true;
								}

								Voting::<T>::insert(proposal_id, vote);

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
					} else if ProposalStatus::Pending == proposal.status && !signal {
						proposal.status = ProposalStatus::Finished;

						vote.nays.push(caller.clone());

						<Voting<T>>::insert(proposal_id, vote);

						Self::deposit_event(Event::RejectProposal {
							proposal_id,
							who: caller,
							vote: proposal.vote,
						});
					}
				},
			}

			result
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
