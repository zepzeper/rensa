use rensa_core::types::{Dependency, Vulnerability, UpdateInfo};
use rensa_core::{VersionComparator, UpdateType};
use crate::registry::PackagistClient;
use crate::osv::OsvClient;

pub struct ComposerScanner {
    registry: PackagistClient,
    vuln_scanner: OsvClient,
}

pub struct ScannedDependency {
    pub dependency: Dependency,
    pub update: Option<UpdateInfo>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub update_type: Option<UpdateType>,
    pub is_security_update: bool,
}

impl ComposerScanner {
    pub fn new() -> Self {
        Self {
            registry: PackagistClient::new(),
            vuln_scanner: OsvClient::new(),
        }
    }

    pub fn with_cache(cache: rensa_core::CacheManager) -> Self {
        Self {
            registry: PackagistClient::new().with_cache(cache.clone()),
            vuln_scanner: OsvClient::new().with_cache(cache),
        }
    }

    pub async fn scan_dependency(&self, dependency: &Dependency) -> ScannedDependency {
        // Get update info
        let update = self.registry.get_update_info(dependency).await.ok().flatten();
        
        // Get vulnerabilities
        let vulnerabilities = self.vuln_scanner.scan(dependency).await.unwrap_or_default();
        
        // Determine update type
        let update_type = update.as_ref().map(|u| {
            VersionComparator::classify_update(&u.current_version, &u.latest_version)
        });
        
        // Check if this is a security update
        let is_security_update = if let Some(ref u) = update {
            vulnerabilities.iter().any(|v| {
                v.fixed_versions.contains(&u.latest_version)
            })
        } else {
            false
        };
        
        ScannedDependency {
            dependency: dependency.clone(),
            update,
            vulnerabilities,
            update_type,
            is_security_update,
        }
    }

    /// Scan multiple dependencies
    pub async fn scan_dependencies(&self, dependencies: &[Dependency]) -> Vec<ScannedDependency> {
        let mut scanned = Vec::new();
        for dep in dependencies {
            let result = self.scan_dependency(dep).await;
            scanned.push(result);
        }
        scanned
    }

    /// Filter dependencies by update type
    pub fn filter_by_update_type(
        scanned: Vec<ScannedDependency>,
        allowed_types: &[UpdateType],
    ) -> Vec<ScannedDependency> {
        scanned
            .into_iter()
            .filter(|s| {
                if let Some(update_type) = s.update_type {
                    allowed_types.contains(&update_type) || 
                    (s.is_security_update && allowed_types.contains(&UpdateType::Security))
                } else {
                    false
                }
            })
            .collect()
    }

    /// Get only security updates
    pub fn get_security_updates(scanned: &[ScannedDependency]) -> Vec<&ScannedDependency> {
        scanned
            .iter()
            .filter(|s| s.is_security_update)
            .collect()
    }

    /// Get updates that can be safely applied (patch and minor)
    pub fn get_safe_updates(scanned: &[ScannedDependency]) -> Vec<&ScannedDependency> {
        scanned
            .iter()
            .filter(|s| {
                if let Some(update_type) = s.update_type {
                    matches!(update_type, UpdateType::Patch | UpdateType::Minor)
                } else {
                    false
                }
            })
            .collect()
    }

    /// Sort by priority (security first, then by update type priority)
    pub fn sort_by_priority(scanned: &mut [ScannedDependency]) {
        scanned.sort_by(|a, b| {
            // Security updates first
            let a_security = if a.is_security_update { 0 } else { 1 };
            let b_security = if b.is_security_update { 0 } else { 1 };
            
            if a_security != b_security {
                return a_security.cmp(&b_security);
            }
            
            // Then by update type priority
            let a_priority = a.update_type.map(|t| t.priority()).unwrap_or(255);
            let b_priority = b.update_type.map(|t| t.priority()).unwrap_or(255);
            
            a_priority.cmp(&b_priority)
        });
    }

    /// Get summary statistics
    pub fn get_summary(scanned: &[ScannedDependency]) -> ScanSummary {
        let mut summary = ScanSummary::default();
        
        for s in scanned {
            summary.total_dependencies += 1;
            
            if s.update.is_some() {
                summary.updates_available += 1;
            }
            
            if !s.vulnerabilities.is_empty() {
                summary.vulnerabilities_found += 1;
                summary.critical_vulnerabilities += s.vulnerabilities.iter()
                    .filter(|v| matches!(v.severity, rensa_core::types::Severity::Critical))
                    .count();
            }
            
            if s.is_security_update {
                summary.security_updates += 1;
            }
            
            if let Some(update_type) = s.update_type {
                match update_type {
                    UpdateType::Major => summary.major_updates += 1,
                    UpdateType::Minor => summary.minor_updates += 1,
                    UpdateType::Patch => summary.patch_updates += 1,
                    _ => {}
                }
            }
        }
        
        summary
    }
}

#[derive(Debug, Default)]
pub struct ScanSummary {
    pub total_dependencies: usize,
    pub updates_available: usize,
    pub security_updates: usize,
    pub vulnerabilities_found: usize,
    pub critical_vulnerabilities: usize,
    pub major_updates: usize,
    pub minor_updates: usize,
    pub patch_updates: usize,
}

impl ScanSummary {
    pub fn has_critical_issues(&self) -> bool {
        self.critical_vulnerabilities > 0
    }
    
    pub fn has_updates(&self) -> bool {
        self.updates_available > 0
    }
    
    pub fn has_security_issues(&self) -> bool {
        self.security_updates > 0 || self.vulnerabilities_found > 0
    }
}
