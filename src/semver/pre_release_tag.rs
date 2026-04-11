use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub struct SemanticVersionPreReleaseTag {
    pub name: String,
    pub number: Option<i64>,
    pub promote_tag_even_if_name_is_empty: bool,
}

impl SemanticVersionPreReleaseTag {
    pub fn has_tag(&self) -> bool {
        !self.name.is_empty() || (self.number.is_some() && self.promote_tag_even_if_name_is_empty)
    }

    pub fn parse(s: &str) -> Self {
        if s.is_empty() {
            return Self::default();
        }
        if let Ok(number) = s.parse::<i64>() {
            return Self {
                name: String::new(),
                number: Some(number),
                promote_tag_even_if_name_is_empty: true,
            };
        }
        let mut split = s.split('.');
        let first = split.next().unwrap_or_default().to_string();
        let second = split.next().and_then(|v| v.parse::<i64>().ok());
        Self {
            name: first,
            number: second,
            promote_tag_even_if_name_is_empty: false,
        }
    }
}

impl Display for SemanticVersionPreReleaseTag {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match (&self.name.is_empty(), self.number) {
            (false, Some(n)) => write!(f, "{}.{}", self.name, n),
            (false, None) => write!(f, "{}", self.name),
            (true, Some(n)) => write!(f, "{}", n),
            (true, None) => Ok(()),
        }
    }
}

impl PartialOrd for SemanticVersionPreReleaseTag {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SemanticVersionPreReleaseTag {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.has_tag(), other.has_tag()) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            _ => {
                let name_cmp = self
                    .name
                    .to_ascii_lowercase()
                    .cmp(&other.name.to_ascii_lowercase());
                if name_cmp != Ordering::Equal {
                    return name_cmp;
                }
                self.number.unwrap_or(0).cmp(&other.number.unwrap_or(0))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::SemanticVersionPreReleaseTag;

    #[test]
    fn has_tag_is_false_for_default() {
        assert!(!SemanticVersionPreReleaseTag::default().has_tag());
    }

    #[test]
    fn has_tag_is_true_when_name_is_present() {
        let tag = SemanticVersionPreReleaseTag {
            name: "beta".to_string(),
            ..Default::default()
        };

        assert!(tag.has_tag());
    }

    #[test]
    fn has_tag_is_true_for_numeric_tag_with_promote_flag() {
        let tag = SemanticVersionPreReleaseTag {
            number: Some(3),
            promote_tag_even_if_name_is_empty: true,
            ..Default::default()
        };

        assert!(tag.has_tag());
    }

    #[rstest]
    #[case("", "", None, false)]
    #[case("beta", "beta", None, false)]
    #[case("beta.3", "beta", Some(3), false)]
    #[case("3", "", Some(3), true)]
    fn parse_understands_name_and_number_parts(
        #[case] input: &str,
        #[case] expected_name: &str,
        #[case] expected_number: Option<i64>,
        #[case] expected_promote: bool,
    ) {
        let parsed = SemanticVersionPreReleaseTag::parse(input);

        assert_eq!(parsed.name, expected_name);
        assert_eq!(parsed.number, expected_number);
        assert_eq!(parsed.promote_tag_even_if_name_is_empty, expected_promote);
    }

    #[rstest]
    #[case("beta", "beta")]
    #[case("beta.4", "beta.4")]
    #[case("5", "5")]
    #[case("", "")]
    fn display_outputs_expected_format(#[case] input: &str, #[case] expected: &str) {
        let parsed = SemanticVersionPreReleaseTag::parse(input);
        assert_eq!(parsed.to_string(), expected);
    }

    #[test]
    fn no_tag_sorts_after_tagged_versions() {
        let tagged = SemanticVersionPreReleaseTag::parse("beta.1");
        let release = SemanticVersionPreReleaseTag::default();

        assert!(release > tagged);
    }

    #[test]
    fn compare_is_case_insensitive_and_uses_number() {
        let lower = SemanticVersionPreReleaseTag::parse("beta.1");
        let upper = SemanticVersionPreReleaseTag::parse("BETA.2");

        assert!(upper > lower);
    }
}
