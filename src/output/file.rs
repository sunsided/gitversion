use std::fs;
use std::path::Path;

use eyre::Result;

use crate::output::json::to_json;
use crate::output::variables::GitVersionVariables;

pub fn write_output_file(path: &Path, variables: &GitVersionVariables) -> Result<()> {
    fs::write(path, to_json(variables)?)?;
    Ok(())
}
