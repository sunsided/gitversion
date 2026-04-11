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
