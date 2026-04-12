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

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::agents::BuildAgent;
    use crate::agents::space_automation::SpaceAutomation;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_space_api_url_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("JB_SPACE_API_URL", "https://space.example");
        }

        let agent = SpaceAutomation;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("JB_SPACE_API_URL");
        }
    }

    #[test]
    fn get_current_branch_reads_space_branch_env() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("JB_SPACE_GIT_BRANCH", "refs/heads/main");
        }

        let agent = SpaceAutomation;
        assert_eq!(
            agent.get_current_branch(false).as_deref(),
            Some("refs/heads/main")
        );

        unsafe {
            std::env::remove_var("JB_SPACE_GIT_BRANCH");
        }
    }

    #[test]
    fn set_output_variables_uses_export_format() {
        let agent = SpaceAutomation;
        assert_eq!(
            agent.set_output_variables("Foo", Some("bar")),
            vec!["export Foo=bar"]
        );
    }
}
