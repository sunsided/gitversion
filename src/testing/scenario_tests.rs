use crate::calculation::increment_strategy::IncrementStrategyFinder;
use crate::config::enums::{DeploymentMode, VersionStrategies};
use crate::config::gitversion_config::GitVersionConfiguration;
use crate::config::workflows;
use crate::semver::{SemanticVersion, VersionField};
use crate::testing::repository_fixture::RepositoryFixture;

fn gitflow_configuration() -> GitVersionConfiguration {
    let mut config = GitVersionConfiguration::default();
    config.branches = workflows::resolve(&config.workflow);
    config
}

fn githubflow_configuration() -> GitVersionConfiguration {
    let mut config = GitVersionConfiguration {
        workflow: "GitHubFlow/v1".to_string(),
        ..GitVersionConfiguration::default()
    };
    config.branches = workflows::resolve(&config.workflow);
    config
}

fn expected_full_semver(base: &str, fixture: &RepositoryFixture, branch: &str) -> String {
    format!(
        "{base}+{branch}.{}",
        fixture.head_sha().expect("head sha for metadata")
    )
}

fn current_branch_name(fixture: &RepositoryFixture) -> String {
    let repo = git2::Repository::open(fixture.path()).expect("open fixture repository");
    repo.head()
        .expect("head")
        .shorthand()
        .unwrap_or("master")
        .to_string()
}

fn assert_core(version: &SemanticVersion, major: i64, minor: i64, patch: i64) {
    assert_eq!(
        (version.major, version.minor, version.patch),
        (major, minor, patch)
    );
}

