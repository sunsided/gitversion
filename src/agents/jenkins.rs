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
    use crate::agents::jenkins::Jenkins;
    use crate::output::variables::GitVersionVariables;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_jenkins_url_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("JENKINS_URL", "https://jenkins.example");
        }

        let agent = Jenkins;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("JENKINS_URL");
        }
    }

    #[test]
    fn get_current_branch_prefers_git_local_branch() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("GIT_LOCAL_BRANCH", "main");
            std::env::set_var("GIT_BRANCH", "origin/main");
        }

        let agent = Jenkins;
        assert_eq!(agent.get_current_branch(false).as_deref(), Some("main"));

        unsafe {
            std::env::remove_var("GIT_LOCAL_BRANCH");
            std::env::remove_var("GIT_BRANCH");
        }
    }

    #[test]
    fn set_build_number_uses_build_number_assignment() {
        let vars = GitVersionVariables {
            full_sem_ver: "1.2.3".to_string(),
            ..Default::default()
        };

        let agent = Jenkins;
        assert_eq!(
            agent.set_build_number(&vars).as_deref(),
            Some("BUILD_NUMBER=1.2.3")
        );
    }
}
