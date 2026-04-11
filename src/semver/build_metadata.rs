use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::semver::{SemanticVersion, VersionField};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemanticVersionBuildMetaData {
    pub commits_since_tag: Option<i64>,
    pub branch: Option<String>,
    pub sha: Option<String>,
    pub short_sha: Option<String>,
    pub other_metadata: Option<String>,
    pub commit_date: Option<DateTime<Utc>>,
    pub version_source_semver: Option<Box<SemanticVersion>>,
    pub version_source_sha: Option<String>,
    pub version_source_distance: i64,
    pub uncommitted_changes: i64,
    pub version_source_increment: VersionField,
}

impl SemanticVersionBuildMetaData {
    pub fn parse(s: &str) -> Self {
        let mut out = Self::default();
        if s.is_empty() {
            return out;
        }
        let mut pieces = s.split('.');
        out.commits_since_tag = pieces.next().and_then(|v| v.parse::<i64>().ok());
        out.branch = pieces.next().map(ToString::to_string);
        out.sha = pieces.next().map(ToString::to_string);
        out
    }
}

impl Display for SemanticVersionBuildMetaData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut parts = Vec::new();
        if let Some(v) = self.commits_since_tag {
            parts.push(v.to_string());
        }
        if let Some(v) = &self.branch {
            parts.push(v.clone());
        }
        if let Some(v) = &self.sha {
            parts.push(v.clone());
        }
        if let Some(v) = &self.other_metadata {
            parts.push(v.clone());
        }
        write!(f, "{}", parts.join("."))
    }
}

impl PartialEq for SemanticVersionBuildMetaData {
    fn eq(&self, other: &Self) -> bool {
        self.commits_since_tag == other.commits_since_tag
            && self.branch == other.branch
            && self.sha == other.sha
    }
}

impl Eq for SemanticVersionBuildMetaData {}

impl Hash for SemanticVersionBuildMetaData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.commits_since_tag.hash(state);
        self.branch.hash(state);
        self.sha.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::SemanticVersionBuildMetaData;

    #[test]
    fn parse_empty_returns_default() {
        let parsed = SemanticVersionBuildMetaData::parse("");
        assert_eq!(parsed, SemanticVersionBuildMetaData::default());
    }

    #[test]
    fn parse_reads_commits_branch_and_sha() {
        let parsed = SemanticVersionBuildMetaData::parse("3.main.a1b2c3");

        assert_eq!(parsed.commits_since_tag, Some(3));
        assert_eq!(parsed.branch.as_deref(), Some("main"));
        assert_eq!(parsed.sha.as_deref(), Some("a1b2c3"));
    }

    #[test]
    fn display_joins_available_parts() {
        let metadata = SemanticVersionBuildMetaData {
            commits_since_tag: Some(2),
            branch: Some("feature".to_string()),
            sha: Some("deadbeef".to_string()),
            other_metadata: Some("extra".to_string()),
            ..Default::default()
        };

        assert_eq!(metadata.to_string(), "2.feature.deadbeef.extra");
    }

    #[test]
    fn equality_and_hash_ignore_non_core_fields() {
        let a = SemanticVersionBuildMetaData {
            commits_since_tag: Some(7),
            branch: Some("main".to_string()),
            sha: Some("abc".to_string()),
            other_metadata: Some("x".to_string()),
            ..Default::default()
        };
        let b = SemanticVersionBuildMetaData {
            commits_since_tag: Some(7),
            branch: Some("main".to_string()),
            sha: Some("abc".to_string()),
            other_metadata: Some("y".to_string()),
            ..Default::default()
        };

        assert_eq!(a, b);

        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }
}
