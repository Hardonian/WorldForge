//! World manifest parsing and validation.

use serde::{Deserialize, Serialize};
use worldforge_core::error::{ErrorCode, WorldForgeError};

/// The world manifest (world.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldManifest {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default)]
    pub extends: Vec<String>,
    #[serde(default)]
    pub mods: Vec<String>,
}

fn default_version() -> String {
    "0.1.0".to_string()
}

impl WorldManifest {
    /// Load a manifest from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self, WorldForgeError> {
        toml::from_str(content).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::WorldManifestInvalid,
                format!("failed to parse world.toml: {}", e),
            )
        })
    }

    /// Load a manifest from a file path.
    pub fn from_file(path: &std::path::Path) -> Result<Self, WorldForgeError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::WorldManifestMissing,
                format!("cannot read {}: {}", path.display(), e),
            )
        })?;
        Self::from_toml(&content)
    }

    /// Validate the manifest.
    pub fn validate(&self) -> Result<(), WorldForgeError> {
        if self.name.is_empty() {
            return Err(WorldForgeError::new(
                ErrorCode::WorldManifestInvalid,
                "world name cannot be empty",
            ));
        }
        if self.name.len() > 128 {
            return Err(WorldForgeError::new(
                ErrorCode::WorldManifestInvalid,
                "world name exceeds 128 characters",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_manifest() {
        let toml = r#"
            name = "test-world"
        "#;
        let manifest = WorldManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.name, "test-world");
        assert!(manifest.extends.is_empty());
    }

    #[test]
    fn parse_full_manifest() {
        let toml = r#"
            name = "zombie-toronto"
            description = "Zombies in Toronto"
            version = "1.0.0"
            extends = ["worldforge/base-earth@1", "worldforge/modern-economy@1"]
            mods = ["hardonia/zombie-pathogen@1"]
        "#;
        let manifest = WorldManifest::from_toml(toml).unwrap();
        assert_eq!(manifest.name, "zombie-toronto");
        assert_eq!(manifest.extends.len(), 2);
        assert_eq!(manifest.mods.len(), 1);
    }

    #[test]
    fn empty_name_fails_validation() {
        let manifest = WorldManifest {
            name: "".to_string(),
            description: String::new(),
            version: "0.1.0".to_string(),
            extends: vec![],
            mods: vec![],
        };
        assert!(manifest.validate().is_err());
    }
}
