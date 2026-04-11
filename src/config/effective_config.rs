use crate::config::branch_config::BranchConfiguration;
use crate::config::enums::{CommitMessageIncrementMode, DeploymentMode, IncrementStrategy};
use crate::config::gitversion_config::GitVersionConfiguration;

#[derive(Debug, Clone)]
pub struct EffectiveConfiguration {
    pub deployment_mode: DeploymentMode,
    pub label: String,
    pub increment: IncrementStrategy,
    pub track_merge_target: bool,
    pub track_merge_message: bool,
    pub commit_message_incrementing: CommitMessageIncrementMode,
    pub tracks_release_branches: bool,
    pub is_release_branch: bool,
    pub is_main_branch: bool,
    pub pre_release_weight: i64,
}

impl EffectiveConfiguration {
    pub fn from(config: &GitVersionConfiguration, branch: &BranchConfiguration) -> Self {
        let resolved = branch.inherit(&config.branch_defaults);
        Self {
            deployment_mode: resolved.deployment_mode.unwrap_or_default(),
            label: resolved.label.unwrap_or_default(),
            increment: resolved.increment.unwrap_or_default(),
            track_merge_target: resolved.track_merge_target.unwrap_or(false),
            track_merge_message: resolved.track_merge_message.unwrap_or(true),
            commit_message_incrementing: resolved.commit_message_incrementing.unwrap_or_default(),
            tracks_release_branches: resolved.tracks_release_branches.unwrap_or(false),
            is_release_branch: resolved.is_release_branch.unwrap_or(false),
            is_main_branch: resolved.is_main_branch.unwrap_or(false),
            pre_release_weight: resolved.pre_release_weight.unwrap_or(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::config::branch_config::BranchConfiguration;
    use crate::config::effective_config::EffectiveConfiguration;
    use crate::config::enums::{CommitMessageIncrementMode, DeploymentMode, IncrementStrategy};
    use crate::config::gitversion_config::GitVersionConfiguration;

    #[test]
    fn from_uses_branch_values_when_present() {
        let config = GitVersionConfiguration::default();
        let branch = BranchConfiguration {
            deployment_mode: Some(DeploymentMode::ContinuousDeployment),
            label: Some("alpha".to_string()),
            increment: Some(IncrementStrategy::Major),
            track_merge_target: Some(true),
            track_merge_message: Some(false),
            commit_message_incrementing: Some(CommitMessageIncrementMode::MergeMessageOnly),
            tracks_release_branches: Some(true),
            is_release_branch: Some(true),
            is_main_branch: Some(true),
            pre_release_weight: Some(12345),
            ..Default::default()
        };

        let effective = EffectiveConfiguration::from(&config, &branch);

        assert_eq!(
            effective.deployment_mode,
            DeploymentMode::ContinuousDeployment
        );
        assert_eq!(effective.label, "alpha");
        assert_eq!(effective.increment, IncrementStrategy::Major);
        assert!(effective.track_merge_target);
        assert!(!effective.track_merge_message);
        assert_eq!(
            effective.commit_message_incrementing,
            CommitMessageIncrementMode::MergeMessageOnly
        );
        assert!(effective.tracks_release_branches);
        assert!(effective.is_release_branch);
        assert!(effective.is_main_branch);
        assert_eq!(effective.pre_release_weight, 12345);
    }

    #[test]
    fn from_falls_back_to_branch_defaults_for_missing_values() {
        let mut config = GitVersionConfiguration::default();
        config.branch_defaults = BranchConfiguration {
            deployment_mode: Some(DeploymentMode::ContinuousDelivery),
            label: Some("beta".to_string()),
            increment: Some(IncrementStrategy::Minor),
            track_merge_target: Some(true),
            track_merge_message: Some(false),
            commit_message_incrementing: Some(CommitMessageIncrementMode::Disabled),
            tracks_release_branches: Some(true),
            is_release_branch: Some(true),
            is_main_branch: Some(true),
            pre_release_weight: Some(99),
            ..Default::default()
        };

        let effective = EffectiveConfiguration::from(&config, &BranchConfiguration::default());

        assert_eq!(
            effective.deployment_mode,
            DeploymentMode::ContinuousDelivery
        );
        assert_eq!(effective.label, "beta");
        assert_eq!(effective.increment, IncrementStrategy::Minor);
        assert!(effective.track_merge_target);
        assert!(!effective.track_merge_message);
        assert_eq!(
            effective.commit_message_incrementing,
            CommitMessageIncrementMode::Disabled
        );
        assert!(effective.tracks_release_branches);
        assert!(effective.is_release_branch);
        assert!(effective.is_main_branch);
        assert_eq!(effective.pre_release_weight, 99);
    }
}
