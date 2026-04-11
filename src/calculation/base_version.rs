use crate::git::git2_impl::commit::Git2Commit;
use crate::semver::{IncrementMode, SemanticVersion, VersionField};

#[derive(Debug, Clone)]
pub struct BaseVersionOperand {
    pub source: String,
    pub semantic_version: SemanticVersion,
    pub base_version_source: Option<Git2Commit>,
}

#[derive(Debug, Clone)]
pub struct BaseVersionOperator {
    pub source: String,
    pub base_version_source: Option<Git2Commit>,
    pub increment: VersionField,
    pub force_increment: bool,
    pub label: Option<String>,
    pub alternative_semantic_version: Option<SemanticVersion>,
}

#[derive(Debug, Clone)]
pub struct BaseVersion {
    pub operand: BaseVersionOperand,
    pub operator: Option<BaseVersionOperator>,
}

impl BaseVersion {
    pub fn get_incremented_version(&self) -> SemanticVersion {
        if let Some(op) = &self.operator {
            self.operand.semantic_version.increment(
                op.increment,
                op.label.as_deref(),
                if op.force_increment {
                    IncrementMode::Force
                } else {
                    IncrementMode::Standard
                },
                &[],
            )
        } else {
            self.operand.semantic_version.clone()
        }
    }

    pub fn apply(mut self, operator: BaseVersionOperator) -> Self {
        self.operator = Some(operator);
        self
    }
}
