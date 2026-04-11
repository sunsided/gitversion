use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub enum VersionField {
    #[default]
    None,
    Patch,
    Minor,
    Major,
}

impl VersionField {
    pub fn consolidate(self, other: Self) -> Self {
        self.max(other)
    }
}

#[cfg(test)]
mod tests {
    use super::VersionField;

    #[test]
    fn consolidate_returns_max_field() {
        assert_eq!(
            VersionField::None.consolidate(VersionField::Patch),
            VersionField::Patch
        );
        assert_eq!(
            VersionField::Patch.consolidate(VersionField::Minor),
            VersionField::Minor
        );
        assert_eq!(
            VersionField::Minor.consolidate(VersionField::Major),
            VersionField::Major
        );
        assert_eq!(
            VersionField::Major.consolidate(VersionField::None),
            VersionField::Major
        );
    }

    #[test]
    fn fields_have_expected_ordering() {
        assert!(VersionField::None < VersionField::Patch);
        assert!(VersionField::Patch < VersionField::Minor);
        assert!(VersionField::Minor < VersionField::Major);
    }
}
