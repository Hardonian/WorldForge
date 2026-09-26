pub mod achievements;
pub mod cloud;
pub mod context;
pub mod input;
pub mod workshop;

pub use achievements::{AchievementManager, SteamAchievement};
pub use cloud::{CloudFileMetadata, SteamCloudStorage};
pub use context::{SteamConnectionStatus, SteamContext, SteamTelemetrySnapshot, SteamUserProfile};
pub use input::{ControllerAction, ControllerGlyphFamily, SteamInputProfile};
pub use workshop::{SteamWorkshopManager, WorkshopItem};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_achievements_unlock_and_list() {
        let mut mgr = AchievementManager::new();
        assert_eq!(mgr.list().len(), 15);
        assert_eq!(mgr.total_unlocked(), 0);

        let newly = mgr.unlock("ACH_DYSON_ARCHITECT");
        assert!(newly);
        assert_eq!(mgr.total_unlocked(), 1);

        let ach = mgr.get("ACH_DYSON_ARCHITECT").unwrap();
        assert!(ach.unlocked);
        assert!(ach.unlock_time.is_some());

        // Unlocking already unlocked returns false
        assert!(!mgr.unlock("ACH_DYSON_ARCHITECT"));
    }

    #[test]
    fn test_progressive_achievements() {
        let mut mgr = AchievementManager::new();
        assert!(!mgr.update_progress("ACH_QUANTUM_TACTICIAN", 5000));
        assert!(!mgr.get("ACH_QUANTUM_TACTICIAN").unwrap().unlocked);

        assert!(mgr.update_progress("ACH_QUANTUM_TACTICIAN", 10000));
        assert!(mgr.get("ACH_QUANTUM_TACTICIAN").unwrap().unlocked);
    }

    #[test]
    fn test_cloud_storage_write_and_quota() {
        let mut cloud = SteamCloudStorage::new(480);
        let data = b"WorldForge Save Slot 001 Payload";
        let meta = cloud.write_cloud_file("slot_001.wfslot", data);

        assert_eq!(meta.name, "slot_001.wfslot");
        assert_eq!(meta.size_bytes, data.len() as u64);
        assert_eq!(cloud.used_quota_bytes, data.len() as u64);
        assert_eq!(cloud.list_files().len(), 1);
    }

    #[test]
    fn test_workshop_publish_and_subscribe() {
        let mut ws = SteamWorkshopManager::new(480);
        let initial_count = ws.list_all().len();

        let item = ws.publish_mod(
            "Hyper-Vortex Storms",
            "Enhances atmospheric weather physics",
            "Commander",
            "OnWeatherTick",
            vec!["Weather".into()],
            b"test_mod_code()",
        );

        assert_eq!(ws.list_all().len(), initial_count + 1);
        assert_eq!(ws.list_subscribed().len(), 1);
        assert_eq!(ws.list_subscribed()[0].published_id, item.published_id);
    }

    #[test]
    fn test_steam_context_telemetry_snapshot() {
        let ctx = SteamContext::new(480);
        let snapshot = ctx.get_telemetry_snapshot();

        assert_eq!(snapshot.app_id, 480);
        assert_eq!(snapshot.total_achievements, 15);
        assert_eq!(snapshot.unlocked_achievements, 0);
    }
}
