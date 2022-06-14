## Substrate storage
`#[pallet::storage]`允许在运行时存储中定义一些抽象存储并设置其元数据。该属性可以多次使用。
```rust
#[pallet::storage]
#[pallet::getter(fn $getter_name)] // optional
$vis type $StorageName<$some_generic> $optional_where_clause
	= $StorageType<$generic_name = $some_generics, $other_name = $some_other, ...>;
```
具有泛型：T 或 `T:Config`， `StorageType`必须是 `StorageValue`、`StorageMap` 或 `StorageDoubleMap` `SomeNMap`

```rust
/// u32 表明存储的数据类型， ValueQuery 当存储中有值，get会返回值，无值则返回`OnEmpty`
#[pallet::storage]
type SomePrivateValue<T> = StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
#[pallet::getter(fn some_primitive_value)]
pub(super) type SomePrimitiveValue<T> = StorageValue<_, u32, ValueQuery>;

#[pallet::storage]
pub(super) type SomeComplexValue<T: Config> = StorageValue<_, T::AccountId, ValueQuery>;

/// k==V 存储 kv之间使用的  Blake2_128Concat hash算法 且k、v类型都为u32
#[pallet::storage]
#[pallet::getter(fn some_map)]
pub(super) type SomeMap<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u32, ValueQuery>;

/// 双键存储
#[pallet::storage]
pub(super) type SomeDoubleMap<T: Config> = StorageDoubleMap<_, Blake2_128Concat, u32, Blake2_128Concat, T::AccountId, u32, ValueQuery>;
/// 多键存储
#[pallet::storage]
#[pallet::getter(fn some_nmap)]
pub(super) type SomeNMap<T: Config> = StorageNMap<
    _,
    (
        NMapKey<Blake2_128Concat, u32>,
        NMapKey<Blake2_128Concat, T::AccountId>,
        NMapKey<Twox64Concat, u32>,
    ),
    u32,
    ValueQuery,
>;
```
>传递给存储项的 QueryKindTrait 的实现决定了当存储中没有值时应该如何处理存储。使用 OptionQuery，当存储中没有值时，get 方法将返回 None。使用 ValueQuery，当存储中没有值时，get 方法将返回使用 OnEmpty 泛型配置的值。对于需要配置特定默认值的情况，建议使用 ValueQuery。
>
> SomePrivateValue 之外的所有存储项都通过 pub 关键字公开。区块链存储始终从运行时外部公开可见； Substrate 存储项目的可见性仅影响运行时中的其他pallet是否能够访问存储项目。

> `#[pallet::getter(fn name)]` 提供可选的 get 扩展，可用于为包含该存储项的模块上的存储项实现 getter 方法；该扩展将所需的 getter 函数名称作为参数。如果省略了这个可选的扩展，仍然可以访问存储项的值，但是将无法通过在模块上实现的 getter 方法来访问；相反，将需要使用存储项目的 get 方法。
> 换句话说则是 可以在下边的调度函数中直接使用 fn name 获取存储中的值，前提是此处已经设置


默认值指定：
```rust

#[pallet::type_value]
pub(super) fn MyDefault<T: Config>() -> T::Balance { 3.into() }
#[pallet::storage]
pub(super) type MyStorageValue<T: Config> =
    StorageValue<Value = T::Balance, QueryKind = ValueQuery, OnEmpty = MyDefault<T>>;
```



## substrate pallet error

`#[pallet::error]` 允许定义错误类型以在错误时从可调度返回。这种错误类型信息被放入元数据中。 本质上是个枚举类型
```rust
#[pallet::error]
pub enum Error<T> {
	/// $some_optional_doc
	$SomeFieldLessVariant,
	...
}
```
tip:泛型 T 不能绑定任何东西，并且不允许使用 where 子句。但是任何用例都不需要 bounds 和 where 子句。


## substrate pallet events

