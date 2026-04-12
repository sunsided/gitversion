use chrono::Utc;
use eyre::Result;

use crate::calculation::deployment_mode::continuous_delivery::ContinuousDeliveryCalculator;
use crate::calculation::deployment_mode::continuous_deployment::ContinuousDeploymentCalculator;
use crate::calculation::deployment_mode::manual::ManualDeploymentCalculator;
use crate::calculation::effective_branch::{
    EffectiveBranchConfiguration, EffectiveBranchConfigurationFinder,
};
use crate::calculation::strategies::configured_next_version::ConfiguredNextVersionStrategy;
use crate::calculation::strategies::fallback::FallbackVersionStrategy;
use crate::calculation::strategies::mainline::MainlineVersionStrategy;
use crate::calculation::strategies::merge_message::MergeMessageVersionStrategy;
use crate::calculation::strategies::tagged_commit::TaggedCommitVersionStrategy;
use crate::calculation::strategies::track_release_branches::TrackReleaseBranchesVersionStrategy;
use crate::calculation::strategies::version_in_branch_name::VersionInBranchNameStrategy;
use crate::calculation::strategies::VersionStrategy;
use crate::config::enums::{DeploymentMode, VersionStrategies};
use crate::context::GitVersionContext;
use crate::semver::SemanticVersion;

#[derive(Debug, Clone)]
pub struct NextVersion {
    pub incremented_version: SemanticVersion,
    pub branch_configuration: EffectiveBranchConfiguration,
}

#[derive(Debug, Default)]
pub struct NextVersionCalculator;

impl NextVersionCalculator {
    pub fn find_version(&self, ctx: &GitVersionContext) -> Result<SemanticVersion> {
        let branch_config = EffectiveBranchConfigurationFinder
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .ok_or_else(|| eyre::eyre!("missing branch configuration"))?;

        let mut candidates = self.calculate_next_versions(ctx, &branch_config);
        candidates.sort_by(|a, b| a.incremented_version.cmp(&b.incremented_version));
        let mut base = candidates
            .pop()
            .map(|v| v.incremented_version)
            .unwrap_or_else(|| SemanticVersion::new(0, 0, 0));

        if let Some(label) = branch_config
            .branch
            .label
            .as_deref()
            .filter(|label| !label.is_empty())
        {
            base = base.with_label(label);
        }

        let mut deployed = match branch_config
            .branch
            .deployment_mode
            .unwrap_or(DeploymentMode::ManualDeployment)
        {
            DeploymentMode::ManualDeployment => ManualDeploymentCalculator.calculate(base),
            DeploymentMode::ContinuousDelivery => ContinuousDeliveryCalculator.calculate(base, 1),
            DeploymentMode::ContinuousDeployment => {
                ContinuousDeploymentCalculator.calculate(base, 1)
            }
        };

        deployed.build_metadata.sha = Some(ctx.current_commit.sha().to_string());
        deployed.build_metadata.short_sha =
            Some(ctx.current_commit.sha().chars().take(7).collect::<String>());
        deployed.build_metadata.branch = Some(ctx.current_branch.name.friendly());
        deployed.build_metadata.commit_date = Some(ctx.current_commit.when.with_timezone(&Utc));
        deployed.build_metadata.uncommitted_changes = ctx.number_of_uncommitted_changes;

        Ok(deployed)
    }

