use std::collections::HashMap;

use regex::Regex;

use crate::calculation::base_version::BaseVersion;
use crate::calculation::base_version::BaseVersionOperand;
use crate::calculation::base_version::BaseVersionOperator;
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::config::enums::IncrementStrategy;
use crate::context::GitVersionContext;
use crate::semver::{SemanticVersion, VersionField};

#[derive(Debug, Default)]
pub struct TrackReleaseBranchesVersionStrategy;

impl VersionStrategy for TrackReleaseBranchesVersionStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        let current_friendly = ctx.current_branch.name.friendly();
        let inherit_feature_tracking = (current_friendly.starts_with("feature/")
            || current_friendly.starts_with("pull-request/"))
            && matches!(
                config.branch.increment,
                Some(IncrementStrategy::Minor | IncrementStrategy::Inherit)
            );
        if !config.branch.tracks_release_branches.unwrap_or(false) && !inherit_feature_tracking {
            return Vec::new();
        }

        let release_regexes = ctx
            .configuration
            .branches
            .iter()
            .filter(|(_, branch_config)| branch_config.is_release_branch.unwrap_or(false))
            .filter_map(|(_, branch_config)| branch_config.regular_expression.as_deref())
            .filter_map(|pattern| Regex::new(pattern).ok())
            .collect::<Vec<_>>();

        let mut versions = Vec::new();
        let tagged_versions = parse_semver_tags_by_commit(ctx);
        let develop_tip = branch_tip_sha(ctx, "develop");
        let current_tip = ctx.current_branch.tip_sha.clone();

        if let Ok(branches) = ctx.repository.branches() {
            for branch in branches {
                let friendly = branch.name.friendly();
                if !is_release_branch(&friendly, &release_regexes) {
                    continue;
                }

                let Some(version) = extract_version_from_branch_name(
                    &friendly,
                    &ctx.configuration.tag_prefix_pattern,
                    ctx.configuration.semantic_version_format,
                ) else {
                    continue;
                };

                let base_version_source = branch.tip_sha.as_deref().and_then(|sha| {
                    git2::Oid::from_str(sha)
                        .ok()
                        .and_then(|oid| ctx.repository.repo.find_commit(oid).ok())
                        .map(|commit| crate::git::git2_impl::commit::Git2Commit::from_git2(&commit))
                });

                let increment = increment_for_tracked_release(
                    config,
                    &current_friendly,
                    current_tip.as_deref(),
                    branch.tip_sha.as_deref(),
                    develop_tip.as_deref(),
                    ctx,
                );

                versions.push(BaseVersion {
                    operand: BaseVersionOperand {
                        source: format!("Release branch {friendly}"),
                        semantic_version: version,
                        base_version_source,
                    },
                    operator: Some(BaseVersionOperator {
                        source: "Increment from tracking branch config".to_string(),
                        base_version_source: None,
                        increment,
                        force_increment: true,
                        label: None,
                        alternative_semantic_version: None,
                    }),
                });
            }
        }

        if versions.is_empty()
            && config.branch.tracks_release_branches.unwrap_or(false)
            && let Some(main_tip) = find_main_tip_sha(ctx)
            && let Some(main_version) =
                nearest_version_on_commit_chain(ctx, &main_tip, &tagged_versions)
        {
            versions.push(BaseVersion {
                operand: BaseVersionOperand {
                    source: "Main branch tagged version while tracking releases".to_string(),
                    semantic_version: main_version,
                    base_version_source: None,
                },
                operator: Some(BaseVersionOperator {
                    source: "Increment from tracking branch config".to_string(),
                    base_version_source: None,
                    increment: increment_from_branch(config),
                    force_increment: true,
                    label: None,
                    alternative_semantic_version: None,
                }),
            });
        }

        versions
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

