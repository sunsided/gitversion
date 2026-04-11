use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct ContinuousDeliveryCalculator;

impl ContinuousDeliveryCalculator {
    pub fn calculate(
        &self,
        mut version: SemanticVersion,
        commits_since_tag: i64,
    ) -> SemanticVersion {
        if version.pre_release_tag.has_tag() {
            let base = version.pre_release_tag.number.unwrap_or(1);
            version.pre_release_tag.number = Some(base + commits_since_tag.saturating_sub(1));
        }
        version
    }
}
