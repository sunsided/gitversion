use crate::calculation::base_version::{BaseVersion, BaseVersionOperand};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::regex_patterns::VERSION_IN_BRANCH;
use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct VersionInBranchNameStrategy;

impl VersionStrategy for VersionInBranchNameStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        let friendly = ctx.current_branch.name.friendly();
        VERSION_IN_BRANCH
            .captures(&friendly)
            .and_then(|m| m.name("version").map(|v| v.as_str().to_string()))
            .and_then(|v| {
                SemanticVersion::try_parse(
                    &v,
                    Some(&ctx.configuration.tag_prefix_pattern),
                    ctx.configuration.semantic_version_format,
                )
            })
            .map(|semantic_version| {
                vec![BaseVersion {
                    operand: BaseVersionOperand {
                        source: "Version in branch name".to_string(),
                        semantic_version,
                        base_version_source: Some(ctx.current_commit.clone()),
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
    use crate::calculation::strategies::version_in_branch_name::VersionInBranchNameStrategy;
    use crate::calculation::strategies::VersionStrategy;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::context::GitVersionContext;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn extracts_version_from_branch_name() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture.branch_to("release/4.5.6").expect("branch");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = VersionInBranchNameStrategy.get_base_versions(&ctx, &effective);

        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].operand.semantic_version.to_string(), "4.5.6");
    }

    #[test]
    fn returns_no_versions_when_branch_has_no_semver() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture.branch_to("feature/no-version").expect("branch");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = VersionInBranchNameStrategy.get_base_versions(&ctx, &effective);
        assert!(versions.is_empty());
    }
}
