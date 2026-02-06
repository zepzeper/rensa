use crate::types::VersionConstraint;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateType {
    Major,
    Minor,
    Patch,
    Security,
    None,
    Unknown,
}

impl UpdateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            UpdateType::Major => "major",
            UpdateType::Minor => "minor",
            UpdateType::Patch => "patch",
            UpdateType::Security => "security",
            UpdateType::None => "none",
            UpdateType::Unknown => "unknown",
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            UpdateType::Security => 0,
            UpdateType::Major => 1,
            UpdateType::Minor => 2,
            UpdateType::Patch => 3,
            UpdateType::None => 4,
            UpdateType::Unknown => 5,
        }
    }
}

pub struct VersionComparator;

impl VersionComparator {
    pub fn satisfies(constraint: &VersionConstraint, version: &str) -> bool {
        match constraint {
            VersionConstraint::Exact(v) => version == v,
            VersionConstraint::Range(v) => {
                // Parse semver
                if let Ok(req) = VersionReq::parse(v) {
                    if let Ok(ver) = Version::parse(version) {
                        return req.matches(&ver);
                    }
                }
                false
            }
            VersionConstraint::GreaterThanEqual(v) => {
                // Use centralized version parsing
                if let Some(min) = Self::parse_version(v) {
                    if let Ok(ver) = Version::parse(version) {
                        return ver >= min;
                    }
                }
                false
            }
            VersionConstraint::Caret(v) => {
                // ^1.2.3 matches >=1.2.3 <2.0.0
                Self::parse_caret_range(v, version)
            }
            VersionConstraint::Tilde(v) => {
                // ~1.2.3 matches >=1.2.3 <1.3.0
                Self::parse_tilde_range(v, version)
            }
        }
    }

    pub fn classify_update(current: &str, latest: &str) -> UpdateType {
        let current = Version::parse(current).ok();
        let latest = Version::parse(latest).ok();

        match (current, latest) {
            (Some(c), Some(l)) => {
                if l.major > c.major {
                    UpdateType::Major
                } else if l.minor > c.minor {
                    UpdateType::Minor
                } else if l.patch > c.patch {
                    UpdateType::Patch
                } else {
                    UpdateType::None
                }
            }
            _ => UpdateType::Unknown,
        }
    }

    fn parse_caret_range(constraint: &str, version: &str) -> bool {
        // Use centralized version parsing to handle partial versions like "1" or "5.0"
        let constraint_ver = match Self::parse_version(constraint) {
            Some(v) => v,
            None => return false,
        };

        let ver = match Version::parse(version) {
            Ok(v) => v,
            Err(_) => return false,
        };

        // For 0.x versions, caret behaves like tilde (minor version is significant)
        if constraint_ver.major == 0 {
            // Must match major (0) and minor, patch can increase
            return ver.major == 0
                && ver.minor == constraint_ver.minor
                && ver.patch >= constraint_ver.patch;
        }

        // For non-zero major: must match major, minor/patch can increase
        if ver.major != constraint_ver.major {
            return false;
        }

        // If constraint has minor specified, version's minor must be >=
        if ver.minor > constraint_ver.minor {
            return true;
        }

        if ver.minor == constraint_ver.minor && ver.patch >= constraint_ver.patch {
            return true;
        }

        false
    }

    fn parse_tilde_range(constraint: &str, version: &str) -> bool {
        // Handle partial versions like "2.0" by padding with zeros
        let padded_constraint = Self::pad_version(constraint);

        // Parse the constraint version
        let constraint_ver = match Version::parse(&padded_constraint) {
            Ok(v) => v,
            Err(_) => return false,
        };

        let ver = match Version::parse(version) {
            Ok(v) => v,
            Err(_) => return false,
        };

        // Must be >= constraint version
        if ver < constraint_ver {
            return false;
        }

        let parts: Vec<&str> = constraint.split('.').collect();

        // ~1 := >=1.0.0 <2.0.0
        if parts.len() == 1 {
            return ver.major == constraint_ver.major;
        }

        // ~1.2 := >=1.2.0 <1.3.0
        // ~1.2.3 := >=1.2.3 <1.3.0
        // Must match major and be within same minor range
        if ver.major != constraint_ver.major {
            return false;
        }

        // ~1.2 and ~1.2.3 both require matching exact minor version
        // The upper bound is the next minor version (1.3.0)
        ver.minor == constraint_ver.minor
    }

