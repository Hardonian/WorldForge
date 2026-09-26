use crate::achievements::AchievementManager;
use crate::cloud::SteamCloudStorage;
use crate::input::SteamInputProfile;
use crate::workshop::SteamWorkshopManager;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// Connection state with the Steamworks runtime.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SteamConnectionStatus {
    Connected,
    StandaloneSimulated,
    OfflineMode,
    Disabled,
}

/// Consolidated Steam user profile metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamUserProfile {
    pub steam_id: u64,
    pub persona_name: String,
    pub is_steam_deck: bool,
    pub language: String,
    pub beta_branch: String,
}

/// Central Steam Context coordinating achievements, cloud saves, workshop, and controller inputs.
#[derive(Debug, Clone)]
pub struct SteamContext {
    pub app_id: u32,
    pub status: SteamConnectionStatus,
    pub profile: SteamUserProfile,
    pub achievements: Arc<Mutex<AchievementManager>>,
    pub cloud: Arc<Mutex<SteamCloudStorage>>,
    pub workshop: Arc<Mutex<SteamWorkshopManager>>,
    pub input: Arc<Mutex<SteamInputProfile>>,
}

impl Default for SteamContext {
    fn default() -> Self {
        Self::new(480)
    }
}

impl SteamContext {
    pub fn new(app_id: u32) -> Self {
        let is_deck = Self::detect_steam_deck();
        let status = if std::env::var("STEAM_APP_ID").is_ok() || std::env::var("SteamAppId").is_ok() {
            SteamConnectionStatus::Connected
        } else {
            SteamConnectionStatus::StandaloneSimulated
        };

        let profile = SteamUserProfile {
            steam_id: 76561198000000000 + (app_id as u64),
            persona_name: if is_deck { "SteamDeck Commander".into() } else { "Forge Commander".into() },
            is_steam_deck: is_deck,
            language: "english".into(),
            beta_branch: "public".into(),
        };

        let input_profile = if is_deck {
            SteamInputProfile::new_steam_deck()
        } else {
            SteamInputProfile::default()
        };

        Self {
            app_id,
            status,
            profile,
            achievements: Arc::new(Mutex::new(AchievementManager::new())),
            cloud: Arc::new(Mutex::new(SteamCloudStorage::new(app_id))),
            workshop: Arc::new(Mutex::new(SteamWorkshopManager::new(app_id))),
            input: Arc::new(Mutex::new(input_profile)),
        }
    }

    /// Detects if the current system environment is a Steam Deck handheld.
    pub fn detect_steam_deck() -> bool {
        if std::env::var("SteamDeck").map(|v| v == "1").unwrap_or(false) {
            return true;
        }
        if std::env::var("STEAM_DECK").map(|v| v == "1").unwrap_or(false) {
            return true;
        }
        #[cfg(target_os = "linux")]
        {
            if let Ok(os_release) = std::fs::read_to_string("/etc/os-release") {
                if os_release.to_lowercase().contains("steamos") {
                    return true;
                }
            }
        }
        false
    }

    /// Returns a comprehensive snapshot of the Steam state for telemetry and dashboard UI.
    pub fn get_telemetry_snapshot(&self) -> SteamTelemetrySnapshot {
        let ach = self.achievements.lock().unwrap();
        let cl = self.cloud.lock().unwrap();
        let ws = self.workshop.lock().unwrap();
        let inp = self.input.lock().unwrap();

        SteamTelemetrySnapshot {
            app_id: self.app_id,
            status: self.status,
            user_profile: self.profile.clone(),
            unlocked_achievements: ach.total_unlocked(),
            total_achievements: ach.list().len(),
            cloud_files_count: cl.files.len(),
            cloud_quota_used_bytes: cl.used_quota_bytes,
            cloud_quota_total_bytes: cl.total_quota_bytes,
            workshop_items_count: ws.list_all().len(),
            workshop_subscribed_count: ws.list_subscribed().len(),
            controller_connected: true,
            is_steam_deck: inp.is_steam_deck,
        }
    }
}

/// JSON-serializable telemetry payload for `/api/steam/status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamTelemetrySnapshot {
    pub app_id: u32,
    pub status: SteamConnectionStatus,
    pub user_profile: SteamUserProfile,
    pub unlocked_achievements: usize,
    pub total_achievements: usize,
    pub cloud_files_count: usize,
    pub cloud_quota_used_bytes: u64,
    pub cloud_quota_total_bytes: u64,
    pub workshop_items_count: usize,
    pub workshop_subscribed_count: usize,
    pub controller_connected: bool,
    pub is_steam_deck: bool,
}
