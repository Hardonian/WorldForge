use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Steam Achievement descriptor with title, description, milestone mapping, and unlock state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SteamAchievement {
    pub id: String,
    pub title: String,
    pub description: String,
    pub milestone_suite: u32,
    pub unlocked: bool,
    pub unlock_time: Option<DateTime<Utc>>,
    pub progress: u32,
    pub max_progress: u32,
    pub icon_name: String,
}

/// Registry of all WorldForge Steam Achievements mapped across the 15 simulation milestones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementManager {
    achievements: HashMap<String, SteamAchievement>,
}

impl Default for AchievementManager {
    fn default() -> Self {
        let mut mgr = Self {
            achievements: HashMap::new(),
        };
        mgr.register_all_milestones();
        mgr
    }
}

impl AchievementManager {
    pub fn new() -> Self {
        Self::default()
    }

    fn register(&mut self, ach: SteamAchievement) {
        self.achievements.insert(ach.id.clone(), ach);
    }

    /// Registers the canonical 15 Steam Achievements corresponding to Milestones 1-15.
    pub fn register_all_milestones(&mut self) {
        let canonical = vec![
            SteamAchievement {
                id: "ACH_GPU_OVERDRIVE".into(),
                title: "Hardware Accelerated Vanguard".into(),
                description: "Ignite the WebGL / WebGPU compute pipeline with over 5,000 active particles at 60 FPS.".into(),
                milestone_suite: 1,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 1,
                icon_name: "ach_gpu_overdrive".into(),
            },
            SteamAchievement {
                id: "ACH_NEURAL_NETCODE".into(),
                title: "Synchronized Mesh Commander".into(),
                description: "Establish a deterministic P2P lockstep session with under 25ms simulation jitter.".into(),
                milestone_suite: 2,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 1,
                icon_name: "ach_neural_netcode".into(),
            },
            SteamAchievement {
                id: "ACH_SPECTATOR_DRONE".into(),
                title: "Eye in the Stratosphere".into(),
                description: "Deploy an orbital observation drone and record 300 ticks of continuous battlefield telemetry.".into(),
                milestone_suite: 3,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 300,
                icon_name: "ach_spectator_drone".into(),
            },
            SteamAchievement {
                id: "ACH_NODEGRAPH_ARCHITECT".into(),
                title: "Circuitry of War".into(),
                description: "Construct and execute a custom visual NodeGraph logic tree with at least 8 connected logic gates.".into(),
                milestone_suite: 4,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 8,
                icon_name: "ach_nodegraph_architect".into(),
            },
            SteamAchievement {
                id: "ACH_NEMESIS_CONQUEROR".into(),
                title: "Blood Oath Avenge".into(),
                description: "Defeat a ranked procedural Nemesis Commander who has killed you in a prior skirmish.".into(),
                milestone_suite: 5,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 1,
                icon_name: "ach_nemesis_conqueror".into(),
            },
            SteamAchievement {
                id: "ACH_BEHAVIOR_TRAINER".into(),
                title: "Deep-Q Grand Strategist".into(),
                description: "Train a reinforcement learning behavior tree model through 500 combat iterations with >80% win rate.".into(),
                milestone_suite: 6,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 500,
                icon_name: "ach_behavior_trainer".into(),
            },
            SteamAchievement {
                id: "ACH_PLANET_EXPLORER".into(),
                title: "Planetary Voxel Cartographer".into(),
                description: "Survey a procedural 3D spherical planetoid across all 5 distinct thermodynamic biomes.".into(),
                milestone_suite: 7,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 5,
                icon_name: "ach_planet_explorer".into(),
            },
            SteamAchievement {
                id: "ACH_REPLAY_DIRECTOR".into(),
                title: "Cinematic War Historian".into(),
                description: "Export a cryptographically verified Blake3 replay with a custom 5-keyframe spline camera trajectory.".into(),
                milestone_suite: 8,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 1,
                icon_name: "ach_replay_director".into(),
            },
            SteamAchievement {
                id: "ACH_MACRO_TYCOON".into(),
                title: "Galactic Trade Sovereign".into(),
                description: "Accumulate 1,000,000 Credits across the Galactic Commodity Exchange and dispatch 10 macro trade fleets.".into(),
                milestone_suite: 9,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 10,
                icon_name: "ach_macro_tycoon".into(),
            },
            SteamAchievement {
                id: "ACH_MOD_ARCHITECT".into(),
                title: "Steam Workshop Fabricator".into(),
                description: "Compile and inject a custom sandboxed .wfmod package into a live active simulation.".into(),
                milestone_suite: 10,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 1,
                icon_name: "ach_mod_architect".into(),
            },
            SteamAchievement {
                id: "ACH_CORIOLIS_SURVIVOR".into(),
                title: "Eye of the Supercell".into(),
                description: "Survive a Category 5 Supercell Lightning Hurricane while maintaining combat effectiveness under 70+ km/h wind shear.".into(),
                milestone_suite: 11,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 1,
                icon_name: "ach_coriolis_survivor".into(),
            },
            SteamAchievement {
                id: "ACH_SHADOW_SYNDICATE".into(),
                title: "Master of Shadows".into(),
                description: "Infiltrate all 5 sovereign galactic factions and execute 3 simultaneous Black-Ops grid sabotage missions.".into(),
                milestone_suite: 12,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 3,
                icon_name: "ach_shadow_syndicate".into(),
            },
            SteamAchievement {
                id: "ACH_QUANTUM_TACTICIAN".into(),
                title: "Deterministic Prophet".into(),
                description: "Execute 10,000 parallel Monte-Carlo probabilistic rollouts to secure an optimal tactical branch vector with >90% predicted win rate.".into(),
                milestone_suite: 13,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 10000,
                icon_name: "ach_quantum_tactician".into(),
            },
            SteamAchievement {
                id: "ACH_DYSON_ARCHITECT".into(),
                title: "Monument to Eternity".into(),
                description: "Complete all 4 construction stages of an orbital Dyson Swarm Collector Array delivering >2,500 MW energy.".into(),
                milestone_suite: 14,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 4,
                icon_name: "ach_dyson_architect".into(),
            },
            SteamAchievement {
                id: "ACH_AUDIO_SYMPHONY".into(),
                title: "Maestro of the Battlefield".into(),
                description: "Compose a dynamic 4-stem procedural orchestral symphony and fire every synthetic weapon sound on the soundboard.".into(),
                milestone_suite: 15,
                unlocked: false,
                unlock_time: None,
                progress: 0,
                max_progress: 1,
                icon_name: "ach_audio_symphony".into(),
            },
        ];

        for ach in canonical {
            self.register(ach);
        }
    }