fn increment_for_tracked_release(
    config: &EffectiveBranchConfiguration,
    current_branch: &str,
    current_tip: Option<&str>,
    release_tip: Option<&str>,
    develop_tip: Option<&str>,
    ctx: &GitVersionContext,
) -> VersionField {
    if current_branch.starts_with("feature/") {
        let current_from_develop = matches!(
            (current_tip, develop_tip),
            (Some(current), Some(develop))
                if ctx
                    .repository
                    .find_merge_base(current, develop)
                    .ok()
                    .flatten()
                    .as_deref()
                    == Some(develop)
        );

        if current_from_develop {
            return VersionField::Minor;
        }

        let current_from_release = matches!(
            (current_tip, release_tip),
            (Some(current), Some(release))
                if ctx
                    .repository
                    .find_merge_base(current, release)
                    .ok()
                    .flatten()
                    .as_deref()
                    == Some(release)
        );
        if current_from_release {
            return VersionField::None;
        }
    }

    increment_from_branch(config)
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

fn nearest_version_on_commit_chain(
    ctx: &GitVersionContext,
    start_sha: &str,
    tags_by_commit: &HashMap<String, SemanticVersion>,
) -> Option<SemanticVersion> {
    let start = git2::Oid::from_str(start_sha).ok()?;
    let mut revwalk = ctx.repository.repo.revwalk().ok()?;
    revwalk.push(start).ok()?;
    for oid in revwalk.flatten() {
        if let Some(version) = tags_by_commit.get(&oid.to_string()) {
            return Some(version.clone());
        }
    }
    None
}

fn branch_tip_sha(ctx: &GitVersionContext, branch_name: &str) -> Option<String> {
    let target = format!("refs/heads/{branch_name}");
    ctx.repository
        .branches()
        .ok()?
        .into_iter()
        .find(|branch| branch.name.canonical == target)
        .and_then(|branch| branch.tip_sha)
}

fn find_main_tip_sha(ctx: &GitVersionContext) -> Option<String> {
    let mut main_keys = ctx
        .configuration
        .branches
        .iter()
        .filter(|(_, branch_config)| branch_config.is_main_branch.unwrap_or(false))
        .map(|(name, _)| name.clone())
        .collect::<Vec<_>>();
    if main_keys.is_empty() {
        main_keys.push("main".to_string());
        main_keys.push("master".to_string());
    }

    for key in main_keys {
        if let Some(tip) = branch_tip_sha(ctx, &key) {
            return Some(tip);
        }
    }
    None
}

fn is_release_branch(friendly_branch_name: &str, release_regexes: &[Regex]) -> bool {
    if release_regexes.is_empty() {
        return friendly_branch_name.starts_with("release/");
    }
    release_regexes
        .iter()
        .any(|regex| regex.is_match(friendly_branch_name))
}

fn extract_version_from_branch_name(
    friendly_branch_name: &str,
    tag_prefix_pattern: &str,
    format: crate::config::enums::SemanticVersionFormat,
) -> Option<SemanticVersion> {
    crate::regex_patterns::VERSION_IN_BRANCH
        .captures(friendly_branch_name)
        .and_then(|captures| {
            captures
                .name("version")
                .map(|capture| capture.as_str().to_string())
        })
        .and_then(|version| SemanticVersion::try_parse(&version, Some(tag_prefix_pattern), format))
}

#[cfg(test)]
mod tests {
    use crate::calculation::effective_branch::EffectiveBranchConfigurationFinder;
    use crate::calculation::strategies::VersionStrategy;
    use crate::calculation::strategies::track_release_branches::TrackReleaseBranchesVersionStrategy;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::config::workflows;
    use crate::context::GitVersionContext;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn returns_release_branch_versions_when_tracking_is_enabled() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        let initial_branch = {
            let repo = git2::Repository::open(fixture.path()).expect("repo");
            repo.head()
                .expect("head")
                .shorthand()
                .unwrap_or("master")
                .to_string()
        };
        fixture
            .branch_to("release/1.4.0")
            .expect("create release branch");
        fixture.make_a_commit("release commit").expect("commit");
        fixture
            .checkout(&initial_branch)
            .expect("checkout initial branch");
        fixture.branch_to("develop").expect("checkout develop");

        let repo = Git2Repository::open(fixture.path()).expect("open repo");
        let mut config = GitVersionConfiguration::default();
        config.branches = workflows::resolve(&config.workflow);
        let ctx = GitVersionContext::from_repository(repo, config).expect("context");
        let effective = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .expect("effective configuration");

        let versions = TrackReleaseBranchesVersionStrategy.get_base_versions(&ctx, &effective);
        assert!(versions.iter().any(|version| {
            version.operand.semantic_version.major == 1
                && version.operand.semantic_version.minor == 4
                && version.operand.semantic_version.patch == 0
        }));
    }
}
