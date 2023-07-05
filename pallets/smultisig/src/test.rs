use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		// Dispatch a signed extrinsic.

		assert_ok!(MultisigModule::create_multisig_group(
			RuntimeOrigin::signed(1),
			vec![1, 2, 3, 5]
		));

		// Read pallet storage and assert an expected result.

		assert_eq!(MultisigModule::remove_member(RuntimeOrigin::signed(1), 2), Ok(()));
	});
}
