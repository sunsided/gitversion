use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct Drone;

impl BuildAgent for Drone {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("DRONE").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        env::var("DRONE_BRANCH").ok()
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

    use crate::agents::drone::Drone;
    use crate::agents::BuildAgent;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_drone_env_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("DRONE", "true");
        }

        let agent = Drone;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("DRONE");
        }
    }

    #[test]
    fn get_current_branch_reads_drone_branch() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("DRONE_BRANCH", "develop");
        }

        let agent = Drone;
        assert_eq!(agent.get_current_branch(false).as_deref(), Some("develop"));

        unsafe {
            std::env::remove_var("DRONE_BRANCH");
        }
    }

    #[test]
    fn set_output_variables_uses_export_format() {
        let agent = Drone;
        assert_eq!(
            agent.set_output_variables("Foo", Some("bar")),
            vec!["export Foo=bar"]
        );
    }
}
