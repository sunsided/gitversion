use std::fs;
use std::path::Path;

use eyre::Result;

use crate::output::variables::GitVersionVariables;

pub fn write_dotenv(path: &Path, variables: &GitVersionVariables) -> Result<()> {
    let body = variables
        .iter()
        .into_iter()
        .filter_map(|(k, v)| v.map(|v| format!("{k}={v}")))
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, body)?;
    Ok(())
}
