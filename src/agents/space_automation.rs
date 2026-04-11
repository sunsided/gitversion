use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct SpaceAutomation;

impl BuildAgent for SpaceAutomation {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("JB_SPACE_API_URL").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        env::var("JB_SPACE_GIT_BRANCH").ok()
    }
    fn set_build_number(&self, _variables: &GitVersionVariables) -> Option<String> {
        None
    }
    fn set_output_variables(&self, name: &str, value: Option<&str>) -> Vec<String> {
        value
            .map(|v| vec![format!("export {name}={v}")])
            .unwrap_or_default()
    }
}
