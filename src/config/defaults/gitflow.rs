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
    map.insert(
        "feature".to_string(),
        BranchConfiguration {
            increment: Some(IncrementStrategy::Inherit),
            ..Default::default()
        },
    );
    map.insert(
        "pull-request".to_string(),
        BranchConfiguration {
            increment: Some(IncrementStrategy::Inherit),
            ..Default::default()
        },
    );
    map.insert("hotfix".to_string(), BranchConfiguration::default());
    map.insert("support".to_string(), BranchConfiguration::default());
    map.insert("unknown".to_string(), BranchConfiguration::default());
    map
}

#[cfg(test)]
mod tests {
    use crate::config::defaults::gitflow;
    use crate::config::enums::{DeploymentMode, IncrementStrategy};

    #[test]
    fn defaults_contains_expected_branch_keys() {
        let branches = gitflow::defaults();

        assert_eq!(branches.len(), 8);
        assert!(branches.contains_key("main"));
        assert!(branches.contains_key("develop"));
        assert!(branches.contains_key("release"));
        assert!(branches.contains_key("feature"));
        assert!(branches.contains_key("pull-request"));
        assert!(branches.contains_key("hotfix"));
        assert!(branches.contains_key("support"));
        assert!(branches.contains_key("unknown"));
    }

    #[test]
    fn defaults_main_branch_has_expected_behavior() {
        let branches = gitflow::defaults();
        let main = branches.get("main").expect("main branch exists");

        assert_eq!(main.increment, Some(IncrementStrategy::Patch));
        assert_eq!(main.is_main_branch, Some(true));
        assert_eq!(main.label.as_deref(), Some(""));
    }

    #[test]
    fn defaults_develop_branch_tracks_release_branches() {
        let branches = gitflow::defaults();
        let develop = branches.get("develop").expect("develop branch exists");

        assert_eq!(develop.increment, Some(IncrementStrategy::Minor));
        assert_eq!(develop.label.as_deref(), Some("alpha"));
        assert_eq!(develop.tracks_release_branches, Some(true));
        assert_eq!(develop.track_merge_target, Some(true));
    }

    #[test]
    fn defaults_release_branch_is_marked_release_and_manual() {
        let branches = gitflow::defaults();
        let release = branches.get("release").expect("release branch exists");

        assert_eq!(release.increment, Some(IncrementStrategy::Minor));
        assert_eq!(release.label.as_deref(), Some("beta"));
        assert_eq!(release.is_release_branch, Some(true));
        assert_eq!(
            release.deployment_mode,
            Some(DeploymentMode::ManualDeployment)
        );
    }

    #[test]
    fn defaults_generic_branches_use_base_configuration() {
        let branches = gitflow::defaults();
        let feature = branches.get("feature").expect("feature branch exists");

        assert_eq!(feature.increment, Some(IncrementStrategy::Inherit));
        assert!(feature.label.is_none());
        assert!(feature.is_main_branch.is_none());
        assert!(feature.is_release_branch.is_none());
    }
}
