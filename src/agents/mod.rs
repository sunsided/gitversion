pub mod appveyor;
pub mod azure_pipelines;
pub mod bitbucket;
pub mod buildkite;
pub mod codebuild;
pub mod continua_ci;
pub mod drone;
pub mod envrun;
pub mod github_actions;
pub mod gitlab_ci;
pub mod jenkins;
pub mod local;
pub mod myget;
pub mod space_automation;
pub mod teamcity;
pub mod travis_ci;

use crate::output::variables::GitVersionVariables;

pub trait BuildAgent {
    fn is_default(&self) -> bool {
        false
    }
    fn can_apply_to_current_context(&self) -> bool;
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        None
    }
    fn prevent_fetch(&self) -> bool {
        true
    }
    fn should_clean_up_remotes(&self) -> bool {
        false
    }
    fn set_build_number(&self, variables: &GitVersionVariables) -> Option<String>;
    fn set_output_variables(&self, name: &str, value: Option<&str>) -> Vec<String>;
    fn write_integration(
        &self,
        writer: &mut dyn FnMut(Option<&str>),
        variables: &GitVersionVariables,
        update_build_number: bool,
    ) {
        if update_build_number {
            writer(self.set_build_number(variables).as_deref());
        }
        for (name, value) in variables.iter() {
            for line in self.set_output_variables(name, value) {
                writer(Some(&line));
            }
        }
    }
}

pub fn detect_build_agent() -> Box<dyn BuildAgent> {
    let agents: Vec<Box<dyn BuildAgent>> = vec![
        Box::new(github_actions::GitHubActions),
        Box::new(azure_pipelines::AzurePipelines),
        Box::new(gitlab_ci::GitLabCI),
        Box::new(jenkins::Jenkins),
        Box::new(teamcity::TeamCity),
        Box::new(bitbucket::BitBucketPipelines),
        Box::new(travis_ci::TravisCI),
        Box::new(appveyor::AppVeyor),
        Box::new(buildkite::BuildKite),
        Box::new(drone::Drone),
        Box::new(codebuild::CodeBuild),
        Box::new(continua_ci::ContinuaCI),
        Box::new(myget::MyGet),
        Box::new(envrun::EnvRun),
        Box::new(space_automation::SpaceAutomation),
        Box::new(local::LocalBuild),
    ];
    agents
        .into_iter()
        .find(|agent| agent.can_apply_to_current_context())
        .unwrap_or_else(|| Box::new(local::LocalBuild))
}
