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
        let branch_config = EffectiveBranchConfigurationFinder::default()
            .get_configurations(&ctx.current_branch, &ctx.configuration)
            .into_iter()
            .next()
            .ok_or_else(|| eyre::eyre!("missing branch configuration"))?;

        let mut candidates = self.calculate_next_versions(ctx, &branch_config);
        candidates.sort_by(|a, b| a.incremented_version.cmp(&b.incremented_version));
        let base = candidates
            .pop()
            .map(|v| v.incremented_version)
            .unwrap_or_else(|| SemanticVersion::new(0, 0, 0));

        let mut deployed = match branch_config
            .branch
            .deployment_mode
            .unwrap_or(DeploymentMode::ManualDeployment)
        {
            DeploymentMode::ManualDeployment => {
                ManualDeploymentCalculator::default().calculate(base)
            }
            DeploymentMode::ContinuousDelivery => {
                ContinuousDeliveryCalculator::default().calculate(base, 1)
            }
            DeploymentMode::ContinuousDeployment => {
                ContinuousDeploymentCalculator::default().calculate(base, 1)
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
    use crate::config::gitversion_config::GitVersionConfiguration;
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
}
