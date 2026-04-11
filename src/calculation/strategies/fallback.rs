use crate::calculation::base_version::{BaseVersion, BaseVersionOperand};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct FallbackVersionStrategy;

impl VersionStrategy for FallbackVersionStrategy {
    fn get_base_versions(
        &self,
        _ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        vec![BaseVersion {
            operand: BaseVersionOperand {
                source: "Fallback base version".to_string(),
                semantic_version: SemanticVersion::new(0, 0, 0),
                base_version_source: None,
            },
            operator: None,
        }]
    }
}
