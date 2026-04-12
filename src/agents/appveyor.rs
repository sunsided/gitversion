use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct AppVeyor;

impl BuildAgent for AppVeyor {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("APPVEYOR").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        env::var("APPVEYOR_REPO_BRANCH").ok()
    }
    fn set_build_number(&self, variables: &GitVersionVariables) -> Option<String> {
        Some(format!(
            "appveyor UpdateBuild -Version {}",
            variables.full_sem_ver
        ))
    }
    fn set_output_variables(&self, name: &str, value: Option<&str>) -> Vec<String> {
        value
            .map(|v| vec![format!("appveyor SetVariable -Name {name} -Value {v}")])
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::agents::BuildAgent;
    use crate::agents::appveyor::AppVeyor;
    use crate::output::variables::GitVersionVariables;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_appveyor_env_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("APPVEYOR", "true");
        }

        let agent = AppVeyor;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("APPVEYOR");
        }
    }

    #[test]
    fn get_current_branch_reads_appveyor_repo_branch() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("APPVEYOR_REPO_BRANCH", "release/3.0.0");
        }

        let agent = AppVeyor;
        assert_eq!(
            agent.get_current_branch(false).as_deref(),
            Some("release/3.0.0")
        );

        unsafe {
            std::env::remove_var("APPVEYOR_REPO_BRANCH");
        }
    }

    #[test]
    fn set_output_variables_uses_appveyor_command() {
        let agent = AppVeyor;
        assert_eq!(
            agent.set_output_variables("Foo", Some("bar")),
            vec!["appveyor SetVariable -Name Foo -Value bar"]
        );
    }

    #[test]
    fn set_build_number_uses_appveyor_update_command() {
        let vars = GitVersionVariables {
            full_sem_ver: "4.5.6".to_string(),
            ..Default::default()
        };

        let agent = AppVeyor;
        assert_eq!(
            agent.set_build_number(&vars).as_deref(),
            Some("appveyor UpdateBuild -Version 4.5.6")
        );
    }
}
