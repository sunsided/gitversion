use std::fs;
use std::path::{Path, PathBuf};

use eyre::{eyre, Result};
use git2::{build::CheckoutBuilder, Repository, Signature};
use tempfile::TempDir;

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

    pub fn apply_tag(&mut self, tag_name: &str) -> Result<()> {
        let head = self.repo.head()?;
        let commit_oid = head.target().ok_or_else(|| eyre!("HEAD has no target"))?;
        let commit = self.repo.find_commit(commit_oid)?;
        self.repo
            .tag_lightweight(tag_name, commit.as_object(), false)?;
        Ok(())
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
