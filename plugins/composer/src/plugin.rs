use rensa_core::types::Ecosystem;

pub struct ComposerPlugin {
    cache: Option<rensa_core::CacheManager>,
}

impl ComposerPlugin {
    pub fn new() -> Self {
        Self { cache: None }
    }

    pub fn with_cache(cache: rensa_core::CacheManager) -> Self {
        Self { cache: Some(cache) }
    }

    pub fn create_detector(&self) -> Option<Box<dyn rensa_core::Detector>> {
        Some(Box::new(super::detector::ComposerDetector::new()))
    }

    pub fn create_parser(&self) -> Option<Box<dyn rensa_core::Parser>> {
        Some(Box::new(super::parser::ComposerParser::new()))
    }

    pub fn create_registry_client(&self) -> Option<Box<dyn rensa_core::RegistryClient>> {
        let client = match &self.cache {
            Some(cache) => super::registry::PackagistClient::new().with_cache(cache.clone()),
            None => super::registry::PackagistClient::new(),
        };
        Some(Box::new(client))
    }

    pub fn create_vulnerability_scanner(
        &self,
    ) -> Option<Box<dyn rensa_core::VulnerabilityScanner>> {
        let client = match &self.cache {
            Some(cache) => super::osv::OsvScanner::new().with_cache(cache.clone()),
            None => super::osv::OsvScanner::new(),
        };
        Some(Box::new(client))
    }
}

impl rensa_core::Plugin for ComposerPlugin {
    fn name(&self) -> &'static str {
        "composer"
    }

    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Composer
    }
}
