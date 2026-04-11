use std::collections::HashMap;

use eyre::Result;

use crate::config::gitversion_config::GitVersionConfiguration;
use crate::git::git2_impl::branch::Git2Branch;
use crate::git::git2_impl::repository::Git2Repository;
use crate::git::tagged_semver::SemanticVersionWithTag;
use crate::semver::SemanticVersion;

#[derive(Debug, Default)]
pub struct TaggedSemanticVersionService;

impl TaggedSemanticVersionService {
    pub fn get_tagged_semantic_versions(
        &self,
        repo: &Git2Repository,
        _branch: &Git2Branch,
        config: &GitVersionConfiguration,
    ) -> Result<HashMap<String, Vec<SemanticVersionWithTag>>> {
        let mut map: HashMap<String, Vec<SemanticVersionWithTag>> = HashMap::new();
        for tag in repo.tags()? {
            let name = tag.name.friendly();
            if let Some(version) = SemanticVersion::try_parse(
                &name,
                Some(&config.tag_prefix_pattern),
                config.semantic_version_format,
            ) {
                map.entry(tag.commit_sha.clone())
                    .or_default()
                    .push(SemanticVersionWithTag {
                        value: version,
                        tag,
                    });
            }
        }
        Ok(map)
    }
}

#[cfg(test)]
mod tests {
    use super::TaggedSemanticVersionService;
    use crate::config::enums::SemanticVersionFormat;
    use crate::config::gitversion_config::GitVersionConfiguration;
    use crate::git::git2_impl::repository::Git2Repository;
    use crate::testing::repository_fixture::RepositoryFixture;

    #[test]
    fn returns_empty_map_when_repository_has_no_tags() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let branch = repo.head().expect("head branch");

        let tagged = TaggedSemanticVersionService
            .get_tagged_semantic_versions(&repo, &branch, &GitVersionConfiguration::default())
            .expect("tagged semantic versions");

        assert!(tagged.is_empty());
    }

    #[test]
    fn groups_multiple_semver_tags_by_commit_sha() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        let commit_sha = fixture.make_a_commit("initial commit").expect("commit");
        fixture.apply_tag("1.2.3").expect("tag");
        fixture.apply_tag("2.0.0").expect("tag");
        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let branch = repo.head().expect("head branch");

        let tagged = TaggedSemanticVersionService
            .get_tagged_semantic_versions(&repo, &branch, &GitVersionConfiguration::default())
            .expect("tagged semantic versions");

        let versions = tagged.get(&commit_sha).expect("versions for commit");
        assert_eq!(versions.len(), 2);
    }

    #[test]
    fn ignores_tags_that_are_not_semantic_versions() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        let commit_sha = fixture.make_a_commit("initial commit").expect("commit");
        fixture.apply_tag("not-a-version").expect("tag");
        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let branch = repo.head().expect("head branch");

        let tagged = TaggedSemanticVersionService
            .get_tagged_semantic_versions(&repo, &branch, &GitVersionConfiguration::default())
            .expect("tagged semantic versions");

        assert!(tagged.get(&commit_sha).is_none());
    }

    #[test]
    fn strict_format_rejects_v_prefix_tags() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        fixture.make_a_commit("initial commit").expect("commit");
        fixture.apply_tag("v1.2.3").expect("tag");
        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let branch = repo.head().expect("head branch");
        let config = GitVersionConfiguration::default();

        let tagged = TaggedSemanticVersionService
            .get_tagged_semantic_versions(&repo, &branch, &config)
            .expect("tagged semantic versions");

        assert!(tagged.is_empty());
    }

    #[test]
    fn loose_format_accepts_v_prefix_tags() {
        let mut fixture = RepositoryFixture::new().expect("fixture");
        let commit_sha = fixture.make_a_commit("initial commit").expect("commit");
        fixture.apply_tag("v1.2.3").expect("tag");
        let repo = Git2Repository::open(fixture.path()).expect("open repository");
        let branch = repo.head().expect("head branch");
        let config = GitVersionConfiguration {
            semantic_version_format: SemanticVersionFormat::Loose,
            ..GitVersionConfiguration::default()
        };

        let tagged = TaggedSemanticVersionService
            .get_tagged_semantic_versions(&repo, &branch, &config)
            .expect("tagged semantic versions");

        let versions = tagged.get(&commit_sha).expect("versions for commit");
        assert_eq!(versions.len(), 1);
        assert_eq!(versions[0].value.to_string(), "1.2.3");
    }
}
