use crate::calculation::base_version::{BaseVersion, BaseVersionOperand};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct TaggedCommitVersionStrategy;

impl VersionStrategy for TaggedCommitVersionStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        let current_sha = ctx.current_commit.sha();
        let mut out = Vec::new();
        if let Ok(tags) = ctx.repository.tags() {
            for tag in tags {
                if tag.commit_sha != current_sha {
                    continue;
                }
                let name = tag.name.friendly();
                if let Some(version) = SemanticVersion::try_parse(
                    &name,
                    Some(&ctx.configuration.tag_prefix_pattern),
                    ctx.configuration.semantic_version_format,
                ) {
                    out.push(BaseVersion {
                        operand: BaseVersionOperand {
                            source: format!("Tag {}", tag.name.friendly()),
                            semantic_version: version,
                            base_version_source: Some(ctx.current_commit.clone()),
                        },
                        operator: None,
                    });
                }
            }
        }
        out
    }
}