#[test]
fn main_scenario_uses_tagged_version_on_main_branch() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    if current_branch_name(&fixture) != "main" {
        fixture.branch_to("main").expect("switch to main");
    }
    fixture.make_a_commit("release commit").expect("commit");
    fixture.apply_tag("1.2.3").expect("apply tag");

    let config = gitflow_configuration();
    let expected = expected_full_semver("1.2.3", &fixture, "main");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn feature_branch_scenario_uses_version_in_branch_name_and_label() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture
        .branch_to("feature/2.4.0-search")
        .expect("switch to feature branch");
    fixture.make_a_commit("feature commit").expect("commit");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.label = Some("feat".to_string());
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    let expected = expected_full_semver("2.4.0-feat.1", &fixture, "feature/2.4.0-search");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn release_branch_scenario_applies_release_label() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture
        .branch_to("release/3.2.0")
        .expect("switch to release branch");
    fixture.make_a_commit("release commit").expect("commit");

    let config = gitflow_configuration();
    let expected = expected_full_semver("3.2.0-beta", &fixture, "release/3.2.0");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn develop_scenario_uses_alpha_prerelease_in_continuous_delivery_mode() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.branch_to("develop").expect("switch to develop");
    fixture.make_a_commit("develop commit").expect("commit");

    let mut config = gitflow_configuration();
    let develop = config
        .branches
        .get_mut("develop")
        .expect("develop branch configuration");
    develop.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    let expected = expected_full_semver("0.0.0-alpha.1", &fixture, "develop");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn hotfix_branch_scenario_uses_version_from_branch_name() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture
        .branch_to("hotfix/2.4.1")
        .expect("switch to hotfix branch");
    fixture.make_a_commit("hotfix commit").expect("commit");

    let config = gitflow_configuration();
    let expected = expected_full_semver("2.4.1", &fixture, "hotfix/2.4.1");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn pull_request_scenario_applies_pr_label() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture
        .branch_to("pull-request/123")
        .expect("switch to pull request branch");
    fixture
        .make_a_commit("pull request commit")
        .expect("commit");

    let mut config = gitflow_configuration();
    let pull_request = config
        .branches
        .get_mut("pull-request")
        .expect("pull-request branch configuration");
    pull_request.label = Some("pr".to_string());
    pull_request.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    let expected = expected_full_semver("0.0.0-pr.1", &fixture, "pull-request/123");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn mainline_development_scenario_increments_patch_after_tag() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    if current_branch_name(&fixture) != "main" {
        fixture.branch_to("main").expect("switch to main");
    }
    fixture.make_a_commit("release commit").expect("commit");
    fixture.apply_tag("1.0.0").expect("apply tag");
    fixture
        .make_a_commit("post-release commit")
        .expect("commit");

    let mut config = gitflow_configuration();
    config.version_strategy = VersionStrategies::Mainline;
    let expected = expected_full_semver("1.0.1", &fixture, "main");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn continuous_delivery_scenario_uses_configured_next_version_with_main_ci_label() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    if current_branch_name(&fixture) != "main" {
        fixture.branch_to("main").expect("switch to main");
    }

    let mut config = gitflow_configuration();
    config.next_version = Some("1.0.0".to_string());
    let main = config
        .branches
        .get_mut("main")
        .expect("main branch configuration");
    main.label = Some("ci".to_string());
    main.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    let expected = expected_full_semver("1.0.0-ci.1", &fixture, "main");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn continuous_delivery_scenario_prefers_higher_tag_over_next_version() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    if current_branch_name(&fixture) != "main" {
        fixture.branch_to("main").expect("switch to main");
    }
    fixture.make_a_commit("release commit").expect("commit");
    fixture.apply_tag("1.1.0").expect("apply tag");

    let mut config = gitflow_configuration();
    config.next_version = Some("1.0.0".to_string());
    let main = config
        .branches
        .get_mut("main")
        .expect("main branch configuration");
    main.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    let expected = expected_full_semver("1.1.0", &fixture, "main");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn gitflow_scenario_merge_message_uses_release_branch_version_on_main() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    let main_branch = current_branch_name(&fixture);
    fixture
        .branch_to("release/2.0.0")
        .expect("switch to release branch");
    fixture.make_a_commit("release hardening").expect("commit");
    fixture
        .checkout(&main_branch)
        .expect("switch back to main branch");
    fixture
        .merge("release/2.0.0", "Merge branch 'release/2.0.0' into main")
        .expect("merge release branch");

    let config = gitflow_configuration();
    let expected = expected_full_semver("2.0.0", &fixture, &main_branch);

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn support_branch_scenario_uses_version_in_branch_name() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture
        .branch_to("support/1.5.0")
        .expect("switch to support branch");
    fixture.make_a_commit("support patch").expect("commit");

    let config = gitflow_configuration();
    let expected = expected_full_semver("1.5.0", &fixture, "support/1.5.0");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn release_branch_scenario_continuous_deployment_strips_prerelease_tag() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture
        .branch_to("release/4.0.0")
        .expect("switch to release branch");
    fixture.make_a_commit("release prep").expect("commit");

    let mut config = gitflow_configuration();
    let release = config
        .branches
        .get_mut("release")
        .expect("release branch configuration");
    release.deployment_mode = Some(DeploymentMode::ContinuousDeployment);
    let expected = expected_full_semver("4.0.0", &fixture, "release/4.0.0");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn unknown_branch_scenario_uses_fallback_configuration() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture
        .branch_to("random-branch")
        .expect("switch to unknown branch");
    fixture
        .make_a_commit("unknown branch commit")
        .expect("commit");

    let mut config = gitflow_configuration();
    let unknown = config
        .branches
        .get_mut("unknown")
        .expect("unknown branch configuration");
    unknown.label = Some("ci".to_string());
    unknown.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    let expected = expected_full_semver("0.0.0-ci.1", &fixture, "random-branch");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn develop_scenario_manual_deployment_preserves_alpha_label_without_number() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.branch_to("develop").expect("switch to develop");
    fixture.make_a_commit("develop commit").expect("commit");

    let mut config = gitflow_configuration();
    let develop = config
        .branches
        .get_mut("develop")
        .expect("develop branch configuration");
    develop.deployment_mode = Some(DeploymentMode::ManualDeployment);
    let expected = expected_full_semver("0.0.0-alpha", &fixture, "develop");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn mainline_continuous_delivery_multiple_commits_after_tag_uses_consistent_prerelease_number() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    if current_branch_name(&fixture) != "main" {
        fixture.branch_to("main").expect("switch to main");
    }
    fixture.make_a_commit("release commit").expect("commit");
    fixture.apply_tag("1.0.0").expect("apply tag");
    fixture
        .make_commits(3, "post-release commit")
        .expect("create post-release commits");

    let mut config = gitflow_configuration();
    config.version_strategy = VersionStrategies::Mainline;
    let main = config
        .branches
        .get_mut("main")
        .expect("main branch configuration");
    main.label = Some("ci".to_string());
    main.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    let expected = expected_full_semver("1.0.1-ci.1", &fixture, "main");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn githubflow_scenario_uses_workflow_specific_branch_defaults() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    if current_branch_name(&fixture) != "main" {
        fixture.branch_to("main").expect("switch to main");
    }
    fixture.make_a_commit("release commit").expect("commit");
    fixture.apply_tag("2.1.0").expect("apply tag");

    let config = githubflow_configuration();
    let expected = expected_full_semver("2.1.0", &fixture, "main");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn tagged_pre_release_scenario_uses_pre_release_tag_from_head_commit() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    if current_branch_name(&fixture) != "main" {
        fixture.branch_to("main").expect("switch to main");
    }
    fixture.make_a_commit("release candidate").expect("commit");
    fixture
        .apply_tag("1.0.0-rc.1")
        .expect("apply pre-release tag");

    let config = gitflow_configuration();
    let expected = expected_full_semver("1.0.0-rc.1", &fixture, "main");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn multiple_tags_on_same_commit_scenario_prefers_highest_semver_tag() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    if current_branch_name(&fixture) != "main" {
        fixture.branch_to("main").expect("switch to main");
    }
    fixture.make_a_commit("release commit").expect("commit");
    fixture.apply_tag("1.0.0").expect("apply lower tag");
    fixture.apply_tag("1.1.0").expect("apply higher tag");

    let config = gitflow_configuration();
    let expected = expected_full_semver("1.1.0", &fixture, "main");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn hotfix_branch_scenario_continuous_delivery_keeps_release_version_without_label() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture
        .branch_to("hotfix/2.0.1")
        .expect("switch to hotfix branch");
    fixture.make_a_commit("hotfix commit").expect("commit");

    let mut config = gitflow_configuration();
    let hotfix = config
        .branches
        .get_mut("hotfix")
        .expect("hotfix branch configuration");
    hotfix.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    let expected = expected_full_semver("2.0.1", &fixture, "hotfix/2.0.1");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn develop_scenario_continuous_deployment_strips_alpha_prerelease_tag() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.branch_to("develop").expect("switch to develop");
    fixture.make_a_commit("develop commit").expect("commit");

    let mut config = gitflow_configuration();
    let develop = config
        .branches
        .get_mut("develop")
        .expect("develop branch configuration");
    develop.deployment_mode = Some(DeploymentMode::ContinuousDeployment);
    let expected = expected_full_semver("0.0.0", &fixture, "develop");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
