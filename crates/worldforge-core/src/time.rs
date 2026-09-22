//! Tick and simulation time types.
//!
//! World Forge uses a fixed-timestep simulation model. The `Tick` type
//! represents a discrete simulation step. All simulation state advances
//! by exactly one tick at a time.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

/// A discrete simulation timestep.
///
/// Ticks are monotonically increasing, starting from 0.
/// The simulation advances exactly one tick per step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tick(u64);

impl Tick {
    /// The initial tick (before any simulation has run).
    pub const ZERO: Tick = Tick(0);

    /// Create a tick from a raw value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Get the raw tick value.
    pub const fn value(&self) -> u64 {
        self.0
    }

    /// Advance to the next tick.
    pub const fn next(&self) -> Self {
        Self(self.0 + 1)
    }

    /// Check if this tick has reached or exceeded a target.
    pub const fn reached(&self, target: u64) -> bool {
        self.0 >= target
    }

    /// Get the number of ticks elapsed since another tick.
    pub fn elapsed_since(&self, other: Tick) -> u64 {
        self.0.saturating_sub(other.0)
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tick:{}", self.0)
    }
}

impl Add<u64> for Tick {
    type Output = Tick;
    fn add(self, rhs: u64) -> Self::Output {
        Tick(self.0 + rhs)
    }
}

impl Sub<Tick> for Tick {
    type Output = u64;
    fn sub(self, rhs: Tick) -> Self::Output {
        self.0 - rhs.0
    }
}

impl From<u64> for Tick {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<Tick> for u64 {
    fn from(t: Tick) -> Self {
        t.0
    }
}

/// Configuration for simulation timing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationTiming {
    /// Duration of the simulation in ticks.
    pub duration_ticks: u64,
    /// Logical time per tick in seconds (for display/agent reasoning only).
    /// Does not affect determinism.
    pub seconds_per_tick: f64,
}

impl Default for SimulationTiming {
    fn default() -> Self {
        Self {
            duration_ticks: 1000,
            seconds_per_tick: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_ordering() {
        assert!(Tick::new(0) < Tick::new(1));
        assert!(Tick::new(100) > Tick::new(99));
    }

    #[test]
    fn tick_arithmetic() {
        let t = Tick::new(10);
        assert_eq!((t + 5).value(), 15);
        assert_eq!(Tick::new(15) - Tick::new(10), 5);
    }

    #[test]
    fn tick_reached() {
        assert!(Tick::new(250).reached(250));
        assert!(Tick::new(251).reached(250));
        assert!(!Tick::new(249).reached(250));
    }
}
