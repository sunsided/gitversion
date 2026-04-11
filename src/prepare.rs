use std::fs;
use std::path::Path;

use eyre::{Result, bail};
use git2::{Cred, FetchOptions, Oid, RemoteCallbacks, build::RepoBuilder};

use crate::git::git2_impl::repository::Git2Repository;

#[derive(Debug, Clone, Default)]
pub struct GitPrepareOptions {
    pub no_fetch: bool,
    pub no_normalize: bool,
    pub allow_shallow: bool,
    pub target_branch: Option<String>,
    pub target_commit: Option<String>,
    pub auth: GitRemoteAuth,
}

#[derive(Debug, Clone, Default)]
pub struct GitRemoteAuth {
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Default)]
pub struct GitPreparer;

impl GitPreparer {
    pub fn open_or_clone_repository(
        &self,
        target_path: &Path,
        url: Option<&str>,
        auth: &GitRemoteAuth,
    ) -> Result<Git2Repository> {
        match url {
            None => Git2Repository::open(target_path),
            Some(remote_url) => match Git2Repository::open(target_path) {
                Ok(repo) => Ok(repo),
                Err(_) => self.clone_repository(remote_url, target_path, auth),
            },
        }
    }

    pub fn prepare(&self, options: GitPrepareOptions, repo: &mut Git2Repository) -> Result<()> {
        if repo.repo.is_shallow() && !options.allow_shallow {
            bail!("repository is shallow; use --allow-shallow to continue");
        }

        if !options.no_fetch {
            let _ = self.fetch_origin(repo, &options.auth);
        }

        if !options.no_normalize {
            self.normalize_checkout(
                repo,
                options.target_branch.as_deref(),
                options.target_commit.as_deref(),
            )?;
        }

        Ok(())
    }

    fn clone_repository(
        &self,
        remote_url: &str,
        target_path: &Path,
        auth: &GitRemoteAuth,
    ) -> Result<Git2Repository> {
        if target_path.exists() && target_path.read_dir()?.next().is_some() {
            bail!(
                "target path '{}' exists and is not a git repository",
                target_path.display()
            );
        }

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let callbacks = build_remote_callbacks(auth);
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_options);

        Ok(Git2Repository {
            repo: builder.clone(remote_url, target_path)?,
        })
    }

    fn fetch_origin(&self, repo: &mut Git2Repository, auth: &GitRemoteAuth) -> Result<()> {
        let callbacks = build_remote_callbacks(auth);
        let mut fetch_options = FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        let mut remote = repo.repo.find_remote("origin")?;
        remote.fetch(
            &["refs/heads/*:refs/remotes/origin/*"],
            Some(&mut fetch_options),
            None,
        )?;
        Ok(())
    }

    fn normalize_checkout(
        &self,
        repo: &mut Git2Repository,
        target_branch: Option<&str>,
        target_commit: Option<&str>,
    ) -> Result<()> {
        if let Some(branch) = target_branch {
            self.checkout_branch(repo, branch)?;
        }

        if let Some(commit) = target_commit {
            let oid = Oid::from_str(commit)?;
            repo.repo.set_head_detached(oid)?;
            return Ok(());
        }

        let _ = repo.head()?;
        Ok(())
    }

    fn checkout_branch(&self, repo: &mut Git2Repository, branch: &str) -> Result<()> {
        for candidate in branch_ref_candidates(branch) {
            if repo.repo.find_reference(&candidate).is_ok() {
                repo.repo.set_head(&candidate)?;
                return Ok(());
            }
        }

        bail!("unable to resolve branch '{branch}'")
    }
}

fn branch_ref_candidates(branch: &str) -> Vec<String> {
    if branch.starts_with("refs/") {
        return vec![branch.to_string()];
    }

    vec![
        format!("refs/heads/{branch}"),
        format!("refs/remotes/{branch}"),
        format!("refs/remotes/origin/{branch}"),
    ]
}

fn build_remote_callbacks(auth: &GitRemoteAuth) -> RemoteCallbacks<'static> {
    let username = auth.username.clone();
    let password = auth.password.clone();

    let mut callbacks = RemoteCallbacks::new();
    callbacks.credentials(move |_url, username_from_url, _allowed_types| {
        if let Some(user) = username.as_deref() {
            return Cred::userpass_plaintext(user, password.as_deref().unwrap_or(""));
        }

        if let (Some(user), Some(pass)) = (username_from_url, password.as_deref()) {
            return Cred::userpass_plaintext(user, pass);
        }

        Cred::default()
    });
    callbacks
}

#[cfg(test)]
mod tests {
    use super::{GitPrepareOptions, GitPreparer};
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn prepare_can_checkout_target_branch_name() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial").expect("commit");
        fixture.branch_to("feature/test").expect("branch");

        let mut repo = Git2Repository::open(fixture.path()).expect("open");
        let prepare = GitPreparer;
        prepare
            .prepare(
                GitPrepareOptions {
                    no_fetch: true,
                    no_normalize: false,
                    allow_shallow: true,
                    target_branch: Some("feature/test".to_string()),
                    target_commit: None,
                    ..Default::default()
                },
                &mut repo,
            )
            .expect("prepare");

        let head = repo.head().expect("head");
        assert_eq!(head.name.canonical, "refs/heads/feature/test");
    }

    #[test]
    fn prepare_can_checkout_target_commit() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        let first = fixture.make_a_commit("first").expect("first commit");
        fixture.make_a_commit("second").expect("second commit");

        let mut repo = Git2Repository::open(fixture.path()).expect("open");
        let prepare = GitPreparer;
        prepare
            .prepare(
                GitPrepareOptions {
                    no_fetch: true,
                    no_normalize: false,
                    allow_shallow: true,
                    target_branch: None,
                    target_commit: Some(first.clone()),
                    ..Default::default()
                },
                &mut repo,
            )
            .expect("prepare");

        assert_eq!(repo.head_commit().expect("head commit").sha(), first);
    }

    #[test]
    fn open_or_clone_repository_clones_when_target_is_missing() {
        let mut source = RepositoryFixture::new().expect("source fixture");
        source.make_a_commit("initial").expect("commit");

        let destination_root = tempfile::tempdir().expect("destination root");
        let destination = destination_root.path().join("clone-target");
        let url = source.path().to_string_lossy().to_string();

        let preparer = GitPreparer;
        let cloned = preparer
            .open_or_clone_repository(&destination, Some(&url), &Default::default())
            .expect("clone repository");

        assert!(destination.join(".git").exists());
        assert_eq!(
            cloned.head_commit().expect("head commit").sha(),
            source.head_sha().expect("source head")
        );
    }
}
