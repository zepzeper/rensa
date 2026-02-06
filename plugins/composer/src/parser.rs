use rensa_core::plugin::Parser;
use rensa_core::types::{Dependency, DependencyFile, Ecosystem, VersionConstraint};
use rensa_core::Result;
use serde_json::Value;

pub struct ComposerParser;

impl ComposerParser {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Parser for ComposerParser {
    fn ecosystem(&self) -> Ecosystem {
        Ecosystem::Composer
    }

    async fn parse(&self, file: &DependencyFile) -> Result<Vec<Dependency>> {
        let json: Value = serde_json::from_str(&file.content).map_err(|e| rensa_core::RensaError::ParseError {
            file: file.path.clone(),
            source: e,
        })?;

        let mut dependencies = Vec::new();

        // Parse require section
        if let Some(require) = json.get("require").and_then(|r| r.as_object()) {
            for (name, version) in require {
                // Skip PHP version requirements
                if name == "php" {
                    continue;
                }

                dependencies.push(Dependency {
                    name: name.to_string(),
                    version: version.as_str()
                        .ok_or_else(|| rensa_core::RensaError::ParseError {
                            file: file.path.clone(),
                            source: serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "Invalid version format")),
                        })?
                        .to_string(),
                    constraint: parse_constraint(version),
                    file: file.path.clone(),
                });
            }
        }

        // Parse require-dev section (could filter based on config)
        if let Some(require_dev) = json.get("require-dev").and_then(|r| r.as_object()) {
            for (name, version) in require_dev {
                if name == "php" {
                    continue;
                }

                dependencies.push(Dependency {
                    name: name.to_string(),
                    version: version.as_str()
                        .ok_or_else(|| rensa_core::RensaError::ParseError {
                            file: file.path.clone(),
                            source: serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "Invalid version format")),
                        })?
                        .to_string(),
                    constraint: parse_constraint(version),
                    file: file.path.clone(),
                });
            }
        }

        Ok(dependencies)
    }
}

fn parse_constraint(version: &Value) -> VersionConstraint {
    let version_str = version.as_str().unwrap_or("*").to_string();

    if version_str.starts_with('^') {
        VersionConstraint::Caret(version_str.trim_start_matches('^').to_string())
    } else if version_str.starts_with('~') {
        VersionConstraint::Tilde(version_str.trim_start_matches('~').to_string())
    } else if version_str.starts_with(">=") {
        VersionConstraint::GreaterThanEqual(version_str.trim_start_matches(">=").to_string())
    } else if version_str.starts_with('>') {
        VersionConstraint::GreaterThanEqual(version_str.trim_start_matches('>').to_string())
    } else {
        VersionConstraint::Range(version_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parse_simple_composer_json() {
        let parser = ComposerParser::new();
        let file = DependencyFile {
            ecosystem: Ecosystem::Composer,
            path: std::path::PathBuf::from("composer.json"),
            content: r#"{
  "name": "test/package",
  "require": {
    "php": "^8.0",
    "guzzlehttp/guzzle": "^7.0"
  }
}"#.to_string(),
        };

        let deps = parser.parse(&file).await.unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "guzzlehttp/guzzle");
        assert_eq!(deps[0].version, "^7.0");
    }

    #[tokio::test]
    async fn test_parse_with_dev_dependencies() {
        let parser = ComposerParser::new();
        let file = DependencyFile {
            ecosystem: Ecosystem::Composer,
            path: std::path::PathBuf::from("composer.json"),
            content: r#"{
  "require": {
    "php": "^8.0"
  },
  "require-dev": {
    "phpstan/phpstan": "^1.0"
  }
}"#.to_string(),
        };

        let deps = parser.parse(&file).await.unwrap();

        assert_eq!(deps.len(), 1);
        assert_eq!(deps[0].name, "phpstan/phpstan");
    }
}
