//! Objective definitions and evaluation.

use serde::{Deserialize, Serialize};
use worldforge_core::Fixed64;

/// Status of an objective.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectiveStatus {
    #[default]
    Pending,
    Passed,
    Failed,
}

/// Types of objectives.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ObjectiveType {
    #[serde(rename = "maintain_inventory")]
    MaintainInventory {
        resource: String,
        #[serde(default = "default_min")]
        minimum: f64,
    },
    #[serde(rename = "avoid_shortage")]
    AvoidShortage {
        resource: String,
    },
    #[serde(rename = "reach_production_target")]
    ReachProductionTarget {
        resource: String,
        target: f64,
    },
    #[serde(rename = "survive_until_tick")]
    SurviveUntilTick {
        tick: u64,
    },
}

fn default_min() -> f64 {
    0.0
}

/// An objective defined in a scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Objective {
    #[serde(flatten)]
    pub objective_type: ObjectiveType,
    #[serde(skip)]
    pub status: ObjectiveStatus,
    /// Whether this objective has ever been violated.
    #[serde(skip)]
    pub ever_failed: bool,
}

impl Objective {
    /// Evaluate this objective against current inventory state.
    pub fn evaluate(&mut self, get_inventory: &dyn Fn(&str) -> Fixed64, current_tick: u64) {
        match &self.objective_type {
            ObjectiveType::MaintainInventory { resource, minimum } => {
                let amount = get_inventory(resource);
                let min = Fixed64::from_f64_lossy(*minimum);
                if amount < min {
                    self.status = ObjectiveStatus::Failed;
                    self.ever_failed = true;
                } else if !self.ever_failed {
                    self.status = ObjectiveStatus::Passed;
                }
            }
            ObjectiveType::AvoidShortage { resource } => {
                let amount = get_inventory(resource);
                if amount.is_zero() {
                    self.status = ObjectiveStatus::Failed;
                    self.ever_failed = true;
                } else if !self.ever_failed {
                    self.status = ObjectiveStatus::Passed;
                }
            }
            ObjectiveType::ReachProductionTarget { resource, target } => {
                let amount = get_inventory(resource);
                let target_fixed = Fixed64::from_f64_lossy(*target);
                if amount >= target_fixed {
                    self.status = ObjectiveStatus::Passed;
                } else {
                    self.status = ObjectiveStatus::Pending;
                }
            }
            ObjectiveType::SurviveUntilTick { tick } => {
                if current_tick >= *tick {
                    self.status = ObjectiveStatus::Passed;
                } else {
                    self.status = ObjectiveStatus::Pending;
                }
            }
        }
    }

    /// Get a human-readable name for this objective.
    pub fn name(&self) -> String {
        match &self.objective_type {
            ObjectiveType::MaintainInventory { resource, minimum } => {
                format!("maintain {} ≥ {}", resource, minimum)
            }
            ObjectiveType::AvoidShortage { resource } => {
                format!("avoid {} shortage", resource)
            }
            ObjectiveType::ReachProductionTarget { resource, target } => {
                format!("produce {} {} total", target, resource)
            }
            ObjectiveType::SurviveUntilTick { tick } => {
                format!("survive until tick {}", tick)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maintain_inventory_objective() {
        let mut obj = Objective {
            objective_type: ObjectiveType::MaintainInventory {
                resource: "goods".to_string(),
                minimum: 50.0,
            },
            status: ObjectiveStatus::Pending,
            ever_failed: false,
        };

        // Enough inventory
        obj.evaluate(&|_| Fixed64::from_int(100), 0);
        assert_eq!(obj.status, ObjectiveStatus::Passed);

        // Not enough
        obj.evaluate(&|_| Fixed64::from_int(10), 0);
        assert_eq!(obj.status, ObjectiveStatus::Failed);
    }
}
