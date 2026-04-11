use std::collections::HashMap;

use crate::config::branch_config::BranchConfiguration;
use crate::config::defaults::{gitflow, githubflow};

pub fn resolve(name: &str) -> HashMap<String, BranchConfiguration> {
    match name {
        "GitHubFlow/v1" => githubflow::defaults(),
        _ => gitflow::defaults(),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve;

    #[test]
    fn resolve_returns_githubflow_when_requested() {
        let branches = resolve("GitHubFlow/v1");

        assert_eq!(branches.len(), 5);
        assert!(branches.contains_key("main"));
        assert!(branches.contains_key("release"));
        assert!(branches.contains_key("feature"));
        assert!(branches.contains_key("pull-request"));
        assert!(branches.contains_key("unknown"));
    }

    #[test]
    fn resolve_returns_gitflow_for_gitflow_name() {
        let branches = resolve("GitFlow/v1");

        assert_eq!(branches.len(), 8);
        assert!(branches.contains_key("develop"));
        assert!(branches.contains_key("hotfix"));
        assert!(branches.contains_key("support"));
    }

    #[test]
    fn resolve_falls_back_to_gitflow_for_unknown_name() {
        let branches = resolve("TrunkBased/preview1");

        assert_eq!(branches.len(), 8);
        assert!(branches.contains_key("develop"));
        assert!(branches.contains_key("unknown"));
    }

    #[test]
    fn resolve_distinguishes_between_workflow_specific_branch_sets() {
        let github = resolve("GitHubFlow/v1");
        let gitflow = resolve("GitFlow/v1");

        assert!(!github.contains_key("develop"));
        assert!(gitflow.contains_key("develop"));
    }
}
