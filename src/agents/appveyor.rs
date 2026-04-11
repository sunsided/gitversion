use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct AppVeyor;

impl BuildAgent for AppVeyor {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("APPVEYOR").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        env::var("APPVEYOR_REPO_BRANCH").ok()
    }
    fn set_build_number(&self, variables: &GitVersionVariables) -> Option<String> {
        Some(format!(
            "appveyor UpdateBuild -Version {}",
            variables.FullSemVer
        ))
    }
    fn set_output_variables(&self, name: &str, value: Option<&str>) -> Vec<String> {
        value
            .map(|v| vec![format!("appveyor SetVariable -Name {name} -Value {v}")])
            .unwrap_or_default()
    }
}
