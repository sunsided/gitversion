use crate::git::reference_name::ReferenceName;
use crate::git::traits::Tag;

#[derive(Debug, Clone)]
pub struct Git2Tag {
    pub name: ReferenceName,
    pub target_sha: String,
    pub commit_sha: String,
}

impl Tag for Git2Tag {
    fn name(&self) -> &ReferenceName {
        &self.name
    }

    fn target_sha(&self) -> &str {
        &self.target_sha
    }

    fn commit_sha(&self) -> &str {
        &self.commit_sha
    }
}
