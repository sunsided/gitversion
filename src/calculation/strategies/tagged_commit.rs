use crate::calculation::base_version::{BaseVersion, BaseVersionOperand};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct TaggedCommitVersionStrategy;

impl VersionStrategy for TaggedCommitVersionStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        let current_sha = ctx.current_commit.sha();
        let mut out = Vec::new();
        if let Ok(tags) = ctx.repository.tags() {
            for tag in tags {
                if tag.commit_sha != current_sha {
                    continue;
                }
                let name = tag.name.friendly();
                if let Some(version) = SemanticVersion::try_parse(
                    &name,
                    Some(&ctx.configuration.tag_prefix_pattern),
                    ctx.configuration.semantic_version_format,
                ) {
                    out.push(BaseVersion {
                        operand: BaseVersionOperand {
                            source: format!("Tag {}", tag.name.friendly()),
                            semantic_version: version,
                            base_version_source: Some(ctx.current_commit.clone()),
                        },
                        operator: None,
                    });
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use crate::calculation::effective_branch::EffectiveBranchConfigurationFinder;
    use crate::calculation::strategies::tagged_commit::TaggedCommitVersionStrategy;
    use crate::calculation::strategies::VersionStrategy;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::context::GitVersionContext;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn returns_semver_tag_on_current_commit() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture.apply_tag("1.2.3").expect("tag");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = TaggedCommitVersionStrategy.get_base_versions(&ctx, &effective);

        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].operand.semantic_version.to_string(), "1.2.3");
    }

    #[test]
    fn ignores_non_semver_tags_and_tags_on_other_commits() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture.apply_tag("not-a-version").expect("tag");
        fixture.apply_tag("1.0.0").expect("tag");
        fixture.make_a_commit("second commit").expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = TaggedCommitVersionStrategy.get_base_versions(&ctx, &effective);
        assert!(versions.is_empty());
    }
}
