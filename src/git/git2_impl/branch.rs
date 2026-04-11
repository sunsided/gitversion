use crate::git::reference_name::ReferenceName;
use crate::git::traits::Branch;

#[derive(Debug, Clone)]
pub struct Git2Branch {
    pub name: ReferenceName,
    pub tip_sha: Option<String>,
    pub remote: bool,
    pub tracking: bool,
    pub detached_head: bool,
}

impl Branch for Git2Branch {
    fn name(&self) -> &ReferenceName {
        &self.name
    }

    fn tip_sha(&self) -> Option<&str> {
        self.tip_sha.as_deref()
    }

    fn is_remote(&self) -> bool {
        self.remote
    }

    fn is_tracking(&self) -> bool {
        self.tracking
    }

    fn is_detached_head(&self) -> bool {
        self.detached_head
    }
}
