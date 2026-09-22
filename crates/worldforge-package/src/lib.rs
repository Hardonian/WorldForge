//! # worldforge-package
//!
//! World Forge package format (.world).
//! Builds deterministic, reproducible archives from world directories.
//! Same content always produces the same fingerprint.

use std::path::Path;

use worldforge_core::error::{ErrorCode, WorldForgeError};
use worldforge_core::hash::{Fingerprint, FingerprintBuilder};
use worldforge_world::WorldManifest;

/// Metadata about a built package.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub fingerprint: Fingerprint,
    pub file_count: usize,
    pub total_bytes: u64,
    pub files: Vec<PackageFileEntry>,
}

/// An entry in a package.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageFileEntry {
    pub path: String,
    pub size: u64,
    pub fingerprint: Fingerprint,
}

/// Build a .world package from a directory.
///
/// The archive is deterministic: same files → same output bytes.
/// Files are sorted alphabetically, timestamps are zeroed.
pub fn build_package(world_dir: &Path, output: &Path) -> Result<PackageInfo, WorldForgeError> {
    // Load and validate manifest
    let manifest_path = world_dir.join("world.toml");
    let manifest = WorldManifest::from_file(&manifest_path)?;
    manifest.validate()?;

    // Collect all files in sorted order
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    collect_files(world_dir, world_dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0)); // Deterministic ordering

    // Build tar archive
    let output_file = std::fs::File::create(output).map_err(|e| {
        WorldForgeError::new(
            ErrorCode::PackageBuildFailed,
            format!("cannot create output file: {}", e),
        )
    })?;

    let mut archive = tar::Builder::new(output_file);
    let mut entries = Vec::new();
    let mut total_bytes = 0u64;
    let mut content_builder = FingerprintBuilder::new();

    for (rel_path, data) in &files {
        let mut header = tar::Header::new_gnu();
        header.set_size(data.len() as u64);
        header.set_mode(0o644);
        header.set_mtime(0); // Zero timestamp for reproducibility
        header.set_uid(0);
        header.set_gid(0);
        header.set_cksum();

        archive
            .append_data(&mut header, rel_path, data.as_slice())
            .map_err(|e| {
                WorldForgeError::new(
                    ErrorCode::PackageBuildFailed,
                    format!("failed to add {} to archive: {}", rel_path, e),
                )
            })?;

        let file_fp = Fingerprint::hash(data);
        content_builder.update(rel_path.as_bytes());
        content_builder.update_fingerprint(&file_fp);

        entries.push(PackageFileEntry {
            path: rel_path.clone(),
            size: data.len() as u64,
            fingerprint: file_fp,
        });
        total_bytes += data.len() as u64;
    }

    archive.finish().map_err(|e| {
        WorldForgeError::new(
            ErrorCode::PackageBuildFailed,
            format!("failed to finalize archive: {}", e),
        )
    })?;

    let fingerprint = content_builder.finalize();

    Ok(PackageInfo {
        name: manifest.name,
        version: manifest.version,
        fingerprint,
        file_count: entries.len(),
        total_bytes,
        files: entries,
    })
}

/// Validate a world directory without building.
pub fn validate_world(world_dir: &Path) -> Result<(), WorldForgeError> {
    let manifest_path = world_dir.join("world.toml");
    if !manifest_path.exists() {
        return Err(WorldForgeError::new(
            ErrorCode::WorldManifestMissing,
            format!("no world.toml found in {}", world_dir.display()),
        ));
    }

    let manifest = WorldManifest::from_file(&manifest_path)?;
    manifest.validate()?;

    let scenario_path = world_dir.join("scenario.toml");
    if scenario_path.exists() {
        let scenario = worldforge_world::Scenario::from_file(&scenario_path)?;
        scenario.validate()?;
    }

    let entities_path = world_dir.join("entities.toml");
    if !entities_path.exists() {
        return Err(WorldForgeError::new(
            ErrorCode::WorldNotFound,
            format!("no entities.toml found in {}", world_dir.display()),
        ));
    }
    let entities = std::fs::read_to_string(&entities_path).map_err(|e| {
        WorldForgeError::new(
            ErrorCode::WorldNotFound,
            format!("cannot read entities.toml: {e}"),
        )
    })?;
    toml::from_str::<toml::Value>(&entities).map_err(|e| {
        WorldForgeError::new(
            ErrorCode::WorldSchemaViolation,
            format!("invalid entities.toml: {e}"),
        )
    })?;

    Ok(())
}

