use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PreventIncrementConfiguration {
    pub of_merged_branch: Option<bool>,
    pub when_branch_merged: Option<bool>,
    pub when_current_commit_tagged: Option<bool>,
}
