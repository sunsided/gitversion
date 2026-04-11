use std::cmp::Ordering;

use crate::git::git2_impl::tag::Git2Tag;
use crate::semver::SemanticVersion;

#[derive(Debug, Clone)]
pub struct SemanticVersionWithTag {
    pub value: SemanticVersion,
    pub tag: Git2Tag,
}

impl PartialEq for SemanticVersionWithTag {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SemanticVersionWithTag {}

impl PartialOrd for SemanticVersionWithTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersionWithTag {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}
