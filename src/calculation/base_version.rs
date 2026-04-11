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

#[cfg(test)]
mod tests {
    use super::{BaseVersion, BaseVersionOperand, BaseVersionOperator};
    use crate::semver::{SemanticVersion, VersionField};

    #[test]
    fn get_incremented_version_returns_operand_version_when_operator_missing() {
        let base = BaseVersion {
            operand: BaseVersionOperand {
                source: "seed".to_string(),
                semantic_version: SemanticVersion::new(1, 2, 3),
                base_version_source: None,
            },
            operator: None,
        };

        let incremented = base.get_incremented_version();

        assert_eq!(incremented.to_string(), "1.2.3");
    }

    #[test]
    fn apply_sets_operator_on_base_version() {
        let base = BaseVersion {
            operand: BaseVersionOperand {
                source: "seed".to_string(),
                semantic_version: SemanticVersion::new(1, 0, 0),
                base_version_source: None,
            },
            operator: None,
        };
        let operator = BaseVersionOperator {
            source: "operator".to_string(),
            base_version_source: None,
            increment: VersionField::Minor,
            force_increment: false,
            label: None,
            alternative_semantic_version: None,
        };

        let applied = base.apply(operator.clone());

        assert_eq!(applied.operator.expect("operator").source, operator.source);
    }

    #[test]
    fn standard_increment_with_existing_pre_release_increments_pre_release_number() {
        let base = BaseVersion {
            operand: BaseVersionOperand {
                source: "seed".to_string(),
                semantic_version: SemanticVersion::parse(
                    "1.2.3-beta.4",
                    None,
                    crate::config::enums::SemanticVersionFormat::Strict,
                )
                .expect("valid semver"),
                base_version_source: None,
            },
            operator: Some(BaseVersionOperator {
                source: "operator".to_string(),
                base_version_source: None,
                increment: VersionField::Patch,
                force_increment: false,
                label: None,
                alternative_semantic_version: None,
            }),
        };

        let incremented = base.get_incremented_version();

        assert_eq!(incremented.to_string(), "1.2.3-beta.5");
    }

    #[test]
    fn force_increment_uses_version_field_even_with_existing_pre_release() {
        let base = BaseVersion {
            operand: BaseVersionOperand {
                source: "seed".to_string(),
                semantic_version: SemanticVersion::parse(
                    "1.2.3-beta.4",
                    None,
                    crate::config::enums::SemanticVersionFormat::Strict,
                )
                .expect("valid semver"),
                base_version_source: None,
            },
            operator: Some(BaseVersionOperator {
                source: "operator".to_string(),
                base_version_source: None,
                increment: VersionField::Patch,
                force_increment: true,
                label: None,
                alternative_semantic_version: None,
            }),
        };

        let incremented = base.get_incremented_version();

        assert_eq!(incremented.to_string(), "1.2.4-beta.1");
        assert_eq!(incremented.pre_release_tag.number, Some(1));
    }

    #[test]
    fn increment_applies_label_when_provided_by_operator() {
        let base = BaseVersion {
            operand: BaseVersionOperand {
                source: "seed".to_string(),
                semantic_version: SemanticVersion::new(1, 2, 3),
                base_version_source: None,
            },
            operator: Some(BaseVersionOperator {
                source: "operator".to_string(),
                base_version_source: None,
                increment: VersionField::Minor,
                force_increment: true,
                label: Some("alpha".to_string()),
                alternative_semantic_version: None,
            }),
        };

        let incremented = base.get_incremented_version();

        assert_eq!(incremented.to_string(), "1.3.0-alpha.1");
    }

    #[test]
    fn increment_with_none_and_label_creates_labeled_pre_release() {
        let base = BaseVersion {
            operand: BaseVersionOperand {
                source: "seed".to_string(),
                semantic_version: SemanticVersion::new(2, 0, 0),
                base_version_source: None,
            },
            operator: Some(BaseVersionOperator {
                source: "operator".to_string(),
                base_version_source: None,
                increment: VersionField::None,
                force_increment: false,
                label: Some("beta".to_string()),
                alternative_semantic_version: None,
            }),
        };

        let incremented = base.get_incremented_version();

        assert_eq!(incremented.to_string(), "2.0.0-beta.1");
    }
}
