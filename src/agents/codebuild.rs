use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct CodeBuild;

impl BuildAgent for CodeBuild {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("CODEBUILD_CI").is_ok()
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