    /// Unlocks an achievement by ID, recording timestamp. Returns true if newly unlocked.
    pub fn unlock(&mut self, id: &str) -> bool {
        if let Some(ach) = self.achievements.get_mut(id) {
            if !ach.unlocked {
                ach.unlocked = true;
                ach.unlock_time = Some(Utc::now());
                ach.progress = ach.max_progress;
                tracing::info!(id = %id, title = %ach.title, "Steam Achievement Unlocked!");
                return true;
            }
        }
        false
    }

    /// Updates progress for a progressive achievement. Unlocks if progress meets max_progress.
    pub fn update_progress(&mut self, id: &str, progress: u32) -> bool {
        if let Some(ach) = self.achievements.get_mut(id) {
            ach.progress = progress.min(ach.max_progress);
            if ach.progress >= ach.max_progress && !ach.unlocked {
                ach.unlocked = true;
                ach.unlock_time = Some(Utc::now());
                tracing::info!(id = %id, title = %ach.title, "Progressive Steam Achievement Unlocked!");
                return true;
            }
        }
        false
    }

    pub fn get(&self, id: &str) -> Option<&SteamAchievement> {
        self.achievements.get(id)
    }

    pub fn list(&self) -> Vec<SteamAchievement> {
        let mut list: Vec<_> = self.achievements.values().cloned().collect();
        list.sort_by_key(|a| a.milestone_suite);
        list
    }

    pub fn total_unlocked(&self) -> usize {
        self.achievements.values().filter(|a| a.unlocked).count()
    }
}
