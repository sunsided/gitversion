use std::collections::HashMap;
use std::sync::RwLock;

use once_cell::sync::Lazy;
use regex::{Captures, Regex};

pub static SEMVER_STRICT: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?P<major>0|[1-9]\d*)\.(?P<minor>0|[1-9]\d*)\.(?P<patch>0|[1-9]\d*)(?:-(?P<pre>[0-9A-Za-z\-.]+))?(?:\+(?P<meta>[0-9A-Za-z\-.]+))?$")
        .expect("valid semver strict regex")
});

pub static SEMVER_LOOSE: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^(?:v|V)?(?P<major>\d+)\.(?P<minor>\d+)\.(?P<patch>\d+)(?:-(?P<pre>[0-9A-Za-z\-.]+))?(?:\+(?P<meta>[0-9A-Za-z\-.]+))?$")
        .expect("valid semver loose regex")
});

pub static VERSION_IN_BRANCH: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?P<version>\d+\.\d+\.\d+(?:-[0-9A-Za-z\-.]+)?)")
        .expect("valid branch version regex")
});

pub static BUMP_MAJOR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\+semver:\s?(breaking|major)").expect("valid regex"));
pub static BUMP_MINOR: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\+semver:\s?(feature|minor)").expect("valid regex"));
pub static BUMP_PATCH: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\+semver:\s?(fix|patch)").expect("valid regex"));
pub static BUMP_NONE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\+semver:\s?(none|skip)").expect("valid regex"));

pub static TOKEN_EXPANSION: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\{(?P<token>[^{}]+)\}").expect("valid token regex"));

pub static MERGE_MESSAGE_PATTERNS: Lazy<Vec<(&'static str, Regex)>> = Lazy::new(|| {
    vec![
        (
            "Default",
            Regex::new(r"^Merge branch '(?P<source>[^']+)' into (?P<target>\S+)").expect("regex"),
        ),
        (
            "GitHubPull",
            Regex::new(r"^Merge pull request #(\d+) from (?P<source>\S+)").expect("regex"),
        ),
        (
            "AzureDevOps",
            Regex::new(r"^Merged PR (\d+): Merge (?P<source>.+) to (?P<target>.+)").expect("regex"),
        ),
        (
            "BitBucketPull",
            Regex::new(r"^Merge pull request #(\d+) in .+ from (?P<source>.+) to (?P<target>.+)")
                .expect("regex"),
        ),
        (
            "BitBucketCloud",
            Regex::new(r"^Merged in (?P<source>.+) \(pull request #(\d+)\)").expect("regex"),
        ),
        (
            "SmartGit",
            Regex::new(r"^Finish (?P<source>\S+)").expect("regex"),
        ),
        (
            "RemoteTracking",
            Regex::new(r"^Merge remote-tracking branch '(?P<source>[^']+)'").expect("regex"),
        ),
        (
            "BitBucketPullv7",
            Regex::new(r"^Pull request #(\d+): (?P<source>.+)").expect("regex"),
        ),
    ]
});

static DYNAMIC_REGEX: Lazy<RwLock<HashMap<String, Regex>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub fn parse_semver_strict(input: &str) -> Option<Captures<'_>> {
    SEMVER_STRICT.captures(input)
}

pub fn parse_semver_loose(input: &str) -> Option<Captures<'_>> {
    SEMVER_LOOSE.captures(input)
}

pub fn tag_prefix_regex(pattern: &str) -> Regex {
    if let Some(found) = DYNAMIC_REGEX.read().expect("rwlock").get(pattern) {
        return found.clone();
    }
    let compiled = Regex::new(pattern).unwrap_or_else(|_| Regex::new(r"[vV]?").expect("regex"));
    DYNAMIC_REGEX
        .write()
        .expect("rwlock")
        .insert(pattern.to_string(), compiled.clone());
    compiled
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::{
        tag_prefix_regex, BUMP_MAJOR, BUMP_MINOR, BUMP_NONE, BUMP_PATCH, MERGE_MESSAGE_PATTERNS,
        SEMVER_LOOSE, SEMVER_STRICT, VERSION_IN_BRANCH,
    };

    #[rstest]
    #[case("+semver: major", "major")]
    #[case("+semver: breaking", "major")]
    #[case("+semver: minor", "minor")]
    #[case("+semver: feature", "minor")]
    #[case("+semver: patch", "patch")]
    #[case("+semver: fix", "patch")]
    #[case("+semver: none", "none")]
    #[case("+semver: skip", "none")]
    fn bump_patterns_detect_expected_bump_type(#[case] message: &str, #[case] expected: &str) {
        let detected = if BUMP_MAJOR.is_match(message) {
            "major"
        } else if BUMP_MINOR.is_match(message) {
            "minor"
        } else if BUMP_PATCH.is_match(message) {
            "patch"
        } else if BUMP_NONE.is_match(message) {
            "none"
        } else {
            "unknown"
        };

        assert_eq!(detected, expected);
    }

    #[test]
    fn semver_strict_accepts_valid_versions() {
        assert!(SEMVER_STRICT.is_match("1.2.3"));
        assert!(SEMVER_STRICT.is_match("1.2.3-beta.4"));
        assert!(SEMVER_STRICT.is_match("1.2.3+build"));
    }

    #[test]
    fn semver_strict_rejects_invalid_versions() {
        assert!(!SEMVER_STRICT.is_match("v1.2.3"));
        assert!(!SEMVER_STRICT.is_match("1.2"));
    }

    #[test]
    fn semver_loose_accepts_v_prefix() {
        assert!(SEMVER_LOOSE.is_match("v1.2.3"));
        assert!(SEMVER_LOOSE.is_match("V1.2.3"));
    }

    #[test]
    fn version_in_branch_finds_semver_like_substring() {
        let captures = VERSION_IN_BRANCH
            .captures("release/2.3.4-beta")
            .expect("version should be captured");
        assert_eq!(
            captures.name("version").map(|m| m.as_str()),
            Some("2.3.4-beta")
        );
    }

    #[test]
    fn merge_message_patterns_include_common_formats() {
        let names: Vec<&str> = MERGE_MESSAGE_PATTERNS
            .iter()
            .map(|(name, _)| *name)
            .collect();
        assert!(names.contains(&"Default"));
        assert!(names.contains(&"GitHubPull"));
        assert!(names.contains(&"AzureDevOps"));
    }

    #[test]
    fn tag_prefix_regex_uses_fallback_for_invalid_pattern() {
        let regex = tag_prefix_regex("(");
        assert!(regex.is_match("v"));
        assert!(regex.is_match("V"));
    }
}
