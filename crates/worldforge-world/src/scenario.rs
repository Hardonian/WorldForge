//! Scenario definition and loading.

use serde::{Deserialize, Serialize};
use worldforge_core::error::{ErrorCode, WorldForgeError};

use crate::event::ScheduledEvent;
use crate::objective::Objective;

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
            if event.tick > self.duration_ticks {
                return Err(WorldForgeError::new(
                    ErrorCode::ScenarioInvalid,
                    format!(
                        "scheduled event at tick {} exceeds duration {}",
                        event.tick, self.duration_ticks
                    ),
                ));
            }
        }
        Ok(())
    }
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
