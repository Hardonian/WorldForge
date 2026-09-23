//! Typed entity and supply-link configuration.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use worldforge_core::error::{ErrorCode, WorldForgeError};

/// Contents of `entities.toml`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitiesConfig {
    #[serde(default)]
    pub entities: Vec<EntityConfig>,
    #[serde(default)]
    pub links: Vec<LinkConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityConfig {
    pub name: String,
    pub entity_type: String,
    pub region: String,
    #[serde(default)]
    pub initial_inventory: BTreeMap<String, f64>,
    pub production: Option<ProductionConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductionConfig {
    #[serde(default)]
    pub inputs: BTreeMap<String, f64>,
    #[serde(default)]
    pub outputs: BTreeMap<String, f64>,
    #[serde(default)]
    pub energy_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkConfig {
    pub from: String,
    pub to: String,
    pub resource: String,
    pub max_per_tick: f64,
}

impl EntitiesConfig {
    pub fn from_toml(content: &str) -> Result<Self, WorldForgeError> {
        toml::from_str(content).map_err(|error| {
            WorldForgeError::new(
                ErrorCode::WorldSchemaViolation,
                format!("invalid entities.toml: {error}"),
            )
        })
    }

    pub fn from_file(path: &Path) -> Result<Self, WorldForgeError> {
        let content = std::fs::read_to_string(path).map_err(|error| {
            WorldForgeError::new(
                ErrorCode::WorldNotFound,
                format!("cannot read {}: {error}", path.display()),
            )
        })?;
        Self::from_toml(&content)
    }

    /// Validate names, numeric values, and all cross-entity references.
    pub fn validate(&self) -> Result<(), WorldForgeError> {
        if self.entities.is_empty() {
            return Err(schema_error(
                "entities.toml must define at least one entity",
            ));
        }

        let mut names = BTreeSet::new();
        for entity in &self.entities {
            if entity.name.trim().is_empty()
                || entity.entity_type.trim().is_empty()
                || entity.region.trim().is_empty()
            {
                return Err(schema_error(
                    "entity name, entity_type, and region must be non-empty",
                ));
            }
            if !names.insert(entity.name.as_str()) {
                return Err(schema_error(format!(
                    "duplicate entity name '{}'",
                    entity.name
                )));
            }
            validate_amounts(
                &entity.initial_inventory,
                &format!("entity '{}' initial_inventory", entity.name),
                true,
            )?;
            if let Some(production) = &entity.production {
                validate_amounts(
                    &production.inputs,
                    &format!("entity '{}' production inputs", entity.name),
                    true,
                )?;
                validate_amounts(
                    &production.outputs,
                    &format!("entity '{}' production outputs", entity.name),
                    true,
                )?;
                validate_number(
                    production.energy_cost,
                    &format!("entity '{}' energy_cost", entity.name),
                    true,
                )?;
            }
        }

        for link in &self.links {
            if !names.contains(link.from.as_str()) {
                return Err(schema_error(format!(
                    "link source '{}' does not exist",
                    link.from
                )));
            }
            if !names.contains(link.to.as_str()) {
                return Err(schema_error(format!(
                    "link destination '{}' does not exist",
                    link.to
                )));
            }
            if link.from == link.to {
                return Err(schema_error(format!(
                    "link from '{}' cannot target itself",
                    link.from
                )));
            }
            if link.resource.trim().is_empty() {
                return Err(schema_error("link resource must be non-empty"));
            }
            validate_number(link.max_per_tick, "link max_per_tick", false)?;
        }
        Ok(())
    }
}

fn validate_amounts(
    values: &BTreeMap<String, f64>,
    context: &str,
    allow_zero: bool,
) -> Result<(), WorldForgeError> {
    for (resource, value) in values {
        if resource.trim().is_empty() {
            return Err(schema_error(format!(
                "{context} contains an empty resource name"
            )));
        }
        validate_number(*value, &format!("{context}.{resource}"), allow_zero)?;
    }
    Ok(())
}

fn validate_number(value: f64, context: &str, allow_zero: bool) -> Result<(), WorldForgeError> {
    if !value.is_finite() || value < 0.0 || (!allow_zero && value == 0.0) {
        let expectation = if allow_zero {
            "a finite non-negative number"
        } else {
            "a finite positive number"
        };
        return Err(schema_error(format!("{context} must be {expectation}")));
    }
    Ok(())
}

fn schema_error(message: impl Into<String>) -> WorldForgeError {
    WorldForgeError::new(ErrorCode::WorldSchemaViolation, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_duplicate_entities_and_dangling_links() {
        let duplicate = EntitiesConfig::from_toml(
            r#"
            [[entities]]
            name = "one"
            entity_type = "storage"
            region = "here"
            [[entities]]
            name = "one"
            entity_type = "storage"
            region = "there"
            "#,
        )
        .unwrap();
        assert!(duplicate.validate().is_err());

        let dangling = EntitiesConfig::from_toml(
            r#"
            [[entities]]
            name = "one"
            entity_type = "storage"
            region = "here"
            [[links]]
            from = "one"
            to = "missing"
            resource = "ore"
            max_per_tick = 1.0
            "#,
        )
        .unwrap();
        assert!(dangling.validate().is_err());
    }

    #[test]
    fn accepts_valid_config() {
        let config = EntitiesConfig::from_toml(
            r#"
            [[entities]]
            name = "one"
            entity_type = "storage"
            region = "here"
            [entities.initial_inventory]
            ore = 1.0
            "#,
        )
        .unwrap();
        config.validate().unwrap();
    }
}