fn feature_branch_scenario_from_tagged_main_uses_mainline_base_version() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    if current_branch_name(&fixture) != "main" {
        fixture.branch_to("main").expect("switch to main");
    }
    fixture.make_a_commit("release commit").expect("commit");
    fixture.apply_tag("3.0.0").expect("apply tag");
    fixture
        .branch_to("feature/inherit-from-main")
        .expect("switch to feature branch");
    fixture.make_a_commit("feature commit").expect("commit");

    let mut config = gitflow_configuration();
    config.version_strategy = VersionStrategies::Mainline;
    let expected = expected_full_semver("3.0.1", &fixture, "feature/inherit-from-main");

    fixture
        .assert_full_semver(&expected, config)
        .expect("version assertion");
}

#[test]
#[ignore = "requires feature-parent increment inheritance parity"]
fn feature_branch_inherits_increment_with_multiple_possible_parents() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.0").expect("tag");
    fixture.branch_to("develop").expect("branch develop");
    fixture.make_a_commit("develop baseline").expect("commit");
    fixture
        .branch_to("feature/jira-123")
        .expect("branch feature 123");
    fixture.make_a_commit("feature 123").expect("commit");
    fixture.checkout("develop").expect("checkout develop");
    fixture
        .merge_no_ff("feature/jira-123")
        .expect("merge feature 123");
    fixture
        .branch_to("feature/jira-124")
        .expect("branch feature 124");
    fixture.make_a_commit("feature 124").expect("commit");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    feature.label = Some("feat".to_string());

    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_core(&version, 1, 1, 0);
    assert_eq!(version.pre_release_tag.name, "feat");
}

