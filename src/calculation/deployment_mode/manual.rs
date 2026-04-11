use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct ManualDeploymentCalculator;

impl ManualDeploymentCalculator {
    pub fn calculate(&self, version: SemanticVersion) -> SemanticVersion {
        version
    }
}
