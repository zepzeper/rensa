use crate::types::{Dependency, DependencyFile, Ecosystem};
use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

#[async_trait]
pub trait Detector: Send + Sync {
    fn ecosystem(&self) -> Ecosystem;
    
    async fn detect(&self, path: &Path) -> Result<Vec<DependencyFile>>;
}

#[async_trait]
pub trait Parser: Send + Sync {
    fn ecosystem(&self) -> Ecosystem;
    
    async fn parse(&self, file: &DependencyFile) -> Result<Vec<Dependency>>;
}

#[async_trait]
pub trait RegistryClient: Send + Sync {
    fn ecosystem(&self) -> Ecosystem;
    
    async fn get_latest_version(&self, dependency: &Dependency) -> Result<Option<String>>;
    
    async fn get_update_info(&self, dependency: &Dependency) -> Result<Option<crate::types::UpdateInfo>> {
        let latest = match self.get_latest_version(dependency).await? {
            Some(v) => v,
            None => return Ok(None),
        };
        
        Ok(Some(crate::types::UpdateInfo {
            dependency: dependency.clone(),
            current_version: dependency.version.clone(),
            latest_version: latest,
            changelog: None,
        }))
    }
}

#[async_trait]
pub trait VulnerabilityScanner: Send + Sync {
    fn ecosystem(&self) -> Ecosystem;
    
    async fn scan(&self, dependency: &Dependency) -> Result<Vec<crate::types::Vulnerability>>;
}

#[async_trait]
pub trait Plugin: Send + Sync {
    fn name(&self) -> &'static str;
    fn ecosystem(&self) -> Ecosystem;
    
    fn create_detector(&self) -> Option<Box<dyn Detector>> { None }
    fn create_parser(&self) -> Option<Box<dyn Parser>> { None }
    fn create_registry_client(&self) -> Option<Box<dyn RegistryClient>> { None }
    fn create_vulnerability_scanner(&self) -> Option<Box<dyn VulnerabilityScanner>> { None }
}

pub struct PluginRegistry {
    detectors: Vec<(Ecosystem, Box<dyn Detector>)>,
    parsers: Vec<(Ecosystem, Box<dyn Parser>)>,
    registry_clients: Vec<(Ecosystem, Box<dyn RegistryClient>)>,
    vulnerability_scanners: Vec<(Ecosystem, Box<dyn VulnerabilityScanner>)>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            detectors: Vec::new(),
            parsers: Vec::new(),
            registry_clients: Vec::new(),
            vulnerability_scanners: Vec::new(),
        }
    }

    pub fn register_detector(&mut self, detector: Box<dyn Detector>) {
        self.detectors.push((detector.ecosystem(), detector));
    }

    pub fn register_parser(&mut self, parser: Box<dyn Parser>) {
        self.parsers.push((parser.ecosystem(), parser));
    }

    pub fn register_registry_client(&mut self, client: Box<dyn RegistryClient>) {
        self.registry_clients.push((client.ecosystem(), client));
    }

    pub fn register_vulnerability_scanner(&mut self, scanner: Box<dyn VulnerabilityScanner>) {
        self.vulnerability_scanners.push((scanner.ecosystem(), scanner));
    }

    pub fn register_plugin<P: Plugin + 'static>(&mut self, plugin: P) {
        if let Some(detector) = plugin.create_detector() {
            self.register_detector(detector);
        }
        if let Some(parser) = plugin.create_parser() {
            self.register_parser(parser);
        }
        if let Some(client) = plugin.create_registry_client() {
            self.register_registry_client(client);
        }
        if let Some(scanner) = plugin.create_vulnerability_scanner() {
            self.register_vulnerability_scanner(scanner);
        }
    }

    pub fn get_detector(&self, ecosystem: &Ecosystem) -> Option<&dyn Detector> {
        self.detectors.iter()
            .find(|(e, _)| e == ecosystem)
            .map(|(_, d)| d.as_ref())
    }

    pub fn get_parser(&self, ecosystem: &Ecosystem) -> Option<&dyn Parser> {
        self.parsers.iter()
            .find(|(e, _)| e == ecosystem)
            .map(|(_, p)| p.as_ref())
    }

    pub fn get_registry_client(&self, ecosystem: &Ecosystem) -> Option<&dyn RegistryClient> {
        self.registry_clients.iter()
            .find(|(e, _)| e == ecosystem)
            .map(|(_, c)| c.as_ref())
    }

    pub fn get_vulnerability_scanner(&self, ecosystem: &Ecosystem) -> Option<&dyn VulnerabilityScanner> {
        self.vulnerability_scanners.iter()
            .find(|(e, _)| e == ecosystem)
            .map(|(_, s)| s.as_ref())
    }

    pub async fn detect_all(&self, path: &Path) -> Result<Vec<DependencyFile>> {
        let mut all_files = Vec::new();
        
        for (_, detector) in &self.detectors {
            let files = detector.detect(path).await?;
            all_files.extend(files);
        }
        
        Ok(all_files)
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}
