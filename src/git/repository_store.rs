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
    pub fn find_merge_base(
        &self,
        branch_a_sha: &str,
        branch_b_sha: &str,
    ) -> Result<Option<String>> {
        self.repo.find_merge_base(branch_a_sha, branch_b_sha)
    }

    pub fn get_source_branches(
        &self,
        branch: &Git2Branch,
        config: &GitVersionConfiguration,
    ) -> Result<Vec<Git2Branch>> {
        let Some(branch_tip) = branch.tip_sha.as_deref() else {
            return Ok(Vec::new());
        };

        let current_config = config.get_branch_configuration(&branch.name);
        let allowed_sources = current_config.source_branches.unwrap_or_default();

        let mut out = Vec::new();
        for candidate in self.repo.branches()? {
            if candidate.remote || candidate.name == branch.name {
                continue;
            }

            let Some(candidate_tip) = candidate.tip_sha.as_deref() else {
                continue;
            };

            if !allowed_sources.is_empty() {
                let friendly = candidate.name.friendly();
                let matches_allowed = allowed_sources.iter().any(|allowed| {
                    friendly == *allowed
                        || friendly.starts_with(&format!("{allowed}/"))
                        || friendly.contains(allowed)
                });
                if !matches_allowed {
                    continue;
                }
            }

            let Some(merge_base) = self.find_merge_base(branch_tip, candidate_tip)? else {
                continue;
            };

            if merge_base == branch_tip {
                continue;
            }

            out.push(candidate);
        }

        Ok(out)
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

#[cfg(test)]
mod tests {
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::config::workflows;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::git::repository_store::RepositoryStore;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn get_source_branches_excludes_current_and_child_branches() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");

        let initial_branch = {
            let repo = git2::Repository::open(fixture.path()).expect("repo");
            repo.head()
                .expect("head")
                .shorthand()
                .unwrap_or("master")
                .to_string()
        };

        fixture.branch_to("develop").expect("develop branch");
        fixture.make_a_commit("develop commit").expect("commit");
        fixture.branch_to("feature/demo").expect("feature branch");
        fixture.make_a_commit("feature commit").expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repo");
        let current_branch = repo.head().expect("head branch");
        let store = RepositoryStore { repo: &repo };

        let mut config = GitVersionConfiguration::default();
        config.branches = workflows::resolve(&config.workflow);
        let branches = store
            .get_source_branches(&current_branch, &config)
            .expect("source branches");
        let names = branches
            .iter()
            .map(|branch| branch.name.friendly())
            .collect::<Vec<_>>();

        assert!(names.iter().any(|name| name == &"develop".to_string()));
        assert!(names.iter().any(|name| name == &initial_branch));
        assert!(!names.iter().any(|name| name == &"feature/demo".to_string()));
    }
}
