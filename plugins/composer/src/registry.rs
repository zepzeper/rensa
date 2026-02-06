use rensa_core::plugin::RegistryClient;
use rensa_core::types::{Dependency, Ecosystem, UpdateInfo};
use rensa_core::{VersionComparator, UpdateType};
use rensa_core::Result;
use semver::Version;

pub struct PackagistClient {
    client: rensa_core::HttpClient,
    base_url: String,
}

impl PackagistClient {
    pub fn new() -> Self {
        Self {
            client: rensa_core::HttpClient::new(),
            base_url: "https://packagist.org".to_string(),
        }
    }

    pub fn with_base_url(mut self, base_url: String) -> Self {
        self.base_url = base_url;
        self
    }

    pub fn with_cache(mut self, cache: rensa_core::CacheManager) -> Self {
        self.client = rensa_core::HttpClient::with_cache(self.client, cache);
        self
    }
}

#[async_trait::async_trait]
impl RegistryClient for PackagistClient {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Composer
    }

    async fn get_latest_version(&self, dependency: &Dependency) -> Result<Option<String>> {
        let url = format!("{}/packages/{}.json", self.base_url, dependency.name);

        let data: serde_json::Value = self.client.get(&url).await?;

        if let Some(versions) = data.get("package")
            .and_then(|p| p.get("versions"))
            .and_then(|v| v.as_object()) {
            
            let mut parsed_versions: Vec<(Version, String)> = versions
                .keys()
                .filter(|v| {
                    let v = v.to_lowercase();
                    !v.contains("dev") && !v.contains("alpha") && !v.contains("beta") && !v.contains("rc")
                })
                .filter_map(|v| {
                    let clean = v.trim_start_matches('v');
                    Version::parse(clean).ok().map(|parsed| (parsed, v.clone()))
                })
                .collect();
            
            parsed_versions.sort_by(|a, b| b.0.cmp(&a.0));
            
            if let Some((_, original)) = parsed_versions.first() {
                return Ok(Some(original.clone()));
            }
        }
        
        Ok(None)
    }

    async fn get_update_info(&self, dependency: &Dependency) -> Result<Option<UpdateInfo>> {
        let latest = self.get_latest_version(dependency).await?;
        
        if let Some(latest_version) = latest {
            // Clean version strings for comparison (remove 'v' prefix if present)
            let current_clean = dependency.version.trim_start_matches('v');
            let latest_clean = latest_version.trim_start_matches('v');
            
            // Use VersionComparator to classify the update
            let update_type = VersionComparator::classify_update(current_clean, latest_clean);
            
            // Only return if there's an actual update (not None or Unknown)
            match update_type {
                UpdateType::None | UpdateType::Unknown => {
                    // No meaningful update
                    Ok(None)
                }
                _ => {
                    // Return update info with metadata
                    Ok(Some(UpdateInfo {
                        dependency: dependency.clone(),
                        current_version: dependency.version.clone(),
                        latest_version,
                        changelog: None,
                    }))
                }
            }
        } else {
            Ok(None)
        }
    }
}

/// Extension trait for PackagistClient to provide additional functionality
pub trait PackagistClientExt {
    /// Get all available versions for a dependency
    async fn get_all_versions(&self, dependency: &Dependency) -> Result<Vec<String>>;
    
    /// Check if an update is available and classify it
    async fn check_update(&self, dependency: &Dependency) -> Result<Option<UpdateCheck>>;
}

/// Result of checking for an update
pub struct UpdateCheck {
    pub update_info: UpdateInfo,
    pub update_type: UpdateType,
    pub is_significant: bool,
}

impl PackagistClientExt for PackagistClient {
    async fn get_all_versions(&self, dependency: &Dependency) -> Result<Vec<String>> {
        let url = format!("{}/packages/{}.json", self.base_url, dependency.name);

        let data: serde_json::Value = self.client.get(&url).await?;

        let mut versions = Vec::new();

        if let Some(versions_obj) = data.get("package")
            .and_then(|p| p.get("versions"))
            .and_then(|v| v.as_object()) {
            
            for version_key in versions_obj.keys() {
                let v = version_key.to_lowercase();
                // Filter out dev/alpha/beta/rc versions
                if !v.contains("dev") && !v.contains("alpha") && !v.contains("beta") && !v.contains("rc") {
                    versions.push(version_key.clone());
                }
            }
            
            // Sort versions (newest first)
            versions.sort_by(|a, b| {
                let a_clean = a.trim_start_matches('v');
                let b_clean = b.trim_start_matches('v');
                match (Version::parse(a_clean), Version::parse(b_clean)) {
                    (Ok(va), Ok(vb)) => vb.cmp(&va), // Reverse for newest first
                    _ => b.cmp(a), // Fallback to string comparison
                }
            });
        }
        
        Ok(versions)
    }

    async fn check_update(&self, dependency: &Dependency) -> Result<Option<UpdateCheck>> {
        if let Some(update_info) = self.get_update_info(dependency).await? {
            let current_clean = dependency.version.trim_start_matches('v');
            let latest_clean = update_info.latest_version.trim_start_matches('v');
            let update_type = VersionComparator::classify_update(current_clean, latest_clean);
            
            // Consider major and security updates as significant
            let is_significant = matches!(update_type, UpdateType::Major | UpdateType::Security);
            
            Ok(Some(UpdateCheck {
                update_info,
                update_type,
                is_significant,
            }))
        } else {
            Ok(None)
        }
    }
}
