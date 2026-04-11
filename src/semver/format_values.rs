use serde::Serialize;

use crate::config::gitversion_config::GitVersionConfiguration;
use crate::semver::SemanticVersion;

#[derive(Debug, Clone, Default, Serialize)]
pub struct SemanticVersionFormatValues {
    #[serde(rename = "Major")]
    pub major: String,
    #[serde(rename = "Minor")]
    pub minor: String,
    #[serde(rename = "Patch")]
    pub patch: String,
    #[serde(rename = "PreReleaseTag")]
    pub pre_release_tag: String,
    #[serde(rename = "PreReleaseTagWithDash")]
    pub pre_release_tag_with_dash: String,
    #[serde(rename = "PreReleaseLabel")]
    pub pre_release_label: String,
    #[serde(rename = "PreReleaseLabelWithDash")]
    pub pre_release_label_with_dash: String,
    #[serde(rename = "PreReleaseNumber")]
    pub pre_release_number: String,
    #[serde(rename = "WeightedPreReleaseNumber")]
    pub weighted_pre_release_number: String,
    #[serde(rename = "BuildMetaData")]
    pub build_metadata: String,
    #[serde(rename = "FullBuildMetaData")]
    pub full_build_metadata: String,
    #[serde(rename = "MajorMinorPatch")]
    pub major_minor_patch: String,
    #[serde(rename = "SemVer")]
    pub semver: String,
    #[serde(rename = "FullSemVer")]
    pub full_semver: String,
    #[serde(rename = "InformationalVersion")]
    pub informational_version: String,
    #[serde(rename = "BranchName")]
    pub branch_name: String,
    #[serde(rename = "EscapedBranchName")]
    pub escaped_branch_name: String,
    #[serde(rename = "Sha")]
    pub sha: String,
    #[serde(rename = "ShortSha")]
    pub short_sha: String,
    #[serde(rename = "CommitDate")]
    pub commit_date: String,
    #[serde(rename = "VersionSourceDistance")]
    pub version_source_distance: String,
    #[serde(rename = "VersionSourceIncrement")]
    pub version_source_increment: String,
    #[serde(rename = "VersionSourceSemVer")]
    pub version_source_semver: String,
    #[serde(rename = "VersionSourceSha")]
    pub version_source_sha: String,
    #[serde(rename = "UncommittedChanges")]
    pub uncommitted_changes: String,
    #[serde(rename = "AssemblySemVer")]
    pub assembly_semver: String,
    #[serde(rename = "AssemblySemFileVer")]
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

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::SemanticVersionFormatValues;
    use crate::config::enums::SemanticVersionFormat;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::semver::{SemanticVersion, VersionField};

    #[test]
    fn new_populates_expected_values_from_semver_and_metadata() {
        let mut semver = SemanticVersion::new(1, 2, 3);
        semver.pre_release_tag.name = "beta".to_string();
        semver.pre_release_tag.number = Some(4);
        semver.build_metadata.commits_since_tag = Some(9);
        semver.build_metadata.branch = Some("feature/api/v2".to_string());
        semver.build_metadata.sha = Some("0123456789abcdef".to_string());
        semver.build_metadata.short_sha = Some("0123456".to_string());
        semver.build_metadata.commit_date = Some(
            Utc.with_ymd_and_hms(2025, 1, 2, 3, 4, 5)
                .single()
                .expect("valid commit date"),
        );
        semver.build_metadata.version_source_distance = 11;
        semver.build_metadata.version_source_increment = VersionField::Minor;
        semver.build_metadata.version_source_semver = Some(Box::new(SemanticVersion::new(1, 2, 0)));
        semver.build_metadata.version_source_sha = Some("abcdef0123456789".to_string());
        semver.build_metadata.uncommitted_changes = 2;

        let values =
            SemanticVersionFormatValues::new(&semver, &GitVersionConfiguration::default(), 60000);

        assert_eq!(values.major, "1");
        assert_eq!(values.minor, "2");
        assert_eq!(values.patch, "3");
        assert_eq!(values.pre_release_tag, "beta.4");
        assert_eq!(values.pre_release_tag_with_dash, "-beta.4");
        assert_eq!(values.pre_release_label, "beta");
        assert_eq!(values.pre_release_label_with_dash, "-beta");
        assert_eq!(values.pre_release_number, "4");
        assert_eq!(values.weighted_pre_release_number, "60004");
        assert_eq!(values.build_metadata, "9.feature/api/v2.0123456789abcdef");
        assert_eq!(
            values.full_build_metadata,
            "9.feature/api/v2.0123456789abcdef"
        );
        assert_eq!(values.major_minor_patch, "1.2.3");
        assert_eq!(
            values.semver,
            "1.2.3-beta.4+9.feature/api/v2.0123456789abcdef"
        );
        assert_eq!(
            values.full_semver,
            "1.2.3-beta.4+9.feature/api/v2.0123456789abcdef"
        );
        assert_eq!(
            values.informational_version,
            "1.2.3-beta.4+9.feature/api/v2.0123456789abcdef"
        );
        assert_eq!(values.branch_name, "feature/api/v2");
        assert_eq!(values.escaped_branch_name, "feature-api-v2");
        assert_eq!(values.sha, "0123456789abcdef");
        assert_eq!(values.short_sha, "0123456");
        assert_eq!(values.commit_date, "2025-01-02");
        assert_eq!(values.version_source_distance, "11");
        assert_eq!(values.version_source_increment, "Minor");
        assert_eq!(values.version_source_semver, "1.2.0");
        assert_eq!(values.version_source_sha, "abcdef0123456789");
        assert_eq!(values.uncommitted_changes, "2");
        assert_eq!(values.assembly_semver, "1.2.3");
        assert_eq!(values.assembly_file_semver, "1.2.3.4");
    }

