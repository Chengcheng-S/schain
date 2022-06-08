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

