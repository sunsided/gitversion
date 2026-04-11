use regex::Regex;

use crate::calculation::base_version::BaseVersion;
use crate::calculation::base_version::BaseVersionOperand;
use crate::calculation::effective_branch::EffectiveBranchConfiguration;
use crate::calculation::strategies::VersionStrategy;
use crate::context::GitVersionContext;
use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct TrackReleaseBranchesVersionStrategy;

impl VersionStrategy for TrackReleaseBranchesVersionStrategy {
    fn get_base_versions(
        &self,
        ctx: &GitVersionContext,
        config: &EffectiveBranchConfiguration,
    ) -> Vec<BaseVersion> {
        if !config.branch.tracks_release_branches.unwrap_or(false) {
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

                versions.push(BaseVersion {
                    operand: BaseVersionOperand {
                        source: format!("Release branch {friendly}"),
                        semantic_version: version,
                        base_version_source,
                    },
                    operator: None,
                });
            }
        }

        versions
    }
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
