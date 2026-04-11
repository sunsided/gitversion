pub mod configured_next_version;
pub mod fallback;
pub mod mainline;
pub mod merge_message;
pub mod tagged_commit;
pub mod track_release_branches;
pub mod version_in_branch_name;

use crate::calculation::base_version::BaseVersion;
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::context::GitVersionContext;

pub trait VersionStrategy {
    fn get_base_versions(&self, ctx: &GitVersionContext, config: &EffectiveBranchConfiguration) -> Vec<BaseVersion>;
}
