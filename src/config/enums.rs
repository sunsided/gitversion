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

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::{
        AssemblyVersioningScheme, CommitMessageIncrementMode, DeploymentMode, IncrementStrategy,
        SemanticVersionFormat, VersionStrategies,
    };

    #[derive(Debug, Serialize, Deserialize)]
    struct VersionStrategyHolder {
        version_strategy: VersionStrategies,
    }

    #[test]
    fn defaults_match_expected_values() {
        assert_eq!(IncrementStrategy::default(), IncrementStrategy::None);
        assert_eq!(DeploymentMode::default(), DeploymentMode::ManualDeployment);
        assert_eq!(
            CommitMessageIncrementMode::default(),
            CommitMessageIncrementMode::Enabled
        );
        assert_eq!(
            SemanticVersionFormat::default(),
            SemanticVersionFormat::Strict
        );
        assert_eq!(
            AssemblyVersioningScheme::default(),
            AssemblyVersioningScheme::MajorMinorPatchTag
        );
    }

    #[test]
    fn version_strategy_default_contains_all_non_mainline_strategies() {
        let strategy = VersionStrategies::default();

        assert!(strategy.contains(VersionStrategies::Fallback));
        assert!(strategy.contains(VersionStrategies::ConfiguredNextVersion));
        assert!(strategy.contains(VersionStrategies::MergeMessage));
        assert!(strategy.contains(VersionStrategies::TaggedCommit));
        assert!(strategy.contains(VersionStrategies::TrackReleaseBranches));
        assert!(strategy.contains(VersionStrategies::VersionInBranchName));
        assert!(!strategy.contains(VersionStrategies::Mainline));
    }

    #[test]
    fn version_strategy_serializes_and_deserializes_using_bits() {
        let holder = VersionStrategyHolder {
            version_strategy: VersionStrategies::Fallback | VersionStrategies::TaggedCommit,
        };

        let serialized = serde_json::to_string(&holder).expect("serialize strategies");
        assert_eq!(serialized, r#"{"version_strategy":9}"#);

        let deserialized: VersionStrategyHolder =
            serde_json::from_str(&serialized).expect("deserialize strategies");
        assert_eq!(deserialized.version_strategy, holder.version_strategy);
    }

    #[test]
    fn version_strategy_deserialize_fails_for_unknown_bits() {
        let err = serde_json::from_str::<VersionStrategyHolder>(r#"{"version_strategy":1024}"#)
            .expect_err("invalid bits should fail");

        assert!(err.to_string().contains("invalid version strategy bits"));
    }

    #[test]
    fn version_strategy_default_bits_match_expected_mask() {
        assert_eq!(VersionStrategies::default().bits(), 63);
    }

    #[test]
    fn version_strategy_default_serializes_to_expected_mask() {
        let holder = VersionStrategyHolder {
            version_strategy: VersionStrategies::default(),
        };

        let serialized = serde_json::to_string(&holder).expect("serialize default strategies");
        assert_eq!(serialized, r#"{"version_strategy":63}"#);
    }

    #[test]
    fn version_strategy_deserializes_empty_mask() {
        let holder: VersionStrategyHolder =
            serde_json::from_str(r#"{"version_strategy":0}"#).expect("deserialize empty mask");

        assert!(holder.version_strategy.is_empty());
    }

    #[test]
    fn version_strategy_deserializes_mainline_only_mask() {
        let holder: VersionStrategyHolder =
            serde_json::from_str(r#"{"version_strategy":64}"#).expect("deserialize mainline");

        assert_eq!(holder.version_strategy, VersionStrategies::Mainline);
    }

    #[test]
    fn version_strategy_deserialize_fails_for_partially_unknown_bits() {
        let err = serde_json::from_str::<VersionStrategyHolder>(r#"{"version_strategy":129}"#)
            .expect_err("mixed known and unknown bits should fail");

        assert!(err.to_string().contains("invalid version strategy bits"));
    }

    #[test]
    fn enum_variants_roundtrip_via_serde_json() {
        let increment: IncrementStrategy =
            serde_json::from_str(&serde_json::to_string(&IncrementStrategy::Inherit).unwrap())
                .expect("roundtrip increment");
        let deployment: DeploymentMode = serde_json::from_str(
            &serde_json::to_string(&DeploymentMode::ContinuousDeployment).unwrap(),
        )
        .expect("roundtrip deployment");
        let commit_mode: CommitMessageIncrementMode = serde_json::from_str(
            &serde_json::to_string(&CommitMessageIncrementMode::MergeMessageOnly).unwrap(),
        )
        .expect("roundtrip commit mode");
        let semver_format: SemanticVersionFormat =
            serde_json::from_str(&serde_json::to_string(&SemanticVersionFormat::Loose).unwrap())
                .expect("roundtrip semver format");
        let assembly: AssemblyVersioningScheme = serde_json::from_str(
            &serde_json::to_string(&AssemblyVersioningScheme::MajorMinor).unwrap(),
        )
        .expect("roundtrip assembly scheme");

        assert_eq!(increment, IncrementStrategy::Inherit);
        assert_eq!(deployment, DeploymentMode::ContinuousDeployment);
        assert_eq!(commit_mode, CommitMessageIncrementMode::MergeMessageOnly);
        assert_eq!(semver_format, SemanticVersionFormat::Loose);
        assert_eq!(assembly, AssemblyVersioningScheme::MajorMinor);
    }
}
