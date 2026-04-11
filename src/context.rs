use eyre::Result;

use crate::config::gitversion_config::GitVersionConfiguration;
use crate::git::git2_impl::branch::Git2Branch;
use crate::git::git2_impl::commit::Git2Commit;
use crate::git::git2_impl::repository::Git2Repository;

#[derive(Debug)]
pub struct GitVersionContext {
    pub repository: Git2Repository,
    pub current_branch: Git2Branch,
    pub current_commit: Git2Commit,
    pub configuration: GitVersionConfiguration,
    pub is_current_commit_tagged: bool,
    pub number_of_uncommitted_changes: i64,
}

impl GitVersionContext {
    pub fn from_repository(
        repository: Git2Repository,
        configuration: GitVersionConfiguration,
    ) -> Result<Self> {
        let current_branch = repository.head()?;
        let current_commit = repository.head_commit()?;
        let tags = repository.tags()?;
        let tagged = tags.iter().any(|t| t.commit_sha == current_commit.sha());
        let uncommitted = repository.uncommitted_changes_count()? as i64;
        Ok(Self {
            repository,
            current_branch,
            current_commit,
            configuration,
            is_current_commit_tagged: tagged,
            number_of_uncommitted_changes: uncommitted,
        })
    }
}
