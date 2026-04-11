use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct LocalBuild;

impl BuildAgent for LocalBuild {
    fn is_default(&self) -> bool {
        true
    }
    fn can_apply_to_current_context(&self) -> bool {
        true
    }
    fn set_build_number(&self, _variables: &GitVersionVariables) -> Option<String> {
        None
    }
    fn set_output_variables(&self, _name: &str, _value: Option<&str>) -> Vec<String> {
        Vec::new()
    }
}
