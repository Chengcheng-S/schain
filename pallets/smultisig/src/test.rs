use crate::{
	mock::{RuntimeEvent, *},
	Error, Event, ProposalStatus, ProposalThreshold,
};
use frame_support::{assert_ok, event};

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
		System::set_block_number(2);

		assert_ok!(MultisigModule::create_multisig_group(
			RuntimeOrigin::signed(1),
			vec![1, 2, 3, 4]
		));

		/*
		test failed, need to fix
		perhaps the add member have some problem
		 */

		MultisigModule::remove_member(RuntimeOrigin::signed(1), 4);
		assert_events(vec![RuntimeEvent::MultisigModule(Event::CreateProposal {
			who: 1,
			proposal_id: 1,
			threshold: ProposalThreshold::All,
			status: ProposalStatus::Pending,
		})]);
		MultisigModule::approve(RuntimeOrigin::signed(2), 1);
		assert_events(vec![RuntimeEvent::MultisigModule(Event::ApprovalProposal {
			proposal_id: 1,
			vote: 1,
			who: 2,
		})]);

		MultisigModule::approve(RuntimeOrigin::signed(3), 1);

		assert_events(vec![RuntimeEvent::MultisigModule(Event::ApprovalProposal {
			proposal_id: 1,
			vote: 1,
			who: 3,
		})]);

		assert_eq!(MultisigModule::members().contains(&4), false);

		let proposal = MultisigModule::proposals(1).ok_or(Error::<Test>::NotFoundProposal).unwrap();

		assert_eq!(proposal.status, ProposalStatus::Pending);
		assert_eq!(proposal.owner, 1);
		assert_eq!(proposal.threshold, ProposalThreshold::All);
	});
}
