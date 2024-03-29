use crate::{
	mock::{RuntimeEvent, *},
	Event, ProposalStatus, ProposalThreshold,
};
use frame_support::assert_ok;
#[test]
fn it_create_multisig_group() {
	new_test_ext().execute_with(|| {
		assert_ok!(MultisigModule::create_multisig_group(RuntimeOrigin::signed(1), vec![1, 2, 3]));

		assert_ok!(MultisigModule::add_member(RuntimeOrigin::signed(1), 4));

		assert_events(vec![RuntimeEvent::MultisigModule(Event::CreateProposal {
			who: 1,
			proposal_id: 1,
			threshold: ProposalThreshold::All,
			status: ProposalStatus::Pending,
		})]);

		assert_ok!(MultisigModule::approve(RuntimeOrigin::signed(2), 1));

		let proposal_vote = MultisigModule::votings(1).unwrap();
		assert!(proposal_vote.ayes.contains(&1));
		assert!(proposal_vote.ayes.contains(&2));

		assert_ok!(MultisigModule::reject(RuntimeOrigin::signed(3), 1));

		let members = MultisigModule::add_members(1).unwrap();
		assert_eq!(members, 4);

		assert!(!MultisigModule::members().contains(&4));
	});
}

#[test]
fn remove_member_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(MultisigModule::create_multisig_group(
			RuntimeOrigin::signed(1),
			vec![1, 2, 3, 4]
		));
		assert_events(vec![RuntimeEvent::MultisigModule(Event::CreateMultisig {
			who: 1,
			dyn_threshold: 3,
		})]);

		assert_ok!(MultisigModule::remove_member(RuntimeOrigin::signed(1), 4));

		let members = MultisigModule::remove_members(1).unwrap();
		assert_eq!(members, 4);

		assert_events(vec![RuntimeEvent::MultisigModule(Event::CreateProposal {
			who: 1,
			proposal_id: 1,
			threshold: ProposalThreshold::All,
			status: ProposalStatus::Pending,
		})]);

		assert_ok!(MultisigModule::approve(RuntimeOrigin::signed(2), 1));

		assert_ok!(MultisigModule::approve(RuntimeOrigin::signed(3), 1));

		assert_ok!(MultisigModule::approve(RuntimeOrigin::signed(4), 1));

		assert!(!MultisigModule::members().contains(&4));
	});
}

#[test]
fn it_add_members_into_group_work() {
	new_test_ext().execute_with(|| {
		assert_ok!(MultisigModule::create_multisig_group(RuntimeOrigin::signed(1), vec![1, 2, 3]));

		assert_ok!(MultisigModule::add_member(RuntimeOrigin::signed(1), 4));
		assert_events(vec![RuntimeEvent::MultisigModule(Event::CreateProposal {
			who: 1,
			proposal_id: 1,
			threshold: ProposalThreshold::All,
			status: ProposalStatus::Pending,
		})]);

		assert_ok!(MultisigModule::approve(RuntimeOrigin::signed(2), 1));

		let proposal_vote = MultisigModule::votings(1).unwrap();
		assert!(proposal_vote.ayes.contains(&1));
		assert!(proposal_vote.ayes.contains(&2));

		assert_ok!(MultisigModule::approve(RuntimeOrigin::signed(3), 1));

		let members = MultisigModule::add_members(1).unwrap();
		assert_eq!(members, 4);

		assert!(MultisigModule::members().contains(&4));
	});
}
