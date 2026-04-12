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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::agents::buildkite::BuildKite;
    use crate::agents::BuildAgent;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_buildkite_env_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("BUILDKITE", "true");
        }

        let agent = BuildKite;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("BUILDKITE");
        }
    }

    #[test]
    fn get_current_branch_reads_buildkite_branch() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("BUILDKITE_BRANCH", "main");
        }

        let agent = BuildKite;
        assert_eq!(agent.get_current_branch(false).as_deref(), Some("main"));

        unsafe {
            std::env::remove_var("BUILDKITE_BRANCH");
        }
    }

    #[test]
    fn set_output_variables_uses_meta_data_command() {
        let agent = BuildKite;
        assert_eq!(
            agent.set_output_variables("Foo", Some("bar")),
            vec!["buildkite-agent meta-data set Foo bar"]
        );
    }
}
