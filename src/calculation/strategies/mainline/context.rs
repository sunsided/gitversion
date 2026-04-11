use crate::semver::VersionField;

#[derive(Debug, Clone, Default)]
pub struct MainlineContext {
    pub increment: VersionField,
}
