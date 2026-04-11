use crate::calculation::base_version::BaseVersion;
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;

#[derive(Debug, Default)]
pub struct TrackReleaseBranchesVersionStrategy;

impl VersionStrategy for TrackReleaseBranchesVersionStrategy {
    fn get_base_versions(
        &self,
        _ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        Vec::new()
    }
}
