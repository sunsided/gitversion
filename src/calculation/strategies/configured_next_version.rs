use crate::calculation::base_version::{BaseVersion, BaseVersionOperand};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct ConfiguredNextVersionStrategy;

impl VersionStrategy for ConfiguredNextVersionStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        ctx.configuration
            .next_version
            .as_deref()
            .and_then(|v| {
                SemanticVersion::try_parse(
                    v,
                    Some(&ctx.configuration.tag_prefix_pattern),
                    ctx.configuration.semantic_version_format,
                )
            })
            .map(|version| {
                vec![BaseVersion {
                    operand: BaseVersionOperand {
                        source: "Configured next version".to_string(),
                        semantic_version: version,
                        base_version_source: None,
                    },
                    operator: None,
                }]
            })
            .unwrap_or_default()
    }
}
