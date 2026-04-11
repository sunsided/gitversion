use std::collections::HashMap;

use crate::config::branch_config::BranchConfiguration;
use crate::config::enums::{DeploymentMode, IncrementStrategy};

pub fn defaults() -> HashMap<String, BranchConfiguration> {
    let mut map = HashMap::new();
    map.insert(
        "main".to_string(),
        BranchConfiguration {
            increment: Some(IncrementStrategy::Patch),
            is_main_branch: Some(true),
            label: Some(String::new()),
            ..Default::default()
        },
    );
    map.insert(
        "develop".to_string(),
        BranchConfiguration {
            increment: Some(IncrementStrategy::Minor),
            label: Some("alpha".to_string()),
            tracks_release_branches: Some(true),
            track_merge_target: Some(true),
            ..Default::default()
        },
    );
    map.insert(
        "release".to_string(),
        BranchConfiguration {
            increment: Some(IncrementStrategy::Minor),
            label: Some("beta".to_string()),
            is_release_branch: Some(true),
            deployment_mode: Some(DeploymentMode::ManualDeployment),
            ..Default::default()
        },
    );
    map.insert("feature".to_string(), BranchConfiguration::default());
    map.insert("pull-request".to_string(), BranchConfiguration::default());
    map.insert("hotfix".to_string(), BranchConfiguration::default());
    map.insert("support".to_string(), BranchConfiguration::default());
    map.insert("unknown".to_string(), BranchConfiguration::default());
    map
}
