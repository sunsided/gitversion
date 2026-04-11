use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct GitVersionVariables {
    #[serde(rename = "Major")]
    pub major: String,
    #[serde(rename = "Minor")]
    pub minor: String,
    #[serde(rename = "Patch")]
    pub patch: String,
    #[serde(rename = "PreReleaseTag")]
    pub pre_release_tag: String,
    #[serde(rename = "PreReleaseTagWithDash")]
    pub pre_release_tag_with_dash: String,
    #[serde(rename = "PreReleaseLabel")]
    pub pre_release_label: String,
    #[serde(rename = "PreReleaseLabelWithDash")]
    pub pre_release_label_with_dash: String,
    #[serde(rename = "PreReleaseNumber")]
    pub pre_release_number: String,
    #[serde(rename = "WeightedPreReleaseNumber")]
    pub weighted_pre_release_number: String,
    #[serde(rename = "BuildMetaData")]
    pub build_meta_data: String,
    #[serde(rename = "FullBuildMetaData")]
    pub full_build_meta_data: String,
    #[serde(rename = "MajorMinorPatch")]
    pub major_minor_patch: String,
    #[serde(rename = "SemVer")]
    pub sem_ver: String,
    #[serde(rename = "FullSemVer")]
    pub full_sem_ver: String,
    #[serde(rename = "InformationalVersion")]
    pub informational_version: String,
    #[serde(rename = "BranchName")]
    pub branch_name: String,
    #[serde(rename = "EscapedBranchName")]
    pub escaped_branch_name: String,
    #[serde(rename = "Sha")]
    pub sha: String,
    #[serde(rename = "ShortSha")]
    pub short_sha: String,
    #[serde(rename = "CommitDate")]
    pub commit_date: String,
    #[serde(rename = "VersionSourceDistance")]
    pub version_source_distance: String,
    #[serde(rename = "VersionSourceIncrement")]
    pub version_source_increment: String,
    #[serde(rename = "VersionSourceSemVer")]
    pub version_source_sem_ver: String,
    #[serde(rename = "VersionSourceSha")]
    pub version_source_sha: String,
    #[serde(rename = "UncommittedChanges")]
    pub uncommitted_changes: String,
    #[serde(rename = "AssemblySemVer")]
    pub assembly_sem_ver: String,
    #[serde(rename = "AssemblySemFileVer")]
    pub assembly_sem_file_ver: String,
}

impl GitVersionVariables {
    pub const AVAILABLE_VARIABLES: &'static [&'static str] = &[
        "Major",
        "Minor",
        "Patch",
        "PreReleaseTag",
        "PreReleaseTagWithDash",
        "PreReleaseLabel",
        "PreReleaseLabelWithDash",
        "PreReleaseNumber",
        "WeightedPreReleaseNumber",
        "BuildMetaData",
        "FullBuildMetaData",
        "MajorMinorPatch",
        "SemVer",
        "FullSemVer",
        "InformationalVersion",
        "BranchName",
        "EscapedBranchName",
        "Sha",
        "ShortSha",
        "CommitDate",
        "VersionSourceDistance",
        "VersionSourceIncrement",
        "VersionSourceSemVer",
        "VersionSourceSha",
        "UncommittedChanges",
        "AssemblySemVer",
        "AssemblySemFileVer",
    ];

    pub fn iter(&self) -> Vec<(&'static str, Option<&str>)> {
        Self::AVAILABLE_VARIABLES
            .iter()
            .map(|k| (*k, self.get(k)))
            .collect()
    }

    pub fn get(&self, name: &str) -> Option<&str> {
        match name {
            "Major" => Some(&self.major),
            "Minor" => Some(&self.minor),
            "Patch" => Some(&self.patch),
            "PreReleaseTag" => Some(&self.pre_release_tag),
            "PreReleaseTagWithDash" => Some(&self.pre_release_tag_with_dash),
            "PreReleaseLabel" => Some(&self.pre_release_label),
            "PreReleaseLabelWithDash" => Some(&self.pre_release_label_with_dash),
            "PreReleaseNumber" => Some(&self.pre_release_number),
            "WeightedPreReleaseNumber" => Some(&self.weighted_pre_release_number),
            "BuildMetaData" => Some(&self.build_meta_data),
            "FullBuildMetaData" => Some(&self.full_build_meta_data),
            "MajorMinorPatch" => Some(&self.major_minor_patch),
            "SemVer" => Some(&self.sem_ver),
            "FullSemVer" => Some(&self.full_sem_ver),
            "InformationalVersion" => Some(&self.informational_version),
            "BranchName" => Some(&self.branch_name),
            "EscapedBranchName" => Some(&self.escaped_branch_name),
            "Sha" => Some(&self.sha),
            "ShortSha" => Some(&self.short_sha),
            "CommitDate" => Some(&self.commit_date),
            "VersionSourceDistance" => Some(&self.version_source_distance),
            "VersionSourceIncrement" => Some(&self.version_source_increment),
            "VersionSourceSemVer" => Some(&self.version_source_sem_ver),
            "VersionSourceSha" => Some(&self.version_source_sha),
            "UncommittedChanges" => Some(&self.uncommitted_changes),
            "AssemblySemVer" => Some(&self.assembly_sem_ver),
            "AssemblySemFileVer" => Some(&self.assembly_sem_file_ver),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::GitVersionVariables;

    #[test]
    fn get_returns_expected_values_for_known_keys() {
        let vars = GitVersionVariables {
            major: "1".to_string(),
            full_sem_ver: "1.2.3-beta.4".to_string(),
            branch_name: "feature/test".to_string(),
            ..Default::default()
        };

        assert_eq!(vars.get("Major"), Some("1"));
        assert_eq!(vars.get("FullSemVer"), Some("1.2.3-beta.4"));
        assert_eq!(vars.get("BranchName"), Some("feature/test"));
        assert_eq!(vars.get("Unknown"), None);
    }

    #[test]
    fn iter_includes_all_available_variables_once() {
        let vars = GitVersionVariables::default();
        let iterated = vars.iter();

        assert_eq!(
            iterated.len(),
            GitVersionVariables::AVAILABLE_VARIABLES.len()
        );

        let keys: Vec<&str> = iterated.iter().map(|(key, _)| *key).collect();
        assert_eq!(keys, GitVersionVariables::AVAILABLE_VARIABLES);

        let unique: HashSet<&str> = keys.into_iter().collect();
        assert_eq!(unique.len(), GitVersionVariables::AVAILABLE_VARIABLES.len());
    }
}
