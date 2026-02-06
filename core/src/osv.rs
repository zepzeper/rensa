use crate::HttpClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct OsvClient {
    client: HttpClient,
    base_url: String,
}

impl OsvClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: HttpClient::new(),
            base_url: base_url.to_string(),
        }
    }

    pub fn with_cache(mut self, cache: crate::CacheManager) -> Self {
        self.client = HttpClient::with_cache(self.client, cache);
        self
    }

    pub async fn query(&self, query: &OsvQuery) -> crate::Result<Vec<OsvVulnerability>> {
        #[derive(Deserialize, Clone, Serialize)]
        #[serde(from = "OsvResponseHelper")]
        struct OsvResponse {
            vulns: Vec<OsvVulnerability>,
        }

        #[derive(Deserialize)]
        struct OsvResponseHelper {
            vulns: Option<Vec<OsvVulnerability>>,
        }

        impl From<OsvResponseHelper> for OsvResponse {
            fn from(helper: OsvResponseHelper) -> Self {
                Self { vulns: helper.vulns.unwrap_or_default() }
            }
        }

        let response: OsvResponse = self.client.post(&format!("{}/v1/query", self.base_url), query).await?;
        Ok(response.vulns)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct OsvQuery {
    pub package: OsvPackage,
    pub version: String,
}

#[derive(Serialize, Debug, Clone, Deserialize)]
pub struct OsvPackage {
    pub name: String,
    pub ecosystem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsvVulnerability {
    pub id: String,
    pub summary: String,
    pub details: String,
    pub severity: Option<OsvSeverity>,
    pub affected: Vec<OsvAffected>,
    pub references: Vec<OsvReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsvSeverity {
    pub r#type: String,
    pub score: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsvAffected {
    #[serde(default)]
    pub package: Option<OsvPackage>,
    #[serde(default)]
    pub ranges: Vec<OsvRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsvRange {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub events: Vec<OsvEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsvEvent {
    pub introduced: Option<String>,
    pub fixed: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsvReference {
    pub r#type: String,
    pub url: String,
}

impl OsvVulnerability {
    pub fn to_vulnerability(&self) -> crate::types::Vulnerability {
        let severity = self
            .severity
            .as_ref()
            .and_then(|s| {
                s.score
                    .parse::<f64>()
                    .ok()
                    .map(crate::types::Severity::from_cvss_score)
            })
            .unwrap_or(crate::types::Severity::Unknown);

        let fixed_versions: Vec<String> = self
            .affected
            .iter()
            .flat_map(|a| a.ranges.iter())
            .flat_map(|r| r.events.iter())
            .filter_map(|e| e.fixed.clone())
            .collect();

        crate::types::Vulnerability {
            id: self.id.clone(),
            summary: self.summary.clone(),
            details: self.details.clone(),
            severity,
            affected_versions: Vec::new(),
            fixed_versions,
            references: self.references.iter().map(|r| r.url.clone()).collect(),
        }
    }
}