`#[pallet::event]` 允许定义pallet event，event在存放时存储在块中（并在下一个块中删除）。
```rust

#[pallet::event]
#[pallet::generate_deposit($visibility fn deposit_event)] // Optional
pub enum Event<$some_generic> $optional_where_clause {
	/// Some doc
	$SomeName($SomeType, $YetanotherType, ...),
	...
}
```
每个字段都必须实现 `Clone、Eq、PartialEq、Encode、Decode 和 Debug`（仅在 std 上）。为了便于使用，绑定了 `frame_support::pallet_prelude` 中可用的 `trait Member`。

`#[pallet::generate_deposit($visibility fn deposit_event)] `在 Pallet 上生成一个辅助函数来存放事件。

宏将在枚举事件上添加属性：
`#[derive(frame_support::CloneNoBound)]`,
`#[derive(frame_support::EqNoBound)],`
`#[derive(frame_support::PartialEqNoBound)],`
`#[derive(codec::Encode)],`
`#[derive(codec::Decode)],`
`#[derive(frame_support::RuntimeDebugNoBound)]`



## Substrate dispatchables method
```rust
#[pallet::call]
impl<T:Config> Pallet<T>{
  #[pallet::weight($ExpressionResultingInWeight)]
  pub fn $fn_name(origin:Origin<T>,arg_one:types,...)->DispatchResultWithPostInfo{
    // do something


    Ok(().into)
}
}
```
pallet 中调度方法的实现，可参照上述的模板。每个调度方法都需要定义一个带有`#[pallet::weight($expir)]`属性，且第一个参数必须是`Origin<T>`,返回值必须为`DispatchResultWithPostInfo ` 或 `DispatchResult`
参数紧密编码可使用`#[pallet::compact]`。
所有参数都必须实现 `Debug、PartialEq、Eq、Decode、Encode、Clone`。为了便于使用，绑定了 `frame_support::pallet_prelude` 中可用的 `trait Member`。

若不存在`#[pallet::call]`,则自动生成对应于一下代码的实现：
```rust
#[pallet::call]
impl <T>pallet<T>where T:Config{}
```

## substrate runtime pallet weight and fees
区块链系统可用资源是有限的，因此如何合理利用资源十分重要。其中管理的资源包含：
- 内存使用
- 存储存入和取出
- 计算
- 交易、区块size(在substrate中称为 `Extrinsics`)
- 状态数据库

substrate 提供了多种方法来管理对资源的访问，并防止链的各个组件消耗过多任何单一的资源。于是设计了`weight`\ `transaction fee`
weight 管理验证块所需要的时间。weight用于表达在块主体中执行外部调用(tx)所需的时间。通过控制块可以消耗的执行时间，weight设置了存储输入和输出以及计算的限制。但这并不用于限制对其他资源的访问，如存储、内存占用等。

区块允许的一些weihgt作为区块初始化和最终确定的部分消耗。weight也可用于执行固有外部调用。

交易费(tx fee)，通常适用于用户发起的交易，并在执行交易请求之前扣除。

### fee calcuate

参照transaction pallet

- `basefee`:用户为tx支付的最低额度。在运行时被声明为基本weight，并使用`WeightToFee` 转换为fee
- `wight fee`: 与交易耗时(输入输出的计算)成比例的费用
- `lenght fee` :与交易编码长度成比例的费用
- `tip` 增加事务优先级的可选提示，使其有更大的几率被包含在tx queue中

basefee、weight、lenght 构成包含费用**。包含费用是交易被包含在一个区块中必须可用的最低费用。**

```shell
inclusion_fee = base_fee + length_fee + [targeted_fee_adjustment * weight_fee];
final_fee = inclusion_fee + tip;
```

`targeted_fee_adjustment` 可以根据网络的堵塞情况来调整最终费用。

- `basefee` 涵盖了签名验证等

- `lengthfee` 是每字节费用乘以编码的消息长度

- `weight fee` 计算则需要

  - `ExtrinsicBaseWeight`  在运行时声明并适用于外部变量
  - `#[pallet::weight] `



