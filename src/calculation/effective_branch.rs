use crate::config::branch_config::BranchConfiguration;
use crate::config::enums::IncrementStrategy;
use crate::config::gitversion_config::GitVersionConfiguration;
use crate::git::git2_impl::branch::Git2Branch;

#[derive(Debug, Clone)]
pub struct EffectiveBranchConfiguration {
    pub branch: BranchConfiguration,
}

#[derive(Debug, Default)]
pub struct EffectiveBranchConfigurationFinder;

impl EffectiveBranchConfigurationFinder {
    pub fn get_configurations(
        &self,
        branch: &Git2Branch,
        config: &GitVersionConfiguration,
    ) -> Vec<EffectiveBranchConfiguration> {
        let mut current = config.get_branch_configuration(&branch.name);
        if current.increment == Some(IncrementStrategy::Inherit) {
            current = current.inherit(&config.branch_defaults);
        }
        vec![EffectiveBranchConfiguration { branch: current }]
    }
}

#[cfg(test)]
mod tests {
    use super::EffectiveBranchConfigurationFinder;
    use crate::config::branch_config::BranchConfiguration;
    use crate::config::enums::IncrementStrategy;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::git::git2_impl::branch::Git2Branch;
    use crate::git::reference_name::ReferenceName;

    fn branch(name: &str) -> Git2Branch {
        Git2Branch {
            name: ReferenceName::from_branch_name(name),
            tip_sha: None,
            remote: false,
            tracking: false,
            detached_head: false,
        }
    }

    #[test]
    fn preserves_inherit_increment_after_resolution() {
        let mut config = GitVersionConfiguration::default();
        config.branch_defaults = BranchConfiguration {
            increment: Some(IncrementStrategy::Minor),
            label: Some("default".to_string()),
            ..Default::default()
        };
        config.branches.insert(
            "feature".to_string(),
            BranchConfiguration {
                increment: Some(IncrementStrategy::Inherit),
                label: Some("alpha".to_string()),
                ..Default::default()
            },
        );

        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&branch("feature/new-api"), &config)
            .pop()
            .expect("effective branch config");

        assert_eq!(effective.branch.increment, Some(IncrementStrategy::Inherit));
        assert_eq!(effective.branch.label.as_deref(), Some("alpha"));
    }

    #[test]
    fn keeps_non_inherit_increment_as_is() {
        let mut config = GitVersionConfiguration::default();
        config.branch_defaults.increment = Some(IncrementStrategy::Patch);
        config.branches.insert(
            "release".to_string(),
            BranchConfiguration {
                increment: Some(IncrementStrategy::Major),
                ..Default::default()
            },
        );

        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&branch("release/2.0.0"), &config)
            .pop()
            .expect("effective branch config");

        assert_eq!(effective.branch.increment, Some(IncrementStrategy::Major));
    }

    #[test]
    fn uses_fallback_configuration_when_branch_does_not_match() {
        let mut config = GitVersionConfiguration::default();
        config.branch_defaults.increment = Some(IncrementStrategy::Patch);
        config.branches.insert(
            "unknown".to_string(),
            BranchConfiguration {
                increment: Some(IncrementStrategy::None),
                label: Some("ci".to_string()),
                ..Default::default()
            },
        );

        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&branch("bugfix/123"), &config)
            .pop()
            .expect("effective branch config");

        assert_eq!(effective.branch.increment, Some(IncrementStrategy::None));
        assert_eq!(effective.branch.label.as_deref(), Some("ci"));
    }

    #[test]
    fn returns_single_effective_configuration() {
        let config = GitVersionConfiguration::default();

        let configs =
            EffectiveBranchConfigurationFinder.get_configurations(&branch("main"), &config);

        assert_eq!(configs.len(), 1);
    }
}
