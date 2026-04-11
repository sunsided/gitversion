use std::collections::HashSet;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IgnoreConfiguration {
    pub before: Option<DateTime<Utc>>,
    pub shas: HashSet<String>,
    pub paths: HashSet<String>,
}