将`weight` 转换为fee，runtime必须定义一个实现转换函数的`WeightToFee`struct `Convert<Weight,Balance>`

注: 在调用外部消息之前，会向sender收取inclusion fee。即使交易在执行时失败，费用也会从sender的余额中扣除。

足够支付包含费用并维持最低存续存款——那么你应该确保交易被取消，这样就不会扣除任何费用，并且事务没有开始执行。

**如果向链查询交易费用，它只会返回包含费用。**



```rust
#[pallet::weight(100_000)]
fn my_dispatchable() {
    // ...
}
```

对于db读写操作的weight限制

```rust
#[pallet::weight(T::DbWeight::get().reads_writes(1, 2) + 20_000)]
fn my_dispatchable() {
    // ...
}
```

> 除了增加额外 20,000 次的其他操作之外，此可调度程序执行一次数据库读取和两次数据库写入。数据库访问通常是每次访问 #[pallet::storage] 块内声明的值时。但是，只计算唯一访问次数，因为一旦访问了一个值，它就会被缓存，再次访问它不会导致数据库操作。那是：
>
> - 相同的值多次读取记为一次读取
> - 相同值的多次写入算作一次写入
> - 多次读取相同的值，然后写入该值，算作一次读取和一次写入。
> - 写后跟只读算作一次写

### 调度 dispatch

dispatch分为三类：`Normal` `Operational` `Mandatory` 如果在weight注释中没有另外定义，则调度是Normal

也可以指定为其他的类型

```rust
#[pallet::weight((100_000, DispatchClass::Operational))]
fn my_dispatchable() {
    // ...
}
```

此元组表示法还允许指定最终参数，该参数确定是否根据带注释的权重向用户收费。如果未另行定义，则假定 Pays::Yes：

```rust
#[pallet::weight((100_000, DispatchClass::Normal, Pays::No))]
fn my_dispatchable() {
    // ...
}
```

#### Normal

此类中的调度表示**正常的用户触发事务。**这些类型的调度可能只消耗块总重量限制的一部分；这部分可以通过检查` **AvailableBlockRatio`** 找到。正常调度被发送到txpool。



#### operational

代表提供网络能力的调度类型，这些类型的调度可能会消耗一个块的整个权重限制，也就是说它们不受可用块比率的约束。此类中的调度具有最高优先级，并且无需支付 length_fee。



#### mandatory

强制调度将包含在一个块内，即使会导致区块weight超过限制。此调度类只能应用于固有函数，旨在表示作为块验证过程一部分的函数。**由于无论函数权重如何，这些类型的调度总是包含在一个块中**。

### Dynaminc weight

权重计算还可以考虑可调度的输入参数。权重应该可以通过一些基本算术从输入参数中轻松计算

例：

```rust
#[pallet::weight(FunctionOf(
  |args: (&Vec<User>,)| args.0.len().saturating_mul(10_000),
  DispatchClass::Normal,
  Pays::Yes,
))]
fn handle_users(origin, calls: Vec<User>) {
    // Do something per user
}
```



### post dispatch weight correction

```rust
#[pallet::weight(10_000 + 500_000_000)]
fn expensive_or_cheap(input: u64) -> DispatchResultWithPostInfo {
    let was_heavy = do_calculation(input);

    if (was_heavy) {
        // None means "no correction" from the weight annotation.
        Ok(None.into())
    } else {
        // Return the actual weight consumed.
        Ok(Some(10_000).into())
    }
}
```

### 自定义weight计算类型

自定义的类型必须满足trait：

- `WeightData<T>`
- `Classify<T>` 调度的类型
- `PaysFee<T>`  确定sender 是否支付fee

> 然后，Substrate 将两个特征的输出信息捆绑到 [DispatchInfo] 结构中，并通过为所有 Call 变体和不透明的外部类型实现 [GetDispatchInfo] 来提供它。这由系统和执行模块在内部使用

