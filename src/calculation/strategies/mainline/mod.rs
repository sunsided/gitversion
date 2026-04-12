pub mod context;
pub mod enrichers;
pub mod iteration;
pub mod non_trunk;
pub mod trunk;

use std::collections::HashMap;

use crate::calculation::base_version::{BaseVersion, BaseVersionOperand, BaseVersionOperator};
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::semver::{SemanticVersion, VersionField};

#[derive(Debug, Default)]
pub struct MainlineVersionStrategy;

impl VersionStrategy for MainlineVersionStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        _config: &EffectiveBranchConfiguration,
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
            if let Some(version) = tagged_versions.get(&key) {
                let source_commit =
                    ctx.repository.repo.find_commit(oid).ok().map(|commit| {
                        crate::git::git2_impl::commit::Git2Commit::from_git2(&commit)
                    });

                if oid == head.id() {
                    return vec![BaseVersion {
                        operand: BaseVersionOperand {
                            source: format!("Mainline tag {}", version),
                            semantic_version: version.clone(),
                            base_version_source: source_commit,
                        },
                        operator: None,
                    }];
                }

                return vec![BaseVersion {
                    operand: BaseVersionOperand {
                        source: format!("Mainline base from tag {}", version),
                        semantic_version: version.clone(),
                        base_version_source: source_commit,
                    },
                    operator: Some(BaseVersionOperator {
                        source: "Mainline commit after tagged commit".to_string(),
                        base_version_source: None,
                        increment: VersionField::Patch,
                        force_increment: true,
                        label: None,
                        alternative_semantic_version: None,
                    }),
                }];
            }
        }

        vec![BaseVersion {
            operand: BaseVersionOperand {
                source: "Mainline fallback".to_string(),
                semantic_version: SemanticVersion::new(0, 0, 0),
                base_version_source: None,
            },
            operator: Some(BaseVersionOperator {
                source: "Mainline default increment".to_string(),
                base_version_source: None,
                increment: VersionField::Patch,
                force_increment: true,
                label: None,
                alternative_semantic_version: None,
            }),
        }]
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
    use crate::calculation::strategies::mainline::MainlineVersionStrategy;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::context::GitVersionContext;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn returns_tag_version_when_head_is_tagged() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture.apply_tag("1.2.3").expect("tag");

        let repo = Git2Repository::open(fixture.path()).expect("open repo");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = MainlineVersionStrategy.get_base_versions(&ctx, &effective);
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].operand.semantic_version.to_string(), "1.2.3");
    }

    #[test]
    fn increments_patch_when_head_is_after_tagged_commit() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture.apply_tag("1.2.3").expect("tag");
        fixture.make_a_commit("next commit").expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repo");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = MainlineVersionStrategy.get_base_versions(&ctx, &effective);
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].get_incremented_version().to_string(), "1.2.4");
    }

    #[test]
    fn uses_highest_semver_when_multiple_tags_point_to_same_commit() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture.apply_tag("1.2.3").expect("tag 1.2.3");
        fixture.apply_tag("1.3.0").expect("tag 1.3.0");

        let repo = Git2Repository::open(fixture.path()).expect("open repo");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = MainlineVersionStrategy.get_base_versions(&ctx, &effective);

        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].operand.semantic_version.to_string(), "1.3.0");
        assert!(versions[0].operator.is_none());
    }

    #[test]
    fn falls_back_to_zero_zero_one_when_no_tags_exist() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repo");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = MainlineVersionStrategy.get_base_versions(&ctx, &effective);

        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].operand.semantic_version.to_string(), "0.0.0");
        assert_eq!(versions[0].get_incremented_version().to_string(), "0.0.1");
    }
}
