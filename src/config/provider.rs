use std::collections::HashMap;
use std::path::{Path, PathBuf};

use eyre::Result;
use figment::{
    providers::{Format, Serialized, Yaml},
    Figment,
};

use crate::config::gitversion_config::GitVersionConfiguration;
use crate::config::workflows;

#[derive(Debug, Default)]
pub struct ConfigurationProvider;

impl ConfigurationProvider {
    pub fn find_config_file(&self, directory: &Path) -> Option<PathBuf> {
        [
            "GitVersion.yml",
            "GitVersion.yaml",
            ".GitVersion.yml",
            ".GitVersion.yaml",
        ]
        .iter()
        .map(|name| directory.join(name))
        .find(|path| path.exists())
    }

    pub fn provide(
        &self,
        working_dir: &Path,
        explicit_file: Option<&Path>,
        override_config: HashMap<String, String>,
    ) -> Result<GitVersionConfiguration> {
        let mut config = GitVersionConfiguration::default();
        config.branches = workflows::resolve(&config.workflow);

        let file = explicit_file
            .map(ToOwned::to_owned)
            .or_else(|| self.find_config_file(working_dir));

        let mut figment = Figment::new().merge(Serialized::defaults(config.clone()));
        if let Some(file) = file {
            figment = figment.merge(Yaml::file(file));
        }

        for (k, v) in override_config {
            figment = figment.merge(Serialized::from(HashMap::from([(k, v)]), "override"));
        }

        let mut merged: GitVersionConfiguration = figment.extract()?;
        if merged.branches.is_empty() {
            merged.branches = workflows::resolve(&merged.workflow);
        }
        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;

    use tempfile::tempdir;

    use crate::config::provider::ConfigurationProvider;

    #[test]
    fn find_config_file_prefers_primary_name_order() {
        let dir = tempdir().expect("tempdir");
        fs::write(dir.path().join("GitVersion.yml"), "workflow: GitFlow/v1").expect("write");
        fs::write(
            dir.path().join(".GitVersion.yml"),
            "workflow: TrunkBased/preview1",
        )
        .expect("write");

        let provider = ConfigurationProvider;
        let found = provider
            .find_config_file(dir.path())
            .expect("config should exist");

        assert_eq!(
            found.file_name().and_then(|v| v.to_str()),
            Some("GitVersion.yml")
        );
    }

    #[test]
    fn provide_reads_values_from_yaml_file() {
        let dir = tempdir().expect("tempdir");
        fs::write(
            dir.path().join("GitVersion.yml"),
            "workflow: GitFlow/v1\nupdate_build_number: false\n",
        )
        .expect("write");

        let provider = ConfigurationProvider;
        let config = provider
            .provide(dir.path(), None, HashMap::new())
            .expect("configuration should load");

        assert!(!config.update_build_number);
    }
}
