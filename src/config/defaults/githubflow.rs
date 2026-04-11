use std::collections::HashMap;

use crate::config::branch_config::BranchConfiguration;
use crate::config::enums::IncrementStrategy;

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
    map.insert("release".to_string(), BranchConfiguration::default());
    map.insert("feature".to_string(), BranchConfiguration::default());
    map.insert("pull-request".to_string(), BranchConfiguration::default());
    map.insert("unknown".to_string(), BranchConfiguration::default());
    map
}
