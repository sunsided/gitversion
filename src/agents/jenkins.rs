use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct Jenkins;

impl BuildAgent for Jenkins {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("JENKINS_URL").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        env::var("GIT_LOCAL_BRANCH")
            .ok()
            .or_else(|| env::var("GIT_BRANCH").ok())
    }
    fn set_build_number(&self, variables: &GitVersionVariables) -> Option<String> {
        Some(format!("BUILD_NUMBER={}", variables.full_sem_ver))
    }
    fn set_output_variables(&self, _name: &str, _value: Option<&str>) -> Vec<String> {
        Vec::new()
    }
}
