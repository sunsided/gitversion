use std::collections::HashMap;

use crate::calculation::base_version::{BaseVersion, BaseVersionOperand, BaseVersionOperator};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::config::enums::IncrementStrategy;
use crate::context::GitVersionContext;
use crate::semver::{SemanticVersion, VersionField};

#[derive(Debug, Default)]
pub struct TaggedCommitVersionStrategy;

impl VersionStrategy for TaggedCommitVersionStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        let head = match ctx
            .repository
            .repo
            .head()
            .and_then(|head| head.peel_to_commit())
        {
            Ok(commit) => commit,
            Err(_) => return Vec::new(),
        };

        let tagged_versions = parse_semver_tags_by_commit(ctx);
        let mut revwalk = match ctx.repository.repo.revwalk() {
            Ok(revwalk) => revwalk,
            Err(_) => return Vec::new(),
        };
        if revwalk.push(head.id()).is_err() {
            return Vec::new();
        }

        for oid in revwalk.flatten() {
            let key = oid.to_string();
            let Some(version) = tagged_versions.get(&key) else {
                continue;
            };

            let source_commit = ctx
                .repository
                .repo
                .find_commit(oid)
                .ok()
                .map(|commit| crate::git::git2_impl::commit::Git2Commit::from_git2(&commit));

            if oid == head.id() {
                return vec![BaseVersion {
                    operand: BaseVersionOperand {
                        source: format!("Tag {}", version),
                        semantic_version: version.clone(),
                        base_version_source: source_commit,
                    },
                    operator: None,
                }];
            }

            let increment = increment_from_branch(config);
            return vec![BaseVersion {
                operand: BaseVersionOperand {
                    source: format!("Tagged commit base from {}", version),
                    semantic_version: version.clone(),
                    base_version_source: source_commit,
                },
                operator: Some(BaseVersionOperator {
                    source: "Increment from branch config".to_string(),
                    base_version_source: None,
                    increment,
                    force_increment: true,
                    label: None,
                    alternative_semantic_version: None,
                }),
            }];
        }

        Vec::new()
    }
}

fn increment_from_branch(config: &EffectiveBranchConfiguration) -> VersionField {
    match config.branch.increment.unwrap_or(IncrementStrategy::Patch) {
        IncrementStrategy::Major => VersionField::Major,
        IncrementStrategy::Minor => VersionField::Minor,
        IncrementStrategy::Patch | IncrementStrategy::Inherit => VersionField::Patch,
        IncrementStrategy::None => VersionField::None,
    }
}

fn parse_semver_tags_by_commit(ctx: &GitVersionContext) -> HashMap<String, SemanticVersion> {
    let mut versions = HashMap::new();
    if let Ok(tags) = ctx.repository.tags() {
        for tag in tags {
            let friendly = tag.name.friendly();
            let Some(version) = SemanticVersion::try_parse(
                &friendly,
                Some(&ctx.configuration.tag_prefix_pattern),
                ctx.configuration.semantic_version_format,
            ) else {
                continue;
            };

            match versions.get(&tag.commit_sha) {
                Some(existing) if existing >= &version => {}
                _ => {
                    versions.insert(tag.commit_sha, version);
                }
            }
        }
    }
    versions
}

#[cfg(test)]
mod tests {
    use crate::calculation::effective_branch::EffectiveBranchConfigurationFinder;
    use crate::calculation::strategies::VersionStrategy;
    use crate::calculation::strategies::tagged_commit::TaggedCommitVersionStrategy;
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
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].get_incremented_version().to_string(), "1.0.1");
    }

    #[test]
    fn prefers_highest_tag_when_multiple_tags_exist_on_ancestor_commit() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture.apply_tag("1.0.0").expect("tag");
        fixture.apply_tag("1.2.0").expect("tag");
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
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].get_incremented_version().to_string(), "1.2.1");
    }
}
