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
