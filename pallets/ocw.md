## substrate off-chain-worker
### unsigned tx
pallet Config
```rust
 #[pallet::config]
    pub trait Config: frame_system::Config +SendTransactionTypes<Call<Self>> {
        /// Because this pallet emits events, it depends on the runtime's definition of an event.
        type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
    }
```

为Pallet 实现`ValidateUnsigned`
```rust
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
```

外部调用函数实现
```rust0
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
```


钩子函数实现
```rust
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
```
runtime /src/lib.rs
```rust
/// ocw submit unsigned tx
impl pallet_ocw_example::Config for Runtime{
	type Event= Event;
}
///  impl sendtransactiontypes for runtime
impl<C>frame_system::offchain::SendTransactionTypes<C> for Runtime
	where Call: From<C>,
{
	type Extrinsic = UncheckedExtrinsic;
	type OverarchingCall = Call;
}
```

tip:
- 钩子函数实现时**方法中的字段必须和函数签名对应**
- pallet config 的`SendTransactionTypes` trait 必须在runtime中为`Runtime` 实现

