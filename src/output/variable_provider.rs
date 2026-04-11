use crate::config::gitversion_config::GitVersionConfiguration;
use crate::extensions::format_with;
use crate::output::variables::GitVersionVariables;
use crate::semver::{SemanticVersion, SemanticVersionFormatValues};

#[derive(Debug, Default)]
pub struct VariableProvider;

impl VariableProvider {
    pub fn get_variables_for(
        &self,
        semver: &SemanticVersion,
        config: &GitVersionConfiguration,
        pre_release_weight: i64,
    ) -> GitVersionVariables {
        let values = SemanticVersionFormatValues::new(semver, config, pre_release_weight);
        let informational = format_with(&config.assembly_informational_format, &values, &|k| {
            std::env::var(k).ok()
        });
        let assembly_semver = format_with(&config.assembly_versioning_format, &values, &|k| {
            std::env::var(k).ok()
        });
        let assembly_file_semver =
            format_with(&config.assembly_file_versioning_format, &values, &|k| {
                std::env::var(k).ok()
            });
        GitVersionVariables {
            major: values.major,
            minor: values.minor,
            patch: values.patch,
            pre_release_tag: values.pre_release_tag,
            pre_release_tag_with_dash: values.pre_release_tag_with_dash,
            pre_release_label: values.pre_release_label,
            pre_release_label_with_dash: values.pre_release_label_with_dash,
            pre_release_number: values.pre_release_number,
            weighted_pre_release_number: values.weighted_pre_release_number,
            build_meta_data: values.build_metadata,
            full_build_meta_data: values.full_build_metadata,
            major_minor_patch: values.major_minor_patch,
            sem_ver: values.semver,
            full_sem_ver: values.full_semver,
            informational_version: informational,
            branch_name: values.branch_name,
            escaped_branch_name: values.escaped_branch_name,
            sha: values.sha,
            short_sha: values.short_sha,
            commit_date: values.commit_date,
            version_source_distance: values.version_source_distance,
            version_source_increment: values.version_source_increment,
            version_source_sem_ver: values.version_source_semver,
            version_source_sha: values.version_source_sha,
            uncommitted_changes: values.uncommitted_changes,
            assembly_sem_ver: assembly_semver,
            assembly_sem_file_ver: assembly_file_semver,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};

    use super::VariableProvider;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::semver::SemanticVersion;

    #[test]
    fn get_variables_for_includes_build_metadata_fields() {
        let mut semver = SemanticVersion::new(1, 2, 3);
        semver.build_metadata.sha = Some("0123456789abcdef".to_string());
        semver.build_metadata.short_sha = Some("0123456".to_string());
        semver.build_metadata.branch = Some("feature/build-metadata".to_string());
        semver.build_metadata.commit_date = Some(
            Utc.with_ymd_and_hms(2025, 3, 14, 9, 26, 53)
                .single()
                .expect("valid timestamp"),
        );
        semver.build_metadata.uncommitted_changes = 2;

        let variables =
            VariableProvider.get_variables_for(&semver, &GitVersionConfiguration::default(), 0);

        assert_eq!(variables.sha, "0123456789abcdef");
        assert_eq!(variables.short_sha, "0123456");
        assert_eq!(variables.branch_name, "feature/build-metadata");
        assert_eq!(variables.commit_date, "2025-03-14");
        assert_eq!(variables.uncommitted_changes, "2");
    }
}