#[test]
#[ignore = "requires feature-parent increment inheritance parity"]
fn feature_branch_after_fast_forward_merge_inherits_correctly() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.0").expect("tag");
    fixture.branch_to("develop").expect("branch develop");
    fixture.make_a_commit("develop baseline").expect("commit");
    fixture
        .branch_to("feature/jira-123")
        .expect("branch feature 123");
    fixture.make_a_commit("feature 123").expect("commit");
    fixture.checkout("develop").expect("checkout develop");
    fixture
        .merge(
            "feature/jira-123",
            "Merge branch 'feature/jira-123' into develop",
        )
        .expect("merge");
    fixture
        .branch_to("feature/jira-124")
        .expect("branch feature 124");
    fixture.make_a_commit("feature 124").expect("commit");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    feature.label = Some("feat".to_string());

    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_core(&version, 1, 1, 0);
    assert_eq!(version.pre_release_tag.name, "feat");
}

#[test]
fn feature_branch_should_not_use_number_in_branch_name_as_prerelease_number() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.0").expect("tag");
    fixture.branch_to("develop").expect("branch develop");
    fixture
        .branch_to("feature/jira-123")
        .expect("branch feature");
    fixture.make_commits(3, "feature commit").expect("commits");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    feature.label = Some("jira-123".to_string());

    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_eq!(version.pre_release_tag.number, Some(1));
}

#[test]
fn feature_branch_long_running_with_develop_merge() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.0").expect("tag");
    fixture.branch_to("develop").expect("branch develop");
    fixture
        .branch_to("feature/longrunning")
        .expect("branch feature");
    fixture.make_a_commit("feature commit").expect("commit");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    feature.label = Some("longrunning".to_string());
    let before_merge = fixture.calculate_version(config.clone()).expect("version");

    fixture.checkout("develop").expect("checkout develop");
    fixture.make_a_commit("develop commit").expect("commit");
    fixture
        .checkout("feature/longrunning")
        .expect("checkout feature");
    fixture.merge_no_ff("develop").expect("merge develop");

    let after_merge = fixture.calculate_version(config).expect("version");
    assert!(after_merge >= before_merge);
}

#[test]
#[ignore = "requires release-to-feature version inheritance parity"]
fn feature_branch_from_release_uses_branch_name_version() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.branch_to("release/0.3.0").expect("release branch");
    fixture.make_a_commit("release commit").expect("commit");
    fixture.branch_to("feature/proj-1").expect("feature branch");
    fixture.make_a_commit("feature commit").expect("commit");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    feature.label = Some("proj-1".to_string());

    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_core(&version, 0, 3, 0);
}

#[test]
fn feature_branch_configurable_label_with_regex_captures() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.0").expect("tag");
    fixture
        .branch_to("feature/4711_this-is-a-feature")
        .expect("feature branch");
    fixture.make_a_commit("feature commit").expect("commit");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.regular_expression =
        Some("^features?[/-](?<TaskNumber>\\d+)_(?<BranchName>.+)".to_string());
    feature.label = Some("{BranchName}-of-task-number-{TaskNumber}".to_string());
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);

    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_eq!(version.pre_release_tag.number, Some(1));
}

