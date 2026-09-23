//! # worldforge-package
//!
//! World Forge package format (.world).
//! Builds deterministic, reproducible archives from world directories.
//! Same content always produces the same fingerprint.

use std::path::Path;

use worldforge_core::error::{ErrorCode, WorldForgeError};
use worldforge_core::hash::{Fingerprint, FingerprintBuilder};
use worldforge_world::{EntitiesConfig, Scenario, WorldManifest};

mod resolver;
pub use resolver::{resolve_world, ResolvedDependency, ResolvedWorld};

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
    let resolved = resolve_world(world_dir)?;
    let manifest = &resolved.manifest;

    // Collect all files in sorted order
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    collect_files(world_dir, world_dir, &mut files)?;
    if !resolved.dependencies.is_empty() {
        files.push((
            resolver::LOCK_FILE.to_string(),
            resolver::dependency_lock_bytes(&resolved.dependencies)?,
        ));
    }
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
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        fingerprint,
        file_count: entries.len(),
        total_bytes,
        files: entries,
    })
}

/// Validate a world directory without building.
pub fn validate_world(world_dir: &Path) -> Result<(), WorldForgeError> {
    resolve_world(world_dir)?;
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

    let mut previous_path: Option<String> = None;
    let mut manifest_seen = false;
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

        if path.starts_with('/')
            || path.split('/').any(|component| component == "..")
            || path.contains('\\')
        {
            return Err(WorldForgeError::new(
                ErrorCode::PackageInvalid,
                format!("unsafe package path '{path}'"),
            ));
        }
        if previous_path
            .as_ref()
            .is_some_and(|previous| previous >= &path)
        {
            return Err(WorldForgeError::new(
                ErrorCode::PackageInvalid,
                format!("package entries are duplicated or not sorted at '{path}'"),
            ));
        }
        previous_path = Some(path.clone());

        let mut data = Vec::new();
        std::io::Read::read_to_end(&mut entry, &mut data).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::PackageInvalid,
                format!("cannot read entry: {}", e),
            )
        })?;

        // Parse manifest if found
        if path == "world.toml" || path.ends_with("/world.toml") {
            let content = String::from_utf8(data.clone()).map_err(|error| {
                WorldForgeError::new(ErrorCode::PackageInvalid, error.to_string())
            })?;
            let manifest = WorldManifest::from_toml(&content)?;
            manifest.validate()?;
            name = manifest.name;
            version = manifest.version;
            manifest_seen = true;
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

    if !manifest_seen {
        return Err(WorldForgeError::new(
            ErrorCode::PackageInvalid,
            "package does not contain world.toml",
        ));
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

    Ok(fingerprint_files(&files))
}

/// Compute the effective fingerprint used by simulations and replays. For an
/// inherited world this includes a deterministic lock of direct dependency
/// fingerprints, which recursively commits to the full base-world graph.
pub fn fingerprint_resolved_world(world_dir: &Path) -> Result<Fingerprint, WorldForgeError> {
    Ok(resolve_world(world_dir)?.fingerprint)
}

fn fingerprint_files(files: &[(String, Vec<u8>)]) -> Fingerprint {
    let mut builder = FingerprintBuilder::new();
    for (path, data) in files {
        builder.update(path.as_bytes());
        builder.update_fingerprint(&Fingerprint::hash(data));
    }
    builder.finalize()
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

    fn temp_catalog(name: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!(
            "worldforge-package-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_world(
        directory: &Path,
        manifest: &str,
        scenario: &str,
        entities: &str,
    ) {
        fs::create_dir_all(directory).unwrap();
        fs::write(directory.join("world.toml"), manifest).unwrap();
        fs::write(directory.join("scenario.toml"), scenario).unwrap();
        fs::write(directory.join("entities.toml"), entities).unwrap();
    }

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

    #[test]
    fn local_inheritance_merges_content_and_locks_package_fingerprint() {
        let catalog = temp_catalog("inheritance");
        let base = catalog.join("base-world");
        let derived = catalog.join("derived-world");
        write_world(
            &base,
            "name = \"base-world\"\nversion = \"1.0.0\"\n",
            "world = \"base-world\"\nduration_ticks = 10\n",
            r#"[[entities]]
name = "source"
entity_type = "producer"
region = "base"
[entities.initial_inventory]
ore = 10.0
"#,
        );
        write_world(
            &derived,
            "name = \"derived-world\"\nextends = [\"base-world\"]\n",
            "world = \"derived-world\"\nduration_ticks = 10\n",
            r#"[[entities]]
name = "sink"
entity_type = "storage"
region = "derived"

[[links]]
from = "source"
to = "sink"
resource = "ore"
max_per_tick = 1.0
"#,
        );

        let resolved = resolve_world(&derived).unwrap();
        assert_eq!(resolved.entities.entities.len(), 2);
        assert_eq!(resolved.entities.links.len(), 1);
        assert_eq!(resolved.dependencies.len(), 1);
        let first_fingerprint = resolved.fingerprint;

        let archive = catalog.join("derived-world.world");
        let built = build_package(&derived, &archive).unwrap();
        assert_eq!(built.fingerprint, first_fingerprint);
        assert!(built.files.iter().any(|file| file.path == "worldforge.lock"));
        assert_eq!(inspect_package(&archive).unwrap().fingerprint, built.fingerprint);

        let updated = fs::read_to_string(base.join("entities.toml"))
            .unwrap()
            .replace("ore = 10.0", "ore = 20.0");
        fs::write(base.join("entities.toml"), updated).unwrap();
        assert_ne!(
            fingerprint_resolved_world(&derived).unwrap(),
            first_fingerprint
        );
        fs::remove_dir_all(catalog).unwrap();
    }

    #[test]
    fn inheritance_rejects_cycles_and_non_local_references() {
        let catalog = temp_catalog("cycles");
        let a = catalog.join("a");
        let b = catalog.join("b");
        write_world(
            &a,
            "name = \"a\"\nextends = [\"b\"]\n",
            "world = \"a\"\nduration_ticks = 10\n",
            "",
        );
        write_world(
            &b,
            "name = \"b\"\nextends = [\"a\"]\n",
            "world = \"b\"\nduration_ticks = 10\n",
            "",
        );
        let cycle = resolve_world(&a).unwrap_err();
        assert_eq!(cycle.code, ErrorCode::PackageDependencyMissing);
        assert!(cycle.message.contains("cycle"));

        fs::write(a.join("world.toml"), "name = \"a\"\nextends = [\"../b\"]\n")
            .unwrap();
        let traversal = resolve_world(&a).unwrap_err();
        assert_eq!(traversal.code, ErrorCode::PackageDependencyMissing);
        assert!(traversal.message.contains("local world id"));
        fs::remove_dir_all(catalog).unwrap();
    }
}
