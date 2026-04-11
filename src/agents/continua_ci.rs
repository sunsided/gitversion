use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct ContinuaCI;

impl BuildAgent for ContinuaCI {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("ContinuaCI.Version").is_ok()
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
