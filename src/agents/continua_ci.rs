use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct ContinuaCI;

impl BuildAgent for ContinuaCI {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("ContinuaCI.Version").is_ok()
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
    use crate::agents::continua_ci::ContinuaCI;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_continua_version_env_is_set() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("ContinuaCI.Version", "1");
        }

        let agent = ContinuaCI;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("ContinuaCI.Version");
        }
    }

    #[test]
    fn set_output_variables_uses_export_format() {
        let agent = ContinuaCI;
        assert_eq!(
            agent.set_output_variables("Foo", Some("bar")),
            vec!["export Foo=bar"]
        );
    }
}
