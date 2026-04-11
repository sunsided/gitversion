pub mod context;
pub mod enrichers;
pub mod iteration;
pub mod non_trunk;
pub mod trunk;

use crate::calculation::base_version::BaseVersion;
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;

#[derive(Debug, Default)]
pub struct MainlineVersionStrategy;

impl VersionStrategy for MainlineVersionStrategy {
    fn get_base_versions(&self, _ctx: &GitVersionContext, _config: &EffectiveBranchConfiguration) -> Vec<BaseVersion> {
        Vec::new()
    }
}