```rust
struct LenWeight(u32);

impl<T>WeightData for LenWeight{
    fn weight_data(&self,target:T)->Weight{
        let multiplier = self.0;
        let encode_len=target.encode().len() as u32;
        multiplier * encode_len
    }
}

impl<T> ClassifyDispatch<T> for LenWeight {
    fn classify_dispatch(&self, target: T) -> DispatchClass {
        let encoded_len = target.encode().len() as u32;
        if encoded_len > 100 {
            DispatchClass::Operational
        } else {
            DispatchClass::Normal
        }
    }
}

impl<T> PaysFee<T> {
    fn pays_fee(&self, target: T) -> Pays {
        let encoded_len = target.encode().len() as u32;
        if encoded_len > 10 {
            Pays::Yes
        } else {
            Pays::No
        }
    }
}

```


将权重计算为 m * len(args) 其中 m 是给定的乘数， args 是所有调度参数的连接元组。此外，如果事务的参数长度超过 100 个字节，则调度类是可操作的，如果编码长度大于 10 个字节，则将支付费用。


`#[transactional]` 函数上有此属性表明函数是原子执行的，函数中的任何位置错误返回都会导致函数中设置的所有状态回滚
```rust

#[pallet::weight(0)]
		#[transactional]
		pub fn set_param(origin:OriginFor<T>,param:u32)->DispatchResult{

			ensure_signed(origin)?;

			SetFlag::<T>::put(true);

			if param <100u32{
				return Err(Error::<T>::ParamInvalid.into());
			}

			Param::<T>::put(&param);


			Self::deposit_event(Event::SetParams(param));

			Ok(().into())
		}
```
若没有#[transactional]，则需要调动程序的位置
```rust
#[pallet::weight(0)]

		pub fn set_param(origin:OriginFor<T>,param:u32)->DispatchResult{

			ensure_signed(origin)?;



			if param <100u32{
				return Err(Error::<T>::ParamInvalid.into());
			}

			Param::<T>::put(&param);

      SetFlag::<T>::put(true);

			Self::deposit_event(Event::SetParams(param));

			Ok(().into())
		}
```

## Execution
Substrate runtime 的执行有Executive module决定，不同于Frame中的其他模块，这不是运行时模块。其调用区块中包含的各种运行时模块。执行公开的`execute_block` 方法

- initialize block
- Execute extrinsics
- Finalize block

### initialize block
system module 和其他runtime module 都执行`on_initialize` 方法。顺序则是按照在runtime/src/lib.rs 中`construct_runtime!` 中定义的顺序执行，但是system永远是第一位执行。

### Execute extrinsics
当区块初始化完成之后，根据extrinsics 的优先级顺序执行

### Finalize block
当消息执行完毕之后，Executive module 调用每个模块的`on_idle` 、`on_finalize` 来执行在区块结束时的逻辑。模块在此按照在`construct_runtime!`中定义的顺序执行，system最后执行
`on_idle` 也会通过区块的剩余权重，以允许根据区块链的使用情况执行。

```rust

pub trait Hooks<BlockNumber> {
    fn on_finalize(_n: BlockNumber) { ... }
    fn on_idle(_n: BlockNumber, _remaining_weight: Weight) -> Weight { ... }
    fn on_initialize(_n: BlockNumber) -> Weight { ... }
    fn on_runtime_upgrade() -> Weight { ... }
    fn pre_upgrade() -> Result<(), &'static str> { ... }
    fn post_upgrade() -> Result<(), &'static str> { ... }
    fn offchain_worker(_n: BlockNumber) { ... }
    fn integrity_test() { ... }
}
```
- on_finalize 在区块finalize的时候调用。
- on_idle 区块finalize的时候调用，不过比on_finalize先调用。如果剩余权重为 0，则不会触发。返回使用的权重，Hook将从当前使用的权重中减去它，并将结果传递给下一个 on_idle （如果存在）。
- on_initialize  区块初始化时执行，返回区块中固有的weight
- on_runtime_upgrade  模块升级时使用，这并不包含运行时升级触发的所有pallet 逻辑。该函数将在我们初始化任何运行时状态之前调用，也就是尚未调用 on_initialize。因此，无法访问块编号和任何其他块本地数据等信息。 返回运行时升级消耗的固有0权重。该函数将在初始化任何运行时状态之前调用，也就是尚未调用 on_initialize。因此，无法访问块编号和任何其他块本地数据等信息。 返回运行时升级消耗的不可协商权重。
- pre_upgrade 在运行时升级之前执行一些预检查。,作为测试工具使用
- post_upgrade  在运行时升级后执行一些后期检查,仅作为测试工具使用
- offchain_worker 在一个pallet上实现此函数后可以在此函数中长时间的执行需要链下执行的功能。该函数会在每次区块导入的时候调用。可以读取链的状态
- integrity_test  集成测试



