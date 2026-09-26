use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Cloud File Metadata tracked by Steam Remote Storage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CloudFileMetadata {
    pub name: String,
    pub size_bytes: u64,
    pub timestamp: DateTime<Utc>,
    pub blake3_hash: String,
    pub synced: bool,
}

/// Steam Cloud save storage manager.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteamCloudStorage {
    pub app_id: u32,
    pub total_quota_bytes: u64,
    pub used_quota_bytes: u64,
    pub files: HashMap<String, CloudFileMetadata>,
}

impl Default for SteamCloudStorage {
    fn default() -> Self {
        Self {
            app_id: 480, // Default Spacewar / WorldForge Dev AppID
            total_quota_bytes: 100 * 1024 * 1024, // 100 MB Steam Auto-Cloud quota
            used_quota_bytes: 0,
            files: HashMap::new(),
        }
    }
}

impl SteamCloudStorage {
    pub fn new(app_id: u32) -> Self {
        Self {
            app_id,
            ..Default::default()
        }
    }

    /// Scans a local directory (e.g. `.worldforge/saves/`) and stages all save files for cloud sync.
    pub fn sync_from_local_dir(&mut self, local_dir: &Path) -> std::io::Result<Vec<CloudFileMetadata>> {
        let mut synced_files = Vec::new();
        if !local_dir.exists() {
            return Ok(synced_files);
        }

        for entry in std::fs::read_dir(local_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.ends_with(".json") || name.ends_with(".wfslot") {
                        let content = std::fs::read(&path)?;
                        let meta = self.write_cloud_file(name, &content);
                        synced_files.push(meta);
                    }
                }
            }
        }

        Ok(synced_files)
    }

    /// Stores or updates a file in the virtual cloud remote storage.
    pub fn write_cloud_file(&mut self, name: &str, data: &[u8]) -> CloudFileMetadata {
        let hash = blake3::hash(data).to_hex().to_string();
        let size_bytes = data.len() as u64;

        if let Some(old) = self.files.get(name) {
            self.used_quota_bytes = self.used_quota_bytes.saturating_sub(old.size_bytes);
        }
        self.used_quota_bytes += size_bytes;

        let meta = CloudFileMetadata {
            name: name.to_string(),
            size_bytes,
            timestamp: Utc::now(),
            blake3_hash: hash,
            synced: true,
        };

        self.files.insert(name.to_string(), meta.clone());
        meta
    }

    /// Lists all cloud files currently backed up.
    pub fn list_files(&self) -> Vec<CloudFileMetadata> {
        let mut list: Vec<_> = self.files.values().cloned().collect();
        list.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        list
    }

    /// Gets available remaining quota in bytes.
    pub fn available_quota_bytes(&self) -> u64 {
        self.total_quota_bytes.saturating_sub(self.used_quota_bytes)
    }

    /// Generates Steam Auto-Cloud VDF configuration section for `app_build.vdf`.
    pub fn generate_steam_auto_cloud_vdf(&self) -> String {
        format!(
            r#""AutoCloud"
{{
    "RootPath"    "%USERPROFILE%\\.worldforge\\saves"
    "Pattern"     "*.json"
    "Pattern"     "*.wfslot"
    "OS"          "All"
    "Recursive"   "0"
}}"#
        )
    }
}