#[test]
#[ignore = "requires release-finish develop inheritance parity"]
fn feature_branch_after_release_finish_inherits_from_main_tag() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    let main_branch = current_branch_name(&fixture);
    fixture.branch_to("develop").expect("develop");
    fixture.make_a_commit("develop commit").expect("commit");
    fixture.branch_to("release/0.2.0").expect("release");
    fixture.make_a_commit("release commit").expect("commit");
    fixture.checkout(&main_branch).expect("main");
    fixture.merge_no_ff("release/0.2.0").expect("merge release");
    fixture.apply_tag("0.2.0").expect("tag");
    fixture.checkout("develop").expect("develop");
    fixture
        .merge_no_ff("release/0.2.0")
        .expect("merge to develop");
    fixture.make_a_commit("post release").expect("commit");
    fixture.branch_to("feature/test-1").expect("feature");
    fixture.make_a_commit("feature commit").expect("commit");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    feature.label = Some("test-1".to_string());

    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_core(&version, 0, 3, 0);
}

#[test]
#[ignore = "requires track-release-branches inheritance on feature branches"]
fn feature_branch_picks_up_version_after_release_branch_created() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.branch_to("develop").expect("develop");
    fixture.make_a_commit("develop commit").expect("commit");
    fixture.branch_to("release/1.0.0").expect("release");
    fixture.make_a_commit("release commit").expect("commit");
    fixture.checkout("develop").expect("develop");
    fixture.make_a_commit("develop commit 2").expect("commit");
    fixture.branch_to("feature/test").expect("feature");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    feature.label = Some("test".to_string());

    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_core(&version, 1, 1, 0);
}

#[test]
#[ignore = "requires track-release-branches inheritance on feature branches"]
fn feature_branch_picks_up_version_after_release_merged_back() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.branch_to("develop").expect("develop");
    fixture.make_a_commit("develop commit").expect("commit");
    fixture.branch_to("release/1.0.0").expect("release");
    fixture.make_a_commit("release commit").expect("commit");
    fixture.checkout("develop").expect("develop");
    fixture.merge_no_ff("release/1.0.0").expect("merge release");
    fixture.branch_to("feature/test").expect("feature");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    feature.label = Some("test".to_string());

    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_core(&version, 1, 1, 0);
}

#[test]
fn feature_branch_has_greater_semver_after_develop_merged_into_feature() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.branch_to("develop").expect("develop");
    fixture.make_a_commit("develop commit").expect("commit");
    fixture.apply_tag("16.23.0").expect("tag");
    fixture.make_a_commit("develop commit 2").expect("commit");
    fixture.branch_to("feature/featx").expect("feature");
    fixture.make_a_commit("feature commit").expect("commit");

    let mut config = gitflow_configuration();
    let feature = config
        .branches
        .get_mut("feature")
        .expect("feature branch configuration");
    feature.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    feature.label = Some("feat-featx".to_string());
    let before_merge = fixture
        .calculate_version(config.clone())
        .expect("before merge");

    fixture.checkout("develop").expect("develop");
    fixture.make_a_commit("develop commit 3").expect("commit");
    fixture.checkout("feature/featx").expect("feature");
    fixture.merge_no_ff("develop").expect("merge develop");

    let after_merge = fixture.calculate_version(config).expect("after merge");
    assert!(after_merge >= before_merge);
}

#[test]
#[ignore = "requires release merge precedence parity on develop"]
fn release_branch_no_merge_backs_when_no_changes() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    let main_branch = current_branch_name(&fixture);
    fixture.branch_to("develop").expect("develop");
    fixture.make_commits(3, "develop").expect("commits");
    fixture.branch_to("release/1.0.0").expect("release");
    fixture.checkout(&main_branch).expect("main");
    fixture.merge_no_ff("release/1.0.0").expect("merge release");
    fixture.apply_tag("1.0.0").expect("tag");
    fixture.checkout("develop").expect("develop");
    fixture
        .delete_branch("release/1.0.0")
        .expect("delete release");

    let mut config = gitflow_configuration();
    let develop = config
        .branches
        .get_mut("develop")
        .expect("develop branch configuration");
    develop.deployment_mode = Some(DeploymentMode::ContinuousDelivery);

    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_core(&version, 1, 1, 0);
}

