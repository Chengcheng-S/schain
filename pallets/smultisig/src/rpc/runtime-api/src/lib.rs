// This file is part of Schain

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

//! Runtime API definition required by Smultisig RPC extensions.
//!
//! This API should be imported and implemented by the runtime,
//! of a node that wants to use the custom RPC extension
//!

#![cfg_attr(not(feature = "std"), no_std)]

sp_api::decl_runtime_apis! {
	/// The API to query account nonce.
	pub trait Proposals<AccountId, Nonce> where
		AccountId: codec::Codec,
		Nonce: codec::Codec,
	{
		/// proposal info
		fn proposal_info() -> Vec<(u32, Proposal)>;

		/// finish proposal
		fn finish_proposal() -> Vec<(u32, Proposal)>;

		/// accound who in multisig  group
		fn multisig_members() -> Vec<AccountId>;
	}
}
