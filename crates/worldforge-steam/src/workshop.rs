use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

/// Steam Workshop UGC Mod descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct WorkshopItem {
    pub published_id: u64,
    pub title: String,
    pub description: String,
    pub author_steam_id: u64,
    pub author_name: String,
    pub version: String,
    pub hook: String,
    pub tags: Vec<String>,
    pub file_size_bytes: u64,
    pub blake3_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub subscribers_count: u32,
    pub upvotes: u32,
    pub downvotes: u32,
    pub local_installed_path: Option<PathBuf>,
}

/// Steam Workshop UGC Manager for discovering, publishing, and subscribing to .wfmod packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamWorkshopManager {
    pub app_id: u32,
    items: HashMap<u64, WorkshopItem>,
    subscribed_ids: Vec<u64>,
}

impl Default for SteamWorkshopManager {
    fn default() -> Self {
        let mut mgr = Self {
            app_id: 480,
            items: HashMap::new(),
            subscribed_ids: Vec::new(),
        };
        mgr.seed_canonical_workshop_items();
        mgr
    }
}

impl SteamWorkshopManager {
    pub fn new(app_id: u32) -> Self {
        Self {
            app_id,
            ..Default::default()
        }
    }

    /// Seeds curated community mods for initial display.
    pub fn seed_canonical_workshop_items(&mut self) {
        let items = vec![
            WorkshopItem {
                published_id: 2894101901,
                title: "Plasma Ion Superstorm Hazard".into(),
                description: "Overhauls planetary weather with hyper-ion plasma arcs that recharge shields but cause chassis EMP drift.".into(),
                author_steam_id: 76561198000000001,
                author_name: "Valen_Aegis".into(),
                version: "1.2.0".into(),
                hook: "OnBattleTick".into(),
                tags: vec!["Weather".into(), "Hazard".into(), "Combat".into()],
                file_size_bytes: 42_500,
                blake3_hash: "a3f890b1c2d3e4f5".into(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                subscribers_count: 14_280,
                upvotes: 890,
                downvotes: 12,
                local_installed_path: None,
            },
            WorkshopItem {
                published_id: 2894101902,
                title: "Titan Railgun Hyper-Overdrive".into(),
                description: "Gives Vanguard Heavy Titan railguns a 3-round kinetic hyper-burst with realistic recoil physics.".into(),
                author_steam_id: 76561198000000002,
                author_name: "Kovacs_Iron".into(),
                version: "2.0.4".into(),
                hook: "OnUnitSpawn".into(),
                tags: vec!["Weapons".into(), "Titans".into(), "Balance".into()],
                file_size_bytes: 84_100,
                blake3_hash: "b7e210c4f5a6d7e8".into(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                subscribers_count: 22_510,
                upvotes: 1_450,
                downvotes: 24,
                local_installed_path: None,
            },
            WorkshopItem {
                published_id: 2894101903,
                title: "Synthetic Cyber-Symphony Stems".into(),
                description: "6 additional procedural synthesizer audio stem patterns featuring 80s dark synthwave basslines.".into(),
                author_steam_id: 76561198000000003,
                author_name: "Synth_Architect".into(),
                version: "1.0.1".into(),
                hook: "OnAudioTick".into(),
                tags: vec!["Audio".into(), "Music".into(), "Procedural".into()],
                file_size_bytes: 112_000,
                blake3_hash: "c9d8e7f6a5b4c3d2".into(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
                subscribers_count: 9_340,
                upvotes: 620,
                downvotes: 8,
                local_installed_path: None,
            },
        ];

        for item in items {
            self.items.insert(item.published_id, item);
        }
    }

    /// Publishes a local .wfmod package to the Steam Workshop.
    pub fn publish_mod(
        &mut self,
        title: &str,
        description: &str,
        author_name: &str,
        hook: &str,
        tags: Vec<String>,
        mod_bytes: &[u8],
    ) -> WorkshopItem {
        let published_id = 3000000000 + (Uuid::new_v4().as_u128() % 900000000) as u64;
        let hash = blake3::hash(mod_bytes).to_hex().to_string();

        let item = WorkshopItem {
            published_id,
            title: title.to_string(),
            description: description.to_string(),
            author_steam_id: 76561198999999999, // Active player steam ID
            author_name: author_name.to_string(),
            version: "1.0.0".to_string(),
            hook: hook.to_string(),
            tags,
            file_size_bytes: mod_bytes.len() as u64,
            blake3_hash: hash,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            subscribers_count: 1,
            upvotes: 1,
            downvotes: 0,
            local_installed_path: None,
        };

        self.items.insert(published_id, item.clone());
        self.subscribed_ids.push(published_id);
        item
    }

    /// Subscribes to a workshop item by published ID.
    pub fn subscribe(&mut self, published_id: u64) -> bool {
        if self.items.contains_key(&published_id) && !self.subscribed_ids.contains(&published_id) {
            self.subscribed_ids.push(published_id);
            if let Some(item) = self.items.get_mut(&published_id) {
                item.subscribers_count += 1;
            }
            return true;
        }
        false
    }

    /// Unsubscribes from a workshop item.
    pub fn unsubscribe(&mut self, published_id: u64) -> bool {
        if let Some(pos) = self.subscribed_ids.iter().position(|&id| id == published_id) {
            self.subscribed_ids.remove(pos);
            if let Some(item) = self.items.get_mut(&published_id) {
                item.subscribers_count = item.subscribers_count.saturating_sub(1);
            }
            return true;
        }
        false
    }

    pub fn list_all(&self) -> Vec<WorkshopItem> {
        let mut list: Vec<_> = self.items.values().cloned().collect();
        list.sort_by(|a, b| b.subscribers_count.cmp(&a.subscribers_count));
        list
    }

    pub fn list_subscribed(&self) -> Vec<WorkshopItem> {
        self.subscribed_ids
            .iter()
            .filter_map(|id| self.items.get(id).cloned())
            .collect()
    }
}
