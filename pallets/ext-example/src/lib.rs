#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	//denpendencies
	use crate::pallet;
	use frame_support::{pallet_prelude::*, transactional};
	use frame_system::pallet_prelude::*;

	// pallet type
	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	//runtime config
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	//runtime storage
	#[pallet::storage]
	#[pallet::getter(fn user_param)]
	pub type Param<T: Config> = StorageValue<_, u32, ValueQuery>;

	#[pallet::storage]
	pub type SetFlag<T: Config> = StorageValue<_, bool, ValueQuery>;

	//events and errors
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		SetParams(u32),
	}

	#[pallet::error]
	pub enum Error<T> {
		// 参数>100
		ParamInvalid,
	}

	//hooks
	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(_n: BlockNumberFor<T>) -> Weight {
			log::info!(target:"on_initialize","on initilize the block number is {:?}",_n);
			0
		}

		fn on_finalize(_n: BlockNumberFor<T>) {
			log::info!(target:"on_initialize","on finalize the block number   {:?}",_n);
		}
	}

	// dispatch
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::weight(0)]
		#[transactional]
		pub fn set_param(origin: OriginFor<T>, param: u32) -> DispatchResult {
			ensure_signed(origin)?;

			SetFlag::<T>::put(true);

			if param < 100u32 {
				return Err(Error::<T>::ParamInvalid.into())
			}

			Param::<T>::put(&param);

			log::info!(target:"on_initialize","setting the param is  {:?}",&param);

			Self::deposit_event(Event::SetParams(param));

			Ok(().into())
		}
	}
}
