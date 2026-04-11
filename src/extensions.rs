use regex::Regex;
use serde::Serialize;
use serde_json::Value;

use crate::regex_patterns::TOKEN_EXPANSION;

pub fn is_equivalent_to(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

pub fn with_prefix_if_not_empty(s: &str, prefix: &str) -> String {
    if s.is_empty() {
        String::new()
    } else {
        format!("{prefix}{s}")
    }
}

pub fn regex_replace(s: &str, pattern: &Regex, replacement: &str) -> String {
    pattern.replace_all(s, replacement).to_string()
}

pub fn format_with<T: Serialize>(
    template: &str,
    source: &T,
    env: &dyn Fn(&str) -> Option<String>,
) -> String {
    let source_json = serde_json::to_value(source).unwrap_or(Value::Null);
    TOKEN_EXPANSION
        .replace_all(template, |caps: &regex::Captures<'_>| {
            let expr = caps.name("token").map(|m| m.as_str()).unwrap_or_default();
            let mut parts = expr.split("??").map(str::trim);
            let left = parts.next().unwrap_or_default();
            let fallback = parts
                .next()
                .unwrap_or_default()
                .trim_matches('"')
                .trim_matches('\'');

            let value = if let Some(env_key) = left.strip_prefix("env:") {
                env(env_key).unwrap_or_default()
            } else {
                let key = left.split(':').next().unwrap_or_default();
                source_json
                    .get(key)
                    .and_then(|v| {
                        v.as_str()
                            .map(ToString::to_string)
                            .or_else(|| Some(v.to_string()))
                    })
                    .unwrap_or_default()
            };

            if value.is_empty() {
                fallback.to_string()
            } else {
                value
            }
        })
        .to_string()
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use once_cell::sync::Lazy;
    use regex::Regex;
    use serde::Serialize;

    use super::{format_with, is_equivalent_to, regex_replace, with_prefix_if_not_empty};

    static ENV_LOCK: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

    #[derive(Serialize)]
    struct Vars {
        #[serde(rename = "Major")]
        major: String,
        #[serde(rename = "Minor")]
        minor: String,
        #[serde(rename = "Missing")]
        missing: Option<String>,
    }

    #[test]
    fn is_equivalent_to_is_case_insensitive() {
        assert!(is_equivalent_to("main", "Main"));
        assert!(is_equivalent_to("FEATURE", "feature"));
        assert!(!is_equivalent_to("main", "develop"));
    }

    #[test]
    fn with_prefix_if_not_empty_applies_prefix_conditionally() {
        assert_eq!(with_prefix_if_not_empty("foo", "-"), "-foo");
        assert_eq!(with_prefix_if_not_empty("", "-"), "");
    }

    #[test]
    fn regex_replace_substitutes_all_matches() {
        let pattern = Regex::new("a+").expect("valid regex");
        assert_eq!(regex_replace("caaab aa", &pattern, "x"), "cxb x");
    }

    #[test]
    fn format_with_resolves_tokens() {
        let vars = Vars {
            major: "1".to_string(),
            minor: "2".to_string(),
            missing: None,
        };

        let result = format_with("{Major}.{Minor}.0", &vars, &|_| None);
        assert_eq!(result, "1.2.0");
    }

    #[test]
    fn format_with_resolves_env_tokens() {
        let _guard = ENV_LOCK.lock().expect("env lock poisoned");
        unsafe {
            std::env::set_var("GITVERSION_TEST_VAR", "hello");
        }

        let vars = Vars {
            major: "1".to_string(),
            minor: "2".to_string(),
            missing: None,
        };
        let result = format_with("{env:GITVERSION_TEST_VAR}", &vars, &|k| {
            std::env::var(k).ok()
        });

        unsafe {
            std::env::remove_var("GITVERSION_TEST_VAR");
        }

        assert_eq!(result, "hello");
    }

    #[test]
    fn format_with_uses_fallback_for_missing_values() {
        let vars = Vars {
            major: "1".to_string(),
            minor: "2".to_string(),
            missing: None,
        };

        let result = format_with("{Unknown ?? 'default'}", &vars, &|_| None);
        assert_eq!(result, "default");
    }
}
