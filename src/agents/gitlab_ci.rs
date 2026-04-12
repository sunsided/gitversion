use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct GitLabCI;

impl BuildAgent for GitLabCI {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("GITLAB_CI").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        env::var("CI_COMMIT_REF_NAME").ok()
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

    use crate::agents::gitlab_ci::GitLabCI;
    use crate::agents::BuildAgent;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_gitlab_ci_env_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("GITLAB_CI", "true");
        }

        let agent = GitLabCI;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("GITLAB_CI");
        }
    }

    #[test]
    fn get_current_branch_reads_ci_commit_ref_name() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("CI_COMMIT_REF_NAME", "release/1.2.3");
        }

        let agent = GitLabCI;
        assert_eq!(
            agent.get_current_branch(false).as_deref(),
            Some("release/1.2.3")
        );

        unsafe {
            std::env::remove_var("CI_COMMIT_REF_NAME");
        }
    }

    #[test]
    fn set_output_variables_emits_export_command() {
        let agent = GitLabCI;
        let output = agent.set_output_variables("Foo", Some("bar"));

        assert_eq!(output, vec!["export Foo=bar"]);
    }
}
