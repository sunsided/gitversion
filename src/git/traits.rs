use std::fmt::Display;
use std::hash::Hash;

use chrono::{DateTime, FixedOffset};

use crate::git::reference_name::ReferenceName;

pub trait ObjectId: Display + Clone + Eq + Hash {
    fn sha(&self) -> &str;
    fn to_short_string(&self, len: usize) -> String;
}

pub trait Commit: Clone {
    type Id: ObjectId;
    fn id(&self) -> &Self::Id;
    fn sha(&self) -> &str;
    fn when(&self) -> DateTime<FixedOffset>;
    fn message(&self) -> &str;
    fn parent_shas(&self) -> &[String];
    fn is_merge_commit(&self) -> bool {
        self.parent_shas().len() > 1
    }
}

pub trait Branch {
    fn name(&self) -> &ReferenceName;
    fn tip_sha(&self) -> Option<&str>;
    fn is_remote(&self) -> bool;
    fn is_tracking(&self) -> bool;
    fn is_detached_head(&self) -> bool;
}

pub trait Tag {
    fn name(&self) -> &ReferenceName;
    fn target_sha(&self) -> &str;
    fn commit_sha(&self) -> &str;
}
