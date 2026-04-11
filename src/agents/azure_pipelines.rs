use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct AzurePipelines;

impl BuildAgent for AzurePipelines {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("TF_BUILD").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        env::var("GIT_BRANCH")
            .ok()
            .or_else(|| env::var("BUILD_SOURCEBRANCH").ok())
    }
    fn set_build_number(&self, variables: &GitVersionVariables) -> Option<String> {
        Some(format!(
            "##vso[build.updatebuildnumber]{}",
            variables.full_sem_ver
        ))
    }
    fn set_output_variables(&self, name: &str, value: Option<&str>) -> Vec<String> {
        value
            .map(|v| vec![format!("##vso[task.setvariable variable={name};]{v}")])
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::agents::azure_pipelines::AzurePipelines;
    use crate::agents::BuildAgent;
    use crate::output::variables::GitVersionVariables;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_tf_build_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("TF_BUILD", "True");
        }

        let agent = AzurePipelines;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("TF_BUILD");
        }
    }

    #[test]
    fn get_current_branch_prefers_git_branch() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("GIT_BRANCH", "refs/heads/main");
            std::env::set_var("BUILD_SOURCEBRANCH", "refs/heads/fallback");
        }

        let agent = AzurePipelines;
        assert_eq!(
            agent.get_current_branch(false).as_deref(),
            Some("refs/heads/main")
        );

        unsafe {
            std::env::remove_var("GIT_BRANCH");
            std::env::remove_var("BUILD_SOURCEBRANCH");
        }
    }

    #[test]
    fn set_build_number_outputs_vso_command() {
        let vars = GitVersionVariables {
            full_sem_ver: "0.0.0-Unstable4".to_string(),
            ..Default::default()
        };

        let agent = AzurePipelines;
        let result = agent
            .set_build_number(&vars)
            .expect("build number command should be generated");
        assert!(result.starts_with("##vso[build.updatebuildnumber]"));
    }

    #[test]
    fn set_output_variables_outputs_vso_variable_command() {
        let agent = AzurePipelines;
        let result = agent.set_output_variables("Foo", Some("1.0.0"));

        assert_eq!(result, vec!["##vso[task.setvariable variable=Foo;]1.0.0"]);
    }
}
