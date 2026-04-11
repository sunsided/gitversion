use std::env;
use std::fs::OpenOptions;
use std::io::Write;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct GitHubActions;

impl BuildAgent for GitHubActions {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("GITHUB_ACTIONS").is_ok()
    }
    fn get_current_branch(&self, _using_dynamic_repos: bool) -> Option<String> {
        if env::var("GITHUB_REF_TYPE").ok().as_deref() == Some("tag") {
            return None;
        }
        env::var("GITHUB_REF").ok()
    }
    fn set_build_number(&self, variables: &GitVersionVariables) -> Option<String> {
        Some(format!("::notice::Build {}", variables.FullSemVer))
    }
    fn set_output_variables(&self, name: &str, value: Option<&str>) -> Vec<String> {
        if let (Ok(path), Some(value)) = (env::var("GITHUB_ENV"), value)
            && let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path)
        {
            let _ = writeln!(file, "{name}={value}");
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use tempfile::NamedTempFile;

    use crate::agents::BuildAgent;
    use crate::agents::github_actions::GitHubActions;
    use crate::output::variables::GitVersionVariables;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_env_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("GITHUB_ACTIONS", "true");
        }

        let agent = GitHubActions;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("GITHUB_ACTIONS");
        }
    }

    #[test]
    fn get_current_branch_returns_none_for_tags() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("GITHUB_REF_TYPE", "tag");
            std::env::set_var("GITHUB_REF", "refs/tags/v1.2.3");
        }

        let agent = GitHubActions;
        assert_eq!(agent.get_current_branch(false), None);

        unsafe {
            std::env::remove_var("GITHUB_REF_TYPE");
            std::env::remove_var("GITHUB_REF");
        }
    }

    #[test]
    fn set_output_variables_writes_to_github_env_file() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        let env_file = NamedTempFile::new().expect("temp file");
        unsafe {
            std::env::set_var("GITHUB_ENV", env_file.path());
        }

        let agent = GitHubActions;
        let _ = agent.set_output_variables("Foo", Some("1.0.0"));

        let content = fs::read_to_string(env_file.path()).expect("read github env file");
        assert!(content.contains("Foo=1.0.0"));

        unsafe {
            std::env::remove_var("GITHUB_ENV");
        }
    }

    #[test]
    fn set_build_number_uses_notice_format() {
        let vars = GitVersionVariables {
            FullSemVer: "1.2.3".to_string(),
            ..Default::default()
        };

        let agent = GitHubActions;
        assert_eq!(agent.set_build_number(&vars).as_deref(), Some("::notice::Build 1.2.3"));
    }
}
