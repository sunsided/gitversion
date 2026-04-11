use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct MyGet;

impl BuildAgent for MyGet {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("BuildRunner").ok().as_deref() == Some("MyGet")
    }
    fn set_build_number(&self, _variables: &GitVersionVariables) -> Option<String> {
        None
    }
    fn set_output_variables(&self, _name: &str, _value: Option<&str>) -> Vec<String> {
        Vec::new()
    }
}
