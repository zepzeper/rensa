use anyhow::Result;
use std::path::PathBuf;
use rensa_core::{PluginRegistry, scan_path, ScanReport};

#[cfg(feature = "composer")]
use rensa_plugin_composer::ComposerPlugin;

pub async fn run_scan(path: &PathBuf) -> Result<ScanReport> {
    let mut registry = PluginRegistry::new();

    #[cfg(feature = "composer")]
    {
        registry.register_plugin(ComposerPlugin::new());
    }

    let report = scan_path(path.clone(), &registry).await?;
    Ok(report)
}