## pallet 中调用其他的pallet
自定义的pallet中使用其他的pallet的情况:
- 在pallet的config中定义类型，然后runtime中使用时指定这个类型为frame中指定某个现成的pallet。
- 在pallet的config 中定义类型，然后runtime中使用时指定这个类型为frame中指定某个自定义的pallet。
- 封装和扩展现有的pallet。

确指定pallet的 Config 受要使用的另一个pallet的 Config 的约束。

```rust
pub trait Config: frame_system::Config + some_pallet::Config {
    // --snip--
}
```

耦合性低的另一只写法

在runtime 配置，此类型的实际发生在pallet之外(runtime/src/lib.rs)。可以让另一个实现了这个trait的pallet进行配置，或者声明一个全新的struct，实现这些trait，然后再runtime中进行配置

```rust
pub trait Currency<AccountId> {
    // -- snip --
    fn transfer(
        source: &AccountId,
        dest: &AccountId,
        value: Self::Balance,
        // don't worry about the last parameter for now
        existence_requirement: ExistenceRequirement,
    ) -> DispatchResult;
}
```



然后在自定义的pallet中定义相关的类型，如此就可以通过`T::mycurrency::transfer()`

```rust
pub trait Config: frame_system::Config {
    type MyCurrency: Currency<Self::AccountId>;
}

impl<T: Config> Pallet<T> {
    pub fn my_function() {
        T::MyCurrency::transfer(&buyer, &seller, price, ExistenceRequirement::KeepAlive)?;
    }
}
```

然后再runtime中进行`my_pallet` 的配置

```rust
impl my_pallet::Config for Runtime{
    type MyCurrency = Balances;
}
```







## pallet debug
- 可以使用Rust的 log api进行调试
- 使用log crate `log = { version = "0.4.14", default-features = false }`
- 使用 `Printable trait`
  - `Printable trait`是一种在 `no_std` 和 `std` 中从运行时打印的方法。
  - ```rust
    use sp_runtime::traits::Printable;
    use sp_runtime::print;
    ```
- substrate `print` function
  - `  print!("After storing my_val");`
  - 使用Rust_LOG 启动 `RUST_LOG=runtime=debug ./target/release/node-template --dev`

## substrate randomness
substrate 中提供了一个`Randomness` trait,其编码了生成随机数的逻辑和使用逻辑之间的接口。允许两个逻辑片段彼此独立编写。
该trait提供了两种随机性方法：
- `random_seed`  无需参数，返回一个原始的随机数。**在一个块中多次调用此方法将每次返回相同的值。**
- `random` 用一个字节数组，用作上下文标识符，并返回一个对该上下文唯一的结果，并且在底层随机源允许的情况下独立于其他上下文。
### generate randomness
Substrate 带有两种randomness trait实现:
- `Randomness Colletctive pallet`  基于collective coin flip，高效但并不安全。此pallet仅在测试消耗随机数的pallet时使用，并不用于生产。
- `BABE pallet` 它使用可验证的随机函数。该托盘提供生产级随机性，并用于 Polkadot。选择此随机源表明您的区块链使用 Babe 共识。







