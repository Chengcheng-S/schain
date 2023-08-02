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

#![cfg_attr(not(feature = "std"), no_std)]
use codec::{Decode, Encode};
use frame_support::dispatch::Vec;

// pub use pallet_smultisig::Proposal;

sp_api::decl_runtime_apis! {
	/// The API to query account nonce.
	pub trait SmultisigApi<AccountId,Proposal> where
		AccountId: Decode + Encode,
		Proposal: Encode +  Decode,
	{
		/// proposal info
		fn proposal_info(id:u32) -> (u32, Proposal);

		/// finish proposal
		fn finish_proposal(id:u32) -> (u32, Proposal);

		/// accound who in multisig  group
		fn multisig_members() -> Vec<AccountId>;
	}
}