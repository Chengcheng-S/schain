#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
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

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		#[pallet::constant]
		type MaxMultisigNumber: Get<u32>;

		#[pallet::constant]
		type MaxProposalNumber: Get<u32>;

		#[pallet::constant]
		type MinMultisigNumber: Get<u32>;
	}

	#[pallet::storage]
	#[pallet::getter(fn members)]
	pub type MultisigMembers<T: Config> =
		StorageValue<_, BoundedVec<T::AccountId, T::MaxMultisigNumber>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn proposals)]
	pub type Propoasals<T: Config> = StorageMap<
		_,
		Twox64Concat,
		u32,
		BoundedVec<T::AccountId, T::MaxMultisigNumber>,
		ValueQuery,
	>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		CreateMultisig { who: T::AccountId },
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
		NotFoundAccount,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// create multisig group
		#[pallet::call_index(0)]
		#[pallet::weight(0)]
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
					Self::deposit_event(Event::CreateMultisig { who });
					// todo! Dynamically adjust signing thresholds
				},
				_ => return Err(Error::<T>::MaxMultisigNumber.into()),
			}

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// create proposal
		#[pallet::call_index(1)]
		#[pallet::weight(0)]
		pub fn create_propoasl(origin: OriginFor<T>, threshold: u32) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// does account contain the multisig group?
			match MultisigMembers::<T>::get().contains(&who) {
				true => {
					let multisig_members = MultisigMembers::<T>::get();
					let multisig_members_len = multisig_members.len();

					if multisig_members_len > 10 {
						return Err(Error::<T>::MaxPropolasNumber.into())
					} else {
						let proposal_id = T::MaxProposalNumber::get();
						// let threshold: PropoaslThreshold = match threshold {
						//     1 => PropoaslThreshold::All,
						//     2 => PropoaslThreshold::MoreThanhalf,
						//     3|_ => PropoaslThreshold::MoreThanTwoThirds,
						//     4 => PropoaslThreshold::MoreThanhalf,
						// };

						Self::deposit_event(Event::CreateProposal { threshold, who, proposal_id });
					}
				},
				false => return Err(Error::<T>::NotFoundAccount.into()),
			}

			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		fn change_multisig_members(members: &mut Vec<T::AccountId>) -> DispatchResult {
			MultisigMembers::<T>::try_mutate(|accounts| -> DispatchResult {
				accounts.try_append(members).map_err(|_| Error::<T>::MaxMultisigNumber)?;
				accounts.sort();
				Ok(())
			})?;
			Ok(())
		}
	}
}
