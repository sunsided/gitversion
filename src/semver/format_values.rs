use serde::Serialize;

use crate::config::gitversion_config::GitVersionConfiguration;
use crate::semver::SemanticVersion;

#[derive(Debug, Clone, Default, Serialize)]
pub struct SemanticVersionFormatValues {
    pub major: String,
    pub minor: String,
    pub patch: String,
    pub pre_release_tag: String,
    pub pre_release_tag_with_dash: String,
    pub pre_release_label: String,
    pub pre_release_label_with_dash: String,
    pub pre_release_number: String,
    pub weighted_pre_release_number: String,
    pub build_metadata: String,
    pub full_build_metadata: String,
    pub major_minor_patch: String,
    pub semver: String,
    pub full_semver: String,
    pub informational_version: String,
    pub branch_name: String,
    pub escaped_branch_name: String,
    pub sha: String,
    pub short_sha: String,
    pub commit_date: String,
    pub version_source_distance: String,
    pub version_source_increment: String,
    pub version_source_semver: String,
    pub version_source_sha: String,
    pub uncommitted_changes: String,
    pub assembly_semver: String,
    pub assembly_file_semver: String,
}

impl SemanticVersionFormatValues {
    pub fn new(
        semver: &SemanticVersion,
        config: &GitVersionConfiguration,
        pre_release_weight: i64,
    ) -> Self {
        let pre_release_tag = semver.pre_release_tag.to_string();
        let pre_release_number = semver.pre_release_tag.number.unwrap_or(0);
        let build = semver.build_metadata.to_string();
        let commit_date = semver
            .build_metadata
            .commit_date
            .map(|d| d.format(&config.commit_date_format).to_string())
            .unwrap_or_else(|| format!("{:04}-{:02}-{:02}", 1970, 1, 1));
        Self {
            major: semver.major.to_string(),
            minor: semver.minor.to_string(),
            patch: semver.patch.to_string(),
            pre_release_tag_with_dash: if pre_release_tag.is_empty() {
                String::new()
            } else {
                format!("-{pre_release_tag}")
            },
            pre_release_label_with_dash: if semver.pre_release_tag.name.is_empty() {
                String::new()
            } else {
                format!("-{}", semver.pre_release_tag.name)
            },
            pre_release_label: semver.pre_release_tag.name.clone(),
            pre_release_tag,
            pre_release_number: pre_release_number.to_string(),
            weighted_pre_release_number: (pre_release_number + pre_release_weight).to_string(),
            build_metadata: build.clone(),
            full_build_metadata: build.clone(),
            major_minor_patch: format!("{}.{}.{}", semver.major, semver.minor, semver.patch),
            semver: semver.to_string(),
            full_semver: semver.to_string(),
            informational_version: semver.to_string(),
            branch_name: semver.build_metadata.branch.clone().unwrap_or_default(),
            escaped_branch_name: semver
                .build_metadata
                .branch
                .clone()
                .unwrap_or_default()
                .replace('/', "-"),
            sha: semver.build_metadata.sha.clone().unwrap_or_default(),
            short_sha: semver.build_metadata.short_sha.clone().unwrap_or_default(),
            commit_date,
            version_source_distance: semver.build_metadata.version_source_distance.to_string(),
            version_source_increment: format!(
                "{:?}",
                semver.build_metadata.version_source_increment
            ),
            version_source_semver: semver
                .build_metadata
                .version_source_semver
                .as_ref()
                .map(|v| v.to_string())
                .unwrap_or_default(),
            version_source_sha: semver
                .build_metadata
                .version_source_sha
                .clone()
                .unwrap_or_default(),
            uncommitted_changes: semver.build_metadata.uncommitted_changes.to_string(),
            assembly_semver: format!("{}.{}.{}", semver.major, semver.minor, semver.patch),
            assembly_file_semver: format!(
                "{}.{}.{}.{}",
                semver.major, semver.minor, semver.patch, pre_release_number
            ),
        }
    }
}
