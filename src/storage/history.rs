use std::fs;
use std::path::{Path, PathBuf};

/// Base directory for application data (%LOCALAPPDATA%/Siro or temp fallback)
pub fn siro_data_dir() -> PathBuf {
    if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
        PathBuf::from(local_app_data).join(crate::APP_NAME)
    } else if let Ok(home) = std::env::var("USERPROFILE") {
        PathBuf::from(home).join(format!(".{}", crate::APP_NAME.to_lowercase()))
    } else {
        std::env::temp_dir().join(crate::APP_NAME)
    }
}

/// Default folder where user projects are stored (%USERPROFILE%/Documents/Siro Projects or fallback)
pub fn default_projects_dir() -> PathBuf {
    let dir = if let Ok(profile) = std::env::var("USERPROFILE") {
        PathBuf::from(profile).join("Documents").join(format!("{} Projects", crate::APP_NAME))
    } else {
        siro_data_dir().join("projects")
    };
    let _ = fs::create_dir_all(&dir);
    dir
}

/// Path to starred boards JSON (%LOCALAPPDATA%/Siro/starred.json)
pub fn starred_file_path() -> PathBuf {
    siro_data_dir().join("starred.json")
}

/// Path to recent boards JSON (%LOCALAPPDATA%/Siro/recents.json)
pub fn recents_file_path() -> PathBuf {
    siro_data_dir().join("recents.json")
}

/// Load file paths from a specific JSON path
pub fn load_paths_from(path: &Path) -> Vec<PathBuf> {
    if let Ok(data) = fs::read_to_string(path) {
        if let Ok(list) = serde_json::from_str::<Vec<PathBuf>>(&data) {
            return list;
        }
    }
    Vec::new()
}

/// Save file paths to a specific JSON path atomically
pub fn save_paths_to(path: &Path, paths: &[PathBuf]) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;

    let json = serde_json::to_string_pretty(paths).map_err(|e| e.to_string())?;
    let temp_name = format!(
        ".{}.tmp_{}",
        path.file_name().and_then(|s| s.to_str()).unwrap_or("history"),
        uuid::Uuid::new_v4()
    );
    let temp_path = parent.join(temp_name);

    fs::write(&temp_path, json).map_err(|e| e.to_string())?;
    if let Err(_err) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(path);
        fs::rename(&temp_path, path).map_err(|e| format!("Atomic write rename failed: {}", e))?;
    }
    Ok(())
}

/// Load starred file paths from disk
pub fn load_starred() -> Vec<PathBuf> {
    load_paths_from(&starred_file_path())
}

/// Save starred file paths to disk
pub fn save_starred(paths: &[PathBuf]) -> Result<(), String> {
    save_paths_to(&starred_file_path(), paths)
}

/// Load recent file paths from disk
pub fn load_recents() -> Vec<PathBuf> {
    load_paths_from(&recents_file_path())
}

/// Save recent file paths to disk
pub fn save_recents(paths: &[PathBuf]) -> Result<(), String> {
    save_paths_to(&recents_file_path(), paths)
}

/// Path to theme preference JSON (%LOCALAPPDATA%/Siro/theme.json)
pub fn theme_file_path() -> PathBuf {
    siro_data_dir().join("theme.json")
}

/// Load saved theme dark mode setting (defaults to false / light mode)
pub fn load_theme_dark_mode() -> bool {
    let path = theme_file_path();
    if let Ok(data) = fs::read_to_string(&path) {
        if let Ok(dark_mode) = serde_json::from_str::<bool>(&data) {
            return dark_mode;
        }
    }
    false
}

/// Save theme dark mode setting to disk atomically
pub fn save_theme_dark_mode(dark_mode: bool) -> Result<(), String> {
    let path = theme_file_path();
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|e| e.to_string())?;

    let json = serde_json::to_string(&dark_mode).map_err(|e| e.to_string())?;
    let temp_name = format!(".theme.tmp_{}", uuid::Uuid::new_v4());
    let temp_path = parent.join(temp_name);

    fs::write(&temp_path, json).map_err(|e| e.to_string())?;
    if let Err(_err) = fs::rename(&temp_path, &path) {
        let _ = fs::remove_file(&path);
        fs::rename(&temp_path, &path).map_err(|e| format!("Atomic theme write rename failed: {}", e))?;
    }
    Ok(())
}

/// Returns common navigation shortcuts for browsing (Projects, Documents, Desktop)
pub fn get_common_directories() -> Vec<(&'static str, PathBuf)> {
    let mut dirs = Vec::new();
    dirs.push(("Projects", default_projects_dir()));

    if let Ok(profile) = std::env::var("USERPROFILE") {
        let docs = PathBuf::from(&profile).join("Documents");
        if docs.exists() {
            dirs.push(("Documents", docs));
        }
        let desktop = PathBuf::from(&profile).join("Desktop");
        if desktop.exists() {
            dirs.push(("Desktop", desktop));
        }
    } else if let Ok(home) = std::env::var("HOME") {
        let docs = PathBuf::from(&home).join("Documents");
        if docs.exists() {
            dirs.push(("Documents", docs));
        }
    }

    dirs
}

/// Format file byte size into human-readable representation
pub fn format_file_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Format standard system time to a readable string
pub fn format_system_time(time: std::time::SystemTime) -> String {
    let dt = chrono::DateTime::<chrono::Local>::from(time);
    dt.format("%Y-%m-%d %H:%M").to_string()
}

/// Clean Windows extended path prefixes (\\?\ and \\?\UNC\) and normalize slashes
pub fn clean_windows_path(path: &Path) -> PathBuf {
    let mut s = path.to_string_lossy().to_string();
    s = s.replace('/', "\\");
    if let Some(rest) = s.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{}", rest))
    } else if let Some(rest) = s.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        PathBuf::from(s)
    }
}

/// Safely truncate path string on Unicode character boundaries to prevent UTF-8 slicing panics
pub fn truncate_path_string(path_str: &str, max_chars: usize) -> String {
    let char_count = path_str.chars().count();
    if char_count <= max_chars {
        path_str.to_string()
    } else {
        let keep_count = max_chars.saturating_sub(3);
        let skip_count = char_count.saturating_sub(keep_count);
        let tail: String = path_str.chars().skip(skip_count).collect();
        format!("...{}", tail)
    }
}

/// Normalize path for consistent comparison across platforms and slashes
pub fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        clean_windows_path(&canonical)
    } else {
        clean_windows_path(path)
    }
}

/// Robust path equality check that handles slashes and casing
pub fn are_paths_equal(a: &Path, b: &Path) -> bool {
    if a == b {
        return true;
    }
    let norm_a = normalize_path(a);
    let norm_b = normalize_path(b);
    if norm_a == norm_b {
        return true;
    }
    norm_a.to_string_lossy().to_lowercase() == norm_b.to_string_lossy().to_lowercase()
}
