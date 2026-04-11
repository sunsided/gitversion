use crate::config::gitversion_config::GitVersionConfiguration;
use crate::git::reference_name::ReferenceName;
use crate::regex_patterns::{MERGE_MESSAGE_PATTERNS, VERSION_IN_BRANCH};
use crate::semver::SemanticVersion;

#[derive(Debug, Clone)]
pub struct MergeMessage {
    pub format_name: String,
    pub target_branch: String,
    pub merged_branch: Option<ReferenceName>,
    pub pull_request_number: Option<i32>,
    pub version: Option<SemanticVersion>,
}

impl MergeMessage {
    pub fn try_parse(message: &str, config: &GitVersionConfiguration) -> Option<Self> {
        for (name, re) in MERGE_MESSAGE_PATTERNS.iter() {
            if let Some(caps) = re.captures(message) {
                let source = caps.name("source").map(|m| m.as_str().to_string());
                let target = caps
                    .name("target")
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_default();
                let pr = caps.get(1).and_then(|m| m.as_str().parse::<i32>().ok());
                let version = source
                    .as_deref()
                    .and_then(|s| VERSION_IN_BRANCH.captures(s))
                    .and_then(|m| m.name("version").map(|v| v.as_str().to_string()))
                    .and_then(|v| {
                        SemanticVersion::try_parse(
                            &v,
                            Some(&config.tag_prefix_pattern),
                            config.semantic_version_format,
                        )
                    });

                return Some(Self {
                    format_name: (*name).to_string(),
                    target_branch: target,
                    merged_branch: source.as_deref().map(ReferenceName::from_branch_name),
                    pull_request_number: pr,
                    version,
                });
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::git::merge_message::MergeMessage;

    #[rstest]
    #[case(
        "Merge branch 'feature/foo' into main",
        "Default",
        Some("refs/heads/feature/foo"),
        "main",
        None
    )]
    #[case(
        "Merge pull request #123 from feature/bar",
        "GitHubPull",
        Some("refs/heads/feature/bar"),
        "",
        Some(123)
    )]
    #[case(
        "Merged PR 456: Merge feature/baz to develop",
        "AzureDevOps",
        Some("refs/heads/feature/baz"),
        "develop",
        Some(456)
    )]
    fn try_parse_matches_known_merge_patterns(
        #[case] message: &str,
        #[case] expected_format: &str,
        #[case] expected_source: Option<&str>,
        #[case] expected_target: &str,
        #[case] expected_pr: Option<i32>,
    ) {
        let config = GitVersionConfiguration::default();
        let parsed = MergeMessage::try_parse(message, &config).expect("message should parse");

        assert_eq!(parsed.format_name, expected_format);
        assert_eq!(parsed.target_branch, expected_target);
        assert_eq!(
            parsed.merged_branch.as_ref().map(|v| v.canonical.as_str()),
            expected_source
        );
        assert_eq!(parsed.pull_request_number, expected_pr);
    }

    #[test]
    fn try_parse_extracts_version_from_source_branch() {
        let config = GitVersionConfiguration::default();
        let parsed = MergeMessage::try_parse("Merge pull request #9 from release/1.2.3", &config)
            .expect("message should parse");

        let version = parsed.version.expect("version should be extracted");
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
    }

    #[test]
    fn try_parse_returns_none_for_non_merge_message() {
        let config = GitVersionConfiguration::default();
        assert!(MergeMessage::try_parse("feat: add login", &config).is_none());
    }
}