#[test]
fn release_branch_merge_to_main_carries_version() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.3").expect("tag");
    let main_branch = current_branch_name(&fixture);
    fixture.make_a_commit("main commit").expect("commit");
    fixture.branch_to("release/2.0.0").expect("release");
    fixture.make_commits(2, "release").expect("commits");
    fixture.checkout(&main_branch).expect("main");
    fixture.merge_no_ff("release/2.0.0").expect("merge release");

    let version = fixture
        .calculate_version(gitflow_configuration())
        .expect("calculate version");
    assert_core(&version, 2, 0, 0);
}

#[test]
#[ignore = "requires highest release precedence parity on develop"]
fn release_branch_highest_version_wins_with_multiple_releases() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.3").expect("tag");
    fixture.branch_to("develop").expect("develop");
    fixture.make_a_commit("develop commit").expect("commit");
    fixture.branch_to("release/2.0.0").expect("release 2");
    fixture.make_a_commit("release 2 commit").expect("commit");
    fixture.checkout("develop").expect("develop");
    fixture
        .merge_no_ff("release/2.0.0")
        .expect("merge release 2");
    fixture.branch_to("release/1.0.0").expect("release 1");
    fixture.make_a_commit("release 1 commit").expect("commit");
    fixture.checkout("develop").expect("develop");
    fixture
        .merge_no_ff("release/1.0.0")
        .expect("merge release 1");

    let mut config = gitflow_configuration();
    let develop = config
        .branches
        .get_mut("develop")
        .expect("develop branch configuration");
    develop.deployment_mode = Some(DeploymentMode::ContinuousDelivery);
    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_core(&version, 2, 1, 0);
}

#[test]
fn release_branch_beta_version_not_reset_after_merge_to_develop() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.3").expect("tag");
    fixture.branch_to("develop").expect("develop");
    fixture.make_a_commit("develop commit").expect("commit");
    fixture.branch_to("release/2.0.0").expect("release");
    fixture.make_a_commit("release commit").expect("commit");

    let release_version_before = fixture
        .calculate_version(gitflow_configuration())
        .expect("release version before");
    fixture.checkout("develop").expect("develop");
    fixture.merge_no_ff("release/2.0.0").expect("merge release");
    fixture.checkout("release/2.0.0").expect("release");
    fixture.make_a_commit("release continued").expect("commit");
    let release_version_after = fixture
        .calculate_version(gitflow_configuration())
        .expect("release version after");

    assert_core(&release_version_before, 2, 0, 0);
    assert_core(&release_version_after, 2, 0, 0);
}

#[test]
fn release_branch_hotfix_off_release_preserves_count() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.3").expect("tag");
    fixture.branch_to("develop").expect("develop");
    fixture.make_a_commit("develop commit").expect("commit");
    fixture.branch_to("release/2.0.0").expect("release");
    fixture.make_commits(2, "release").expect("release commits");
    let before_hotfix = fixture
        .calculate_version(gitflow_configuration())
        .expect("release before hotfix");

    fixture.branch_to("hotfix/2.0.0").expect("hotfix");
    fixture.make_commits(2, "hotfix").expect("hotfix commits");
    fixture.checkout("release/2.0.0").expect("release");
    fixture.merge_no_ff("hotfix/2.0.0").expect("merge hotfix");

    let after_hotfix = fixture
        .calculate_version(gitflow_configuration())
        .expect("release after hotfix");
    assert_core(&before_hotfix, 2, 0, 0);
    assert_core(&after_hotfix, 2, 0, 0);
}

