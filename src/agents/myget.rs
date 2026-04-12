use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct MyGet;

impl BuildAgent for MyGet {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("BuildRunner").ok().as_deref() == Some("MyGet")
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
    use crate::agents::myget::MyGet;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_only_when_build_runner_is_myget() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("BuildRunner", "MyGet");
        }

        let agent = MyGet;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::set_var("BuildRunner", "Other");
        }
        assert!(!agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("BuildRunner");
        }
    }

    #[test]
    fn set_output_variables_uses_export_format() {
        let agent = MyGet;
        assert_eq!(
            agent.set_output_variables("Foo", Some("bar")),
            vec!["export Foo=bar"]
        );
    }
}
