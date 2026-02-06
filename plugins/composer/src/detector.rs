use rensa_core::plugin::Detector;
use rensa_core::types::{DependencyFile, Ecosystem};
use rensa_core::Result;
use std::path::Path;
use walkdir::WalkDir;

pub struct ComposerDetector;

impl ComposerDetector {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Detector for ComposerDetector {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Composer
    }

    async fn detect(&self, path: &Path) -> Result<Vec<DependencyFile>> {
        let mut files = Vec::new();

        let walker = WalkDir::new(path)
            .follow_links(true)
            .into_iter();

        for entry in walker {
            let entry = match entry {
                Ok(e) => e,
                Err(_) => continue,
            };

            let file_path = entry.path();

            // Only look for composer.json at root or specific locations
            if file_path.file_name() == Some(std::ffi::OsStr::new("composer.json")) {
                // Skip vendor directories
                if let Some(parent) = file_path.parent() {
                    if parent.to_string_lossy().contains("/vendor/") {
                        continue;
                    }
                }

                // Skip nested composer.json files (e.g., in vendor)
                let relative = file_path.strip_prefix(path).unwrap_or(file_path);
                let parts_str = relative.to_string_lossy();
                let parts: Vec<&str> = parts_str.split('/').collect();
                
                // Allow only:
                // - composer.json (root)
                // - subdir/composer.json
                // Disallow:
                // - vendor/**/composer.json
                // - **/vendor/**/composer.json
                if parts.iter().any(|p| *p == "vendor") {
                    continue;
                }

                match std::fs::read_to_string(file_path) {
                    Ok(content) => {
                        files.push(DependencyFile {
                            ecosystem: Ecosystem::Composer,
                            path: file_path.to_path_buf(),
                            content,
                        });
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to read {}: {}", file_path.display(), e);
                    }
                }
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_detect_finds_composer_json() {
        let temp_dir = TempDir::new().unwrap();
        let composer_json = r#"{
  "name": "test/package",
  "require": {
    "php": "^8.0"
  }
}"#;

        let json_path = temp_dir.path().join("composer.json");
        fs::write(&json_path, composer_json).unwrap();

        let detector = ComposerDetector::new();
        let files = detector.detect(temp_dir.path()).await.unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, json_path);
    }

    #[tokio::test]
    async fn test_detect_skips_vendor() {
        let temp_dir = TempDir::new().unwrap();
        
        // Create root composer.json
        let root_json = temp_dir.path().join("composer.json");
        fs::write(&root_json, r#"{"name":"test/package"}"#).unwrap();
        
        // Create vendor composer.json (should be skipped)
        let vendor_dir = temp_dir.path().join("vendor").join("some/package");
        fs::create_dir_all(&vendor_dir).unwrap();
        let vendor_json = vendor_dir.join("composer.json");
        fs::write(&vendor_json, r#"{"name":"some/package"}"#).unwrap();

        let detector = ComposerDetector::new();
        let files = detector.detect(temp_dir.path()).await.unwrap();

        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, root_json);
    }
}
