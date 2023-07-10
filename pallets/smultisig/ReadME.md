## Error log

When I added the Votes struct, the cargo check kept getting an error
```shell
     Compiling pallet-smultisig v0.1.0 (/root/rustcode/schain/pallets/smultisig)
  error[E0277]: the trait bound `Votes<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber>: parity_scale_codec::MaxEncodedLen` is not satisfied
    --> /root/rustcode/schain/pallets/smultisig/src/lib.rs:26:12
     |
  26 |     #[pallet::pallet]
     |               ^^^^^^ the trait `parity_scale_codec::MaxEncodedLen` is not implemented for `Votes<<T as frame_system::Config>::AccountId, <T as frame_system::Config>::BlockNumber>`
     |
     = help: the following other types implement trait `parity_scale_codec::MaxEncodedLen`:
               ()
               (TupleElement0, TupleElement1)
               (TupleElement0, TupleElement1, TupleElement2)
               (TupleElement0, TupleElement1, TupleElement2, TupleElement3)
               (TupleElement0, TupleElement1, TupleElement2, TupleElement3, TupleElement4)
               (TupleElement0, TupleElement1, TupleElement2, TupleElement3, TupleElement4, TupleElement5)
               (TupleElement0, TupleElement1, TupleElement2, TupleElement3, TupleElement4, TupleElement5, TupleElement6)
               (TupleElement0, TupleElement1, TupleElement2, TupleElement3, TupleElement4, TupleElement5, TupleElement6, TupleElement7)
             and 107 others
     = note: required for `StorageMap<_GeneratedPrefixForStorageVoting<T>, Twox64Concat, u32, Votes<<T as Config>::AccountId, ...>>` to implement `StorageInfoTrait`
     = note: the full type name has been written to '/root/rustcode/schain/target/debug/wbuild/schain-runtime/target/wasm32-unknown-unknown/release/deps/pallet_smultisig-3305752161e15988.long-type-17515771729039869851.txt'
```

This is a pallet configuration problem, so I referred to substrate/frame/collective/lib.rs line 178 to solve the check error perfectly
```shell
...
#[pallet::pallet]
#[pallet::without_storage_info]
pub struct Pallet<T>(PhantomData<T>);
...
```
As for MultisigMembers struct uses BoundedVec, I may replace it later, after all, it is too resource-intensive to perform one operation.

Record it
```shell
#[derive(PartialEq, Eq, Clone, Copy, Encode, Decode, TypeInfo, MaxEncodedLen)]
	#[scale_info(skip_type_params(T))]   //This line is essential
	pub struct Proposal<T: Config> {
		pub proposal_id: u32,
		pub threshold: ProposalThreshold,
		pub status: ProposalStatus,
		pub vote: u32,
		pub owner: T::AccountId,
	}
```
