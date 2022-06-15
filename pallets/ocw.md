## substrate off-chain-worker

### signed tx
第一部分
这部分主要是用来在offchain worker提交签名交易时的签名的子模块。在实际的开发中，这部分基本上是固定的写法。在substrate中支持ed25519和sr25519，我们此处使用的是sr29915作为例子。其中KEY_TYPE是offchain worker签名时检索key使用的类型，由开发者指定，我们这里指定为“demo”。

第二部分主要是支持offchain提交签名交易的config配置，需要注意两点：

1、config需要继承 CreateSignedTransaction；

2、需要定义类型type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

第三部分。从上面的代码可以知道，调用offchain worker是在钩子函数中实现，这个比较好理解。在offchain worker中调用交易的方式是这样，
```rust
let result = signer.send_signed_transaction(|_account| {
                Call::off_chain_signed_tx { number };
            });
```
然后就是在runtime中的配置
```rust
///  impl sendtransactiontypes for runtime
impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
    where Call: From<C>,
{
    type Extrinsic = UncheckedExtrinsic;
    type OverarchingCall = Call;
}

pub struct  MyAuthorityId;

impl frame_system::offchain::AppCrypto<<Signature as Verify>::Signer, Signature> for MyAuthorityId {
	type RuntimeAppPublic = pallet_ocw_sigtx::crypto::Public;
	type GenericSignature = sp_core::sr25519::Signature;
	type GenericPublic = sp_core::sr25519::Public;
}

/// impl send signed tx by off-chain-worker on chain
impl pallet_ocw_signed_example::Config for Runtime {
    type Event = Event;
	type AuthorityId = MyAuthorityId;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Runtime
    where
        Call: From<LocalCall>,
{
    fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
        call: Call,
        public: <Signature as sp_runtime::traits::Verify>::Signer,
        account: AccountId,
        index: Index,
    ) -> Option<(Call, <UncheckedExtrinsic as sp_runtime::traits::Extrinsic>::SignaturePayload)> {
        let period = BlockHashCount::get() as u64;
        let current_block = System::block_number()
            .saturated_into::<u64>()
            .saturating_sub(1);
        let tip = 0;


        let extra: SignedExtra = (

            frame_system::CheckSpecVersion::<Runtime>::new(),
            frame_system::CheckTxVersion::<Runtime>::new(),
            frame_system::CheckGenesis::<Runtime>::new(),
            frame_system::CheckEra::<Runtime>::from(generic::Era::mortal(period, current_block)),
            frame_system::CheckNonce::<Runtime>::from(index),
            frame_system::CheckWeight::<Runtime>::new(),
            pallet_transaction_payment::ChargeTransactionPayment::<Runtime>::from(tip),
        );

        let raw_payload = SignedPayload::new(call, extra)
            .map_err(|e| {
                log::warn!("Unable to create signed payload: {:?}", e);
            })
            .ok()?;
        let signature = raw_payload.using_encoded(|payload| C::sign(payload, public))?;
        let address = account;
        let (call, extra, _) = raw_payload.deconstruct();
        Some((call, (sp_runtime::MultiAddress::Id(address), signature.into(), extra)))
    }
}


impl frame_system::offchain::SigningTypes for Runtime {
    type Public = <Signature as sp_runtime::traits::Verify>::Signer;
    type Signature = Signature;
}

impl<C> frame_system::offchain::SendTransactionTypes<C> for Runtime
    where
        Call: From<C>,
{
    type OverarchingCall = Call;
    type Extrinsic = UncheckedExtrinsic;
}
```

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

