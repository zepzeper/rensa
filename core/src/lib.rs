//! Rensa Core Library
//!
//! Core types and traits for the Rensa dependency checker.

pub mod types;
pub mod plugin;
pub mod error;
pub mod http;
pub mod cache;
pub mod config;
pub mod job;
pub mod version;
pub mod report;
pub mod scanner;
pub mod osv;

pub use types::*;
pub use error::{RensaError, Result};
pub use plugin::{Plugin, Detector, Parser, RegistryClient, VulnerabilityScanner, PluginRegistry};
pub use report::ScanReport;
pub use scanner::scan_path;
pub use http::HttpClient;
pub use cache::{CacheManager, CacheEntry};
pub use config::{Config, EcosystemConfig, SeverityThreshold};
pub use job::{JobDescription, JobConfig, ScheduleConfig, ScheduleInterval};
pub use version::{VersionComparator, UpdateType};
