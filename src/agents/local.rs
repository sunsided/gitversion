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

#[cfg(test)]
mod tests {
    use crate::agents::BuildAgent;
    use crate::agents::local::LocalBuild;
    use crate::output::variables::GitVersionVariables;

    #[test]
    fn local_build_is_default_and_always_applicable() {
        let agent = LocalBuild;

        assert!(agent.is_default());
        assert!(agent.can_apply_to_current_context());
    }

    #[test]
    fn local_build_does_not_emit_build_or_variable_commands() {
        let agent = LocalBuild;

        assert!(
            agent
                .set_build_number(&GitVersionVariables::default())
                .is_none()
        );
        assert!(agent.set_output_variables("Foo", Some("bar")).is_empty());
    }
}
