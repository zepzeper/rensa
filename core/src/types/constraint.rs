use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionConstraint {
    Range(String),
    Exact(String),
    GreaterThanEqual(String),
    Caret(String),
    Tilde(String),
}

impl Default for VersionConstraint {
    fn default() -> Self {
        VersionConstraint::Range("*".to_string())
    }
}

impl std::fmt::Display for VersionConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VersionConstraint::Range(v) => write!(f, "^{}", v),
            VersionConstraint::Exact(v) => write!(f, "{}", v),
            VersionConstraint::GreaterThanEqual(v) => write!(f, ">={}", v),
            VersionConstraint::Caret(v) => write!(f, "^{}", v),
            VersionConstraint::Tilde(v) => write!(f, "~{}", v),
        }
    }
}
