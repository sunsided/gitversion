use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct TravisCI;

impl BuildAgent for TravisCI {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("TRAVIS").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        env::var("TRAVIS_BRANCH").ok()
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
    use crate::agents::travis_ci::TravisCI;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_travis_env_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("TRAVIS", "true");
        }

        let agent = TravisCI;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("TRAVIS");
        }
    }

    #[test]
    fn get_current_branch_reads_travis_branch() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("TRAVIS_BRANCH", "feature/x");
        }

        let agent = TravisCI;
        assert_eq!(
            agent.get_current_branch(false).as_deref(),
            Some("feature/x")
        );

        unsafe {
            std::env::remove_var("TRAVIS_BRANCH");
        }
    }

    #[test]
    fn set_output_variables_uses_export_format() {
        let agent = TravisCI;
        assert_eq!(
            agent.set_output_variables("Foo", Some("bar")),
            vec!["export Foo=bar"]
        );
    }
}
