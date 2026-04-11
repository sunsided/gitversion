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
        self.branches
            .iter()
            .find(|(key, _)| branch.friendly().contains(key.as_str()))
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
