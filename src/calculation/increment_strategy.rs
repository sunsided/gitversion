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

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::IncrementStrategyFinder;
    use crate::config::enums::CommitMessageIncrementMode;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::semver::VersionField;

    #[test]
    fn returns_none_when_commit_message_incrementing_is_disabled() {
        let mut config = GitVersionConfiguration::default();
        config.branch_defaults.commit_message_incrementing =
            Some(CommitMessageIncrementMode::Disabled);

        let increment =
            IncrementStrategyFinder.get_increment_forced_by_commit("+semver: major", &config);

        assert_eq!(increment, VersionField::None);
    }

    #[rstest]
    #[case("feat: new api", VersionField::None)]
    #[case("chore: update deps\n\n+semver: major", VersionField::Major)]
    #[case("fix: typo +semver: feature", VersionField::Minor)]
    #[case("fix: bug +semver: patch", VersionField::Patch)]
    #[case("docs: skip +semver: none", VersionField::None)]
    fn detects_increment_from_commit_message(
        #[case] message: &str,
        #[case] expected: VersionField,
    ) {
        let increment = IncrementStrategyFinder
            .get_increment_forced_by_commit(message, &GitVersionConfiguration::default());

        assert_eq!(increment, expected);
    }

    #[test]
    fn no_bump_marker_takes_precedence_over_other_markers() {
        let message = "+semver: major\n+semver: none";

        let increment = IncrementStrategyFinder
            .get_increment_forced_by_commit(message, &GitVersionConfiguration::default());

        assert_eq!(increment, VersionField::None);
    }

    #[test]
    fn major_marker_takes_precedence_over_minor_and_patch() {
        let message = "+semver: patch\n+semver: feature\n+semver: major";

        let increment = IncrementStrategyFinder
            .get_increment_forced_by_commit(message, &GitVersionConfiguration::default());

        assert_eq!(increment, VersionField::Major);
    }

    #[test]
    fn marker_matching_is_case_sensitive() {
        let message = "+SEMVER: MAJOR";

        let increment = IncrementStrategyFinder
            .get_increment_forced_by_commit(message, &GitVersionConfiguration::default());

        assert_eq!(increment, VersionField::None);
    }
}
