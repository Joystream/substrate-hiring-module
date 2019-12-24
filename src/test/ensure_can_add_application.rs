use super::*;
use crate::mock::*;

use crate::hiring::*;
use add_opening::{AddOpeningFixture, OPENING_HUMAN_READABLE_TEXT};
use rstd::collections::btree_set::BTreeSet;
/*
Not covered:
- ApplicationRationingPolicy
*/
#[test]
fn ensure_can_add_application_fails_with_no_opening() {
    build_test_externalities().execute_with(|| {
        assert_eq!(
            Hiring::ensure_can_add_application(2, None, None),
            Err(AddApplicationError::OpeningDoesNotExist)
        );
    });
}

#[test]
fn ensure_can_add_application_fails_with_redundant_role_stake() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();
        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        assert_eq!(
            Hiring::ensure_can_add_application(opening_id, Some(200), None),
            Err(AddApplicationError::StakeProvidedWhenRedundant(
                StakePurpose::Role
            ))
        );
    });
}

#[test]
fn ensure_can_add_application_fails_with_too_low_role_stake_amout() {
    build_test_externalities().execute_with(|| {
        let mut opening_fixture = AddOpeningFixture::default();
        opening_fixture.role_staking_policy = Some(StakingPolicy {
            amount: 100,
            amount_mode: StakingAmountLimitMode::Exact,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        assert_eq!(
            Hiring::ensure_can_add_application(opening_id, Some(200), None),
            Err(AddApplicationError::StakeAmountTooLow(StakePurpose::Role))
        );
    });
}

#[test]
fn ensure_can_add_application_fails_with_missing_role_stake_when_required() {
    build_test_externalities().execute_with(|| {
        //**** stake provided when redundant
        let mut opening_fixture = AddOpeningFixture::default();
        opening_fixture.role_staking_policy = Some(StakingPolicy {
            amount: 100,
            amount_mode: StakingAmountLimitMode::Exact,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        assert_eq!(
            Hiring::ensure_can_add_application(opening_id, None, None),
            Err(AddApplicationError::StakeMissingWhenRequired(
                StakePurpose::Role
            ))
        );
    });
}

#[test]
fn ensure_can_add_application_fails_with_redundant_application_stake() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();
        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        assert_eq!(
            Hiring::ensure_can_add_application(opening_id, None, Some(200),),
            Err(AddApplicationError::StakeProvidedWhenRedundant(
                StakePurpose::Application
            ))
        );
    });
}

#[test]
fn ensure_can_add_application_fails_with_too_low_application_stake_amout() {
    build_test_externalities().execute_with(|| {
        let mut opening_fixture = AddOpeningFixture::default();
        opening_fixture.application_staking_policy = Some(StakingPolicy {
            amount: 100,
            amount_mode: StakingAmountLimitMode::Exact,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        assert_eq!(
            Hiring::ensure_can_add_application(opening_id, None, Some(200),),
            Err(AddApplicationError::StakeAmountTooLow(
                StakePurpose::Application
            ))
        );
    });
}

#[test]
fn ensure_can_add_application_fails_with_missing_application_stake_when_required() {
    build_test_externalities().execute_with(|| {
        //**** stake provided when redundant
        let mut opening_fixture = AddOpeningFixture::default();
        opening_fixture.application_staking_policy = Some(StakingPolicy {
            amount: 100,
            amount_mode: StakingAmountLimitMode::Exact,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });

        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        assert_eq!(
            Hiring::ensure_can_add_application(opening_id, None, None),
            Err(AddApplicationError::StakeMissingWhenRequired(
                StakePurpose::Application
            ))
        );
    });
}

#[test]
fn ensure_can_add_application_fails_with_non_active_opening() {
    build_test_externalities().execute_with(|| {
        let mut opening_fixture = AddOpeningFixture::default();
        opening_fixture.activate_at = ActivateOpeningAt::ExactBlock(22);
        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        assert_eq!(
            Hiring::ensure_can_add_application(opening_id, None, None),
            Err(AddApplicationError::OpeningNotInAcceptingApplicationsStage)
        );
    });
}

#[test]
fn ensure_can_add_application_fails_with_non_accepting_application_stage() {
    build_test_externalities().execute_with(|| {
        let opening_fixture = AddOpeningFixture::default();
        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        assert_eq!(Hiring::begin_review(opening_id), Ok(()));

        assert_eq!(
            Hiring::ensure_can_add_application(opening_id, None, None),
            Err(AddApplicationError::OpeningNotInAcceptingApplicationsStage)
        );
    });
}

#[test]
fn ensure_can_add_application_success() {
    build_test_externalities().execute_with(|| {
        let mut opening_fixture = AddOpeningFixture::default();
        opening_fixture.application_staking_policy = Some(StakingPolicy {
            amount: 100,
            amount_mode: StakingAmountLimitMode::Exact,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });
        opening_fixture.role_staking_policy = Some(StakingPolicy {
            amount: 100,
            amount_mode: StakingAmountLimitMode::Exact,
            crowded_out_unstaking_period_length: None,
            review_period_expired_unstaking_period_length: None,
        });
        let add_opening_result = opening_fixture.add_opening();
        let opening_id = add_opening_result.unwrap();

        let ensure_can_add_application_result =
            Hiring::ensure_can_add_application(opening_id, Some(100), Some(100));

        assert_eq!(
            ensure_can_add_application_result,
            Ok(DestructuredApplicationCanBeAddedEvaluation {
                opening: Opening {
                    created: 1,
                    stage: hiring::OpeningStage::Active {
                        stage: hiring::ActiveOpeningStage::AcceptingApplications {
                            started_accepting_applicants_at_block: 1
                        },
                        applications_added: BTreeSet::new(),
                        active_application_count: 0,
                        unstaking_application_count: 0,
                        deactivated_application_count: 0
                    },
                    max_review_period_length: 672,
                    application_rationing_policy: None,
                    application_staking_policy: Some(StakingPolicy {
                        amount: 100,
                        amount_mode: StakingAmountLimitMode::Exact,
                        crowded_out_unstaking_period_length: None,
                        review_period_expired_unstaking_period_length: None
                    }),
                    role_staking_policy: Some(StakingPolicy {
                        amount: 100,
                        amount_mode: StakingAmountLimitMode::Exact,
                        crowded_out_unstaking_period_length: None,
                        review_period_expired_unstaking_period_length: None
                    }),
                    human_readable_text: OPENING_HUMAN_READABLE_TEXT.to_vec()
                },
                active_stage: hiring::ActiveOpeningStage::AcceptingApplications {
                    started_accepting_applicants_at_block: 1
                },
                applications_added: BTreeSet::new(),
                active_application_count: 0,
                unstaking_application_count: 0,
                deactivated_application_count: 0,
                would_get_added_success: ApplicationAddedSuccess::Unconditionally
            })
        );
    });
}

//#[test]
//fn ensure_can_add_application_fails_with_application_rationing_policy() {
//    build_test_externalities().execute_with(|| {
//        let mut opening_fixture = AddOpeningFixture::default();
//        opening_fixture.application_rationing_policy = Some(ApplicationRationingPolicy{max_active_applicants: 1});
//        let add_opening_result = opening_fixture.add_opening();
//        let opening_id = add_opening_result.unwrap();
//
//   //     assert_eq!(Hiring::begin_review(opening_id), Ok(()));
//
//        assert_eq!(
//            Hiring::ensure_can_add_application(opening_id, None, None),
//            Ok(DestructuredApplicationCanBeAddedEvaluation{..})
//        );
//    });
//}