// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! RPC interface for the transaction payment pallet.

use std::sync::Arc;
use jsonrpsee::{
	core::RpcResult,
	proc_macros::rpc,
};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::traits::Block as BlockT;


use smultisig_rpc_runtime_api::SmultisigApi;

#[rpc(client, server)]
pub trait SumtisigModuleApi {
	#[method(name = "group_members")]
	fn group_members(&self) -> RpcResult<Vec<u32>>;
}

/// Provides RPC methods to query a dispatchable's class, weight and fee.
pub struct SmultisigModule<C, P> {
	/// Shared reference to the client.
	client: Arc<C>,
	_marker: std::marker::PhantomData<P>,
}

// Error of this RPC api
pub enum Error {
	EmptyGroup,

}

impl From<Error> for i32{
	fn from(value: Error) -> Self {
		match value {
			Error::EmptyGroup => 1,
		}
	}
}

impl<C, P> SmultisigModule<C, P> {
	/// Creates a new instance of the TransactionPayment Rpc helper.
	pub fn new(client: Arc<C>) -> Self {
		Self { client, _marker: Default::default() }
	}
}

impl<C, Block> SumtisigModuleApiServer for SmultisigModule<Block, C>
where
	Block: BlockT,
	C: ProvideRuntimeApi<Block> + HeaderBackend<Block> + Send + Sync + 'static,
	C::Api: SmultisigApi<Block, u32, u32>,
{
	fn group_members(&self) -> RpcResult<Vec<u32>> {
		
		let api = self.client.runtime_api();
		
		self.client.hash();

		Ok(vec![1,2,3])
	}
}