    #[test]
    fn new_uses_defaults_when_no_pre_release_or_metadata_present() {
        let semver = SemanticVersion::new(4, 5, 6);

        let values =
            SemanticVersionFormatValues::new(&semver, &GitVersionConfiguration::default(), 7);

        assert_eq!(values.pre_release_tag, "");
        assert_eq!(values.pre_release_tag_with_dash, "");
        assert_eq!(values.pre_release_label, "");
        assert_eq!(values.pre_release_label_with_dash, "");
        assert_eq!(values.pre_release_number, "0");
        assert_eq!(values.weighted_pre_release_number, "7");
        assert_eq!(values.build_metadata, "");
        assert_eq!(values.full_build_metadata, "");
        assert_eq!(values.branch_name, "");
        assert_eq!(values.escaped_branch_name, "");
        assert_eq!(values.sha, "");
        assert_eq!(values.short_sha, "");
        assert_eq!(values.commit_date, "1970-01-01");
        assert_eq!(values.version_source_distance, "0");
        assert_eq!(values.version_source_increment, "None");
        assert_eq!(values.version_source_semver, "");
        assert_eq!(values.version_source_sha, "");
        assert_eq!(values.uncommitted_changes, "0");
        assert_eq!(values.assembly_semver, "4.5.6");
        assert_eq!(values.assembly_file_semver, "4.5.6.0");
    }

    #[test]
    fn new_formats_commit_date_using_configuration_pattern() {
        let mut semver = SemanticVersion::new(1, 0, 0);
        semver.build_metadata.commit_date = Some(
            Utc.with_ymd_and_hms(2026, 4, 11, 9, 8, 7)
                .single()
                .expect("valid commit date"),
        );

        let mut config = GitVersionConfiguration::default();
        config.commit_date_format = "%d/%m/%Y %H:%M".to_string();

        let values = SemanticVersionFormatValues::new(&semver, &config, 0);
        assert_eq!(values.commit_date, "11/04/2026 09:08");
    }

    #[test]
    fn new_handles_numeric_pre_release_tag_without_label() {
        let semver = SemanticVersion::parse("1.0.0-7", None, SemanticVersionFormat::Strict)
            .expect("valid numeric pre-release");

        let values =
            SemanticVersionFormatValues::new(&semver, &GitVersionConfiguration::default(), 10);

        assert_eq!(values.pre_release_tag, "7");
        assert_eq!(values.pre_release_tag_with_dash, "-7");
        assert_eq!(values.pre_release_label, "");
        assert_eq!(values.pre_release_label_with_dash, "");
        assert_eq!(values.pre_release_number, "7");
        assert_eq!(values.weighted_pre_release_number, "17");
        assert_eq!(values.assembly_file_semver, "1.0.0.7");
    }

    #[test]
    fn new_replaces_all_forward_slashes_when_escaping_branch_name() {
        let mut semver = SemanticVersion::new(2, 1, 0);
        semver.build_metadata.branch = Some("feature/a/b/c".to_string());

        let values =
            SemanticVersionFormatValues::new(&semver, &GitVersionConfiguration::default(), 0);

        assert_eq!(values.branch_name, "feature/a/b/c");
        assert_eq!(values.escaped_branch_name, "feature-a-b-c");
    }

    #[test]
    fn new_preserves_branch_name_without_slashes_when_escaping() {
        let mut semver = SemanticVersion::new(2, 1, 0);
        semver.build_metadata.branch = Some("release-2026.q2".to_string());

        let values =
            SemanticVersionFormatValues::new(&semver, &GitVersionConfiguration::default(), 0);

        assert_eq!(values.branch_name, "release-2026.q2");
        assert_eq!(values.escaped_branch_name, "release-2026.q2");
    }

    #[test]
    fn new_allows_negative_pre_release_weight() {
        let mut semver = SemanticVersion::new(3, 2, 1);
        semver.pre_release_tag.name = "alpha".to_string();
        semver.pre_release_tag.number = Some(2);

        let values =
            SemanticVersionFormatValues::new(&semver, &GitVersionConfiguration::default(), -5);

        assert_eq!(values.pre_release_number, "2");
        assert_eq!(values.weighted_pre_release_number, "-3");
    }

    #[test]
    fn new_stringifies_version_source_increment_variant_name() {
        let mut semver = SemanticVersion::new(1, 0, 0);
        semver.build_metadata.version_source_increment = VersionField::Major;

        let values =
            SemanticVersionFormatValues::new(&semver, &GitVersionConfiguration::default(), 0);

        assert_eq!(values.version_source_increment, "Major");
    }
}
