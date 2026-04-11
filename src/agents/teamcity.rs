use std::env;

use crate::agents::BuildAgent;
use crate::output::variables::GitVersionVariables;

#[derive(Debug)]
pub struct TeamCity;

impl BuildAgent for TeamCity {
    fn can_apply_to_current_context(&self) -> bool {
        env::var("TEAMCITY_VERSION").is_ok()
    }
    fn set_build_number(&self, variables: &GitVersionVariables) -> Option<String> {
        Some(format!(
            "##teamcity[buildNumber '{}']",
            variables.full_sem_ver
        ))
    }
    fn set_output_variables(&self, name: &str, value: Option<&str>) -> Vec<String> {
        value
            .map(|v| {
                vec![format!(
                    "##teamcity[setParameter name='env.{name}' value='{v}']"
                )]
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;

    use crate::agents::teamcity::TeamCity;
    use crate::agents::BuildAgent;
    use crate::output::variables::GitVersionVariables;

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[test]
    fn can_apply_when_teamcity_version_is_present() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("TEAMCITY_VERSION", "2024.1");
        }

        let agent = TeamCity;
        assert!(agent.can_apply_to_current_context());

        unsafe {
            std::env::remove_var("TEAMCITY_VERSION");
        }
    }

    #[test]
    fn set_build_number_teamcity_format() {
        let vars = GitVersionVariables {
            full_sem_ver: "1.0.0".to_string(),
            ..Default::default()
        };

        let agent = TeamCity;
        assert_eq!(
            agent.set_build_number(&vars),
            Some("##teamcity[buildNumber '1.0.0']".to_string())
        );
    }

    #[test]
    fn set_output_variables_teamcity_format() {
        let agent = TeamCity;
        let result = agent.set_output_variables("Foo", Some("bar"));

        assert_eq!(
            result,
            vec!["##teamcity[setParameter name='env.Foo' value='bar']".to_string()]
        );
    }
}
