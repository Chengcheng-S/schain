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
	pub type Proposals<T: Config> = StorageMap<_, Twox64Concat, u32, Proposal<T>>;

	#[pallet::storage]
	#[pallet::getter(fn votings)]
	pub type Voting<T: Config> = StorageMap<_, Identity, u32, Votes<T>, OptionQuery>;

	// add member
	#[pallet::storage]
	#[pallet::getter(fn add_members)]
	pub type AddMember<T: Config> = StorageMap<_, Twox64Concat, u32, T::AccountId>;

	// remove member
	#[pallet::storage]
	#[pallet::getter(fn remove_members)]
	pub type RemoveMember<T: Config> = StorageMap<_, Twox64Concat, u32, T::AccountId>;

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
		pub proposaltype: ProposalType,
		pub owner: T::AccountId,
	}

	#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
	pub enum ProposalType {
		AddMember,
		RemoveMember,
		//etc
	}

	/// Info for keeping track of a motion being voted on.
	#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub struct Votes<T: Config> {
		/// The proposal's unique index.
		pub index: ProposalIndex,
		/// The number of approval votes that are needed to pass the motion.
		pub threshold: Threshold,
		/// The current set of voters that approved it.
		pub ayes: Vec<T::AccountId>,
		/// The current set of voters that rejected it.
		pub nays: Vec<T::AccountId>,
		// /// The hard end time of this vote.
		// end: T::BlockNumber,
	}

	#[derive(Clone, PartialEq, Eq, Debug, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
	pub enum ProposalThreshold {
		// 100%
		All,
		// 1/2 +
		MoreThanhalf,
		//  2/3+
		MoreThanTwoThirds,
		// 3/4 +
		MoreThanThreeQuarters,
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
		NotFoundAddAccount,
		NotFoundRemoveAccount,
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

			let mut add_members = members.to_vec();

			match add_members.len() > 0 && add_members.len() == members.len() {
				false => return Err(Error::<T>::MinMultisigNumber.into()),
				true =>
					if members.contains(&who) {
						Self::change_multisig_members(&mut add_members, true)?;
						let dyn_threshold = Self::calculate_dyn_threshold(&members);

						Self::deposit_event(Event::CreateMultisig { who, dyn_threshold });
					} else {
						return Err(Error::<T>::MinMultisigNumber.into())
					},
			}

			//generate a multisig account address

			Ok(())
		}

		/// create proposal
		#[pallet::call_index(1)]
		#[pallet::weight(Weight::from_parts(3_000, 0))]
		pub fn create_proposal(
			origin: OriginFor<T>,
			threshold: u32,
			proposaltype: u32,
		) -> DispatchResult {
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

						let vote: Votes<T> = Votes {
							index: proposal_id,
							threshold,
							ayes: vec![who.clone()],
							nays: Vec::new(),
						};

						Voting::<T>::insert(&proposal_id, &vote);

						let threshold = match threshold {
							1 => ProposalThreshold::All,
							2 => ProposalThreshold::MoreThanhalf,
							3 | _ => ProposalThreshold::MoreThanTwoThirds,
						};

						let protype = match proposaltype {
							1 => ProposalType::AddMember,
							2 | _ => ProposalType::RemoveMember,
						};

						let status = ProposalStatus::Pending;

						let proposal = Proposal {
							proposal_id,
							threshold,
							status,
							vote: 1,
							proposaltype: protype,
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
					// vote for proposal and execute the proposal if vote had enough approval

					let dyn_threshold = Self::calculate_dyn_threshold(&MultisigMembers::<T>::get());

					let should_execute =
						Self::do_vote(who.clone(), proposal_id, true, dyn_threshold)?;

					if should_execute {
						Self::exe_proposal(proposal_id)?;
					}
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
					//only reject the proposal
					let dyn_threshold = Self::calculate_dyn_threshold(&MultisigMembers::<T>::get());
					Self::do_vote(who.clone(), proposal_id, false, dyn_threshold)?;
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
					// just create remove member proposal
					Self::create_a_proposal(who, 1, 2, false, member)?;
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
					// just create add member proposal
					Self::create_a_proposal(who, 3, 1, true, member)?;
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
			approve: bool,
			dynthreshold: u32,
		) -> Result<bool, DispatchError> {
			// should be execute the proposal
			let mut result: bool = false;

			let mut vote = match Self::votings(&proposal_id) {
				Some(vote) => vote,
				None => return Err(Error::<T>::InvalidVote.into()),
			};

			let mut proposal = match Self::proposals(&proposal_id) {
				Some(proposal) => proposal,
				None => return Err(Error::<T>::NotFoundProposal.into()),
			};

			let threshold = {
				let members = Self::members().len() as u32;

				let proposal_threshold = match proposal.threshold {
					ProposalThreshold::All => members,
					ProposalThreshold::MoreThanTwoThirds => 2 * (members % 3) + 1,
					ProposalThreshold::MoreThanhalf => (members / 2) + 1,
					ProposalThreshold::MoreThanThreeQuarters => 3 * (members % 4) + 1,
				};

				if proposal_threshold > dynthreshold {
					proposal_threshold
				} else {
					dynthreshold
				}
			};

			// check if proposal is pending
			match !vote.ayes.contains(&caller) && !vote.nays.contains(&caller) {
				false => {},
				true => {
					// check if proposal is pending and approved this proposal
					if proposal.status == ProposalStatus::Pending && approve {
						match proposal.vote < threshold {
							true => {
								// approve
								proposal.vote += 1;

								vote.ayes.push(caller.clone());

								if vote.ayes.len() as u32 >= dynthreshold {
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
					} else if proposal.status == ProposalStatus::Pending && !approve {
						// proposal.status = ProposalStatus::Finished;

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

			Ok(result)
		}

		// execute proopsal
		pub fn exe_proposal(proposal_id: u32) -> DispatchResult {
			//get proposal status  && proposal vote yes_number > dynthreshold than approve the
			// proposal such as add member | remove member | transfer etc

			let mut proposal = Self::proposals(&proposal_id).ok_or(Error::<T>::NotFoundProposal)?;

			proposal.status = ProposalStatus::Finished;

			match proposal.proposaltype {
				ProposalType::AddMember => {
					let member = match Self::add_members(&proposal_id) {
						Some(member) => member,
						None => return Err(Error::<T>::NotFoundAddAccount.into()),
					};

					let mut members = vec![member];

					Self::do_change_members(proposal.owner, &mut members, true);
					// Self::change_multisig_members(&mut members)?;
				},
				ProposalType::RemoveMember => {
					let member = match Self::remove_members(&proposal_id) {
						Some(member) => member,
						None => return Err(Error::<T>::NotFoundRemoveAccount.into()),
					};

					let mut members = vec![member];

					Self::do_change_members(proposal.owner, &mut members, false);
				},
			}

			Ok(())
		}

		// create a proposal by user behavior
		pub fn create_a_proposal(
			caller: T::AccountId,
			threshold_u32: u32,
			proposaltype: u32,
			signal: bool,
			change_member: T::AccountId,
		) -> DispatchResult {
			// does account contain the multisig group?
			match MultisigMembers::<T>::get().contains(&caller) {
				true => {
					let multisig_members = MultisigMembers::<T>::get();
					let multisig_members_len = multisig_members.len();

					if multisig_members_len > 5 {
						return Err(Error::<T>::MaxProposalNumber.into())
					} else {
						let proposal_id = Proposals::<T>::iter().count() as u32 + 1;

						let vote: Votes<T> = Votes {
							index: proposal_id,
							threshold: threshold_u32,
							ayes: vec![caller.clone()],
							nays: Vec::new(),
						};

						Voting::<T>::insert(&proposal_id, &vote);

						let threshold = match threshold_u32 {
							1..=3 => ProposalThreshold::All,
							6 => ProposalThreshold::MoreThanhalf,
							5 | _ => ProposalThreshold::MoreThanTwoThirds,
						};

						let protype = match proposaltype {
							1 => ProposalType::AddMember,
							2 | _ => ProposalType::RemoveMember,
						};

						let status = ProposalStatus::Pending;

						let proposal = Proposal {
							proposal_id,
							threshold,
							status,
							vote: 0,
							proposaltype: protype,
							owner: caller.clone(),
						};

						match signal {
							true => {
								AddMember::<T>::insert(&proposal_id, &change_member);
							},
							false => {
								RemoveMember::<T>::insert(&proposal_id, &change_member);
							},
						}

						Proposals::<T>::insert(&proposal_id, &proposal);

						Self::do_vote(caller.clone(), proposal_id, signal, threshold_u32)?;

						Self::deposit_event(Event::CreateProposal {
							who: caller,
							proposal_id,
							threshold,
							status,
						})
					}
				},
				false => return Err(Error::<T>::NotFoundAccount.into()),
			}

			Ok(())
		}

		pub fn do_change_members(who: T::AccountId, members: &mut Vec<T::AccountId>, signal: bool) {
			let _ = Self::change_multisig_members(members, signal);

			let dyn_threshold = Self::calculate_dyn_threshold(&members);

			Self::deposit_event(Event::ChangeGroup { account: who, dynthreshold: dyn_threshold });
		}

		fn change_multisig_members(
			members: &mut Vec<T::AccountId>,
			singal: bool,
		) -> DispatchResult {
			match singal {
				true => {
					MultisigMembers::<T>::try_mutate(|accounts| -> DispatchResult {
						accounts.try_append(members).map_err(|_| Error::<T>::MaxMultisigNumber)?;
						accounts.sort();
						Ok(())
					})?;
				},
				false => {
					MultisigMembers::<T>::try_mutate(|accounts| -> DispatchResult {
						if let Some(index) = accounts.iter().position(|x| x == &members[0]) {
							accounts.remove(index);
							Ok(())
						} else {
							Err(Error::<T>::NotFoundAccount.into())
						}
					})?;
				},
			}

			Ok(())
		}

		// multisig group dyn threshold
		fn calculate_dyn_threshold(members: &Vec<T::AccountId>) -> u32 {
			let member_numbers = members.len() as u32;
			match member_numbers {
				0..=3 => member_numbers,           // must all
				4..=5 => 2 * (member_numbers % 3), // must 2/3 +
				_ => member_numbers / 2 + 1,       // must 1/2 +
			}
		}
	}
}
