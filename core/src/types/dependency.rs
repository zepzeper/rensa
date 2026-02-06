use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::constraint::VersionConstraint;
use super::ecosystem::Ecosystem;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyFile {
    pub ecosystem: Ecosystem,
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    #[serde(skip)]
    pub constraint: VersionConstraint,
    pub file: PathBuf,
}
