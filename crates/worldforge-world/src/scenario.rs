//! Scenario definition and loading.

use serde::{Deserialize, Serialize};
use worldforge_core::error::{ErrorCode, WorldForgeError};

use crate::event::ScheduledEvent;
use crate::objective::{Objective, ObjectiveType};
use crate::{EntitiesConfig, ScheduledEventType, WorldManifest};

/// A scenario defines how to run a world: seed, duration, events, objectives.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    pub world: String,
    #[serde(default = "default_seed")]
    pub seed: u64,
    #[serde(default = "default_duration")]
    pub duration_ticks: u64,
    #[serde(default)]
    pub events: Vec<ScheduledEvent>,
    #[serde(default)]
    pub objectives: Vec<Objective>,
}

fn default_seed() -> u64 {
    42
}

fn default_duration() -> u64 {
    1000
}

impl Scenario {
    /// Load a scenario from a TOML string.
    pub fn from_toml(content: &str) -> Result<Self, WorldForgeError> {
        toml::from_str(content).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                format!("failed to parse scenario: {}", e),
            )
        })
    }

    /// Load from a file.
    pub fn from_file(path: &std::path::Path) -> Result<Self, WorldForgeError> {
        let content = std::fs::read_to_string(path).map_err(|e| {
            WorldForgeError::new(
                ErrorCode::ScenarioMissing,
                format!("cannot read {}: {}", path.display(), e),
            )
        })?;
        Self::from_toml(&content)
    }

    /// Validate the scenario.
    pub fn validate(&self) -> Result<(), WorldForgeError> {
        if self.world.is_empty() {
            return Err(WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                "scenario must specify a world",
            ));
        }
        if self.duration_ticks == 0 {
            return Err(WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                "duration_ticks must be > 0",
            ));
        }
        // Validate events are within duration
        for event in &self.events {
            if event.tick >= self.duration_ticks {
                return Err(WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    format!(
                        "scheduled event at tick {} is outside duration {}",
                        event.tick, self.duration_ticks
                    ),
                ));
            }
        }
        for objective in &self.objectives {
            match &objective.objective_type {
                ObjectiveType::MaintainInventory { resource, minimum } => {
                    validate_resource(resource)?;
                    validate_non_negative(*minimum, "objective minimum")?;
                }
                ObjectiveType::AvoidShortage { resource } => validate_resource(resource)?,
                ObjectiveType::ReachProductionTarget { resource, target } => {
                    validate_resource(resource)?;
                    validate_non_negative(*target, "objective target")?;
                }
                ObjectiveType::SurviveUntilTick { tick } => {
                    if *tick >= self.duration_ticks {
                        return Err(WorldForgeError::new(
                            ErrorCode::ScenarioInvalid,
                            format!(
                                "survive_until_tick {} is outside duration {}",
                                tick, self.duration_ticks
                            ),
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Validate references that span the manifest and entity configuration.
    pub fn validate_against(
        &self,
        manifest: &WorldManifest,
        entities: &EntitiesConfig,
    ) -> Result<(), WorldForgeError> {
        self.validate()?;
        if self.world != manifest.name {
            return Err(WorldForgeError::new(
                ErrorCode::ScenarioInvalid,
                format!(
                    "scenario world '{}' does not match manifest name '{}'",
                    self.world, manifest.name
                ),
            ));
        }
        for event in &self.events {
            match &event.event_type {
                ScheduledEventType::CapacityChange { target, value } => {
                    if !entities.entities.iter().any(|entity| entity.name == *target) {
                        return Err(WorldForgeError::new(
                            ErrorCode::ScenarioInvalid,
                            format!("capacity_change target '{target}' does not exist"),
                        ));
                    }
                    validate_non_negative(*value, "capacity_change value")?;
                }
            }
        }
        Ok(())
    }
}

fn validate_resource(resource: &str) -> Result<(), WorldForgeError> {
    if resource.trim().is_empty() {
        return Err(WorldForgeError::new(
            ErrorCode::ScenarioInvalid,
            "objective resource must be non-empty",
        ));
    }
    Ok(())
}

fn validate_non_negative(value: f64, field: &str) -> Result<(), WorldForgeError> {
    if !value.is_finite() || value < 0.0 {
        return Err(WorldForgeError::new(
            ErrorCode::ScenarioInvalid,
            format!("{field} must be a finite non-negative number"),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_scenario() {
        let toml = r#"
            world = "supply-chain"
            seed = 42
            duration_ticks = 1000

            [[events]]
            tick = 250
            type = "capacity_change"
            target = "factory"
            value = 0.5

            [[objectives]]
            type = "maintain_inventory"
            resource = "goods"
            minimum = 50.0
        "#;
        let scenario = Scenario::from_toml(toml).unwrap();
        assert_eq!(scenario.world, "supply-chain");
        assert_eq!(scenario.seed, 42);
        assert_eq!(scenario.events.len(), 1);
        assert_eq!(scenario.objectives.len(), 1);
    }

    #[test]
    fn empty_world_fails() {
        let scenario = Scenario {
            world: "".to_string(),
            seed: 42,
            duration_ticks: 100,
            events: vec![],
            objectives: vec![],
        };
        assert!(scenario.validate().is_err());
    }
}
