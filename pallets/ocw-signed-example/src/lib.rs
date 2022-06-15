#![cfg_attr(not(feature = "std"), no_std)]


use sp_core::{crypto::KeyTypeId};

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"demo");

pub type AuthorityId = crypto::Public;

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::offchain::{CreateSignedTransaction, SendSignedTransaction, Signer, SubmitTransaction, AppCrypto};
    use frame_system::pallet_prelude::*;

    use crate::KEY_TYPE;

    /// Configure the pallet by specifying the parameters and types on which it depends.
    #[pallet::config]
    pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

        type AuthorityId: AppCrypto<Self::Public, Self::Signature>;
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
    pub type Something<T> = StorageMap<_, Blake2_128Concat, u64, u64>;

    // Pallets use events to inform users when important changes are made.
    // https://docs.substrate.io/v3/runtime/events-and-errors
    #[pallet::event]
    #[pallet::generate_deposit(pub (super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event documentation should end with an array that provides descriptive names for event
        /// parameters. [something, who]
        SomethingStored(u64, u64),
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
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(block_number: T::BlockNumber) {
            /// initial a signature account that can submit tx on chain
            /// all_accounts  check the pallet all signature account
            let signer = Signer::<T, T::AuthorityId>::all_accounts();
            let number = block_number.try_into().unwrap_or(0);
            /// send extrinsic
            let result = signer.send_signed_transaction(|_account|Call::off_chain_signed_tx { number });

            for (acc, res) in &result {
                match res {
                    Ok(()) => log::info!("[{:?}]: submit transaction success.", acc.id),
                    Err(e) => log::error!("[{:?}]: submit transaction failure. Reason: {:?}", acc.id, e),
                }
            }
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
        pub fn off_chain_signed_tx(origin: OriginFor<T>, number: u64) -> DispatchResult {
            log::info!(target:"ocw","start send signed tx");
            ensure_signed(origin)?;

            let mut cnt: u64 = 0;
            if number > 0 {
                cnt = number;
            }

            log::info!(target:"ocw","signed tx  by off chain worker");

            Something::<T>::insert(&number, cnt);
            // Emit an event.
            Self::deposit_event(Event::SomethingStored(number, cnt));
            // Return a successful DispatchResultWithPostInfo
            Ok(())
        }
    }
}

/// app_crypto! declare a account that has sr25519 signature，and the signature that is definded by KET_TYPE (*b"demo")
///tip: this doesn't crate new account,only declare a crypto account is available for this pallets
///
pub mod crypto {
    use super::KEY_TYPE;
    use sp_core::sr25519::Signature as Sr25519Signature;
    use sp_runtime::{
        app_crypto::{app_crypto, sr25519},
        traits::Verify, MultiSignature, MultiSigner,
    };
    // Declare a new set of crypto types using Ed25519 logic that identifies as `KeyTypeId` of value *b"demo"
    app_crypto!(sr25519,KEY_TYPE);
}