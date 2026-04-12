use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::config::branch_config::BranchConfiguration;
use crate::config::enums::{
    AssemblyFileVersioningScheme, AssemblyVersioningScheme, CommitMessageIncrementMode,
    DeploymentMode, IncrementStrategy, SemanticVersionFormat, VersionStrategies,
};
use crate::config::ignore_config::IgnoreConfiguration;
use crate::git::reference_name::ReferenceName;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitVersionConfiguration {
    pub workflow: String,
    pub assembly_versioning_scheme: AssemblyVersioningScheme,
    pub assembly_file_versioning_scheme: AssemblyFileVersioningScheme,
    pub assembly_informational_format: String,
    pub assembly_versioning_format: String,
    pub assembly_file_versioning_format: String,
    pub tag_prefix_pattern: String,
    pub version_in_branch_pattern: String,
    pub next_version: Option<String>,
    pub major_version_bump_message: String,
    pub minor_version_bump_message: String,
    pub patch_version_bump_message: String,
    pub no_bump_message: String,
    pub commit_date_format: String,
    pub update_build_number: bool,
    pub semantic_version_format: SemanticVersionFormat,
    pub version_strategy: VersionStrategies,
    pub merge_message_formats: HashMap<String, String>,
    pub tag_pre_release_weight: i64,
    pub ignore: IgnoreConfiguration,
    pub branch_defaults: BranchConfiguration,
    pub branches: HashMap<String, BranchConfiguration>,
}

impl Default for GitVersionConfiguration {
    fn default() -> Self {
        Self {
            workflow: "GitFlow/v1".to_string(),
            assembly_versioning_scheme: AssemblyVersioningScheme::MajorMinorPatchTag,
            assembly_file_versioning_scheme: AssemblyFileVersioningScheme::MajorMinorPatchTag,
            assembly_informational_format: "{InformationalVersion}".to_string(),
            assembly_versioning_format: "{Major}.{Minor}.{Patch}".to_string(),
            assembly_file_versioning_format: "{Major}.{Minor}.{Patch}.{WeightedPreReleaseNumber}"
                .to_string(),
            tag_prefix_pattern: "[vV]?".to_string(),
            version_in_branch_pattern: "(?<version>\\d+\\.\\d+\\.\\d+)".to_string(),
            next_version: None,
            major_version_bump_message: r"\+semver:\s?(breaking|major)".to_string(),
            minor_version_bump_message: r"\+semver:\s?(feature|minor)".to_string(),
            patch_version_bump_message: r"\+semver:\s?(fix|patch)".to_string(),
            no_bump_message: r"\+semver:\s?(none|skip)".to_string(),
            commit_date_format: "%Y-%m-%d".to_string(),
            update_build_number: true,
            semantic_version_format: SemanticVersionFormat::Strict,
            version_strategy: VersionStrategies::default(),
            merge_message_formats: HashMap::new(),
            tag_pre_release_weight: 60000,
            ignore: IgnoreConfiguration::default(),
            branch_defaults: BranchConfiguration {
                deployment_mode: Some(DeploymentMode::ManualDeployment),
                label: Some(String::new()),
                increment: Some(IncrementStrategy::Patch),
                prevent_increment: None,
                track_merge_target: Some(false),
                track_merge_message: Some(true),
                commit_message_incrementing: Some(CommitMessageIncrementMode::Enabled),
                regular_expression: None,
                source_branches: Some(Vec::new()),
                is_source_branch_for: Some(Vec::new()),
                tracks_release_branches: Some(false),
                is_release_branch: Some(false),
                is_main_branch: Some(false),
                pre_release_weight: Some(0),
            },
            branches: HashMap::new(),
        }
    }
}

impl GitVersionConfiguration {
    pub fn get_branch_configuration(&self, branch: &ReferenceName) -> BranchConfiguration {
        let friendly = branch.friendly();
        let without_origin = branch.without_origin();
        self.branches
            .iter()
            .find(|(key, _)| {
                branch_key_matches(&friendly, key.as_str())
                    || branch_key_matches(&without_origin, key.as_str())
            })
            .map(|(_, cfg)| cfg.clone().inherit(&self.branch_defaults))
            .unwrap_or_else(|| self.get_fallback_branch_configuration())
    }

    pub fn get_fallback_branch_configuration(&self) -> BranchConfiguration {
        self.branches
            .get("unknown")
            .cloned()
            .unwrap_or_default()
            .inherit(&self.branch_defaults)
    }
}

fn branch_key_matches(branch_name: &str, key: &str) -> bool {
    branch_name.match_indices(key).any(|(start, _)| {
        let end = start + key.len();
        let left_ok = start == 0 || is_boundary(branch_name.as_bytes()[start - 1] as char);
        let right_ok = end == branch_name.len() || is_boundary(branch_name.as_bytes()[end] as char);
        left_ok && right_ok
    })
}

