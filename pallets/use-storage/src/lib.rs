#![cfg_attr(not(feature = "std"), no_std)]

// 1. Imports and Dependencies
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;

	// 2. Declaration of the Pallet type
	// This is a placeholder to implement traits and methods.
	#[pallet::pallet]
	#[pallet::generate_store(pub (super) trait Store)]
	pub struct Pallet<T>(_);

	// 3. Runtime Configuration Trait
	// All types and constants go here.
	// Use #[pallet::constant] and #[pallet::extra_constants]
	// to pass in values to metadata.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	// 4. Runtime Storage
	// class number  origin account
	#[pallet::storage]
	#[pallet::getter(fn one_class)]
	pub type Class<T: Config> = StorageValue<_, u32>;

	// studentinfo
	#[pallet::storage]
	#[pallet::getter(fn student_info)]
	pub type StudentInfo<T>
	where
		T: Config,
	= StorageMap<_, Blake2_128Concat, u32, u128, ValueQuery>;

	//dorminfo
	#[pallet::storage]
	#[pallet::getter(fn dorminfo)]
	pub type DormInfo<T: Config> =
		StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, u32, u32, ValueQuery>;

	// 5. Runtime Events
	// Can stringify event types to metadata.
	#[pallet::event]
	#[pallet::generate_deposit(pub (super) fn deposit_event)]
	pub enum Event<T: Config> {
		SetClass(u32),
		SetStudentInfo(u32, u128),
		SetDormInfo(u32, u32, u32),
	}

	//runtime error
	#[pallet::error]
	pub enum Error<T> {
		SetClassDuplicate,

		SetStudentInfoDuplicate,

		SetDormInfoDuplicate,
	}

	// 7. Extrinsics
	// Functions that are callable from outside the runtime.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::compact]
		#[pallet::weight(0)]
		pub fn setClass(origin: OriginFor<T>, class: u32) -> DispatchResultWithPostInfo {
			// check the sender is origin acc
			ensure_root(origin)?;

			if Class::<T>::exists() {
				return Err(Error::<T>::SetClassDuplicate.into())
			}

			//set the value
			Class::<T>::put(class);

			// call storage geeter method  note argments
			// let _z = Self::one_class();
			//emit setclass event
			Self::deposit_event(Event::SetClass(class));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn setstudentinfo(
			origin: OriginFor<T>,
			class: u32,
			name: u128,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			if StudentInfo::<T>::contains_key(&class) {
				return Err(Error::<T>::SetStudentInfoDuplicate.into())
			}

			StudentInfo::<T>::insert(&class, &name);

			//emit event
			Self::deposit_event(Event::SetStudentInfo(class, name));

			Ok(().into())
		}

		#[pallet::weight(0)]
		pub fn setdorminfo(
			origin: OriginFor<T>,
			dorm_number: u32,
			bed_number: u32,
			student_number: u32,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;

			if DormInfo::<T>::contains_key(&dorm_number, &bed_number) {
				return Err(Error::<T>::SetDormInfoDuplicate.into())
			}

			DormInfo::<T>::insert(&dorm_number, &bed_number, &student_number);

			Self::deposit_event(Event::SetDormInfo(dorm_number, bed_number, student_number));
			Ok(().into())
		}
	}
}
