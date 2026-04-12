use crate::calculation::base_version::{BaseVersion, BaseVersionOperand};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct ConfiguredNextVersionStrategy;

impl VersionStrategy for ConfiguredNextVersionStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        ctx.configuration
            .next_version
            .as_deref()
            .and_then(|v| {
                SemanticVersion::try_parse(
                    v,
                    Some(&ctx.configuration.tag_prefix_pattern),
                    ctx.configuration.semantic_version_format,
                )
            })
            .map(|version| {
                vec![BaseVersion {
                    operand: BaseVersionOperand {
                        source: "Configured next version".to_string(),
                        semantic_version: version,
                        base_version_source: None,
                    },
                    operator: None,
                }]
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::calculation::effective_branch::EffectiveBranchConfigurationFinder;
    use crate::calculation::strategies::VersionStrategy;
    use crate::calculation::strategies::configured_next_version::ConfiguredNextVersionStrategy;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::context::GitVersionContext;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn returns_version_from_configuration_when_present() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let config = GitVersionConfiguration {
            next_version: Some("2.4.6".to_string()),
            ..GitVersionConfiguration::default()
        };
        let ctx = GitVersionContext::from_repository(repo, config).expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = ConfiguredNextVersionStrategy.get_base_versions(&ctx, &effective);

        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].operand.semantic_version.to_string(), "2.4.6");
    }

    #[test]
    fn returns_no_versions_when_next_version_is_missing_or_invalid() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let config = GitVersionConfiguration {
            next_version: Some("not-a-semver".to_string()),
            ..GitVersionConfiguration::default()
        };
        let ctx = GitVersionContext::from_repository(repo, config).expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = ConfiguredNextVersionStrategy.get_base_versions(&ctx, &effective);
        assert!(versions.is_empty());
    }
}
