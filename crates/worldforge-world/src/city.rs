//! Data-driven city construction and research configuration.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use worldforge_core::error::{ErrorCode, WorldForgeError};

use crate::EntitiesConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CityConfig {
    pub treasury: String,
    #[serde(default = "default_population_resource")]
    pub population_resource: String,
    #[serde(default)]
    pub population_growth_per_tick: f64,
    #[serde(default)]
    pub population_needs: BTreeMap<String, f64>,
    #[serde(default)]
    pub districts: Vec<DistrictDefinition>,
    #[serde(default)]
    pub buildings: Vec<BuildingDefinition>,
    #[serde(default)]
    pub technologies: Vec<TechnologyDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistrictDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub slots: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub category: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub allowed_districts: Vec<String>,
    #[serde(default = "default_footprint")]
    pub footprint: u32,
    #[serde(default = "default_max_count")]
    pub max_count: u32,
    #[serde(default)]
    pub cost: BTreeMap<String, f64>,
    #[serde(default)]
    pub upkeep: BTreeMap<String, f64>,
    #[serde(default)]
    pub outputs: BTreeMap<String, f64>,
    #[serde(default)]
    pub housing: u32,
    #[serde(default)]
    pub jobs: u32,
    #[serde(default)]
    pub wellbeing: f64,
    #[serde(default)]
    pub requires_technologies: Vec<String>,
    #[serde(default)]
    pub synergies: Vec<BuildingSynergy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingSynergy {
    pub with_tag: String,
    pub output_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyDefinition {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub branch: String,
    #[serde(default)]
    pub cost: BTreeMap<String, f64>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    #[serde(default)]
    pub excludes: Vec<String>,
    #[serde(default)]
    pub effects: TechnologyEffects,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TechnologyEffects {
    #[serde(default)]
    pub resource_multipliers: BTreeMap<String, f64>,
    #[serde(default)]
    pub building_multipliers: BTreeMap<String, f64>,
    #[serde(default)]
    pub housing_multiplier: f64,
    #[serde(default)]
    pub jobs_multiplier: f64,
    #[serde(default)]
    pub wellbeing_bonus: f64,
}

fn default_population_resource() -> String {
    "population".to_string()
}

const fn default_footprint() -> u32 {
    1
}

const fn default_max_count() -> u32 {
    100
}

impl CityConfig {
    pub fn from_toml(content: &str) -> Result<Self, WorldForgeError> {
        toml::from_str(content).map_err(|error| {
            WorldForgeError::new(
                ErrorCode::WorldSchemaViolation,
                format!("invalid city.toml: {error}"),
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

    pub fn validate(&self, entities: &EntitiesConfig) -> Result<(), WorldForgeError> {
        if self.treasury.trim().is_empty()
            || !entities
                .entities
                .iter()
                .any(|entity| entity.name == self.treasury)
        {
            return Err(city_error(format!(
                "city treasury '{}' is not a world entity",
                self.treasury
            )));
        }
        validate_id(&self.population_resource, "population resource")?;
        validate_amount(self.population_growth_per_tick, "population growth", true)?;
        validate_amounts(&self.population_needs, "population needs")?;
        if self.districts.is_empty() || self.buildings.is_empty() {
            return Err(city_error(
                "city.toml requires at least one district and one building",
            ));
        }

        let mut district_ids = BTreeSet::new();
        for district in &self.districts {
            validate_id(&district.id, "district id")?;
            if district.name.trim().is_empty() || district.slots == 0 {
                return Err(city_error(format!(
                    "district '{}' needs a name and at least one slot",
                    district.id
                )));
            }
            if !district_ids.insert(district.id.as_str()) {
                return Err(city_error(format!("duplicate district '{}'", district.id)));
            }
        }

        let mut building_ids = BTreeSet::new();
        for building in &self.buildings {
            validate_id(&building.id, "building id")?;
            if building.name.trim().is_empty() || building.footprint == 0 || building.max_count == 0
            {
                return Err(city_error(format!(
                    "building '{}' needs a name, footprint, and max_count",
                    building.id
                )));
            }
            if !building_ids.insert(building.id.as_str()) {
                return Err(city_error(format!("duplicate building '{}'", building.id)));
            }
            validate_amounts(&building.cost, &format!("building '{}' cost", building.id))?;
            validate_amounts(
                &building.upkeep,
                &format!("building '{}' upkeep", building.id),
            )?;
            validate_amounts(
                &building.outputs,
                &format!("building '{}' outputs", building.id),
            )?;
            validate_amount(building.wellbeing, "building wellbeing", true)?;
            if building.allowed_districts.is_empty()
                || building
                    .allowed_districts
                    .iter()
                    .any(|id| !district_ids.contains(id.as_str()))
            {
                return Err(city_error(format!(
                    "building '{}' must reference valid allowed districts",
                    building.id
                )));
            }
            let mut tags = BTreeSet::new();
            for tag in &building.tags {
                validate_id(tag, "building tag")?;
                if !tags.insert(tag) {
                    return Err(city_error(format!(
                        "building '{}' has duplicate tag '{}'",
                        building.id, tag
                    )));
                }
            }
            for synergy in &building.synergies {
                validate_id(&synergy.with_tag, "synergy tag")?;
                if !synergy.output_multiplier.is_finite()
                    || synergy.output_multiplier < 1.0
                    || synergy.output_multiplier > 10.0
                {
                    return Err(city_error(format!(
                        "building '{}' synergy multiplier must be between 1 and 10",
                        building.id
                    )));
                }
            }
        }

        let mut technology_ids = BTreeSet::new();
        for technology in &self.technologies {
            validate_id(&technology.id, "technology id")?;
            if technology.name.trim().is_empty() || technology.branch.trim().is_empty() {
                return Err(city_error(format!(
                    "technology '{}' needs a name and branch",
                    technology.id
                )));
            }
            if !technology_ids.insert(technology.id.as_str()) {
                return Err(city_error(format!(
                    "duplicate technology '{}'",
                    technology.id
                )));
            }
            validate_amounts(
                &technology.cost,
                &format!("technology '{}' cost", technology.id),
            )?;
        }
        for building in &self.buildings {
            for technology in &building.requires_technologies {
                if !technology_ids.contains(technology.as_str()) {
                    return Err(city_error(format!(
                        "building '{}' requires unknown technology '{}'",
                        building.id, technology
                    )));
                }
            }
        }
        for technology in &self.technologies {
            for related in technology
                .prerequisites
                .iter()
                .chain(technology.excludes.iter())
            {
                if related == &technology.id || !technology_ids.contains(related.as_str()) {
                    return Err(city_error(format!(
                        "technology '{}' has invalid relationship '{}'",
                        technology.id, related
                    )));
                }
            }
            for (building, multiplier) in &technology.effects.building_multipliers {
                if !building_ids.contains(building.as_str()) {
                    return Err(city_error(format!(
                        "technology '{}' modifies unknown building '{}'",
                        technology.id, building
                    )));
                }
                validate_multiplier(*multiplier, "building multiplier")?;
            }
            for multiplier in technology.effects.resource_multipliers.values() {
                validate_multiplier(*multiplier, "resource multiplier")?;
            }
            for (value, label) in [
                (technology.effects.housing_multiplier, "housing multiplier"),
                (technology.effects.jobs_multiplier, "jobs multiplier"),
            ] {
                if value != 0.0 {
                    validate_multiplier(value, label)?;
                }
            }
            if !technology.effects.wellbeing_bonus.is_finite() {
                return Err(city_error("technology wellbeing bonus must be finite"));
            }
        }
        validate_technology_dag(&self.technologies)?;
        Ok(())
    }
}

fn validate_technology_dag(technologies: &[TechnologyDefinition]) -> Result<(), WorldForgeError> {
    fn visit<'a>(
        id: &'a str,
        by_id: &BTreeMap<&'a str, &'a TechnologyDefinition>,
        visiting: &mut BTreeSet<&'a str>,
        visited: &mut BTreeSet<&'a str>,
    ) -> Result<(), WorldForgeError> {
        if visited.contains(id) {
            return Ok(());
        }
        if !visiting.insert(id) {
            return Err(city_error(format!(
                "technology prerequisite cycle at '{id}'"
            )));
        }
        for prerequisite in &by_id[id].prerequisites {
            visit(prerequisite, by_id, visiting, visited)?;
        }
        visiting.remove(id);
        visited.insert(id);
        Ok(())
    }

    let by_id = technologies
        .iter()
        .map(|technology| (technology.id.as_str(), technology))
        .collect::<BTreeMap<_, _>>();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for id in by_id.keys() {
        visit(id, &by_id, &mut visiting, &mut visited)?;
    }
    Ok(())
}

fn validate_amounts(values: &BTreeMap<String, f64>, context: &str) -> Result<(), WorldForgeError> {
    for (resource, amount) in values {
        validate_id(resource, "resource")?;
        validate_amount(*amount, &format!("{context}.{resource}"), false)?;
    }
    Ok(())
}

fn validate_amount(value: f64, context: &str, allow_zero: bool) -> Result<(), WorldForgeError> {
    if !value.is_finite() || value < 0.0 || (!allow_zero && value == 0.0) {
        return Err(city_error(format!(
            "{context} must be a finite positive number"
        )));
    }
    Ok(())
}

fn validate_multiplier(value: f64, context: &str) -> Result<(), WorldForgeError> {
    if !value.is_finite() || !(0.1..=10.0).contains(&value) {
        return Err(city_error(format!("{context} must be between 0.1 and 10")));
    }
    Ok(())
}

fn validate_id(value: &str, context: &str) -> Result<(), WorldForgeError> {
    if value.is_empty()
        || value.len() > 64
        || value.starts_with('-')
        || value.ends_with('-')
        || !value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '_'
        })
    {
        return Err(city_error(format!("{context} '{value}' is not a valid id")));
    }
    Ok(())
}

fn city_error(message: impl Into<String>) -> WorldForgeError {
    WorldForgeError::new(ErrorCode::WorldSchemaViolation, message)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entities() -> EntitiesConfig {
        EntitiesConfig::from_toml(
            r#"[[entities]]
            name = "city-hall"
            entity_type = "civic"
            region = "center"
            "#,
        )
        .unwrap()
    }

    #[test]
    fn validates_branching_city_tree_and_rejects_cycles() {
        let source = r#"
            treasury = "city-hall"
            population_growth_per_tick = 1.0
            [population_needs]
            food = 0.1

            [[districts]]
            id = "core"
            name = "Core"
            slots = 4

            [[buildings]]
            id = "lab"
            name = "Lab"
            allowed_districts = ["core"]
            tags = ["science"]
            [buildings.cost]
            credits = 10.0
            [buildings.outputs]
            research = 1.0

            [[technologies]]
            id = "networks"
            name = "Networks"
            branch = "systems"
            [technologies.cost]
            research = 10.0

            [[technologies]]
            id = "automation"
            name = "Automation"
            branch = "systems"
            prerequisites = ["networks"]
            [technologies.cost]
            research = 20.0
            [technologies.effects.resource_multipliers]
            research = 1.5
        "#;
        CityConfig::from_toml(source)
            .unwrap()
            .validate(&entities())
            .unwrap();

        let cyclic = source.replace(
            "id = \"networks\"\n            name",
            "id = \"networks\"\n            prerequisites = [\"automation\"]\n            name",
        );
        assert!(CityConfig::from_toml(&cyclic)
            .unwrap()
            .validate(&entities())
            .is_err());
    }
}
