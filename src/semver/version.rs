use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

use crate::config::enums::SemanticVersionFormat;
use crate::regex_patterns::{parse_semver_loose, parse_semver_strict};
use crate::semver::{SemanticVersionBuildMetaData, SemanticVersionPreReleaseTag, VersionField};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IncrementMode {
    Standard,
    Force,
    EnsureIntegrity,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SemanticVersion {
    pub major: i64,
    pub minor: i64,
    pub patch: i64,
    pub pre_release_tag: SemanticVersionPreReleaseTag,
    pub build_metadata: SemanticVersionBuildMetaData,
}

impl SemanticVersion {
    pub fn new(major: i64, minor: i64, patch: i64) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release_tag: SemanticVersionPreReleaseTag::default(),
            build_metadata: SemanticVersionBuildMetaData::default(),
        }
    }

    pub fn parse(
        version: &str,
        _tag_prefix_regex: Option<&str>,
        format: SemanticVersionFormat,
    ) -> eyre::Result<Self> {
        Self::try_parse(version, None, format)
            .ok_or_else(|| eyre::eyre!("invalid semantic version: {version}"))
    }

    pub fn try_parse(
        version: &str,
        _tag_prefix_regex: Option<&str>,
        format: SemanticVersionFormat,
    ) -> Option<Self> {
        let caps = match format {
            SemanticVersionFormat::Strict => parse_semver_strict(version),
            SemanticVersionFormat::Loose => parse_semver_loose(version),
        }?;
        let major = caps.name("major")?.as_str().parse().ok()?;
        let minor = caps.name("minor")?.as_str().parse().ok()?;
        let patch = caps.name("patch")?.as_str().parse().ok()?;
        let pre = caps
            .name("pre")
            .map(|m| SemanticVersionPreReleaseTag::parse(m.as_str()))
            .unwrap_or_default();
        let meta = caps
            .name("meta")
            .map(|m| SemanticVersionBuildMetaData::parse(m.as_str()))
            .unwrap_or_default();
        Some(Self {
            major,
            minor,
            patch,
            pre_release_tag: pre,
            build_metadata: meta,
        })
    }

    pub fn compare_to(&self, other: &Self, include_pre_release: bool) -> Ordering {
        let core =
            (self.major, self.minor, self.patch).cmp(&(other.major, other.minor, other.patch));
        if core != Ordering::Equal {
            return core;
        }
        if include_pre_release {
            self.pre_release_tag.cmp(&other.pre_release_tag)
        } else {
            Ordering::Equal
        }
    }

    pub fn increment(
        &self,
        field: VersionField,
        label: Option<&str>,
        mode: IncrementMode,
        _alternatives: &[Option<&SemanticVersion>],
    ) -> Self {
        let mut next = self.clone();
        let has_pre = self.pre_release_tag.has_tag();

        match (field, mode, has_pre) {
            (VersionField::None, _, true) | (_, IncrementMode::Standard, true) => {
                let current = self.pre_release_tag.number.unwrap_or(0);
                next.pre_release_tag.number = Some(current + 1);
            }
            (VersionField::Major, _, _) => {
                next.major += 1;
                next.minor = 0;
                next.patch = 0;
                next.pre_release_tag.number = Some(1);
            }
            (VersionField::Minor, _, _) => {
                next.minor += 1;
                next.patch = 0;
                next.pre_release_tag.number = Some(1);
            }
            (VersionField::Patch, _, _) => {
                next.patch += 1;
                next.pre_release_tag.number = Some(1);
            }
            (VersionField::None, _, false) => {}
        }

        if let Some(label) = label {
            next.pre_release_tag.name = label.to_string();
            if next.pre_release_tag.number.is_none() {
                next.pre_release_tag.number = Some(1);
            }
        }
        next
    }

    pub fn with_label(mut self, label: &str) -> Self {
        self.pre_release_tag.name = label.to_string();
        self
    }

    pub fn is_pre_release(&self) -> bool {
        self.pre_release_tag.has_tag()
    }

    pub fn is_labeled_with(&self, value: &str) -> bool {
        self.pre_release_tag.name.eq_ignore_ascii_case(value)
    }

    pub fn is_match_for_branch_specific_label(&self, value: &str) -> bool {
        self.is_labeled_with(value) || self.pre_release_tag.name.starts_with(value)
    }
}

impl Display for SemanticVersion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if self.pre_release_tag.has_tag() {
            write!(f, "-{}", self.pre_release_tag)?;
        }
        let meta = self.build_metadata.to_string();
        if !meta.is_empty() {
            write!(f, "+{meta}")?;
        }
        Ok(())
    }
}

