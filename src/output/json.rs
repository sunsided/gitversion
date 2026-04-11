use eyre::Result;

use crate::output::variables::GitVersionVariables;

pub fn to_json(variables: &GitVersionVariables) -> Result<String> {
    Ok(serde_json::to_string_pretty(variables)?)
}

#[cfg(test)]
mod tests {
    use super::to_json;
    use crate::output::variables::GitVersionVariables;

    #[test]
    fn to_json_serializes_expected_property_names() {
        let variables = GitVersionVariables {
            major: "1".to_string(),
            full_sem_ver: "1.2.3-beta.1".to_string(),
            ..Default::default()
        };

        let json = to_json(&variables).expect("json serialization succeeds");

        assert!(json.contains("\"Major\": \"1\""));
        assert!(json.contains("\"FullSemVer\": \"1.2.3-beta.1\""));
    }

    #[test]
    fn to_json_outputs_pretty_printed_json() {
        let variables = GitVersionVariables::default();

        let json = to_json(&variables).expect("json serialization succeeds");

        assert!(json.starts_with("{\n"));
        assert!(json.contains("\n  \"Major\": \"\","));
        assert!(json.ends_with("\n}"));
    }

    #[test]
    fn to_json_produces_valid_json_document() {
        let variables = GitVersionVariables {
            branch_name: "feature/sprint-2".to_string(),
            ..Default::default()
        };

        let json = to_json(&variables).expect("json serialization succeeds");
        let value: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        assert_eq!(
            value.get("BranchName").and_then(|v| v.as_str()),
            Some("feature/sprint-2")
        );
    }
}
