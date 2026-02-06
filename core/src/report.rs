use crate::{Dependency, Ecosystem, Severity, UpdateInfo, UpdateType, VersionComparator, Vulnerability};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub timestamp: DateTime<Utc>,

    pub scanned_path: PathBuf,

    pub elapsed: u64,

    pub total_dependency_files: usize,

    pub total_dependencies: usize,

    pub summary: ScanSummary,

    pub ecosystem_results: HashMap<Ecosystem, EcosystemScanResult>,

    pub updates: Vec<UpdateInfo>,

    pub vulnerabilities: Vec<Vulnerability>,

    pub warnings: Vec<String>,

    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScanSummary {
    pub updates_available: usize,
    pub vulnerabilities_found: usize,
    pub critical_vulnerabilities: usize,
    pub high_vulnerabilities: usize,
    pub medium_vulnerabilities: usize,
    pub low_vulnerabilities: usize,
    pub outdated_dependencies: usize,
    pub up_to_date_dependencies: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemScanResult {
    pub ecosystem: Ecosystem,
    pub files_found: Vec<PathBuf>,
    pub dependencies: Vec<Dependency>,
    pub updates: Vec<UpdateInfo>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub errors: Vec<String>,
}

impl ScanReport {
    pub fn new(scanned_path: PathBuf) -> Self {
        Self {
            timestamp: Utc::now(),
            scanned_path,
            elapsed: 0,
            total_dependency_files: 0,
            total_dependencies: 0,
            summary: ScanSummary::default(),
            ecosystem_results: HashMap::new(),
            updates: Vec::new(),
            vulnerabilities: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn add_ecosystem_result(&mut self, ecosystem: Ecosystem, result: EcosystemScanResult) {
        self.total_dependency_files += result.files_found.len();
        self.total_dependencies += result.dependencies.len();

        self.summary.updates_available += result.updates.len(); 
        self.summary.low_vulnerabilities += result.vulnerabilities.len(); 

        for vuln in &result.vulnerabilities {
            match vuln.severity {
                Severity::Low => self.summary.low_vulnerabilities += 1,
                Severity::Medium => self.summary.medium_vulnerabilities += 1,
                Severity::High => self.summary.high_vulnerabilities += 1,
                Severity::Critical => self.summary.critical_vulnerabilities += 1,
                Severity::Unknown => {}
            }
        }

        self.summary.outdated_dependencies += result.updates.len();
        self.summary.up_to_date_dependencies += 
            result.dependencies.len().saturating_sub(result.updates.len());

        self.ecosystem_results.insert(ecosystem, result);
    }

    pub fn has_critical_vulnerabilities(&self) -> bool {
        self.summary.critical_vulnerabilities > 0
    }

    pub fn has_updates(&self) -> bool {
        self.summary.updates_available > 0
    }

    pub fn updates_by_type(&self) -> HashMap<UpdateType, Vec<&UpdateInfo>> {
        let mut grouped: HashMap<UpdateType, Vec<&UpdateInfo>> = HashMap::new();

        for update in &self.updates {
            let update_type = VersionComparator::classify_update(&update.current_version, &update.latest_version);
            grouped.entry(update_type).or_default().push(update);
        }

        grouped
    }

    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
