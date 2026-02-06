use crate::report::{ScanReport, EcosystemScanResult};
use crate::Result;
use std::path::PathBuf;
use std::time::Instant;

pub struct Scanner<'a> {
    registry: &'a crate::plugin::PluginRegistry,
}

impl<'a> Scanner<'a> {
    pub fn new(registry: &'a crate::plugin::PluginRegistry) -> Self {
        Self { registry }
    }

    pub async fn scan(&self, path: PathBuf) -> Result<ScanReport> {
        let start = Instant::now();
        let mut report = ScanReport::new(path.clone());

        let files = self.registry.detect_all(&path).await?;

        for file in files {
            let ecosystem = file.ecosystem;
            let parser = self.registry.get_parser(&ecosystem);
            let registry_client = self.registry.get_registry_client(&ecosystem);
            let vulnerability_scanner = self.registry.get_vulnerability_scanner(&ecosystem);

            let deps = match parser {
                Some(p) => p.parse(&file).await?,
                None => {
                    report.warnings.push(format!("No parser for ecosystem: {:?}", ecosystem));
                    continue;
                }
            };

            let mut ecosystem_result = EcosystemScanResult {
                ecosystem,
                files_found: vec![file.path.clone()],
                dependencies: deps.clone(),
                updates: Vec::new(),
                vulnerabilities: Vec::new(),
                errors: Vec::new(),
            };

            let mut updates = Vec::new();
            let mut vulnerabilities = Vec::new();

            for dep in deps {
                if let Some(client) = registry_client {
                    if let Some(info) = client.get_update_info(&dep).await? {
                        updates.push(info);
                    }
                }
                if let Some(scanner) = vulnerability_scanner {
                    if let Ok(vulns) = scanner.scan(&dep).await {
                        vulnerabilities.extend(vulns);
                    }
                }
            }

            ecosystem_result.updates = updates;
            ecosystem_result.vulnerabilities = vulnerabilities;

            report.add_ecosystem_result(ecosystem, ecosystem_result);
        }

        report.elapsed = start.elapsed().as_millis() as u64;

        Ok(report)
    }
}

pub async fn scan_path(path: PathBuf, registry: &crate::plugin::PluginRegistry) -> Result<ScanReport> {
    let scanner = Scanner::new(registry);
    scanner.scan(path).await
}