#[test]
fn release_branch_merge_on_release_preserves_count() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.3").expect("tag");
    fixture.branch_to("develop").expect("develop");
    fixture.make_a_commit("develop commit").expect("commit");
    fixture.branch_to("release/2.0.0").expect("release");
    fixture.make_a_commit("release commit").expect("commit");
    let before_merge = fixture
        .calculate_version(gitflow_configuration())
        .expect("before merge");

    fixture
        .branch_to("release/2.0.0-xxx")
        .expect("side release");
    fixture
        .make_a_commit("side release commit")
        .expect("commit");
    fixture.checkout("release/2.0.0").expect("release");
    fixture
        .merge_no_ff("release/2.0.0-xxx")
        .expect("merge side release");

    let after_merge = fixture
        .calculate_version(gitflow_configuration())
        .expect("after merge");
    assert_core(&before_merge, 2, 0, 0);
    assert_core(&after_merge, 2, 0, 0);
}

#[test]
fn release_branch_feature_from_release_preserves_count() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.branch_to("develop").expect("develop");
    fixture.branch_to("release/2.0.0").expect("release");
    fixture.make_commits(2, "release").expect("release commits");
    let before_feature_merge = fixture
        .calculate_version(gitflow_configuration())
        .expect("release before feature");

    fixture.branch_to("feature/xxx").expect("feature");
    fixture.make_commits(2, "feature").expect("feature commits");
    fixture.checkout("release/2.0.0").expect("release");
    fixture.merge_no_ff("feature/xxx").expect("merge feature");

    let after_feature_merge = fixture
        .calculate_version(gitflow_configuration())
        .expect("release after feature");
    assert_core(&before_feature_merge, 2, 0, 0);
    assert_core(&after_feature_merge, 2, 0, 0);
}

#[test]
fn release_branch_with_prerelease_weight_for_assembly_version() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.3").expect("tag");
    fixture.make_commits(2, "main").expect("commits");
    fixture.branch_to("release/2.0.0").expect("release");

    let mut config = gitflow_configuration();
    let release = config
        .branches
        .get_mut("release")
        .expect("release branch configuration");
    release.pre_release_weight = Some(1000);

    let version = fixture
        .calculate_version(config)
        .expect("calculate version");
    assert_core(&version, 2, 0, 0);
    assert_eq!(version.pre_release_tag.name, "beta");
}

#[test]
fn release_branch_uses_branch_name_despite_bump_in_previous_commit() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.apply_tag("1.0.0").expect("tag");
    fixture.make_a_commit("+semver:major").expect("commit");
    fixture.make_a_commit("regular commit").expect("commit");
    fixture.branch_to("release/2.0.0").expect("release");

    let version = fixture
        .calculate_version(gitflow_configuration())
        .expect("calculate version");
    assert_core(&version, 2, 0, 0);
}

#[test]
fn release_branch_detection_with_loose_semver_format() {
    let mut fixture = RepositoryFixture::new().expect("fixture");
    fixture.make_a_commit("initial commit").expect("commit");
    fixture.branch_to("release/1.2.0").expect("release branch");
    fixture.make_a_commit("release commit").expect("commit");

    let mut strict = gitflow_configuration();
    strict.semantic_version_format = crate::config::enums::SemanticVersionFormat::Strict;
    let strict_version = fixture.calculate_version(strict).expect("strict version");

    let mut loose = gitflow_configuration();
    loose.semantic_version_format = crate::config::enums::SemanticVersionFormat::Loose;
    let loose_version = fixture.calculate_version(loose).expect("loose version");

    assert_core(&strict_version, 1, 2, 0);
    assert_core(&loose_version, 1, 2, 0);
}

#[test]
fn commit_message_increment_markers_detect_minor_increment() {
    let increment = IncrementStrategyFinder.get_increment_forced_by_commit(
        "feat: add API endpoint\n\n+semver: minor",
        &GitVersionConfiguration::default(),
    );

    assert_eq!(increment, VersionField::Minor);
}
