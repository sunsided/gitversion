use crate::config::enums::{DeploymentMode, VersionStrategies};
use crate::config::gitversion_config::GitVersionConfiguration;
use crate::config::workflows;
use crate::testing::repository_fixture::RepositoryFixture;

fn gitflow_configuration() -> GitVersionConfiguration {
    let mut config = GitVersionConfiguration::default();
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
