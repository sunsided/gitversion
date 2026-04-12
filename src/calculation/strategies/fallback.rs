use crate::calculation::base_version::{BaseVersion, BaseVersionOperand};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct FallbackVersionStrategy;

impl VersionStrategy for FallbackVersionStrategy {
    fn get_base_versions(
        &self,
        _ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        vec![BaseVersion {
            operand: BaseVersionOperand {
                source: "Fallback base version".to_string(),
                semantic_version: SemanticVersion::new(0, 0, 0),
                base_version_source: None,
            },
            operator: None,
        }]
    }
}

#[cfg(test)]
mod tests {
    use crate::calculation::effective_branch::EffectiveBranchConfigurationFinder;
    use crate::calculation::strategies::VersionStrategy;
    use crate::calculation::strategies::fallback::FallbackVersionStrategy;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::context::GitVersionContext;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn always_returns_zero_zero_zero_base_version() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = FallbackVersionStrategy.get_base_versions(&ctx, &effective);

        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].operand.semantic_version.to_string(), "0.0.0");
        assert!(versions[0].operator.is_none());
    }
}
