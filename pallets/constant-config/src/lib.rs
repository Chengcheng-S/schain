#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;


#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;
    use crate::pallet;
    use frame_support::sp_runtime::traits::Zero;
    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        #[pallet::constant]
        type MaxAddend: Get<u32>;

        type ClearFrequency: Get<Self::BlockNumber>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://docs.substrate.io/v3/runtime/storage
    #[pallet::storage]
    #[pallet::getter(fn signle_value)]
    // Learn more about declaring storage items:
    // https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
    pub type Singlevalue<T: Config> = StorageValue<_, u32, ValueQuery>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        Added(u32, u32, u32),

        Cleared(u32),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        Overflow
    }

    //hooks method
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn on_finalize(_n: T::BlockNumber) {
            if (_n % T::ClearFrequency::get()).is_zero() {
                let curr_value = Singlevalue::<T>::get();
                Singlevalue::<T>::put(0u32);
                Self::deposit_event(Event::Cleared(curr_value));
            }
        }
    }


    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// user add value
        #[pallet::weight(10_000)]
        pub fn Add_Value(origin: OriginFor<T>, value: u32) -> DispatchResult {
            // Check that the extrinsic was signed and get the signer.
            // This function will return an error if the extrinsic is not signed.
            // https://docs.substrate.io/v3/runtime/origins
            ensure_signed(origin)?;

            ensure!(value <= T::MaxAddend::get(), "value must be <= maximum add amount constant");


            // check value has overflow
            let curr_value = Singlevalue::<T>::get();
            let res_value = curr_value.checked_add(value).ok_or(Error::<T>::Overflow)?;

            // Update storage.
            <Singlevalue<T>>::put(res_value);
            // Emit an event.
            Self::deposit_event(Event::Added(value,curr_value,res_value));
            // Return a successful DispatchResultWithPostInfo
            Ok(().into())
        }
    }
}