    fn calculate_next_versions(
        &self,
        ctx: &GitVersionContext,
        branch_config: &EffectiveBranchConfiguration,
    ) -> Vec<NextVersion> {
        let mut strategies: Vec<Box<dyn VersionStrategy>> = Vec::new();
        let selected = ctx.configuration.version_strategy;

        if selected.contains(VersionStrategies::ConfiguredNextVersion) {
            strategies.push(Box::new(ConfiguredNextVersionStrategy));
        }
        if selected.contains(VersionStrategies::MergeMessage) {
            strategies.push(Box::new(MergeMessageVersionStrategy));
        }
        if selected.contains(VersionStrategies::TaggedCommit) {
            strategies.push(Box::new(TaggedCommitVersionStrategy));
        }
        if selected.contains(VersionStrategies::TrackReleaseBranches) {
            strategies.push(Box::new(TrackReleaseBranchesVersionStrategy));
        }
        if selected.contains(VersionStrategies::VersionInBranchName) {
            strategies.push(Box::new(VersionInBranchNameStrategy));
        }
        if selected.contains(VersionStrategies::Mainline) {
            strategies.push(Box::new(MainlineVersionStrategy));
        }
        strategies.push(Box::new(FallbackVersionStrategy));

        let mut out = Vec::new();
        for strategy in strategies {
            for base in strategy.get_base_versions(ctx, branch_config) {
                out.push(NextVersion {
                    incremented_version: base.get_incremented_version(),
                    branch_configuration: branch_config.clone(),
                });
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::NextVersionCalculator;
    use crate::config::enums::DeploymentMode;
    use crate::config::enums::VersionStrategies;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::config::workflows;
    use crate::context::GitVersionContext;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn find_version_populates_commit_sha_short_sha_branch_and_date() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture
            .branch_to("feature/build-metadata")
            .expect("branch switch");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("build context");
        let version = NextVersionCalculator
            .find_version(&ctx)
            .expect("calculate version");

        let expected_sha = ctx.current_commit.sha().to_string();
        let expected_short_sha = expected_sha.chars().take(7).collect::<String>();
        let expected_branch = ctx.current_branch.name.friendly();
        let expected_commit_date = ctx.current_commit.when.with_timezone(&chrono::Utc);

        assert_eq!(
            version.build_metadata.sha.as_deref(),
            Some(expected_sha.as_str())
        );
        assert_eq!(
            version.build_metadata.short_sha.as_deref(),
            Some(expected_short_sha.as_str())
        );
        assert_eq!(
            version.build_metadata.branch.as_deref(),
            Some(expected_branch.as_str())
        );
        assert_eq!(
            version.build_metadata.commit_date,
            Some(expected_commit_date)
        );
    }

    #[test]
    fn find_version_populates_uncommitted_changes_count() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture
            .write_uncommitted_file("dirty.txt", "dirty\n")
            .expect("create dirty file");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let ctx = GitVersionContext::from_repository(repo, GitVersionConfiguration::default())
            .expect("build context");
        let version = NextVersionCalculator
            .find_version(&ctx)
            .expect("calculate version");

        assert_eq!(ctx.number_of_uncommitted_changes, 1);
        assert_eq!(version.build_metadata.uncommitted_changes, 1);
    }

    #[test]
    fn find_version_applies_branch_label_before_continuous_delivery() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let mut config = GitVersionConfiguration::default();
        config.branches = workflows::resolve(&config.workflow);
        let main = config
            .branches
            .get_mut("main")
            .expect("main branch configuration");
        main.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
        main.label = Some("alpha".to_string());

        let ctx = GitVersionContext::from_repository(repo, config).expect("build context");
        let version = NextVersionCalculator
            .find_version(&ctx)
            .expect("calculate version");

        assert_eq!(version.pre_release_tag.name, "alpha");
        assert_eq!(version.pre_release_tag.number, Some(1));
    }

    #[test]
    fn find_version_selects_highest_candidate_across_enabled_strategies() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture.branch_to("release/4.5.6").expect("branch");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let config = GitVersionConfiguration {
            next_version: Some("2.0.0".to_string()),
            ..GitVersionConfiguration::default()
        };

        let ctx = GitVersionContext::from_repository(repo, config).expect("build context");
        let version = NextVersionCalculator
            .find_version(&ctx)
            .expect("calculate version");

        assert_eq!(version.major, 4);
        assert_eq!(version.minor, 5);
        assert_eq!(version.patch, 6);
    }

    #[test]
    fn find_version_uses_fallback_when_no_strategies_are_enabled() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");

        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let config = GitVersionConfiguration {
            version_strategy: VersionStrategies::empty(),
            ..GitVersionConfiguration::default()
        };

        let ctx = GitVersionContext::from_repository(repo, config).expect("build context");
        let version = NextVersionCalculator
            .find_version(&ctx)
            .expect("calculate version");

        assert_eq!(version.major, 0);
        assert_eq!(version.minor, 0);
        assert_eq!(version.patch, 0);
    }
}
