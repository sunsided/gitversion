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
    use rstest::rstest;

    use crate::calculation::effective_branch::EffectiveBranchConfigurationFinder;
    use crate::calculation::strategies::merge_message::MergeMessageVersionStrategy;
    use crate::calculation::strategies::VersionStrategy;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::context::GitVersionContext;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    fn strategy_output_for_commit_message(
        message: &str,
    ) -> Vec<crate::calculation::base_version::BaseVersion> {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit(message).expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        MergeMessageVersionStrategy.get_base_versions(&ctx, &effective)
    }

    #[rstest]
    #[case("Merge pull request #42 from release/3.1.0", Some("3.1.0"))]
    #[case("Merge branch 'release/4.2.1' into main", Some("4.2.1"))]
    #[case("Merged PR 456: Merge release/7.8.9 to develop", Some("7.8.9"))]
    #[case("Merge pull request #100 from feature/add-search", None)]
    #[case("feat: add search", None)]
    fn extracts_version_only_when_merge_source_contains_semver(
        #[case] message: &str,
        #[case] expected: Option<&str>,
    ) {
        let versions = strategy_output_for_commit_message(message);

        let actual = versions
            .first()
            .map(|version| version.operand.semantic_version.to_string());
        assert_eq!(actual.as_deref(), expected);

        if expected.is_some() {
            assert_eq!(versions.len(), 1);
        } else {
            assert!(versions.is_empty());
        }
    }
}
