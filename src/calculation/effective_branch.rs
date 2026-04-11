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
