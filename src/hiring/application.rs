use codec::{Decode, Encode};

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use rstd::vec::Vec;

use crate::hiring::StakePurpose;

/// An application for an actor to occupy an opening.
#[derive(Encode, Decode, Default, Debug, Eq, PartialEq, Clone, PartialOrd, Ord)]
pub struct Application<OpeningId, BlockNumber, StakeId> {
    /// Identifier for opening for which this application is for.
    pub opening_id: OpeningId,

    /// Index of arrival across all applications for given opening,
    /// which is needed for strictly ordering applications.
    /// Starts at 0.
    pub application_index_in_opening: u32,

    /// Block at which this application was added.
    pub add_to_opening_in_block: BlockNumber,

    // NB: The given staking identifiers have a bloated purpose,
    // and are mutable, fix this.
    // https://github.com/Joystream/substrate-hiring-module/issues/11
    /// Identifier for stake that may possibly be established for role.
    /// Will be set iff the role staking policy of the corresponding opening
    /// states so AND application is not inactive.
    pub active_role_staking_id: Option<StakeId>,

    /// Identifier for stake that may possibly be established for application
    /// Will be set iff the application staking policy of the corresponding opening
    /// states so.
    pub active_application_staking_id: Option<StakeId>,

    /// Status of this application
    pub stage: ApplicationStage<BlockNumber>,

    // ...
    pub human_readable_text: Vec<u8>,
}

/// Possible status of an application
#[derive(Encode, Decode, Debug, Eq, PartialEq, Clone, PartialOrd, Ord)]
pub enum ApplicationStage<BlockNumber> {
    /// Normal active state
    Active,

    /// Waiting for one or more unstakings, with a non-zero unstaking period, to complete.
    Unstaking {
        // When deactivation was initiated.
        deactivation_initiated: BlockNumber,

        // The cause of the deactivation.
        cause: ApplicationDeactivationCause,
    },

    ///  No longer active, can't do anything fun now.
    Inactive {
        /// When deactivation was initiated.
        deactivation_initiated: BlockNumber,

        /// When deactivation was completed, and the inactive state was established.
        deactivated: BlockNumber,

        /// The cause of the deactivation.
        cause: ApplicationDeactivationCause,
    },
}

/// Possible application deactivation causes
#[derive(Encode, Decode, Debug, Eq, PartialEq, Clone, Copy, PartialOrd, Ord)]
pub enum ApplicationDeactivationCause {
    External, // Add ID here for simplicity?
    Hired,
    NotHired,
    CrowdedOut,
    OpeningCancelled,
    ReviewPeriodExpired,
    OpeningFilled,
}

/// OpeningStage must be default constructible because it indirectly is a value in a storage map.
/// ***SHOULD NEVER ACTUALLY GET CALLED, IS REQUIRED TO DUE BAD STORAGE MODEL IN SUBSTRATE***
impl<BlockNumber> Default for ApplicationStage<BlockNumber> {
    fn default() -> Self {
        ApplicationStage::Active
    }
}

/// How to limit the number of eligible applicants
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Debug, Eq, PartialEq, Clone)]
pub struct ApplicationRationingPolicy {
    /// The maximum number of applications that can be on the list at any time.
    pub max_active_applicants: u32,
    // How applicants will be ranked, in order to respect the maximum simultaneous application limit
    //pub applicant_ranking: ApplicationRankingPolicy
}

#[derive(Encode, Decode, Debug, Eq, PartialEq, Clone)]
pub enum OpeningDeactivationCause {
    CancelledBeforeActivation,
    CancelledAcceptingApplications,
    CancelledInReviewPeriod,
    ReviewPeriodExpired,
    Filled,
}

#[derive(Encode, Decode, Debug, Eq, PartialEq, Clone)]
pub enum ActiveOpeningStage<BlockNumber> {
    AcceptingApplications {
        //
        started_accepting_applicants_at_block: BlockNumber,
    },

    //
    ReviewPeriod {
        started_accepting_applicants_at_block: BlockNumber,

        started_review_period_at_block: BlockNumber,
    },

    //
    Deactivated {
        cause: OpeningDeactivationCause,

        deactivated_at_block: BlockNumber,

        started_accepting_applicants_at_block: BlockNumber,

        /// Whether the review period had ever been started, and if so, at what block.
        /// Deactivation can also occur directly from the AcceptingApplications stage.
        started_review_period_at_block: Option<BlockNumber>,
    },
}

impl<BlockNumber: Clone> ActiveOpeningStage<BlockNumber> {
    /// Ensures that active opening stage is accepting applications.
    pub fn ensure_active_opening_is_accepting_applications<Err>(
        &self,
        error: Err,
    ) -> Result<BlockNumber, Err> {
        if let ActiveOpeningStage::AcceptingApplications {
            started_accepting_applicants_at_block,
        } = self
        {
            return Ok(started_accepting_applicants_at_block.clone());
        }

        Err(error)
    }

    /// Ensures that active opening stage is in review period.
    pub fn ensure_active_opening_is_in_review_period<Err>(
        &self,
        error: Err,
    ) -> Result<(BlockNumber, BlockNumber), Err> {
        match self {
            ActiveOpeningStage::ReviewPeriod {
                started_accepting_applicants_at_block,
                started_review_period_at_block,
            } => Ok((
                started_accepting_applicants_at_block.clone(),
                started_review_period_at_block.clone(),
            )), // <= need proper type here in the future, not param
            _ => Err(error),
        }
    }
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum BeginAcceptingApplicationsError {
    OpeningDoesNotExist,
    OpeningIsNotInWaitingToBeginStage,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum AddApplicationError {
    OpeningDoesNotExist,
    StakeProvidedWhenRedundant(StakePurpose),
    StakeMissingWhenRequired(StakePurpose),
    StakeAmountTooLow(StakePurpose),
    OpeningNotInAcceptingApplicationsStage,
    NewApplicationWasCrowdedOut,
    BrokenInvariant(&'static str),
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub struct ApplicationAdded<ApplicationId> {
    /// ...
    pub application_id_added: ApplicationId,

    /// ...
    pub application_id_crowded_out: Option<ApplicationId>,
}

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum DeactivateApplicationError {
    ApplicationDoesNotExist,
    ApplicationNotActive,
    OpeningNotAcceptingApplications,
    UnstakingPeriodTooShort(StakePurpose),
    RedundantUnstakingPeriodProvided(StakePurpose),
    BrokenInvariant(&'static str),
}
