use chrono::{DateTime, FixedOffset};
use git2::{DiffOptions, Repository};

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
    fn matches(&self, commit: &Git2Commit) -> bool {
        if self.ignored_paths.is_empty() {
            return true;
        }

        let Ok(repo) = Repository::discover(".") else {
            return true;
        };
        let Ok(oid) = git2::Oid::from_str(commit.sha()) else {
            return true;
        };
        let Ok(git_commit) = repo.find_commit(oid) else {
            return true;
        };

        let current_tree = git_commit.tree().ok();
        let parent_tree = if git_commit.parent_count() > 0 {
            git_commit
                .parent(0)
                .ok()
                .and_then(|parent| parent.tree().ok())
        } else {
            None
        };

        let mut options = DiffOptions::new();
        let Ok(diff) = repo.diff_tree_to_tree(
            parent_tree.as_ref(),
            current_tree.as_ref(),
            Some(&mut options),
        ) else {
            return true;
        };

        let mut changed_paths = Vec::new();
        for delta in diff.deltas() {
            if let Some(path) = delta.new_file().path().or_else(|| delta.old_file().path())
                && let Some(path) = path.to_str()
            {
                changed_paths.push(path.to_string());
            }
        }

        if changed_paths.is_empty() {
            return true;
        }

        changed_paths.iter().any(|path| {
            !self
                .ignored_paths
                .iter()
                .any(|ignored| path == ignored || path.starts_with(&format!("{ignored}/")))
        })
    }
}