    fn pad_version(version: &str) -> String {
        let parts: Vec<&str> = version.split('.').collect();
        match parts.len() {
            1 => format!("{}.0.0", version),
            2 => format!("{}.0", version),
            _ => version.to_string(),
        }
    }

    fn parse_version(version: &str) -> Option<Version> {
        let normalized = Self::pad_version(version);
        Version::parse(&normalized).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_type_as_str() {
        assert_eq!(UpdateType::Major.as_str(), "major");
        assert_eq!(UpdateType::Minor.as_str(), "minor");
        assert_eq!(UpdateType::Patch.as_str(), "patch");
        assert_eq!(UpdateType::Security.as_str(), "security");
        assert_eq!(UpdateType::None.as_str(), "none");
        assert_eq!(UpdateType::Unknown.as_str(), "unknown");
    }

    #[test]
    fn test_update_type_priority() {
        // Security should have highest priority (lowest number)
        assert_eq!(UpdateType::Security.priority(), 0);
        assert_eq!(UpdateType::Major.priority(), 1);
        assert_eq!(UpdateType::Minor.priority(), 2);
        assert_eq!(UpdateType::Patch.priority(), 3);
        assert_eq!(UpdateType::None.priority(), 4);
        assert_eq!(UpdateType::Unknown.priority(), 5);
    }

    #[test]
    fn test_classify_major_update() {
        assert_eq!(
            VersionComparator::classify_update("1.0.0", "2.0.0"),
            UpdateType::Major
        );
        assert_eq!(
            VersionComparator::classify_update("0.9.5", "1.0.0"),
            UpdateType::Major
        );
    }

    #[test]
    fn test_classify_minor_update() {
        assert_eq!(
            VersionComparator::classify_update("1.0.0", "1.1.0"),
            UpdateType::Minor
        );
        assert_eq!(
            VersionComparator::classify_update("2.3.0", "2.5.0"),
            UpdateType::Minor
        );
    }

    #[test]
    fn test_classify_patch_update() {
        assert_eq!(
            VersionComparator::classify_update("1.0.0", "1.0.1"),
            UpdateType::Patch
        );
        assert_eq!(
            VersionComparator::classify_update("2.3.4", "2.3.9"),
            UpdateType::Patch
        );
    }

    #[test]
    fn test_classify_no_update() {
        assert_eq!(
            VersionComparator::classify_update("1.0.0", "1.0.0"),
            UpdateType::None
        );
        assert_eq!(
            VersionComparator::classify_update("2.3.4", "2.3.4"),
            UpdateType::None
        );
    }

    #[test]
    fn test_classify_downgrade() {
        // Downgrades should be classified as None
        assert_eq!(
            VersionComparator::classify_update("2.0.0", "1.0.0"),
            UpdateType::None
        );
        assert_eq!(
            VersionComparator::classify_update("1.1.0", "1.0.0"),
            UpdateType::None
        );
    }

    #[test]
    fn test_classify_unknown() {
        // Invalid versions should return Unknown
        assert_eq!(
            VersionComparator::classify_update("invalid", "1.0.0"),
            UpdateType::Unknown
        );
        assert_eq!(
            VersionComparator::classify_update("1.0.0", "invalid"),
            UpdateType::Unknown
        );
        assert_eq!(
            VersionComparator::classify_update("invalid", "also-invalid"),
            UpdateType::Unknown
        );
    }

    #[test]
    fn test_classify_with_pre_release() {
        // Pre-release versions comparison - when numeric parts are equal, it's None
        // (semver treats pre-release as separate from release)
        assert_eq!(
            VersionComparator::classify_update("1.0.0-alpha", "1.0.0"),
            UpdateType::None
        );
        // When numeric parts differ, classification works on numeric parts
        assert_eq!(
            VersionComparator::classify_update("1.0.0-beta.1", "1.1.0"),
            UpdateType::Minor
        );
    }

    #[test]
    fn test_satisfies_exact() {
        let constraint = VersionConstraint::Exact("1.0.0".to_string());
        assert!(VersionComparator::satisfies(&constraint, "1.0.0"));
        assert!(!VersionComparator::satisfies(&constraint, "1.0.1"));
        assert!(!VersionComparator::satisfies(&constraint, "2.0.0"));
    }

    #[test]
    fn test_satisfies_range_caret() {
        // ^1.2.0 matches >=1.2.0 <2.0.0
        let constraint = VersionConstraint::Caret("1.2.0".to_string());

        assert!(VersionComparator::satisfies(&constraint, "1.2.0"));
        assert!(VersionComparator::satisfies(&constraint, "1.3.0"));
        assert!(VersionComparator::satisfies(&constraint, "1.9.9"));
        assert!(!VersionComparator::satisfies(&constraint, "2.0.0"));
        assert!(!VersionComparator::satisfies(&constraint, "0.9.0"));
    }

    #[test]
    fn test_satisfies_range_caret_zero_major() {
        // ^0.x.x has special behavior (only patches within minor)
        let constraint = VersionConstraint::Caret("0.2.0".to_string());

        assert!(VersionComparator::satisfies(&constraint, "0.2.0"));
        assert!(VersionComparator::satisfies(&constraint, "0.2.5"));
        assert!(!VersionComparator::satisfies(&constraint, "0.3.0"));
        assert!(!VersionComparator::satisfies(&constraint, "1.0.0"));
    }

    #[test]
    fn test_satisfies_range_tilde() {
        // ~1.2.0 matches >=1.2.0 <1.3.0
        let constraint = VersionConstraint::Tilde("1.2.0".to_string());

        assert!(VersionComparator::satisfies(&constraint, "1.2.0"));
        assert!(VersionComparator::satisfies(&constraint, "1.2.5"));
        assert!(!VersionComparator::satisfies(&constraint, "1.3.0"));
        assert!(!VersionComparator::satisfies(&constraint, "2.0.0"));
    }

    #[test]
    fn test_satisfies_range_tilde_different_minor() {
        let constraint = VersionConstraint::Tilde("1.2.5".to_string());

        assert!(VersionComparator::satisfies(&constraint, "1.2.5"));
        assert!(VersionComparator::satisfies(&constraint, "1.2.10"));
        assert!(!VersionComparator::satisfies(&constraint, "1.3.0"));
        assert!(!VersionComparator::satisfies(&constraint, "1.2.4"));
    }

    #[test]
    fn test_satisfies_range_generic() {
        // Generic range using semver VersionReq
        let constraint = VersionConstraint::Range(">=1.0.0, <2.0.0".to_string());

        assert!(VersionComparator::satisfies(&constraint, "1.0.0"));
        assert!(VersionComparator::satisfies(&constraint, "1.5.0"));
        assert!(!VersionComparator::satisfies(&constraint, "2.0.0"));
        assert!(!VersionComparator::satisfies(&constraint, "0.9.0"));
    }

    #[test]
    fn test_satisfies_greater_than_equal() {
        let constraint = VersionConstraint::GreaterThanEqual("1.2.0".to_string());

        assert!(VersionComparator::satisfies(&constraint, "1.2.0"));
        assert!(VersionComparator::satisfies(&constraint, "1.3.0"));
        assert!(VersionComparator::satisfies(&constraint, "2.0.0"));
        assert!(!VersionComparator::satisfies(&constraint, "1.1.9"));
        assert!(!VersionComparator::satisfies(&constraint, "0.9.0"));
    }

    #[test]
    fn test_satisfies_invalid_version() {
        let constraint = VersionConstraint::Caret("1.0.0".to_string());

        // Invalid versions should not satisfy any constraint
        assert!(!VersionComparator::satisfies(&constraint, "invalid"));
        assert!(!VersionComparator::satisfies(&constraint, ""));
    }

    #[test]
    fn test_satisfies_with_pre_release() {
        let constraint = VersionConstraint::Caret("1.0.0".to_string());

        // Pre-release versions should work with caret
        assert!(VersionComparator::satisfies(&constraint, "1.0.0-alpha"));
        assert!(VersionComparator::satisfies(&constraint, "1.1.0-beta"));
    }

    #[test]
    fn test_satisfies_empty_constraint() {
        // Empty constraint parts should handle gracefully
        let constraint = VersionConstraint::Caret("".to_string());
        assert!(!VersionComparator::satisfies(&constraint, "1.0.0"));
    }

    #[test]
    fn test_satisfies_single_number_caret() {
        // Single number in caret should work
        let constraint = VersionConstraint::Caret("1".to_string());

        assert!(VersionComparator::satisfies(&constraint, "1.0.0"));
        assert!(VersionComparator::satisfies(&constraint, "1.5.0"));
        assert!(!VersionComparator::satisfies(&constraint, "2.0.0"));
    }

    #[test]
    fn test_satisfies_single_number_tilde() {
        // Single number in tilde: ~1 means >=1.0.0 <2.0.0
        let constraint = VersionConstraint::Tilde("1".to_string());

        assert!(VersionComparator::satisfies(&constraint, "1.0.0"));
        assert!(VersionComparator::satisfies(&constraint, "1.5.0"));
        assert!(!VersionComparator::satisfies(&constraint, "2.0.0"));
        assert!(!VersionComparator::satisfies(&constraint, "0.9.0"));
    }

    #[test]
    fn test_composer_version_scenarios() {
        // Common Composer version constraint scenarios

        // ^5.0 (Symfony major version)
        let symfony = VersionConstraint::Caret("5.0".to_string());
        assert!(VersionComparator::satisfies(&symfony, "5.4.0"));
        assert!(!VersionComparator::satisfies(&symfony, "6.0.0"));

        // ~2.0 (Monolog minor version) - allows >=2.0.0 <2.1.0
        let monolog = VersionConstraint::Tilde("2.0".to_string());
        assert!(VersionComparator::satisfies(&monolog, "2.0.5"));
        assert!(VersionComparator::satisfies(&monolog, "2.0.0"));
        assert!(!VersionComparator::satisfies(&monolog, "2.1.0")); // Different minor
        assert!(!VersionComparator::satisfies(&monolog, "2.8.0")); // Different minor
        assert!(!VersionComparator::satisfies(&monolog, "3.0.0"));

        // ~2 (Monolog major version) - allows >=2.0.0 <3.0.0
        let monolog_major = VersionConstraint::Tilde("2".to_string());
        assert!(VersionComparator::satisfies(&monolog_major, "2.8.0"));
        assert!(!VersionComparator::satisfies(&monolog_major, "3.0.0"));

        // >=7.0 (PHP version requirement)
        let php = VersionConstraint::GreaterThanEqual("7.0".to_string());
        assert!(VersionComparator::satisfies(&php, "7.4.0"));
        assert!(VersionComparator::satisfies(&php, "8.0.0"));
        assert!(!VersionComparator::satisfies(&php, "5.6.0"));
    }

    #[test]
    fn test_update_classification_scenarios() {
        // Real-world update scenarios

        // Security patch
        assert_eq!(
            VersionComparator::classify_update("1.2.3", "1.2.4"),
            UpdateType::Patch
        );

        // Feature release
        assert_eq!(
            VersionComparator::classify_update("1.2.0", "1.3.0"),
            UpdateType::Minor
        );

        // Breaking change
        assert_eq!(
            VersionComparator::classify_update("1.0.0", "2.0.0"),
            UpdateType::Major
        );

        // Laravel-style versioning
        assert_eq!(
            VersionComparator::classify_update("9.0.0", "10.0.0"),
            UpdateType::Major
        );
    }
}
