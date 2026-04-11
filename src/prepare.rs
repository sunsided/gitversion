use eyre::{bail, Result};

use crate::git::git2_impl::repository::Git2Repository;

#[derive(Debug, Clone, Copy, Default)]
pub struct GitPrepareOptions {
    pub no_fetch: bool,
    pub no_normalize: bool,
    pub allow_shallow: bool,
}

#[derive(Debug, Default)]
pub struct GitPreparer;

impl GitPreparer {
    pub fn prepare(&self, options: GitPrepareOptions, repo: &mut Git2Repository) -> Result<()> {
        if repo.repo.is_shallow() && !options.allow_shallow {
            bail!("repository is shallow; use --allow-shallow to continue");
        }
        if !options.no_fetch {
            let _ = repo.fetch_origin();
        }
        if !options.no_normalize {
            let _ = repo.head();
        }
        Ok(())
    }
}
