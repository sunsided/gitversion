use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct BitBucketPipelines;

impl BuildAgent for BitBucketPipelines {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("BITBUCKET_WORKSPACE").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        env::var("BITBUCKET_BRANCH").ok()
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
    use crate::agents::bitbucket::BitBucketPipelines;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_workspace_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("BITBUCKET_WORKSPACE", "workspace");
        }

        let agent = BitBucketPipelines;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("BITBUCKET_WORKSPACE");
        }
    }

    #[test]
    fn get_current_branch_reads_bitbucket_branch() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("BITBUCKET_BRANCH", "feature/test");
        }

        let agent = BitBucketPipelines;
        assert_eq!(
            agent.get_current_branch(false).as_deref(),
            Some("feature/test")
        );

        unsafe {
            std::env::remove_var("BITBUCKET_BRANCH");
        }
    }

    #[test]
    fn set_output_variables_uses_export_format() {
        let agent = BitBucketPipelines;
        assert_eq!(
            agent.set_output_variables("Foo", Some("bar")),
            vec!["export Foo=bar"]
        );
    }
}
