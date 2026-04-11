use crate::calculation::base_version::{BaseVersion, BaseVersionOperand};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::regex_patterns::VERSION_IN_BRANCH;
use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct VersionInBranchNameStrategy;

impl VersionStrategy for VersionInBranchNameStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        let friendly = ctx.current_branch.name.friendly();
        VERSION_IN_BRANCH
            .captures(&friendly)
            .and_then(|m| m.name("version").map(|v| v.as_str().to_string()))
            .and_then(|v| {
                SemanticVersion::try_parse(
                    &v,
                    Some(&ctx.configuration.tag_prefix_pattern),
                    ctx.configuration.semantic_version_format,
                )
            })
            .map(|semantic_version| {
                vec![BaseVersion {
                    operand: BaseVersionOperand {
                        source: "Version in branch name".to_string(),
                        semantic_version,
                        base_version_source: Some(ctx.current_commit.clone()),
                    },
                    operator: None,
                }]
            })
            .unwrap_or_default()
    }
}
