use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Ecosystem {
    Composer,
    Npm,
    Cargo,
    PyPI,
    Pip,
    Go,
    Maven,
    NuGet,
    Gem,
    Dotnet,
    GitHubActions,
}

impl std::fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Ecosystem::Composer => write!(f, "composer"),
            Ecosystem::Npm => write!(f, "npm"),
            Ecosystem::Cargo => write!(f, "cargo"),
            Ecosystem::PyPI => write!(f, "pypi"),
            Ecosystem::Pip => write!(f, "pip"),
            Ecosystem::Go => write!(f, "go"),
            Ecosystem::Maven => write!(f, "maven"),
            Ecosystem::NuGet => write!(f, "nuget"),
            Ecosystem::Gem => write!(f, "gem"),
            Ecosystem::Dotnet => write!(f, "dotnet"),
            Ecosystem::GitHubActions => write!(f, "github_actions"),
        }
    }
}
