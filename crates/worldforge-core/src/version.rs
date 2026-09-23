//! Version types for World Forge.
//!
//! Separate version tracks for engine, world format, replay format,
//! WIT API, and package format.

use serde::{Deserialize, Serialize};

/// Engine version — the simulation runtime itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineVersion {
    pub version: semver::Version,
}

impl EngineVersion {
    pub fn current() -> Self {
        Self {
            version: semver::Version::new(0, 2, 0),
        }
    }
}

impl std::fmt::Display for EngineVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.version)
    }
}

/// Format version for a specific data format.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FormatVersion {
    pub format_name: String,
    pub version: semver::Version,
}

impl FormatVersion {
    pub fn new(name: &str, major: u64, minor: u64, patch: u64) -> Self {
        Self {
            format_name: name.to_string(),
            version: semver::Version::new(major, minor, patch),
        }
    }

    /// Check if this version is compatible with another (same major version).
    pub fn is_compatible_with(&self, other: &FormatVersion) -> bool {
        self.format_name == other.format_name && self.version.major == other.version.major
    }
}

/// Well-known format versions.
pub mod formats {
    use super::FormatVersion;

    pub fn world_format() -> FormatVersion {
        FormatVersion::new("world", 0, 1, 0)
    }

    pub fn replay_format() -> FormatVersion {
        FormatVersion::new("replay", 0, 1, 0)
    }

    pub fn package_format() -> FormatVersion {
        FormatVersion::new("package", 0, 1, 0)
    }

    pub fn wit_api() -> FormatVersion {
        FormatVersion::new("wit-api", 0, 1, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compatibility() {
        assert_eq!(EngineVersion::current().to_string(), "0.2.0");
        let a = FormatVersion::new("world", 0, 1, 0);
        let b = FormatVersion::new("world", 0, 2, 0);
        assert!(a.is_compatible_with(&b)); // Same major

        let c = FormatVersion::new("world", 1, 0, 0);
        assert!(!a.is_compatible_with(&c)); // Different major
    }
}
