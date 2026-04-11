use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct ManualDeploymentCalculator;

impl ManualDeploymentCalculator {
    pub fn calculate(&self, version: SemanticVersion) -> SemanticVersion {
        version
    }
}

#[cfg(test)]
mod tests {
    use crate::calculation::deployment_mode::manual::ManualDeploymentCalculator;
    use crate::config::enums::SemanticVersionFormat;
    use crate::semver::SemanticVersion;

    #[test]
    fn returns_input_version_unchanged() {
        let version = SemanticVersion::parse(
            "1.2.3-beta.4+5.main.deadbee",
            None,
            SemanticVersionFormat::Strict,
        )
        .expect("valid semver");

        let calculated = ManualDeploymentCalculator.calculate(version.clone());

        assert_eq!(calculated, version);
    }
}
