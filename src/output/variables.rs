use serde::Serialize;

#[derive(Debug, Clone, Default, Serialize)]
pub struct GitVersionVariables {
    pub Major: String,
    pub Minor: String,
    pub Patch: String,
    pub PreReleaseTag: String,
    pub PreReleaseTagWithDash: String,
    pub PreReleaseLabel: String,
    pub PreReleaseLabelWithDash: String,
    pub PreReleaseNumber: String,
    pub WeightedPreReleaseNumber: String,
    pub BuildMetaData: String,
    pub FullBuildMetaData: String,
    pub MajorMinorPatch: String,
    pub SemVer: String,
    pub FullSemVer: String,
    pub InformationalVersion: String,
    pub BranchName: String,
    pub EscapedBranchName: String,
    pub Sha: String,
    pub ShortSha: String,
    pub CommitDate: String,
    pub VersionSourceDistance: String,
    pub VersionSourceIncrement: String,
    pub VersionSourceSemVer: String,
    pub VersionSourceSha: String,
    pub UncommittedChanges: String,
    pub AssemblySemVer: String,
    pub AssemblySemFileVer: String,
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
            "Major" => Some(&self.Major),
            "Minor" => Some(&self.Minor),
            "Patch" => Some(&self.Patch),
            "PreReleaseTag" => Some(&self.PreReleaseTag),
            "PreReleaseTagWithDash" => Some(&self.PreReleaseTagWithDash),
            "PreReleaseLabel" => Some(&self.PreReleaseLabel),
            "PreReleaseLabelWithDash" => Some(&self.PreReleaseLabelWithDash),
            "PreReleaseNumber" => Some(&self.PreReleaseNumber),
            "WeightedPreReleaseNumber" => Some(&self.WeightedPreReleaseNumber),
            "BuildMetaData" => Some(&self.BuildMetaData),
            "FullBuildMetaData" => Some(&self.FullBuildMetaData),
            "MajorMinorPatch" => Some(&self.MajorMinorPatch),
            "SemVer" => Some(&self.SemVer),
            "FullSemVer" => Some(&self.FullSemVer),
            "InformationalVersion" => Some(&self.InformationalVersion),
            "BranchName" => Some(&self.BranchName),
            "EscapedBranchName" => Some(&self.EscapedBranchName),
            "Sha" => Some(&self.Sha),
            "ShortSha" => Some(&self.ShortSha),
            "CommitDate" => Some(&self.CommitDate),
            "VersionSourceDistance" => Some(&self.VersionSourceDistance),
            "VersionSourceIncrement" => Some(&self.VersionSourceIncrement),
            "VersionSourceSemVer" => Some(&self.VersionSourceSemVer),
            "VersionSourceSha" => Some(&self.VersionSourceSha),
            "UncommittedChanges" => Some(&self.UncommittedChanges),
            "AssemblySemVer" => Some(&self.AssemblySemVer),
            "AssemblySemFileVer" => Some(&self.AssemblySemFileVer),
            _ => None,
        }
    }
}
