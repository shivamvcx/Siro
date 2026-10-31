use crate::model::board::Board;
use crate::model::planner::PlannerData;
use crate::model::project::{AssetItem, AssetStore, Project, ProjectManifest};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use uuid::Uuid;
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

pub struct SiroFileManager;

impl SiroFileManager {
    /// Atomically writes the project data into a `.siro` ZIP package without mutating project metadata.
    pub fn save_project_to_path(project: &Project, target_path: &Path) -> Result<(), String> {
        let parent_dir = target_path
            .parent()
            .unwrap_or_else(|| Path::new("."));

        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).map_err(|e| format!("Create dir error: {}", e))?;
        }

        let temp_filename = format!(
            ".{}.tmp_{}",
            target_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("project"),
            Uuid::new_v4()
        );
        let temp_path = parent_dir.join(temp_filename);

        // Scope to ensure ZipWriter flushes and file handle is closed
        {
            let temp_file = File::create(&temp_path)
                .map_err(|e| format!("Failed to create temporary file: {}", e))?;
            let mut zip = ZipWriter::new(temp_file);
            let options = SimpleFileOptions::default()
                .compression_method(CompressionMethod::Deflated);

            // 1. Write manifest.json
            zip.start_file("manifest.json", options)
                .map_err(|e| format!("Zip manifest error: {}", e))?;
            let manifest_json = serde_json::to_string_pretty(&project.manifest)
                .map_err(|e| format!("Manifest serialize error: {}", e))?;
            zip.write_all(manifest_json.as_bytes())
                .map_err(|e| format!("Zip write error: {}", e))?;

            // 2. Write boards/{id}.json
            for (board_id, board) in &project.boards {
                let board_entry_name = format!("boards/{}.json", board_id);
                zip.start_file(&board_entry_name, options)
                    .map_err(|e| format!("Zip board entry error: {}", e))?;
                let board_json = serde_json::to_string_pretty(board)
                    .map_err(|e| format!("Board serialize error: {}", e))?;
                zip.write_all(board_json.as_bytes())
                    .map_err(|e| format!("Zip write error: {}", e))?;
            }

            // 3. Write planner.json
            zip.start_file("planner.json", options)
                .map_err(|e| format!("Zip planner error: {}", e))?;
            let planner_json = serde_json::to_string_pretty(&project.planner)
                .map_err(|e| format!("Planner serialize error: {}", e))?;
            zip.write_all(planner_json.as_bytes())
                .map_err(|e| format!("Zip write error: {}", e))?;

            // 4. Write assets/
            for (asset_id, asset_item) in &project.asset_store.assets {
                let asset_entry_name = format!("assets/{}_{}", asset_id, asset_item.filename);
                zip.start_file(&asset_entry_name, options)
                    .map_err(|e| format!("Zip asset entry error: {}", e))?;
                zip.write_all(&asset_item.data)
                    .map_err(|e| format!("Zip asset write error: {}", e))?;
            }

            zip.finish()
                .map_err(|e| format!("Failed to finalize zip container: {}", e))?;
        }

        // Create backup of existing file if it exists
        if target_path.exists() {
            let bak_path = target_path.with_extension("siro.bak");
            let _ = fs::copy(target_path, bak_path);
        }

        // Atomic replace / rename
        if let Err(err) = fs::rename(&temp_path, target_path) {
            // On Windows, if destination exists, rename might fail, so remove dest then rename
            let _ = fs::remove_file(target_path);
            fs::rename(&temp_path, target_path)
                .map_err(|e| format!("Failed atomic rename (orig: {}): {}", err, e))?;
        }

        Ok(())
    }

    /// Atomically saves the user project and updates filepath and dirty state.
    pub fn save_project(project: &mut Project, target_path: &Path) -> Result<(), String> {
        let clean_path = if target_path.extension().is_none() {
            target_path.with_extension("siro")
        } else {
            target_path.to_path_buf()
        };

        Self::save_project_to_path(project, &clean_path)?;
        project.filepath = Some(clean_path);
        project.mark_saved();
        Ok(())
    }

    /// Loads a project from a `.siro` ZIP package.
    pub fn load_project(file_path: &Path) -> Result<Project, String> {
        let file = File::open(file_path)
            .map_err(|e| format!("Failed to open file {:?}: {}", file_path, e))?;
        let mut archive = ZipArchive::new(file)
            .map_err(|e| format!("Failed to read zip archive: {}", e))?;

        // 1. Read manifest.json
        let manifest: ProjectManifest = {
            let mut manifest_entry = archive
                .by_name("manifest.json")
                .map_err(|e| format!("Missing manifest.json in archive: {}", e))?;
            let mut contents = String::new();
            manifest_entry
                .read_to_string(&mut contents)
                .map_err(|e| format!("Failed to read manifest.json: {}", e))?;
            serde_json::from_str(&contents)
                .map_err(|e| format!("Invalid manifest.json format: {}", e))?
        };

        // 2. Read planner.json
        let planner: PlannerData = if let Ok(mut planner_entry) = archive.by_name("planner.json") {
            let mut contents = String::new();
            if planner_entry.read_to_string(&mut contents).is_ok() {
                serde_json::from_str(&contents).unwrap_or_default()
            } else {
                PlannerData::default()
            }
        } else {
            PlannerData::default()
        };

        // 3. Read boards
        let mut boards = HashMap::new();
        for &board_id in &manifest.board_order {
            let entry_name = format!("boards/{}.json", board_id);
            if let Ok(mut board_entry) = archive.by_name(&entry_name) {
                let mut contents = String::new();
                if board_entry.read_to_string(&mut contents).is_ok() {
                    if let Ok(mut board) = serde_json::from_str::<Board>(&contents) {
                        board.id = board_id;
                        boards.insert(board_id, board);
                    }
                }
            }
        }

        // If no boards matched or active_board does not exist, create or pick a safe fallback
        let active_id = if boards.contains_key(&manifest.active_board) {
            manifest.active_board
        } else if let Some(&first_id) = manifest.board_order.iter().find(|id| boards.contains_key(id)) {
            first_id
        } else if let Some(&first_id) = boards.keys().next() {
            first_id
        } else {
            let fallback_board = Board::new("Main Schematic");
            let fid = fallback_board.id;
            boards.insert(fid, fallback_board);
            fid
        };

        // 4. Read assets
        let mut asset_store = AssetStore::new();
        for i in 0..archive.len() {
            if let Ok(mut file) = archive.by_index(i) {
                let name = file.name().to_string();
                if name.starts_with("assets/") && !name.ends_with('/') {
                    let mut data = Vec::new();
                    if file.read_to_end(&mut data).is_ok() {
                        let filename_part = name.trim_start_matches("assets/");
                        let (uuid_str, original_name) = if let Some(idx) = filename_part.find('_') {
                            (&filename_part[..idx], &filename_part[idx + 1..])
                        } else {
                            (filename_part, filename_part)
                        };
                        let asset_id = Uuid::parse_str(uuid_str).unwrap_or_else(|_| Uuid::new_v4());
                        let item = AssetItem {
                            id: asset_id,
                            filename: original_name.to_string(),
                            mime_type: "application/octet-stream".to_string(),
                            data,
                        };
                        asset_store.assets.insert(asset_id, item);
                    }
                }
            }
        }

        let mut undo_stacks = HashMap::new();
        for &id in boards.keys() {
            undo_stacks.insert(id, crate::model::undo::UndoStack::default());
        }

        Ok(Project {
            manifest,
            boards,
            active_board_id: active_id,
            planner,
            asset_store,
            filepath: Some(file_path.to_path_buf()),
            edit_revision: 0,
            saved_revision: 0,
            undo_stacks,
        })
    }
}

/// Sanitizes a board name by trimming whitespace, stripping any redundant `.siro` extension,
/// replacing filesystem-illegal characters (<>:"/\|?*) and control characters with '_',
/// and trimming any trailing dots or spaces which are illegal on Windows filesystems.
pub fn sanitize_board_name(name: &str) -> String {
    let mut clean = name.trim();
    if clean.to_lowercase().ends_with(".siro") {
        clean = clean[..clean.len() - 5].trim();
    }
    let sanitized: String = clean
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    let sanitized = sanitized.trim_matches(|c| c == ' ' || c == '.').to_string();
    if sanitized.is_empty() {
        "Untitled Project".to_string()
    } else {
        sanitized
    }
}

