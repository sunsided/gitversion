use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct CodeBuild;

impl BuildAgent for CodeBuild {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("CODEBUILD_CI").is_ok()
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
    use crate::agents::codebuild::CodeBuild;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_codebuild_ci_env_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("CODEBUILD_CI", "true");
        }

        let agent = CodeBuild;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("CODEBUILD_CI");
        }
    }

    #[test]
    fn set_output_variables_uses_export_format() {
        let agent = CodeBuild;
        assert_eq!(
            agent.set_output_variables("Foo", Some("bar")),
            vec!["export Foo=bar"]
        );
    }
}
