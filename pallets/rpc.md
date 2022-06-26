## substrate RPC server
要将自定义 RPC 客户端连接到 Substrate 节点，必须提供称为 `RPC extension builder`。此函数接受节点是否应接受或拒绝不安全的 RPC 调用的参数，并返回节点创建 JSON RPC 所需的 IoHandler。
RPC 扩展所需的runtime API 定义。此 API 应由runtime导入和实现，想要使用自定义 RPC 扩展的节点添加系统访问方法


```shell
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "rpc_methods"}' http://localhost:9933/
```
使用以上的命令可以查看当前节点支持的rpc服务
Substrate 节点提供以下命令行选项，允许公开`pub RPC` 接口：
```shell
--ws-external
--rpc-external
--unsafe-ws-external
--unsafe-rpc-external
```
在启动substrate  节点时，可以使用以下两个端口：
- HTTP `http://localhost:9933/`
- WebSocket `ws://localhost:9944/`

```shell
curl -H "Content-Type: application/json" -d '{"id":1, "jsonrpc":"2.0", "method": "state_getMetadata"}' http://localhost:9933/
```

## 自定义rpc server
此处还需要说明一下，使用的是substrate -4.0.0-dev 在此版本之前rpc实现有许多的不同之处。
这部分可以参考 substrate-node-template 中`transaction-payment-rpc-server`
源码目录结构
```shell
-substratet
  -frame
   - transaction-payment
     -rpc
      -src 
      - runtime-api
        - lib.rs
```
### rpc trait and implement
首先需要在自定义的pallet中新建一个rpc的包，用来定义和实现用户的rpc方法
Cargo.toml 文件的配置
```toml
[package.metadata.docs.rs]
targets = ["x86_64-unknown-linux-gnu"]

[dependencies]
codec = { package = "parity-scale-codec", version = "3.0.0" }
jsonrpsee = { version = "0.13.0", features = ["server", "macros"] }
pallet-transaction-payment-rpc-runtime-api = { version = "4.0.0-dev", path = "./runtime-api" }
sp-api = { version = "4.0.0-dev", default-features = false, git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23" }
sp-blockchain = { version = "4.0.0-dev", path = "../../../primitives/blockchain" }
sp-core = { version = "6.0.0", default-features = false, git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23" }
sp-rpc = { version = "6.0.0", default-features = false, git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23" }
sp-runtime = { version = "6.0.0", default-features = false, git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23" }

```
rpc trait and implement template
```rust
///first   declare user runtimeapi trait
#[rpc(client,server)]
pub trait TransactionPaymentApi<BlockHash,ResponseType>{
    
    /// declare the rpc method 
    #[method(name="payment_queryInfo")]
    fn query_info(&self,encoded_xt: Bytes, at: Option<BlockHash>) -> RpcResult<ResponseType>;

    #[method(name = "payment_queryFeeDetails")]
    fn query_fee_details(
        &self,
        encoded_xt: Bytes,
        at: Option<BlockHash>,
    ) -> RpcResult<FeeDetails<NumberOrHex>>;
    
    /// and  etc
}

/// secondly providers RPC methods to query a dispathcable's class weight and fee 
pub struct TransactionPayment<C,P>{
    /// shared reference to the clent

    Client:Arc<C>,
    _marker: std::marker::PhantomData<P>,
}
// impl for transactionpayment struct 
impl <C,P> TransactionPayment<C,P>{
    
    ///creates new instance of the transactionpayment rpc helper
    fn new(client:Arc<C>)->Self{Self{client,_marker:Default::default()}}
}      

/// thirdly   user can define error enum
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i32 {
    fn from(e: Error) -> i32 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

/// finally implement for your rpc method
#[async trait]
impl<C,Block,Balance>
TransactionPaymentApiServer<<Block as BlockT>::Hash, RuntimeDispatchInfo<Balance>>
for TransactionPayment<C, Block>
    where
        Block: BlockT,
        C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
        C::Api: TransactionPaymentRuntimeApi<Block, Balance>,  /// can replace user define rpc runtime-api
        Balance: Codec + MaybeDisplay + Copy + TryInto<NumberOrHex> + Send + Sync + 'static,
{
    fn query_info(
        &self,
        encoded_xt: Bytes,
        at: Option<Block::Hash>,
    ) -> RpcResult<RuntimeDispatchInfo<Balance>>{todo()}

    fn query_fee_details(
        &self,
        encoded_xt: Bytes,
        at: Option<Block::Hash>,
    ) -> RpcResult<FeeDetails<NumberOrHex>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        let encoded_len = encoded_xt.len() as u32;

        let uxt: Block::Extrinsic = Decode::decode(&mut &*encoded_xt).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::DecodeError.into(),
                "Unable to query fee details.",
                Some(format!("{:?}", e)),
            ))
        })?;
        let fee_details = api.query_fee_details(&at, uxt, encoded_len).map_err(|e| {
            CallError::Custom(ErrorObject::owned(
                Error::RuntimeError.into(),
                "Unable to query fee details.",
                Some(e.to_string()),
            ))
        })?;

        let try_into_rpc_balance = |value: Balance| {
            value.try_into().map_err(|_| {
                JsonRpseeError::Call(CallError::Custom(ErrorObject::owned(
                    ErrorCode::InvalidParams.code(),
                    format!("{} doesn't fit in NumberOrHex representation", value),
                    None::<()>,
                )))
            })
        };

        Ok(FeeDetails {
            inclusion_fee: if let Some(inclusion_fee) = fee_details.inclusion_fee {
                Some(InclusionFee {
                    base_fee: try_into_rpc_balance(inclusion_fee.base_fee)?,
                    len_fee: try_into_rpc_balance(inclusion_fee.len_fee)?,
                    adjusted_weight_fee: try_into_rpc_balance(inclusion_fee.adjusted_weight_fee)?,
                })
            } else {
                None
            },
            tip: Default::default(),
        })
    }
}
```


