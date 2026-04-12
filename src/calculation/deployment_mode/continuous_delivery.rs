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

#[cfg(test)]
mod tests {
    use crate::calculation::deployment_mode::continuous_delivery::ContinuousDeliveryCalculator;
    use crate::config::enums::SemanticVersionFormat;
    use crate::semver::SemanticVersion;

    #[test]
    fn increments_existing_pre_release_number_by_commit_distance_minus_one() {
        let version = SemanticVersion::parse("1.2.3-alpha.4", None, SemanticVersionFormat::Strict)
            .expect("valid semver");

        let calculated = ContinuousDeliveryCalculator.calculate(version, 3);

        assert_eq!(calculated.to_string(), "1.2.3-alpha.6");
    }

    #[test]
    fn leaves_release_version_unchanged_when_no_pre_release_tag_exists() {
        let version = SemanticVersion::new(1, 2, 3);

        let calculated = ContinuousDeliveryCalculator.calculate(version, 5);

        assert_eq!(calculated.to_string(), "1.2.3");
    }

    #[test]
    fn initializes_pre_release_number_to_one_when_missing() {
        let version = SemanticVersion::parse("1.2.3-alpha", None, SemanticVersionFormat::Strict)
            .expect("valid semver");

        let calculated = ContinuousDeliveryCalculator.calculate(version, 1);

        assert_eq!(calculated.to_string(), "1.2.3-alpha.1");
    }

    #[test]
    fn commit_distance_zero_reduces_pre_release_number_by_one() {
        let version = SemanticVersion::parse("1.2.3-alpha.3", None, SemanticVersionFormat::Strict)
            .expect("valid semver");

        let calculated = ContinuousDeliveryCalculator.calculate(version, 0);

        assert_eq!(calculated.to_string(), "1.2.3-alpha.2");
    }
}