/// Inspect a .world package file.
pub fn inspect_package(package_path: &Path) -> Result<PackageInfo, WorldForgeError> {
    let file = std::fs::File::open(package_path).map_err(|e| {
        WorldForgeError::new(
            ErrorCode::PackageInvalid,
            format!("cannot open package: {}", e),
        )
    })?;

    let mut archive = tar::Archive::new(file);
    let mut entries = Vec::new();
    let mut total_bytes = 0u64;
    let mut content_builder = FingerprintBuilder::new();
    let mut name = String::from("unknown");
    let mut version = String::from("0.0.0");

    for entry_result in archive.entries().map_err(|e| {
        WorldForgeError::new(
            ErrorCode::PackageInvalid,
            format!("cannot read archive: {}", e),
        )
    })? {
        let mut entry = entry_result.map_err(|e| {
            WorldForgeError::new(
                ErrorCode::PackageInvalid,
                format!("corrupt archive entry: {}", e),
            )
        })?;

        let path = entry
            .path()
            .map_err(|e| WorldForgeError::new(ErrorCode::PackageInvalid, e.to_string()))?
            .to_string_lossy()
            .to_string();

        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut data).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::PackageInvalid,
                format!("cannot read entry: {}", e),
            )
        })?;

        // Parse manifest if found
        if path == "world.toml" || path.ends_with("/world.toml") {
            if let Ok(content) = String::from_utf8(data.clone()) {
                if let Ok(manifest) = WorldManifest::from_toml(&content) {
                    name = manifest.name;
                    version = manifest.version;
                }
            }
        }

        let file_fp = Fingerprint::hash(&data);
        content_builder.update(path.as_bytes());
        content_builder.update_fingerprint(&file_fp);

        entries.push(PackageFileEntry {
            path,
            size: data.len() as u64,
            fingerprint: file_fp,
        });
        total_bytes += data.len() as u64;
    }

    Ok(PackageInfo {
        name,
        version,
        fingerprint: content_builder.finalize(),
        file_count: entries.len(),
        total_bytes,
        files: entries,
    })
}

/// Compute the content fingerprint of a world directory (without building).
pub fn fingerprint_world(world_dir: &Path) -> Result<Fingerprint, WorldForgeError> {
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    collect_files(world_dir, world_dir, &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));

    let mut builder = FingerprintBuilder::new();
    for (path, data) in &files {
        builder.update(path.as_bytes());
        builder.update_fingerprint(&Fingerprint::hash(data));
    }
    Ok(builder.finalize())
}

fn collect_files(
    base: &Path,
    dir: &Path,
    files: &mut Vec<(String, Vec<u8>)>,
) -> Result<(), WorldForgeError> {
    let entries = std::fs::read_dir(dir).map_err(|e| {
        WorldForgeError::new(
            ErrorCode::PackageBuildFailed,
            format!("cannot read directory {}: {}", dir.display(), e),
        )
    })?;

    for entry in entries {
        let entry = entry
            .map_err(|e| WorldForgeError::new(ErrorCode::PackageBuildFailed, e.to_string()))?;
        let path = entry.path();

        if path.is_dir() {
            // Skip hidden directories
            if path
                .file_name()
                .map(|n| n.to_string_lossy().starts_with('.'))
                .unwrap_or(false)
            {
                continue;
            }
            collect_files(base, &path, files)?;
        } else {
            if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("replay" | "world")
            ) {
                continue;
            }
            let rel = path
                .strip_prefix(base)
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/");
            let data = std::fs::read(&path).map_err(|e| {
                WorldForgeError::new(
                    ErrorCode::PackageBuildFailed,
                    format!("cannot read {}: {}", path.display(), e),
                )
            })?;
            files.push((rel, data));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn fingerprint_is_reproducible() {
        let dir = std::env::temp_dir().join("wf_test_pkg");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        fs::write(dir.join("world.toml"), "name = \"test\"\n").unwrap();
        fs::write(dir.join("data.txt"), "hello world").unwrap();

        let fp1 = fingerprint_world(&dir).unwrap();
        let fp2 = fingerprint_world(&dir).unwrap();
        assert_eq!(fp1, fp2);

        let _ = fs::remove_dir_all(&dir);
    }
}
