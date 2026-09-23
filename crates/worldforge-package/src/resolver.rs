use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use worldforge_core::error::{ErrorCode, WorldForgeError};
use worldforge_core::hash::Fingerprint;
use worldforge_world::{EntitiesConfig, Scenario, WorldManifest};

use super::{collect_files, fingerprint_files};

const MAX_INHERITANCE_DEPTH: usize = 16;
pub(crate) const LOCK_FILE: &str = "worldforge.lock";

/// A direct, resolved base-world dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDependency {
    pub reference: String,
    pub name: String,
    pub version: String,
    pub fingerprint: Fingerprint,
}

/// The effective content used by validation and simulation after inheritance.
#[derive(Debug, Clone)]
pub struct ResolvedWorld {
    pub path: PathBuf,
    pub manifest: WorldManifest,
    pub scenario: Scenario,
    pub entities: EntitiesConfig,
    pub dependencies: Vec<ResolvedDependency>,
    pub effective_mods: Vec<String>,
    pub fingerprint: Fingerprint,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorldLock {
    format_version: u32,
    dependencies: Vec<LockDependency>,
}

#[derive(Debug, Serialize, Deserialize)]
struct LockDependency {
    reference: String,
    name: String,
    version: String,
    fingerprint: String,
}

/// Resolve a world and all of its local sibling dependencies.
///
/// `extends = ["base-world"]` resolves `base-world` beside the derived world.
/// Slashes, traversal, absolute paths, symlink escapes, and remote references
/// are rejected. Parents are applied in declaration order and the local
/// `entities.toml` fragment is applied last.
pub fn resolve_world(world_dir: &Path) -> Result<ResolvedWorld, WorldForgeError> {
    let world_dir = std::fs::canonicalize(world_dir).map_err(|error| {
        WorldForgeError::new(
            ErrorCode::WorldNotFound,
            format!("cannot resolve world {}: {error}", world_dir.display()),
        )
    })?;
    let catalog_root = world_dir.parent().ok_or_else(|| {
        WorldForgeError::new(
            ErrorCode::PackageDependencyMissing,
            "world directory has no dependency catalog parent",
        )
    })?;
    let catalog_root = std::fs::canonicalize(catalog_root).map_err(io_dependency_error)?;
    let mut stack = Vec::new();
    resolve_internal(&catalog_root, &world_dir, &mut stack, 0)
}

fn resolve_internal(
    catalog_root: &Path,
    world_dir: &Path,
    stack: &mut Vec<PathBuf>,
    depth: usize,
) -> Result<ResolvedWorld, WorldForgeError> {
    if depth >= MAX_INHERITANCE_DEPTH {
        return Err(WorldForgeError::new(
            ErrorCode::PackageDependencyMissing,
            format!("world inheritance exceeds {MAX_INHERITANCE_DEPTH} levels"),
        ));
    }
    let canonical = std::fs::canonicalize(world_dir).map_err(io_dependency_error)?;
    if canonical.parent() != Some(catalog_root) {
        return Err(WorldForgeError::new(
            ErrorCode::PackageDependencyMissing,
            format!(
                "world dependency '{}' escapes the local catalog",
                world_dir.display()
            ),
        ));
    }
    if let Some(cycle_start) = stack.iter().position(|entry| entry == &canonical) {
        let mut cycle = stack[cycle_start..]
            .iter()
            .filter_map(|entry| entry.file_name())
            .map(|entry| entry.to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        cycle.push(
            canonical
                .file_name()
                .map_or_else(|| "?".to_string(), |name| name.to_string_lossy().into_owned()),
        );
        return Err(WorldForgeError::new(
            ErrorCode::PackageDependencyMissing,
            format!("world inheritance cycle: {}", cycle.join(" -> ")),
        ));
    }
    if canonical.join(LOCK_FILE).exists() {
        return Err(WorldForgeError::new(
            ErrorCode::PackageInvalid,
            format!("{LOCK_FILE} is generated during packaging and cannot be a source file"),
        ));
    }

    stack.push(canonical.clone());
    let result = (|| {
        let manifest = WorldManifest::from_file(&canonical.join("world.toml"))?;
        manifest.validate()?;
        let scenario = Scenario::from_file(&canonical.join("scenario.toml"))?;
        let local_entities = EntitiesConfig::from_file(&canonical.join("entities.toml"))?;
        local_entities.validate_fragment()?;

        let mut seen = BTreeSet::new();
        let mut entities = EntitiesConfig {
            entities: Vec::new(),
            links: Vec::new(),
        };
        let mut dependencies = Vec::new();
        let mut effective_mods = Vec::new();

        for reference in &manifest.extends {
            validate_dependency_reference(reference)?;
            if !seen.insert(reference.as_str()) {
                return Err(WorldForgeError::new(
                    ErrorCode::WorldManifestInvalid,
                    format!("duplicate world dependency '{reference}'"),
                ));
            }
            let dependency_path = catalog_root.join(reference);
            let dependency = resolve_internal(catalog_root, &dependency_path, stack, depth + 1)
                .map_err(|error| {
                    WorldForgeError::new(
                        error.code,
                        format!("cannot resolve dependency '{reference}': {}", error.message),
                    )
                })?;
            entities.overlay(dependency.entities.clone());
            extend_unique(&mut effective_mods, dependency.effective_mods);
            dependencies.push(ResolvedDependency {
                reference: reference.clone(),
                name: dependency.manifest.name,
                version: dependency.manifest.version,
                fingerprint: dependency.fingerprint,
            });
        }

        entities.overlay(local_entities);
        entities.validate()?;
        scenario.validate_against(&manifest, &entities)?;
        extend_unique(&mut effective_mods, manifest.mods.clone());
        let fingerprint = fingerprint_with_dependencies(&canonical, &dependencies)?;

        Ok(ResolvedWorld {
            path: canonical.clone(),
            manifest,
            scenario,
            entities,
            dependencies,
            effective_mods,
            fingerprint,
        })
    })();
    stack.pop();
    result
}

fn validate_dependency_reference(reference: &str) -> Result<(), WorldForgeError> {
    if reference.is_empty()
        || reference.len() > 128
        || reference.starts_with('-')
        || reference.ends_with('-')
        || reference.contains("--")
        || !reference.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '_'
        })
    {
        return Err(WorldForgeError::new(
            ErrorCode::PackageDependencyMissing,
            format!(
                "dependency '{reference}' is not a local world id; expected a sibling directory name"
            ),
        ));
    }
    Ok(())
}