### `runtime-api` 
这个包则是用来扩展所需要的runtime-api
Cargo.toml 文件配置
```toml

[package.metadata.docs.rs]
targets = ["x86_64-unknown-linux-gnu"]

[dependencies]
codec = { package = "parity-scale-codec", version = "3.0.0", default-features = false, features = ["derive"] }
pallet-transaction-payment = { version = "4.0.0-dev", default-features = false, path = "../../../transaction-payment" }   # runtime-api需要用到transaction-payment中的部分数据结构
sp-api = { version = "4.0.0-dev", default-features = false, git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23" }
sp-runtime = { version = "6.0.0", default-features = false, git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23" }

[features]
default = ["std"]
std = [
	"codec/std",
	"pallet-transaction-payment/std",
	"sp-api/std",
	"sp-runtime/std",
]
```

runtime-api 参考模板
```rust
#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_runtime::traits::MaybeDisplay;

pub use pallet_transaction_payment::{FeeDetails, InclusionFee, RuntimeDispatchInfo};

sp_api::decl_runtime_apis! {
	pub trait TransactionPaymentApi<Balance> where
		Balance: Codec + MaybeDisplay,
	{
		fn query_info(uxt: Block::Extrinsic, len: u32) -> RuntimeDispatchInfo<Balance>;
		fn query_fee_details(uxt: Block::Extrinsic, len: u32) -> FeeDetails<Balance>;
	}
}
```
在定义好runtime-api之后需要在runtime 进行配置
Cargo.toml 配置
```toml
#...
[dependencies]
pallet-transaction-payment = { version = "4.0.0-dev", default-features = false, git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23" }

# rpc-runtime-api 为runtime 实现runtime-api是很有必要的
pallet-transaction-payment-rpc-runtime-api = { version = "4.0.0-dev", default-features = false, git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23" }

[features]
default = ["std"]
std=[
    "pallet-transaction-payment-rpc-runtime-api/std",
    "pallet-transaction-payment/std",
]

```

runtime/lib.rs
```rust
impl_runtime_api!{
    
        // ....
    	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
	}
    
    // ....
}
```
最后要将自定义的rpc 注册到node中
Cargo.toml
```toml
[dependencies]
pallet-transaction-payment = { version = "4.0.0-dev", default-features = false, git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23" }

pallet-transaction-payment-rpc = { version = "4.0.0-dev", git = "https://github.com/paritytech/substrate.git", branch = "polkadot-v0.9.23" }
```

node/rpc.rs
```rust
pub fn create_full<C, P>(

    deps: FullDeps<C, P>,
) -> Result<RpcModule<()>, Box<dyn std::error::Error + Send + Sync>>
    where
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError> + 'static,
        C: Send + Sync + 'static,
        C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
        C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance>,
        C::Api: BlockBuilder<Block>,
        P: TransactionPool + 'static,
{
    /// ....
    module.merge(TransactionPayment::new(client).into_rpc())?;
    
    /// template module.merge(userRPCtrait::into_rpc(userRpcstructs::new(ReferenceToClient,....)))?;
}
```
之后cargo build --release 
然后启动节点通过curl 就能访问定义的rpc
