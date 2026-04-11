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
            Major: values.major,
            Minor: values.minor,
            Patch: values.patch,
            PreReleaseTag: values.pre_release_tag,
            PreReleaseTagWithDash: values.pre_release_tag_with_dash,
            PreReleaseLabel: values.pre_release_label,
            PreReleaseLabelWithDash: values.pre_release_label_with_dash,
            PreReleaseNumber: values.pre_release_number,
            WeightedPreReleaseNumber: values.weighted_pre_release_number,
            BuildMetaData: values.build_metadata,
            FullBuildMetaData: values.full_build_metadata,
            MajorMinorPatch: values.major_minor_patch,
            SemVer: values.semver,
            FullSemVer: values.full_semver,
            InformationalVersion: informational,
            BranchName: values.branch_name,
            EscapedBranchName: values.escaped_branch_name,
            Sha: values.sha,
            ShortSha: values.short_sha,
            CommitDate: values.commit_date,
            VersionSourceDistance: values.version_source_distance,
            VersionSourceIncrement: values.version_source_increment,
            VersionSourceSemVer: values.version_source_semver,
            VersionSourceSha: values.version_source_sha,
            UncommittedChanges: values.uncommitted_changes,
            AssemblySemVer: assembly_semver,
            AssemblySemFileVer: assembly_file_semver,
        }
    }
}
