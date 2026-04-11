use bitflags::bitflags;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IncrementStrategy {
    #[default]
    None,
    Major,
    Minor,
    Patch,
    Inherit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeploymentMode {
    #[default]
    ManualDeployment,
    ContinuousDelivery,
    ContinuousDeployment,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum CommitMessageIncrementMode {
    #[default]
    Enabled,
    Disabled,
    MergeMessageOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SemanticVersionFormat {
    #[default]
    Strict,
    Loose,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct VersionStrategies: u32 {
        const Fallback = 1;
        const ConfiguredNextVersion = 2;
        const MergeMessage = 4;
        const TaggedCommit = 8;
        const TrackReleaseBranches = 16;
        const VersionInBranchName = 32;
        const Mainline = 64;
    }
}

impl Default for VersionStrategies {
    fn default() -> Self {
        Self::Fallback
            | Self::ConfiguredNextVersion
            | Self::MergeMessage
            | Self::TaggedCommit
            | Self::TrackReleaseBranches
            | Self::VersionInBranchName
    }
}

impl Serialize for VersionStrategies {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u32(self.bits())
    }
}

impl<'de> Deserialize<'de> for VersionStrategies {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u32::deserialize(deserializer)?;
        Self::from_bits(bits)
            .ok_or_else(|| serde::de::Error::custom("invalid version strategy bits"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AssemblyVersioningScheme {
    #[default]
    MajorMinorPatchTag,
    MajorMinorPatch,
    MajorMinor,
    Major,
    None,
}

pub type AssemblyFileVersioningScheme = AssemblyVersioningScheme;
