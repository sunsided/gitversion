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
