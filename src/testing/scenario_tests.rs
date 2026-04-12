use crate::calculation::increment_strategy::IncrementStrategyFinder;
use crate::config::enums::{DeploymentMode, VersionStrategies};
use crate::config::gitversion_config::GitVersionConfiguration;
use crate::config::workflows;
use crate::semver::VersionField;
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
fn commit_message_increment_markers_detect_minor_increment() {
    let increment = IncrementStrategyFinder.get_increment_forced_by_commit(
        "feat: add API endpoint\n\n+semver: minor",
        &GitVersionConfiguration::default(),
    );

    assert_eq!(increment, VersionField::Minor);
}
