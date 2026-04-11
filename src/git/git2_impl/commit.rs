use std::fmt::{Display, Formatter};

use chrono::{DateTime, FixedOffset, TimeZone};

use crate::git::traits::{Commit, ObjectId};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Git2ObjectId(pub String);

impl Display for Git2ObjectId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl ObjectId for Git2ObjectId {
    fn sha(&self) -> &str {
        &self.0
    }

    fn to_short_string(&self, len: usize) -> String {
        self.0.chars().take(len).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Git2Commit {
    pub id: Git2ObjectId,
    pub when: DateTime<FixedOffset>,
    pub message: String,
    pub parent_shas: Vec<String>,
}

impl Git2Commit {
    pub fn from_git2(commit: &git2::Commit<'_>) -> Self {
        let t = commit.time();
        let offset = FixedOffset::east_opt(t.offset_minutes() * 60)
            .unwrap_or_else(|| FixedOffset::east_opt(0).expect("offset"));
        let when = offset
            .timestamp_opt(t.seconds(), 0)
            .single()
            .unwrap_or_else(|| offset.timestamp_opt(0, 0).single().expect("epoch"));
        Self {
            id: Git2ObjectId(commit.id().to_string()),
            when,
            message: commit.message().unwrap_or_default().to_string(),
            parent_shas: commit.parent_ids().map(|id| id.to_string()).collect(),
        }
    }

    pub fn sha(&self) -> &str {
        &self.id.0
    }
}

impl Commit for Git2Commit {
    type Id = Git2ObjectId;

    fn id(&self) -> &Self::Id {
        &self.id
    }

    fn sha(&self) -> &str {
        self.id.sha()
    }

    fn when(&self) -> DateTime<FixedOffset> {
        self.when
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn parent_shas(&self) -> &[String] {
        &self.parent_shas
    }
}
