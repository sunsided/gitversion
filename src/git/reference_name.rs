#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ReferenceName {
    pub canonical: String,
}

impl ReferenceName {
    pub const LOCAL_BRANCH_PREFIX: &'static str = "refs/heads/";
    pub const REMOTE_TRACKING_PREFIX: &'static str = "refs/remotes/";
    pub const TAG_PREFIX: &'static str = "refs/tags/";
    pub const ORIGIN_PREFIX: &'static str = "origin/";

    pub fn parse(canonical: &str) -> Self {
        Self {
            canonical: canonical.to_string(),
        }
    }

    pub fn from_branch_name(name: &str) -> Self {
        if name.starts_with("refs/") {
            Self::parse(name)
        } else {
            Self::parse(&format!("{}{}", Self::LOCAL_BRANCH_PREFIX, name))
        }
    }

    pub fn friendly(&self) -> String {
        self.canonical
            .trim_start_matches(Self::LOCAL_BRANCH_PREFIX)
            .trim_start_matches(Self::REMOTE_TRACKING_PREFIX)
            .trim_start_matches(Self::TAG_PREFIX)
            .to_string()
    }

    pub fn without_origin(&self) -> String {
        self.friendly()
            .trim_start_matches(Self::ORIGIN_PREFIX)
            .to_string()
    }

    pub fn is_local_branch(&self) -> bool {
        self.canonical.starts_with(Self::LOCAL_BRANCH_PREFIX)
    }

    pub fn is_remote_branch(&self) -> bool {
        self.canonical.starts_with(Self::REMOTE_TRACKING_PREFIX)
    }

    pub fn is_tag(&self) -> bool {
        self.canonical.starts_with(Self::TAG_PREFIX)
    }

    pub fn is_pull_request(&self) -> bool {
        let friendly = self.friendly();
        ["pull/", "refs/pull/", "merge-requests/"]
            .iter()
            .any(|p| friendly.starts_with(p))
    }
}

#[cfg(test)]
mod tests {
    use super::ReferenceName;

    #[test]
    fn friendly_strips_known_ref_prefixes() {
        assert_eq!(ReferenceName::parse("refs/heads/main").friendly(), "main");
        assert_eq!(
            ReferenceName::parse("refs/remotes/origin/feature/foo").friendly(),
            "origin/feature/foo"
        );
        assert_eq!(
            ReferenceName::parse("refs/tags/v1.2.3").friendly(),
            "v1.2.3"
        );
    }

    #[test]
    fn from_branch_name_adds_local_prefix_when_missing() {
        assert_eq!(
            ReferenceName::from_branch_name("feature/x").canonical,
            "refs/heads/feature/x"
        );
        assert_eq!(
            ReferenceName::from_branch_name("refs/remotes/origin/main").canonical,
            "refs/remotes/origin/main"
        );
    }

    #[test]
    fn branch_kind_detection_matches_prefixes() {
        let local = ReferenceName::parse("refs/heads/main");
        let remote = ReferenceName::parse("refs/remotes/origin/main");
        let tag = ReferenceName::parse("refs/tags/v1.0.0");

        assert!(local.is_local_branch());
        assert!(!local.is_remote_branch());
        assert!(remote.is_remote_branch());
        assert!(!remote.is_local_branch());
        assert!(tag.is_tag());
    }

    #[test]
    fn without_origin_removes_origin_prefix_after_friendly_conversion() {
        let remote = ReferenceName::parse("refs/remotes/origin/feature/foo");
        assert_eq!(remote.without_origin(), "feature/foo");
    }

    #[test]
    fn pull_request_detection_supports_multiple_prefixes() {
        assert!(ReferenceName::parse("refs/heads/pull/123/merge").is_pull_request());
        assert!(ReferenceName::parse("refs/heads/refs/pull/123/merge").is_pull_request());
        assert!(ReferenceName::parse("refs/heads/merge-requests/45/head").is_pull_request());
        assert!(!ReferenceName::parse("refs/heads/feature/foo").is_pull_request());
    }
}
