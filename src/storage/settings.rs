use crate::storage::history::siro_data_dir;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_AUTOSAVE_INTERVAL_SECS: u64 = 60;
pub const MIN_AUTOSAVE_INTERVAL_SECS: u64 = 5;
pub const MAX_AUTOSAVE_INTERVAL_SECS: u64 = 600;

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettings {
    /// Autosave interval in seconds: Some(n) between 5 and 600; None means Autosave is Off.
    pub autosave_interval_secs: Option<u64>,
    /// Smart snapping toggle (Figma style)
    #[serde(default = "default_true")]
    pub smart_snapping_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autosave_interval_secs: Some(DEFAULT_AUTOSAVE_INTERVAL_SECS),
            smart_snapping_enabled: true,
        }
    }
}

impl AppSettings {
    /// Clamps an optional interval: None remains None (Off); Some(n) is clamped to [5, 600].
    pub fn sanitize(&mut self) {
        if let Some(ref mut secs) = self.autosave_interval_secs {
            *secs = (*secs).clamp(MIN_AUTOSAVE_INTERVAL_SECS, MAX_AUTOSAVE_INTERVAL_SECS);
        }
    }

    pub fn is_autosave_enabled(&self) -> bool {
        self.autosave_interval_secs.is_some()
    }
}

/// Path to settings JSON (%LOCALAPPDATA%/Siro/settings.json)
pub fn settings_file_path() -> PathBuf {
    siro_data_dir().join("settings.json")
}

/// Load settings from a specific JSON path tolerantly.
/// Falls back to default if file is missing, corrupted, or contains invalid types.
pub fn load_settings_from(path: &Path) -> AppSettings {
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(mut settings) = serde_json::from_str::<AppSettings>(&data) {
            settings.sanitize();
            return settings;
        }
    }
    AppSettings::default()
}

/// Load settings from disk tolerantly (%LOCALAPPDATA%/Siro/settings.json).
pub fn load_settings() -> AppSettings {
    load_settings_from(&settings_file_path())
}

/// Save settings to a specific path atomically using a temp file + rename.
pub fn save_settings_to(path: &Path, settings: &AppSettings) -> Result<(), String> {
    let mut clean_settings = settings.clone();
    clean_settings.sanitize();

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| format!("Failed to create settings directory: {}", e))?;

    let json = serde_json::to_string_pretty(&clean_settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;
    let temp_name = format!(".settings.tmp_{}", uuid::Uuid::new_v4());
    let temp_path = parent.join(temp_name);

    fs::write(&temp_path, json).map_err(|e| format!("Failed to write temp settings file: {}", e))?;
    if let Err(_err) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(path);
        fs::rename(&temp_path, path)
            .map_err(|e| format!("Atomic settings rename failed: {}", e))?;
    }
    Ok(())
}

/// Save settings to disk atomically (%LOCALAPPDATA%/Siro/settings.json).
pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    save_settings_to(&settings_file_path(), settings)
}
