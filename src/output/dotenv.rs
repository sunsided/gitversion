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

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::write_dotenv;
    use crate::output::variables::GitVersionVariables;

    #[test]
    fn write_dotenv_writes_expected_key_value_rows() {
        let dir = tempdir().expect("temp directory");
        let path = dir.path().join(".env");
        let variables = GitVersionVariables {
            major: "1".to_string(),
            branch_name: "feature/sprint-2".to_string(),
            ..Default::default()
        };

        write_dotenv(&path, &variables).expect("dotenv write succeeds");

        let content = fs::read_to_string(&path).expect("dotenv file can be read");
        assert!(content.contains("Major=1"));
        assert!(content.contains("BranchName=feature/sprint-2"));
    }

    #[test]
    fn write_dotenv_writes_blank_value_as_empty_assignment() {
        let dir = tempdir().expect("temp directory");
        let path = dir.path().join(".env");

        write_dotenv(&path, &GitVersionVariables::default()).expect("dotenv write succeeds");

        let content = fs::read_to_string(&path).expect("dotenv file can be read");
        assert!(content.contains("Major="));
        assert!(content.contains("FullSemVer="));
        assert!(content.lines().all(|line| line.contains('=')));
    }

    #[test]
    fn write_dotenv_overwrites_existing_file_content() {
        let dir = tempdir().expect("temp directory");
        let path = dir.path().join(".env");
        fs::write(&path, "OLD=VALUE").expect("seed old file");

        let variables = GitVersionVariables {
            major: "2".to_string(),
            ..Default::default()
        };

        write_dotenv(&path, &variables).expect("dotenv write succeeds");

        let content = fs::read_to_string(&path).expect("dotenv file can be read");
        assert!(content.contains("Major=2"));
        assert!(!content.contains("OLD=VALUE"));
    }

    #[test]
    fn write_dotenv_returns_error_when_target_is_directory() {
        let dir = tempdir().expect("temp directory");

        let result = write_dotenv(dir.path(), &GitVersionVariables::default());

        assert!(result.is_err());
    }
}
