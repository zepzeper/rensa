use serde::{Deserialize, Serialize};

use super::dependency::Dependency;
use super::vulnerability::Vulnerability;
use crate::version::UpdateType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub dependency: Dependency,
    pub current_version: String,
    pub latest_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizedUpdate {
    pub update: UpdateInfo,
    pub update_type: UpdateType,
    pub is_security_update: bool,
    pub breaking_changes: Option<String>,
    pub release_notes_url: Option<String>,
}

impl CategorizedUpdate {
    pub fn from_update(update: UpdateInfo, vulnerabilities: &[Vulnerability]) -> Self {
        use crate::version::VersionComparator;

        let update_type =
            VersionComparator::classify_update(&update.current_version, &update.latest_version);

        let is_security_update = vulnerabilities
            .iter()
            .any(|v| v.fixed_versions.contains(&update.latest_version));

        Self {
            update,
            update_type,
            is_security_update,
            breaking_changes: None,
            release_notes_url: None,
        }
    }

    pub fn priority_score(&self) -> u32 {
        let mut score = 0;

        if self.is_security_update {
            score += 1000;
        }

        score += match self.update_type {
            UpdateType::Major => 100,
            UpdateType::Minor => 10,
            UpdateType::Patch => 1,
            _ => 0,
        };

        score
    }
}
