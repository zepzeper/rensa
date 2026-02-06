use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RensaError {
    #[error("Dependency '{name}' not found in registry")]
    DependencyNotFound { name: String },

    #[error("Registry error for '{registry}': {source}")]
    RegistryError {
        registry: String,
        #[source]
        source: reqwest::Error,
    },

    #[error("Vulnerability database error: {source}")]
    VulnerabilityError {
        #[source]
        source: reqwest::Error,
    },

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("File parsing error in {file}: {source}")]
    ParseError {
        file: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Plugin error: {message}")]
    Plugin { message: String },

    #[error("Cache error: {message}")]
    Cache { message: String },

    #[error("Validation error in config file '{file}':\n{}", .errors.iter().map(|e| format!("  - {}", e)).collect::<Vec<_>>().join("\n"))]
    Validation { file: PathBuf, errors: Vec<String> },

    #[error("Config file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("Invalid YAML syntax: {message}")]
    YamlSyntaxError {
        file: PathBuf,
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Invalid configuration: {message}")]
    InvalidConfig { message: String },

    #[error("Unknown ecosystem '{ecosystem}'. Supported ecosystems: {supported}")]
    UnknownEcosystem {
        ecosystem: String,
        supported: String,
    },

    #[error("Invalid directory path '{path}': {reason}")]
    InvalidDirectory { path: String, reason: String },

    #[error("Deprecated configuration format detected. Please migrate to version 2 format.")]
    DeprecatedConfig,
}

pub type Result<T> = std::result::Result<T, RensaError>;
