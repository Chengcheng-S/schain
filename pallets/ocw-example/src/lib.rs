#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/v3/runtime/frame>
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::offchain::{SendTransactionTypes, SubmitTransaction};
    use frame_system::pallet_prelude::*;


    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config +SendTransactionTypes<Call<Self>> {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }

    #[pallet::pallet]
    #[pallet::generate_store(pub (super) trait Store)]
    pub struct Pallet<T>(_);

    // The pallet's runtime storage items.
    // https://docs.substrate.io/v3/runtime/storage
    #[pallet::storage]
    #[pallet::getter(fn something)]
    // Learn more about declaring storage items:
    // https://docs.substrate.io/v3/runtime/storage#declaring-storage-items
    pub type Something<T> = StorageMap<_,Blake2_128Concat,u64, u64>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored(u64,u64),
    }

    // Errors inform users that something went wrong.
    #[pallet::error]
    pub enum Error<T> {
        /// Error names should be descriptive.
        NoneValue,
        /// Errors should have helpful documentation associated with them.
        StorageOverflow,
    }
    
    #[pallet::hooks]
    impl <T:Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number:T::BlockNumber) {
            let number :u64 = 10;
            let call = Call::unsigned_extrinsic1{number};

            SubmitTransaction::<T,Call<T>>::submit_unsigned_transaction(call.into())
                .map_err(|_|{
                    log::info!(target:"ocw","failed submit unsigned tx");
                });

        }
    }
    
    // Dispatchable functions allows users to interact with the pallet and invoke state changes.
    // These functions materialize as "extrinsics", which are often compared to transactions.
    // Dispatchable functions must be annotated with a weight and must return a DispatchResult.
    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// An example dispatchable that takes a singles value as a parameter, writes the value to
        /// storage and emits an event. This function must be dispatched by a signed extrinsic.
        #[pallet::weight(10_000)]
        pub fn unsigned_extrinsic1(origin: OriginFor<T>, number:u64) -> DispatchResult {

            ensure_none(origin)?;

            let mut cnt:u64 = 0;
            if number >0{
                cnt = number;
            }

            log::info!(target:"ocw","unsigned tx  by off chain worker");

            Something::<T>::insert(&number,cnt);
            // Emit an event.
            Self::deposit_event(Event::SomethingStored(number, cnt));
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }


    }

    /// impl validateunsigned for pallet  that off-chain-worker can submit unsigned tx to chain
    #[pallet::validate_unsigned]
    impl<T: Config> ValidateUnsigned for Pallet<T> {
        type Call = Call<T>;


        /// user can call extrinsic1 that unsigned,but not any extrinsics
        fn validate_unsigned(source: TransactionSource, call: &Self::Call) -> TransactionValidity {


            let valid_tx = |provide| ValidTransaction::with_tag_prefix("ocwunsign")
                .priority(TransactionPriority::max_value())   // setting tx priority,
                .propagate(true)// setting propagate flag.
                .and_provides(&provide)
                .longevity(3) // setting tx lifetime, by default tx vailded for ever, will not be revalided by tx pool
                .build();


            /// check extrinsics if the call is allowed,

            match call{
                Call::unsigned_extrinsic1{number}=>valid_tx(b"extrinsic1".to_vec()),
                _=>InvalidTransaction::Call.into(),
            }

        }
    }
}
