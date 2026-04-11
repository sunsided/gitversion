use crate::calculation::base_version::{BaseVersion, BaseVersionOperand};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::git::merge_message::MergeMessage;

#[derive(Debug, Default)]
pub struct MergeMessageVersionStrategy;

impl VersionStrategy for MergeMessageVersionStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        MergeMessage::try_parse(&ctx.current_commit.message, &ctx.configuration)
            .and_then(|m| m.version)
            .map(|version| {
                vec![BaseVersion {
                    operand: BaseVersionOperand {
                        source: "Merge message".to_string(),
                        semantic_version: version,
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
    use crate::calculation::strategies::merge_message::MergeMessageVersionStrategy;
    use crate::calculation::strategies::VersionStrategy;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::context::GitVersionContext;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn extracts_version_from_merge_commit_message() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture
            .make_a_commit("Merge pull request #42 from release/3.1.0")
            .expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = MergeMessageVersionStrategy.get_base_versions(&ctx, &effective);

        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].operand.semantic_version.to_string(), "3.1.0");
    }

    #[test]
    fn returns_no_versions_for_non_merge_message() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("feat: add search").expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = MergeMessageVersionStrategy.get_base_versions(&ctx, &effective);
        assert!(versions.is_empty());
    }
}
