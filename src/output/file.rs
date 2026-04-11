use std::fs;
use std::path::Path;

use eyre::Result;

use crate::output::json::to_json;
use crate::output::variables::GitVersionVariables;

pub fn write_output_file(path: &Path, variables: &GitVersionVariables) -> Result<()> {
    fs::write(path, to_json(variables)?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::write_output_file;
    use crate::output::json::to_json;
    use crate::output::variables::GitVersionVariables;

    #[test]
    fn write_output_file_writes_json_payload() {
        let dir = tempdir().expect("temp directory");
        let path = dir.path().join("gitversion.json");
        let variables = GitVersionVariables {
            major: "3".to_string(),
            ..Default::default()
        };

        write_output_file(&path, &variables).expect("file write succeeds");

        let content = fs::read_to_string(&path).expect("output file can be read");
        let expected = to_json(&variables).expect("json serialization succeeds");
        assert_eq!(content, expected);
    }

    #[test]
    fn write_output_file_overwrites_existing_file() {
        let dir = tempdir().expect("temp directory");
        let path = dir.path().join("gitversion.json");
        fs::write(&path, "old content").expect("seed old file");

        let variables = GitVersionVariables {
            branch_name: "main".to_string(),
            ..Default::default()
        };

        write_output_file(&path, &variables).expect("file write succeeds");

        let content = fs::read_to_string(&path).expect("output file can be read");
        assert!(content.contains("\"BranchName\": \"main\""));
        assert!(!content.contains("old content"));
    }

    #[test]
    fn write_output_file_returns_error_when_target_is_directory() {
        let dir = tempdir().expect("temp directory");

        let result = write_output_file(dir.path(), &GitVersionVariables::default());

        assert!(result.is_err());
    }
}
