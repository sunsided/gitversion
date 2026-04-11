use crate::config::enums::CommitMessageIncrementMode;
use crate::config::gitversion_config::GitVersionConfiguration;
use crate::regex_patterns::{BUMP_MAJOR, BUMP_MINOR, BUMP_NONE, BUMP_PATCH};
use crate::semver::VersionField;

#[derive(Debug, Default)]
pub struct IncrementStrategyFinder;

impl IncrementStrategyFinder {
    pub fn get_increment_forced_by_commit(
        &self,
        commit_message: &str,
        config: &GitVersionConfiguration,
    ) -> VersionField {
        if matches!(
            config.branch_defaults.commit_message_incrementing,
            Some(CommitMessageIncrementMode::Disabled)
        ) {
            return VersionField::None;
        }
        if BUMP_NONE.is_match(commit_message) {
            return VersionField::None;
        }
        if BUMP_MAJOR.is_match(commit_message) {
            return VersionField::Major;
        }
        if BUMP_MINOR.is_match(commit_message) {
            return VersionField::Minor;
        }
        if BUMP_PATCH.is_match(commit_message) {
            return VersionField::Patch;
        }
        VersionField::None
    }
}
