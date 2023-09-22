//! Benchmarking setup for pallet-template
#![cfg(feature = "runtime-benchmarks")]
#![allow(clippy::type_complexity)]
use super::*;

#[allow(unused)]
use crate::{Pallet as Smultisig};
use frame_benchmarking::v1::{benchmarks,account,impl_benchmark_test_suite};
use frame_support::dispatch::RawOrigin::Root;
use sp_std::vec;
fn create_user<T: Config>(string: &'static str, n: u32, seed: u32) -> T::AccountId {
    let user = account(string, n, seed);
    user
}

benchmarks!{
    create_multisig_group{
        let caller = create_user::<T>("caller",0u32,1u32);
        let member_one = create_user::<T>("member_one",0u32,1u32);
        let member_two = create_user::<T>("member_two",0u32,1u32);
        let root = Root.as_signed().unwrap();
        let members = vec![root,&caller,&member_one,&member_two];
    }:_(Root,members)
    verify{
        assert_eq!(Smultisig::<T>::members().contains(&member_two),true);
        assert_eq!(Smultisig::<T>::members().contains(&member_one),true);
    }
}
impl_benchmark_test_suite!(Smultisig, crate::mock::new_test_ext(), crate::mock::Test);