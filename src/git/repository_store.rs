use eyre::Result;

use crate::config::gitversion_config::GitVersionConfiguration;
use crate::git::git2_impl::branch::Git2Branch;
use crate::git::git2_impl::commit::Git2Commit;
use crate::git::git2_impl::repository::Git2Repository;

#[derive(Debug)]
pub struct RepositoryStore<'a> {
    pub repo: &'a Git2Repository,
}

impl<'a> RepositoryStore<'a> {
    pub fn find_merge_base(&self, branch_a_sha: &str, branch_b_sha: &str) -> Result<Option<String>> {
        self.repo.find_merge_base(branch_a_sha, branch_b_sha)
    }

    pub fn get_source_branches(&self, _branch: &Git2Branch, _config: &GitVersionConfiguration) -> Result<Vec<Git2Branch>> {
        self.repo.branches()
    }

    pub fn find_commit_branches_branched_from(
        &self,
        branch: &Git2Branch,
        config: &GitVersionConfiguration,
    ) -> Result<Vec<(Git2Branch, Git2Commit)>> {
        let mut out = Vec::new();
        for b in self.get_source_branches(branch, config)? {
            if let Some(sha) = b.tip_sha.clone()
                && let Ok(commit) = self.repo.repo.find_commit(git2::Oid::from_str(&sha)?)
            {
                out.push((b, Git2Commit::from_git2(&commit)));
            }
        }
        Ok(out)
    }
}
