use std::fs;
use std::path::{Path, PathBuf};

use eyre::{Result, eyre};
use git2::{Oid, Repository, Signature, build::CheckoutBuilder};
use tempfile::TempDir;

use crate::config::gitversion_config::GitVersionConfiguration;
use crate::context::GitVersionContext;
use crate::git::git2_impl::repository::Git2Repository;
use crate::semver::SemanticVersion;

pub struct RepositoryFixture {
    temp_dir: TempDir,
    repo: Repository,
    commit_counter: usize,
}

impl RepositoryFixture {
    pub fn new() -> Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let repo = Repository::init(temp_dir.path())?;
        let mut fixture = Self {
            temp_dir,
            repo,
            commit_counter: 0,
        };
        fixture.configure_git_identity()?;
        Ok(fixture)
    }

    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }

    pub fn make_a_commit(&mut self, message: &str) -> Result<String> {
        self.commit_counter += 1;
        let filename = format!("commit-{}.txt", self.commit_counter);
        let file_path = self.path().join(&filename);
        fs::write(&file_path, format!("{message}\n"))?;

        let mut index = self.repo.index()?;
        index.add_path(Path::new(&filename))?;
        index.write()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;
        let signature = Signature::now("GitVersion Tests", "gitversion-tests@example.com")?;

        let commit_id = match self.repo.head().ok().and_then(|head| head.target()) {
            Some(parent_id) => {
                let parent = self.repo.find_commit(parent_id)?;
                self.repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    message,
                    &tree,
                    &[&parent],
                )?
            }
            None => self
                .repo
                .commit(Some("HEAD"), &signature, &signature, message, &tree, &[])?,
        };

        Ok(commit_id.to_string())
    }

    pub fn make_commits(&mut self, count: usize, message_prefix: &str) -> Result<Vec<String>> {
        let mut commits = Vec::with_capacity(count);
        for index in 0..count {
            let message = format!("{message_prefix} {}", index + 1);
            commits.push(self.make_a_commit(&message)?);
        }
        Ok(commits)
    }

    pub fn branch_to(&mut self, branch_name: &str) -> Result<()> {
        let head = self.repo.head()?;
        let commit_oid = head.target().ok_or_else(|| eyre!("HEAD has no target"))?;
        let commit = self.repo.find_commit(commit_oid)?;
        self.repo.branch(branch_name, &commit, true)?;

        let reference_name = format!("refs/heads/{branch_name}");
        self.repo.set_head(&reference_name)?;
        self.repo
            .checkout_head(Some(CheckoutBuilder::default().safe()))?;

        Ok(())
    }

    pub fn checkout(&mut self, branch_name: &str) -> Result<()> {
        let reference_name = format!("refs/heads/{branch_name}");
        self.repo.set_head(&reference_name)?;
        self.repo
            .checkout_head(Some(CheckoutBuilder::default().safe()))?;
        Ok(())
    }

    pub fn apply_tag(&mut self, tag_name: &str) -> Result<()> {
        let head = self.repo.head()?;
        let commit_oid = head.target().ok_or_else(|| eyre!("HEAD has no target"))?;
        let commit = self.repo.find_commit(commit_oid)?;
        self.repo
            .tag_lightweight(tag_name, commit.as_object(), false)?;
        Ok(())
    }

    pub fn merge(&mut self, source_branch: &str, message: &str) -> Result<String> {
        let source = self
            .repo
            .find_branch(source_branch, git2::BranchType::Local)?;
        let source_commit = source.get().peel_to_commit()?;
        let head_commit = self.repo.head()?.peel_to_commit()?;
        let tree = head_commit.tree()?;
        let signature = Signature::now("GitVersion Tests", "gitversion-tests@example.com")?;
        let merge_commit_id = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&head_commit, &source_commit],
        )?;
        Ok(merge_commit_id.to_string())
    }

    pub fn head_sha(&self) -> Result<String> {
        self.repo
            .head()?
            .target()
            .map(|oid| oid.to_string())
            .ok_or_else(|| eyre!("HEAD has no target"))
    }

    pub fn find_commit(&self, sha: &str) -> Result<git2::Commit<'_>> {
        let oid = Oid::from_str(sha)?;
        Ok(self.repo.find_commit(oid)?)
    }

    pub fn calculate_version(
        &self,
        configuration: GitVersionConfiguration,
    ) -> Result<SemanticVersion> {
        let repository = Git2Repository::open(self.path())?;
        let context = GitVersionContext::from_repository(repository, configuration)?;
        crate::calculation::next_version::NextVersionCalculator.find_version(&context)
    }

    pub fn assert_full_semver(
        &self,
        expected: &str,
        configuration: GitVersionConfiguration,
    ) -> Result<()> {
        let version = self.calculate_version(configuration)?;
        if version.to_string() == expected {
            Ok(())
        } else {
            Err(eyre!("expected full semver {expected}, got {}", version))
        }
    }

    pub fn write_uncommitted_file(&self, relative_path: &str, content: &str) -> Result<PathBuf> {
        let path = self.path().join(relative_path);
        fs::write(&path, content)?;
        Ok(path)
    }

    fn configure_git_identity(&mut self) -> Result<()> {
        let mut config = self.repo.config()?;
        config.set_str("user.name", "GitVersion Tests")?;
        config.set_str("user.email", "gitversion-tests@example.com")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::RepositoryFixture;
    use crate::config::gitversion_config::GitVersionConfiguration;

    #[test]
    fn make_commits_creates_requested_number_of_commits() {
        let mut fixture = RepositoryFixture::new().expect("fixture");

        let commits = fixture.make_commits(3, "seed").expect("create commits");

        assert_eq!(commits.len(), 3);
        let latest = fixture.head_sha().expect("head sha");
        assert_eq!(latest, commits[2]);
    }

    #[test]
    fn checkout_switches_to_existing_branch() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        let initial_branch = fixture
            .repo
            .head()
            .expect("head")
            .shorthand()
            .unwrap_or("master")
            .to_string();
        fixture
            .branch_to("feature/test")
            .expect("create feature branch");
        fixture
            .checkout(&initial_branch)
            .expect("checkout initial branch");

        let head_name = fixture
            .repo
            .head()
            .expect("head")
            .name()
            .unwrap_or_default()
            .to_string();
        assert!(head_name.ends_with(&initial_branch));
    }

    #[test]
    fn merge_creates_merge_commit_with_two_parents() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        let initial_branch = fixture
            .repo
            .head()
            .expect("head")
            .shorthand()
            .unwrap_or("master")
            .to_string();
        fixture.branch_to("feature/merge").expect("branch");
        fixture
            .make_a_commit("feature commit")
            .expect("feature commit");
        fixture
            .checkout(&initial_branch)
            .expect("checkout initial branch");

        let merge_sha = fixture
            .merge("feature/merge", "merge feature")
            .expect("merge commit");
        let merge_commit = fixture.find_commit(&merge_sha).expect("find merge commit");

        assert_eq!(merge_commit.parent_count(), 2);
    }

    #[test]
    fn assert_full_semver_succeeds_for_calculated_value() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");

        let version = fixture
            .calculate_version(GitVersionConfiguration::default())
            .expect("calculate version");
        fixture
            .assert_full_semver(&version.to_string(), GitVersionConfiguration::default())
            .expect("assert semver");
    }
}
