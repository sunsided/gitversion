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

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::sync::Mutex;

    use chrono::{FixedOffset, TimeZone};
    use once_cell::sync::Lazy;
    use serial_test::serial;

    use super::{MinDateVersionFilter, PathFilter, ShaVersionFilter, VersionFilter};
    use crate::git::git2_impl::commit::{Git2Commit, Git2ObjectId};
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    static CWD_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    fn commit_with(sha: &str, unix_seconds: i64) -> Git2Commit {
        let offset = FixedOffset::east_opt(0).expect("offset");
        Git2Commit {
            id: Git2ObjectId(sha.to_string()),
            when: offset
                .timestamp_opt(unix_seconds, 0)
                .single()
                .expect("timestamp"),
            message: "message".to_string(),
            parent_shas: Vec::new(),
        }
    }

    struct CurrentDirGuard {
        previous: std::path::PathBuf,
    }

    impl CurrentDirGuard {
        fn change_to(path: &Path) -> Self {
            let previous = std::env::current_dir().expect("current dir");
            std::env::set_current_dir(path).expect("set current dir");
            Self { previous }
        }
    }

    impl Drop for CurrentDirGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.previous);
        }
    }

    #[test]
    fn min_date_filter_accepts_commit_on_or_after_threshold() {
        let filter = MinDateVersionFilter {
            min_date: commit_with("a", 200).when,
        };

        assert!(filter.matches(&commit_with("b", 200)));
        assert!(filter.matches(&commit_with("c", 201)));
    }

    #[test]
    fn min_date_filter_rejects_commit_before_threshold() {
        let filter = MinDateVersionFilter {
            min_date: commit_with("a", 200).when,
        };

        assert!(!filter.matches(&commit_with("b", 199)));
    }

    #[test]
    fn sha_filter_rejects_ignored_sha() {
        let filter = ShaVersionFilter {
            ignored_shas: vec!["abc123".to_string()],
        };

        assert!(!filter.matches(&commit_with("abc123", 1)));
    }

    #[test]
    fn sha_filter_accepts_non_ignored_sha() {
        let filter = ShaVersionFilter {
            ignored_shas: vec!["abc123".to_string()],
        };

        assert!(filter.matches(&commit_with("def456", 1)));
    }

    #[test]
    fn path_filter_accepts_when_no_paths_are_ignored() {
        let filter = PathFilter {
            ignored_paths: Vec::new(),
        };

        assert!(filter.matches(&commit_with("invalid", 1)));
    }

    #[test]
    fn path_filter_accepts_when_commit_sha_is_invalid() {
        let filter = PathFilter {
            ignored_paths: vec!["src".to_string()],
        };

        assert!(filter.matches(&commit_with("not-a-sha", 1)));
    }

    #[test]
    #[serial]
    fn path_filter_rejects_commit_when_all_changed_paths_are_ignored() {
        let _guard = CWD_LOCK.lock().expect("cwd lock poisoned");
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial").expect("commit");
        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let commit = repo.head_commit().expect("head commit");
        let _cwd = CurrentDirGuard::change_to(fixture.path());

        let filter = PathFilter {
            ignored_paths: vec!["commit-1.txt".to_string()],
        };

        assert!(!filter.matches(&commit));
    }

    #[test]
    #[serial]
    fn path_filter_accepts_commit_when_non_ignored_paths_changed() {
        let _guard = CWD_LOCK.lock().expect("cwd lock poisoned");
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial").expect("commit");
        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let commit = repo.head_commit().expect("head commit");
        let _cwd = CurrentDirGuard::change_to(fixture.path());

        let filter = PathFilter {
            ignored_paths: vec!["docs".to_string()],
        };

        assert!(filter.matches(&commit));
    }
}
