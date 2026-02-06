use rensa_core::plugin::VulnerabilityScanner;
use rensa_core::types::{Dependency, Ecosystem, Vulnerability};
use rensa_core::osv::{OsvClient as OsvClientTrait, OsvQuery, OsvPackage};
use rensa_core::Result;
use async_trait::async_trait;

pub struct OsvScanner {
    client: OsvClientTrait,
}

impl OsvScanner {
    pub fn new() -> Self {
        Self {
            client: OsvClientTrait::new("https://api.osv.dev"),
        }
    }

    pub fn with_cache(self, cache: rensa_core::CacheManager) -> Self {
        Self {
            client: self.client.with_cache(cache),
        }
    }
}

#[async_trait]
impl VulnerabilityScanner for OsvScanner {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Composer
    }

    async fn scan(&self, dependency: &Dependency) -> Result<Vec<Vulnerability>> {
        let query = OsvQuery {
            package: OsvPackage {
                name: dependency.name.clone(),
                ecosystem: "Packagist".to_string(),
            },
            version: dependency.version.clone(),
        };

        let osv_vulns = self.client.query(&query).await?;
        let vulnerabilities: Vec<Vulnerability> = osv_vulns.into_iter().map(|v| v.to_vulnerability()).collect();

        Ok(vulnerabilities)
    }
}
