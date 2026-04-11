use std::collections::HashMap;

use crate::config::branch_config::BranchConfiguration;
use crate::config::defaults::{gitflow, githubflow};

pub fn resolve(name: &str) -> HashMap<String, BranchConfiguration> {
    match name {
        "GitHubFlow/v1" => githubflow::defaults(),
        _ => gitflow::defaults(),
    }
}
