use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct BuildKite;

impl BuildAgent for BuildKite {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("BUILDKITE").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        env::var("BUILDKITE_BRANCH").ok()
    }
    fn set_build_number(&self, _variables: &GitVersionVariables) -> Option<String> {
        None
    }
    fn set_output_variables(&self, name: &str, value: Option<&str>) -> Vec<String> {
        value
            .map(|v| vec![format!("buildkite-agent meta-data set {name} {v}")])
            .unwrap_or_default()
    }
}
