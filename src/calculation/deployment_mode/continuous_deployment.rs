use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct ContinuousDeploymentCalculator;

impl ContinuousDeploymentCalculator {
    pub fn calculate(
        &self,
        mut version: SemanticVersion,
        commits_since_tag: i64,
    ) -> SemanticVersion {
        version.pre_release_tag = Default::default();
        version.build_metadata.version_source_distance = commits_since_tag;
        version
    }
}

#[cfg(test)]
mod tests {
    use crate::calculation::deployment_mode::continuous_deployment::ContinuousDeploymentCalculator;
    use crate::config::enums::SemanticVersionFormat;
    use crate::semver::SemanticVersion;

    #[test]
    fn strips_pre_release_tag_and_sets_version_source_distance() {
        let version = SemanticVersion::parse("2.0.0-beta.7", None, SemanticVersionFormat::Strict)
            .expect("valid semver");

        let calculated = ContinuousDeploymentCalculator.calculate(version, 9);

        assert_eq!(calculated.to_string(), "2.0.0");
        assert_eq!(calculated.build_metadata.version_source_distance, 9);
    }
}
