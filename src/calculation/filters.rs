use chrono::{DateTime, FixedOffset};

use crate::git::git2_impl::commit::Git2Commit;

pub trait VersionFilter {
    fn matches(&self, commit: &Git2Commit) -> bool;
}

pub struct MinDateVersionFilter {
    pub min_date: DateTime<FixedOffset>,
}

impl VersionFilter for MinDateVersionFilter {
    fn matches(&self, commit: &Git2Commit) -> bool {
        commit.when >= self.min_date
    }
}

pub struct ShaVersionFilter {
    pub ignored_shas: Vec<String>,
}

impl VersionFilter for ShaVersionFilter {
    fn matches(&self, commit: &Git2Commit) -> bool {
        !self.ignored_shas.iter().any(|sha| sha == commit.sha())
    }
}

pub struct PathFilter {
    pub ignored_paths: Vec<String>,
}

impl VersionFilter for PathFilter {
    fn matches(&self, _commit: &Git2Commit) -> bool {
        self.ignored_paths.is_empty()
    }
}
