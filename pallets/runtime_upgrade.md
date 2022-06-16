# substrate runtime upgrades
在用于运行时开发的 FRAME 系统中，system定义了 set_code 调用，用于runtime_upgrade。
扩展运行时而不是更新现有运行时状态来修改运行时。如果运行时升级定义了对现有状态的更改，则可能需要执行“存储迁移”。

## Initiate a Forkless Runtime Upgrade
substrate 可以在无分叉的情况下修改、扩展runtime功能。

runtime的定义本身就是 Substrate 链状态中的一个元素，
所以网络参与者可以通过外部的方式更新这个值，特别是` set_code `函数。
由于**运行时状态的更新受区块链的共识机制和加密保证的约束**，网络参与者可以使用区块链本身以不信任的方式分发更新或扩展的运行时逻辑，
而无需分叉链甚至发布新的区块链客户端。

在`sudo `pallet 中`sudo_unchecked_weight` 用于执行添加`Scheduler` pallet 升级，然后`Scheduler` pallet 用于执行升级，增加网路账户的存在额度。

在`substrate-node-template` 中Alice账户在chain_spec 的`development_config`中配置为Sudo pallet key的持有者，这也就意味着Alice账户用于执行runtime upgrade
详情见 `node/src/chain_spec.rs` `local_testnet_config` 方法

>Substrate 中的 Dispatchable 调用始终与权重相关联，该权重用于资源核算。 
> FRAME 的system module将外部参数限制为块 BlockLength 和 BlockWeights 限制。 
> System 模块中的 set_code 函数有意设计为消耗可能适合块的最大重量。

`set_code` 函数的weight 注释还指定外部调用和可调度函数的`Operational`中，这将其标识为与网络操作相关并影响其资源的记帐，例如通过将其从 TransactionByteFee 中免除。

>在 FRAME 中，Root Origin 用于标识运行时管理员； 
> FRAME 的一些功能，包括通过 set_code 函数更新运行时的能力，只有该管理员可以访问。 
> Sudo pallet维护一个单一的存储项目：有权访问pallet的可调度功能的帐户的 ID。 
> Sudo pallet的 sudo 函数允许此帐户的持有者调用可调度作为根源。

```rust

#[pallet::weight({
			let dispatch_info = call.get_dispatch_info();
			(dispatch_info.weight.saturating_add(10_000), dispatch_info.class)
		})]
		pub fn sudo(
			origin: OriginFor<T>,
			call: Box<<T as Config>::Call>,
		) -> DispatchResultWithPostInfo {
			// This is a public call, so we ensure that the origin is some signed account.
			let sender = ensure_signed(origin)?;
			ensure!(Self::key().map_or(false, |k| sender == k), Error::<T>::RequireSudo);

			let res = call.dispatch_bypass_filter(frame_system::RawOrigin::Root.into());
			Self::deposit_event(Event::Sudid { sudo_result: res.map(|_| ()).map_err(|e| e.error) });
			// Sudo user does not pay a fee.
			Ok(Pays::No.into())
		}
```
`sudo_unchecked_weight`  该函数提供与 sudo 函数相同的功能，但接受一个附加参数，用于指定（可能为零）用于调用的权重。
dependency import
runtime/Cargo.toml
```toml

pallet-scheduler = {version="4.0.0-dev",default-features=false, git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23"}

[features]
default = ["std"]
std = [
    #... 
    "pallet-scheduler/std",
    
]
```

runtime/src/lib.rs  添加scheduler 至runtime
```rust
pub use frame_support::traits::EqualPrivilegeOnly;
// Define the types required by the Scheduler pallet.
parameter_types! {
    pub MaximumSchedulerWeight: Weight = 10_000_000;
    pub const MaxScheduledPerBlock: u32 = 50;
}

// Configure the runtime's implementation of the Scheduler pallet.
impl pallet_scheduler::Config for Runtime {
    type Event = Event;
    type Origin = Origin;
    type PalletsOrigin = OriginCaller;
    type Call = Call;
    type MaximumWeight = MaximumSchedulerWeight;
    type ScheduleOrigin = frame_system::EnsureRoot<AccountId>;
    type MaxScheduledPerBlock = MaxScheduledPerBlock;
    type WeightInfo = ();
    type OriginPrivilegeCmp = EqualPrivilegeOnly;
  type PreimageProvider = ();
  type NoPreimagePostponement = ();
}

// Add the Scheduler pallet inside the construct_runtime! macro.
construct_runtime!(
    pub enum Runtime where
        Block = Block,
        NodeBlock = opaque::Block,
        UncheckedExtrinsic = UncheckedExtrinsic
    {
        /*** snip ***/
        Scheduler: pallet_scheduler,
    }
);
```

准备升级Frame runtime的最后一步添加spec_version
line 101
```rust
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("node-template"),
    impl_name: create_runtime_str!("node-template"),
    authoring_version: 1,
    spec_version: 102,  // *Increment* this value, the template uses 100 as a base
    impl_version: 1,
    apis: RUNTIME_API_VERSIONS,
    transaction_version: 1,
    state_version:1,
};
```

- `spec_name` runtime/chain name
- `impl_name` client name
- `authoring_version` block author version
- `spec_version` runtime/chain version
- `impl_version` client version
- `apis` support api list
- `transaction_version` dispatch function interface version
- `state_version` runtime state version

使用
```shell
cargo build --release -p node-template-reuntime   
```
然后将编译后的wasm文件通过polkadot.js 上传 提交之后即可无分叉升级。

## storage migrations with FRAME
FRAME 存储迁移是通过 `OnRuntimeUpgrade` 特征实现的，它指定了一个函数 `on_runtime_upgrade`。该函数提供了一个钩子，
允许运行时开发人员指定将在运行时升级之后但在任何外部函数甚至 `on_initialize` 函数执行之前立即运行的逻辑

准备存储迁移意味着runtime升级， Substrate 存储库使用 `E1-runtimemigration` 标签来指定此类更改。.


默认情况下，FRAME 会根据托盘在construct_runtime 中出现的顺序对on_runtime_upgrade 函数的执行进行排序。它们将以相反（从上到下）的顺序运行
FRAME storage 迁移将按照以下顺序运行：
- `frame_system::on_runtime_upgrade`
- 自定义的`on_runtime_upgrade`
- 运行时包含的pallet中定义的所有 `on_runtime_upgrade` 函数，按上述顺序




