use crate::{mock::*, Error, Event};
use frame_support::assert_ok;

#[test]
fn it_create_multisig_group() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.

		assert_ok!(MultisigModule::create_multisig_group(RuntimeOrigin::signed(1), vec![1, 2, 3]));

		MultisigModule::add_member(RuntimeOrigin::signed(1), 4);
		MultisigModule::approve(RuntimeOrigin::signed(2), 1);
		MultisigModule::approve(RuntimeOrigin::signed(3), 1);

		assert_eq!(MultisigModule::members().contains(&4), false);
	});
}

#[test]
fn remove_member_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(MultisigModule::create_multisig_group(RuntimeOrigin::signed(1), vec![1, 2, 3]));

		/*
		test failed, need to fix
		perhaps the add member have some problem
		 */

		MultisigModule::add_member(RuntimeOrigin::signed(1), 4);
		MultisigModule::approve(RuntimeOrigin::signed(2), 1);
		MultisigModule::approve(RuntimeOrigin::signed(3), 1);

		assert_eq!(MultisigModule::members().contains(&2), true);

		assert_ok!(MultisigModule::remove_member(RuntimeOrigin::signed(4), 2));
		MultisigModule::approve(RuntimeOrigin::signed(3), 2);
		MultisigModule::approve(RuntimeOrigin::signed(1), 2);

		assert_eq!(MultisigModule::members().contains(&2), false);
	});
}
