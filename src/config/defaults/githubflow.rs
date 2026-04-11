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

#[cfg(test)]
mod tests {
    use crate::config::defaults::githubflow;
    use crate::config::enums::IncrementStrategy;

    #[test]
    fn defaults_contains_expected_branch_keys() {
        let branches = githubflow::defaults();

        assert_eq!(branches.len(), 5);
        assert!(branches.contains_key("main"));
        assert!(branches.contains_key("release"));
        assert!(branches.contains_key("feature"));
        assert!(branches.contains_key("pull-request"));
        assert!(branches.contains_key("unknown"));
    }

    #[test]
    fn defaults_main_branch_has_patch_increment_and_main_marker() {
        let branches = githubflow::defaults();
        let main = branches.get("main").expect("main branch exists");

        assert_eq!(main.increment, Some(IncrementStrategy::Patch));
        assert_eq!(main.is_main_branch, Some(true));
        assert_eq!(main.label.as_deref(), Some(""));
    }

    #[test]
    fn defaults_non_main_branches_use_base_configuration() {
        let branches = githubflow::defaults();
        let release = branches.get("release").expect("release branch exists");

        assert!(release.increment.is_none());
        assert!(release.label.is_none());
        assert!(release.is_main_branch.is_none());
        assert!(release.is_release_branch.is_none());
    }
}