fn extend_unique(target: &mut Vec<String>, values: Vec<String>) {
    for value in values {
        if !target.contains(&value) {
            target.push(value);
        }
    }
}

pub(crate) fn dependency_lock_bytes(
    dependencies: &[ResolvedDependency],
) -> Result<Vec<u8>, WorldForgeError> {
    let lock = WorldLock {
        format_version: 1,
        dependencies: dependencies
            .iter()
            .map(|dependency| LockDependency {
                reference: dependency.reference.clone(),
                name: dependency.name.clone(),
                version: dependency.version.clone(),
                fingerprint: dependency.fingerprint.to_string(),
            })
            .collect(),
    };
    toml::to_string(&lock)
        .map(String::into_bytes)
        .map_err(|error| WorldForgeError::new(ErrorCode::PackageBuildFailed, error.to_string()))
}

fn fingerprint_with_dependencies(
    world_dir: &Path,
    dependencies: &[ResolvedDependency],
) -> Result<Fingerprint, WorldForgeError> {
    let mut files = Vec::new();
    collect_files(world_dir, world_dir, &mut files)?;
    if !dependencies.is_empty() {
        files.push((
            LOCK_FILE.to_string(),
            dependency_lock_bytes(dependencies)?,
        ));
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(fingerprint_files(&files))
}

fn io_dependency_error(error: std::io::Error) -> WorldForgeError {
    WorldForgeError::new(ErrorCode::PackageDependencyMissing, error.to_string())
}
