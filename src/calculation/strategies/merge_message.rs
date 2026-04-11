use crate::calculation::base_version::{BaseVersion, BaseVersionOperand};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::git::merge_message::MergeMessage;

#[derive(Debug, Default)]
pub struct MergeMessageVersionStrategy;

impl VersionStrategy for MergeMessageVersionStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        MergeMessage::try_parse(&ctx.current_commit.message, &ctx.configuration)
            .and_then(|m| m.version)
            .map(|version| {
                vec![BaseVersion {
                    operand: BaseVersionOperand {
                        source: "Merge message".to_string(),
                        semantic_version: version,
                        base_version_source: Some(ctx.current_commit.clone()),
                    },
                    operator: None,
                }]
            })
            .unwrap_or_default()
    }
}
