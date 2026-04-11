use eyre::Result;

use crate::output::variables::GitVersionVariables;

pub fn to_json(variables: &GitVersionVariables) -> Result<String> {
    Ok(serde_json::to_string_pretty(variables)?)
}
