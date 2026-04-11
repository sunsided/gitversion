use serde::{Deserialize, Serialize};

use crate::config::enums::{CommitMessageIncrementMode, DeploymentMode, IncrementStrategy};
use crate::config::prevent_increment::PreventIncrementConfiguration;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BranchConfiguration {
    pub deployment_mode: Option<DeploymentMode>,
    pub label: Option<String>,
    pub increment: Option<IncrementStrategy>,
    pub prevent_increment: Option<PreventIncrementConfiguration>,
    pub track_merge_target: Option<bool>,
    pub track_merge_message: Option<bool>,
    pub commit_message_incrementing: Option<CommitMessageIncrementMode>,
    pub regular_expression: Option<String>,
    pub source_branches: Option<Vec<String>>,
    pub is_source_branch_for: Option<Vec<String>>,
    pub tracks_release_branches: Option<bool>,
    pub is_release_branch: Option<bool>,
    pub is_main_branch: Option<bool>,
    pub pre_release_weight: Option<i64>,
}

impl BranchConfiguration {
    pub fn inherit(&self, fallback: &BranchConfiguration) -> BranchConfiguration {
        BranchConfiguration {
            deployment_mode: self.deployment_mode.or(fallback.deployment_mode),
            label: self.label.clone().or_else(|| fallback.label.clone()),
            increment: self.increment.or(fallback.increment),
            prevent_increment: self
                .prevent_increment
                .clone()
                .or_else(|| fallback.prevent_increment.clone()),
            track_merge_target: self.track_merge_target.or(fallback.track_merge_target),
            track_merge_message: self.track_merge_message.or(fallback.track_merge_message),
            commit_message_incrementing: self
                .commit_message_incrementing
                .or(fallback.commit_message_incrementing),
            regular_expression: self
                .regular_expression
                .clone()
                .or_else(|| fallback.regular_expression.clone()),
            source_branches: self
                .source_branches
                .clone()
                .or_else(|| fallback.source_branches.clone()),
            is_source_branch_for: self
                .is_source_branch_for
                .clone()
                .or_else(|| fallback.is_source_branch_for.clone()),
            tracks_release_branches: self
                .tracks_release_branches
                .or(fallback.tracks_release_branches),
            is_release_branch: self.is_release_branch.or(fallback.is_release_branch),
            is_main_branch: self.is_main_branch.or(fallback.is_main_branch),
            pre_release_weight: self.pre_release_weight.or(fallback.pre_release_weight),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::branch_config::BranchConfiguration;
    use crate::config::enums::{DeploymentMode, IncrementStrategy};

    #[test]
    fn inherit_uses_self_when_available_and_fallback_for_missing_values() {
        let child = BranchConfiguration {
            label: Some("beta".to_string()),
            ..Default::default()
        };
        let parent = BranchConfiguration {
            increment: Some(IncrementStrategy::Minor),
            deployment_mode: Some(DeploymentMode::ContinuousDelivery),
            track_merge_target: Some(true),
            ..Default::default()
        };

        let merged = child.inherit(&parent);

        assert_eq!(merged.label.as_deref(), Some("beta"));
        assert_eq!(merged.increment, Some(IncrementStrategy::Minor));
        assert_eq!(
            merged.deployment_mode,
            Some(DeploymentMode::ContinuousDelivery)
        );
        assert_eq!(merged.track_merge_target, Some(true));
    }

    #[test]
    fn inherit_keeps_existing_child_values() {
        let child = BranchConfiguration {
            increment: Some(IncrementStrategy::Patch),
            deployment_mode: Some(DeploymentMode::ManualDeployment),
            ..Default::default()
        };
        let parent = BranchConfiguration {
            increment: Some(IncrementStrategy::Major),
            deployment_mode: Some(DeploymentMode::ContinuousDeployment),
            ..Default::default()
        };

        let merged = child.inherit(&parent);
        assert_eq!(merged.increment, Some(IncrementStrategy::Patch));
        assert_eq!(
            merged.deployment_mode,
            Some(DeploymentMode::ManualDeployment)
        );
    }
}