fn is_boundary(ch: char) -> bool {
    matches!(ch, '/' | '-' | '_' | '.')
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::GitVersionConfiguration;
    use crate::config::branch_config::BranchConfiguration;
    use crate::config::enums::{DeploymentMode, IncrementStrategy};
    use crate::git::reference_name::ReferenceName;

    #[test]
    fn default_uses_expected_workflow_and_versioning_defaults() {
        let config = GitVersionConfiguration::default();

        assert_eq!(config.workflow, "GitFlow/v1");
        assert_eq!(config.tag_prefix_pattern, "[vV]?");
        assert_eq!(config.commit_date_format, "%Y-%m-%d");
        assert!(config.update_build_number);
    }

    #[test]
    fn get_branch_configuration_inherits_branch_defaults_for_matching_branch() {
        let mut config = GitVersionConfiguration::default();
        config.branch_defaults = BranchConfiguration {
            deployment_mode: Some(DeploymentMode::ContinuousDelivery),
            track_merge_message: Some(false),
            pre_release_weight: Some(42),
            ..Default::default()
        };
        config.branches.insert(
            "feature".to_string(),
            BranchConfiguration {
                increment: Some(IncrementStrategy::Minor),
                label: Some("alpha".to_string()),
                ..Default::default()
            },
        );

        let branch = ReferenceName::from_branch_name("feature/add-tests");
        let resolved = config.get_branch_configuration(&branch);

        assert_eq!(resolved.increment, Some(IncrementStrategy::Minor));
        assert_eq!(resolved.label.as_deref(), Some("alpha"));
        assert_eq!(
            resolved.deployment_mode,
            Some(DeploymentMode::ContinuousDelivery)
        );
        assert_eq!(resolved.track_merge_message, Some(false));
        assert_eq!(resolved.pre_release_weight, Some(42));
    }

    #[test]
    fn get_branch_configuration_uses_fallback_when_no_branch_matches() {
        let mut config = GitVersionConfiguration::default();
        config.branch_defaults = BranchConfiguration {
            track_merge_target: Some(true),
            ..Default::default()
        };
        config.branches.insert(
            "unknown".to_string(),
            BranchConfiguration {
                increment: Some(IncrementStrategy::Patch),
                label: Some("ci".to_string()),
                ..Default::default()
            },
        );

        let branch = ReferenceName::from_branch_name("bugfix/issue-123");
        let resolved = config.get_branch_configuration(&branch);

        assert_eq!(resolved.increment, Some(IncrementStrategy::Patch));
        assert_eq!(resolved.label.as_deref(), Some("ci"));
        assert_eq!(resolved.track_merge_target, Some(true));
    }

    #[test]
    fn get_fallback_branch_configuration_uses_unknown_branch_when_present() {
        let mut config = GitVersionConfiguration {
            branch_defaults: BranchConfiguration {
                deployment_mode: Some(DeploymentMode::ContinuousDelivery),
                ..Default::default()
            },
            ..Default::default()
        };
        config.branches = HashMap::from([(
            "unknown".to_string(),
            BranchConfiguration {
                increment: Some(IncrementStrategy::Major),
                ..Default::default()
            },
        )]);

        let fallback = config.get_fallback_branch_configuration();

        assert_eq!(fallback.increment, Some(IncrementStrategy::Major));
        assert_eq!(
            fallback.deployment_mode,
            Some(DeploymentMode::ContinuousDelivery)
        );
    }

    #[test]
    fn get_fallback_branch_configuration_uses_defaults_when_unknown_missing() {
        let config = GitVersionConfiguration {
            branch_defaults: BranchConfiguration {
                increment: Some(IncrementStrategy::Minor),
                is_main_branch: Some(true),
                ..Default::default()
            },
            ..Default::default()
        };

        let fallback = config.get_fallback_branch_configuration();

        assert_eq!(fallback.increment, Some(IncrementStrategy::Minor));
        assert_eq!(fallback.is_main_branch, Some(true));
    }

    #[test]
    fn get_branch_configuration_matches_remote_reference_names() {
        let mut config = GitVersionConfiguration::default();
        config.branches.insert(
            "feature".to_string(),
            BranchConfiguration {
                increment: Some(IncrementStrategy::Minor),
                ..Default::default()
            },
        );

        let remote_branch = ReferenceName::parse("refs/remotes/origin/feature/xyz");
        let resolved = config.get_branch_configuration(&remote_branch);

        assert_eq!(resolved.increment, Some(IncrementStrategy::Minor));
    }

    #[test]
    fn get_branch_configuration_returns_inherited_unknown_when_no_entries_exist() {
        let config = GitVersionConfiguration {
            branch_defaults: BranchConfiguration {
                label: Some("default-label".to_string()),
                track_merge_message: Some(false),
                ..Default::default()
            },
            ..Default::default()
        };

        let branch = ReferenceName::from_branch_name("main");
        let resolved = config.get_branch_configuration(&branch);

        assert_eq!(resolved.label.as_deref(), Some("default-label"));
        assert_eq!(resolved.track_merge_message, Some(false));
    }

    #[test]
    fn get_branch_configuration_does_not_treat_developer_as_develop() {
        let mut config = GitVersionConfiguration::default();
        config.branches.insert(
            "develop".to_string(),
            BranchConfiguration {
                label: Some("alpha".to_string()),
                ..Default::default()
            },
        );
        config.branches.insert(
            "unknown".to_string(),
            BranchConfiguration {
                label: Some("fallback".to_string()),
                ..Default::default()
            },
        );

        let branch = ReferenceName::from_branch_name("developer");
        let resolved = config.get_branch_configuration(&branch);

        assert_eq!(resolved.label.as_deref(), Some("fallback"));
    }

    #[test]
    fn default_branch_defaults_are_populated_for_inheritance() {
        let config = GitVersionConfiguration::default();

        assert_eq!(
            config.branch_defaults.deployment_mode,
            Some(DeploymentMode::ManualDeployment)
        );
        assert_eq!(
            config.branch_defaults.increment,
            Some(IncrementStrategy::Patch)
        );
        assert_eq!(config.branch_defaults.track_merge_target, Some(false));
        assert_eq!(config.branch_defaults.track_merge_message, Some(true));
        assert_eq!(config.branch_defaults.pre_release_weight, Some(0));
    }
}
