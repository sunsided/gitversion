use std::path::Path;

use eyre::Result;
use git2::{BranchType, Repository, StatusOptions};

use crate::git::git2_impl::branch::Git2Branch;
use crate::git::git2_impl::commit::Git2Commit;
use crate::git::git2_impl::tag::Git2Tag;
use crate::git::reference_name::ReferenceName;

pub struct Git2Repository {
    pub repo: Repository,
}

impl std::fmt::Debug for Git2Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Git2Repository").finish_non_exhaustive()
    }
}

impl Git2Repository {
    pub fn open(path: &Path) -> Result<Self> {
        Ok(Self {
            repo: Repository::discover(path)?,
        })
    }

    pub fn head(&self) -> Result<Git2Branch> {
        let head = self.repo.head()?;
        let name = head.name().unwrap_or("HEAD");
        let detached = head.is_branch().not();
        Ok(Git2Branch {
            name: ReferenceName::parse(name),
            tip_sha: head.target().map(|o| o.to_string()),
            remote: false,
            tracking: false,
            detached_head: detached,
        })
    }

    pub fn head_commit(&self) -> Result<Git2Commit> {
        let commit = self.repo.head()?.peel_to_commit()?;
        Ok(Git2Commit::from_git2(&commit))
    }

    pub fn branches(&self) -> Result<Vec<Git2Branch>> {
        let mut out = Vec::new();
        for item in self.repo.branches(None)? {
            let (branch, kind) = item?;
            let name = branch.get().name().unwrap_or("unknown");
            let tip = branch.get().target().map(|o| o.to_string());
            let tracking = branch.upstream().is_ok();
            out.push(Git2Branch {
                name: ReferenceName::parse(name),
                tip_sha: tip,
                remote: matches!(kind, BranchType::Remote),
                tracking,
                detached_head: false,
            });
        }
        Ok(out)
    }

    pub fn tags(&self) -> Result<Vec<Git2Tag>> {
        let mut out = Vec::new();
        let names = self.repo.tag_names(None)?;
        for name in names.iter().flatten() {
            let full = format!("refs/tags/{name}");
            if let Ok(reference) = self.repo.find_reference(&full) {
                let target = reference
                    .target()
                    .map(|o| o.to_string())
                    .unwrap_or_default();
                let commit_sha = reference
                    .peel_to_commit()
                    .map(|c| c.id().to_string())
                    .unwrap_or_default();
                out.push(Git2Tag {
                    name: ReferenceName::parse(&full),
                    target_sha: target,
                    commit_sha,
                });
            }
        }
        Ok(out)
    }

    pub fn find_merge_base(&self, a: &str, b: &str) -> Result<Option<String>> {
        let a = git2::Oid::from_str(a)?;
        let b = git2::Oid::from_str(b)?;
        Ok(self.repo.merge_base(a, b).ok().map(|o| o.to_string()))
    }

    pub fn uncommitted_changes_count(&self) -> Result<usize> {
        let mut options = StatusOptions::new();
        options.include_untracked(true).recurse_untracked_dirs(true);
        Ok(self.repo.statuses(Some(&mut options))?.len())
    }

    pub fn fetch_origin(&mut self) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        remote.fetch(&["refs/heads/*:refs/remotes/origin/*"], None, None)?;
        Ok(())
    }
}

trait BoolNot {
    fn not(self) -> bool;
}

impl BoolNot for bool {
    fn not(self) -> bool {
        !self
    }
}