impl PartialOrd for SemanticVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare_to(other, true)
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::config::enums::SemanticVersionFormat;
    use crate::semver::{IncrementMode, SemanticVersion, VersionField};

    #[rstest]
    #[case("1.2.3", SemanticVersionFormat::Strict, 1, 2, 3, "", None)]
    #[case("1.2.3-beta", SemanticVersionFormat::Strict, 1, 2, 3, "beta", None)]
    #[case(
        "1.2.3-beta.3",
        SemanticVersionFormat::Strict,
        1,
        2,
        3,
        "beta",
        Some(3)
    )]
    #[case("v1.2.3", SemanticVersionFormat::Loose, 1, 2, 3, "", None)]
    fn parse_reads_core_and_pre_release_parts(
        #[case] input: &str,
        #[case] format: SemanticVersionFormat,
        #[case] expected_major: i64,
        #[case] expected_minor: i64,
        #[case] expected_patch: i64,
        #[case] expected_pre_name: &str,
        #[case] expected_pre_number: Option<i64>,
    ) {
        let version = SemanticVersion::parse(input, None, format).expect("version to parse");

        assert_eq!(version.major, expected_major);
        assert_eq!(version.minor, expected_minor);
        assert_eq!(version.patch, expected_patch);
        assert_eq!(version.pre_release_tag.name, expected_pre_name);
        assert_eq!(version.pre_release_tag.number, expected_pre_number);
    }

    #[rstest]
    #[case("v1.2.3")]
    #[case("1.2")]
    #[case("someText")]
    fn strict_try_parse_rejects_invalid_input(#[case] input: &str) {
        assert!(SemanticVersion::try_parse(input, None, SemanticVersionFormat::Strict).is_none());
    }

    #[test]
    fn release_version_is_greater_than_pre_release() {
        let release = SemanticVersion::parse("1.0.0", None, SemanticVersionFormat::Strict)
            .expect("valid release");
        let prerelease = SemanticVersion::parse("1.0.0-beta", None, SemanticVersionFormat::Strict)
            .expect("valid prerelease");

        assert!(release > prerelease);
    }

    #[test]
    fn pre_release_number_participates_in_ordering() {
        let beta1 = SemanticVersion::parse("1.0.0-beta.1", None, SemanticVersionFormat::Strict)
            .expect("valid prerelease");
        let beta2 = SemanticVersion::parse("1.0.0-beta.2", None, SemanticVersionFormat::Strict)
            .expect("valid prerelease");

        assert!(beta2 > beta1);
    }

    #[test]
    fn compare_to_can_ignore_pre_release() {
        let release = SemanticVersion::parse("1.0.0", None, SemanticVersionFormat::Strict)
            .expect("valid release");
        let prerelease = SemanticVersion::parse("1.0.0-beta", None, SemanticVersionFormat::Strict)
            .expect("valid prerelease");

        assert_eq!(
            release.compare_to(&prerelease, false),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn increment_standard_mode_with_existing_pre_release_increments_number_only() {
        let version = SemanticVersion::parse("1.2.3-beta.4", None, SemanticVersionFormat::Strict)
            .expect("valid version");

        let incremented =
            version.increment(VersionField::Patch, None, IncrementMode::Standard, &[]);

        assert_eq!(incremented.major, 1);
        assert_eq!(incremented.minor, 2);
        assert_eq!(incremented.patch, 3);
        assert_eq!(incremented.pre_release_tag.name, "beta");
        assert_eq!(incremented.pre_release_tag.number, Some(5));
    }

    #[test]
    fn increment_major_resets_minor_patch_and_sets_pre_release_number() {
        let version = SemanticVersion::new(1, 2, 3);
        let incremented = version.increment(VersionField::Major, None, IncrementMode::Force, &[]);

        assert_eq!(incremented.major, 2);
        assert_eq!(incremented.minor, 0);
        assert_eq!(incremented.patch, 0);
        assert_eq!(incremented.pre_release_tag.number, Some(1));
    }

    #[test]
    fn increment_sets_label_when_provided() {
        let version = SemanticVersion::new(1, 2, 3);
        let incremented = version.increment(
            VersionField::Patch,
            Some("alpha"),
            IncrementMode::Force,
            &[],
        );

        assert_eq!(incremented.pre_release_tag.name, "alpha");
        assert_eq!(incremented.pre_release_tag.number, Some(1));
    }

    #[test]
    fn display_writes_semver_with_optional_metadata() {
        let version = SemanticVersion::parse(
            "1.2.3-beta.4+7.main.abc123",
            None,
            SemanticVersionFormat::Strict,
        )
        .expect("valid semver");

        assert_eq!(version.to_string(), "1.2.3-beta.4+7.main.abc123");
    }

    #[test]
    fn label_helpers_are_case_insensitive_for_exact_match() {
        let version = SemanticVersion::parse("1.0.0-beta.1", None, SemanticVersionFormat::Strict)
            .expect("valid version");

        assert!(version.is_pre_release());
        assert!(version.is_labeled_with("BETA"));
        assert!(version.is_match_for_branch_specific_label("bet"));
    }
}
