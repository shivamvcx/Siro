use super::board::Board;
use super::planner::PlannerData;
use super::undo::UndoStack;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub name: String,
    pub version: String,
    pub board_order: Vec<Uuid>,
    pub active_board: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AssetItem {
    pub id: Uuid,
    pub filename: String,
    pub mime_type: String,
    #[serde(skip)]
    pub data: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct AssetStore {
    pub assets: HashMap<Uuid, AssetItem>,
}

impl AssetStore {
    pub fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn insert(&mut self, filename: impl Into<String>, mime: impl Into<String>, data: Vec<u8>) -> Uuid {
        let id = Uuid::new_v4();
        let item = AssetItem {
            id,
            filename: filename.into(),
            mime_type: mime.into(),
            data,
        };
        self.assets.insert(id, item);
        id
    }

    pub fn get(&self, id: &Uuid) -> Option<&AssetItem> {
        self.assets.get(id)
    }

    pub fn get_mut(&mut self, id: &Uuid) -> Option<&mut AssetItem> {
        self.assets.get_mut(id)
    }
}

#[derive(Clone, Debug)]
pub struct Project {
    pub manifest: ProjectManifest,
    pub boards: HashMap<Uuid, Board>,
    pub active_board_id: Uuid,
    pub planner: PlannerData,
    pub asset_store: AssetStore,
    pub filepath: Option<PathBuf>,
    pub edit_revision: u64,
    pub saved_revision: u64,
    pub undo_stacks: HashMap<Uuid, UndoStack>,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        let name_str = name.into();
        let initial_board = Board::new("Main Schematic");
        let initial_id = initial_board.id;
        let now = Utc::now();

        let mut boards = HashMap::new();
        boards.insert(initial_id, initial_board);

        let manifest = ProjectManifest {
            name: name_str,
            version: "0.1.0".to_string(),
            board_order: vec![initial_id],
            active_board: initial_id,
            created_at: now,
            updated_at: now,
        };

        let mut undo_stacks = HashMap::new();
        undo_stacks.insert(initial_id, UndoStack::default());

        Self {
            manifest,
            boards,
            active_board_id: initial_id,
            planner: PlannerData::default(),
            asset_store: AssetStore::new(),
            filepath: None,
            edit_revision: 0,
            saved_revision: 0,
            undo_stacks,
        }
    }

    pub fn mark_edited(&mut self) {
        self.edit_revision = self.edit_revision.wrapping_add(1);
    }

    pub fn mark_saved(&mut self) {
        self.saved_revision = self.edit_revision;
    }

    pub fn is_unsaved(&self) -> bool {
        self.edit_revision != self.saved_revision
    }

    pub fn has_content(&self) -> bool {
        let has_board_elements = self.boards.values().any(|b| !b.elements.is_empty());
        let has_extra_boards = self.boards.len() > 1;
        let has_custom_board_name = self.boards.len() == 1
            && self.boards.values().any(|b| b.name != "Main Schematic" && b.name != "Sheet");
        let has_assets = !self.asset_store.is_empty();
        let has_planner_data = self.planner != PlannerData::default();
        has_board_elements || has_extra_boards || has_custom_board_name || has_assets || has_planner_data
    }

    pub fn active_board(&self) -> &Board {
        self.boards.get(&self.active_board_id).expect("Active board exists")
    }

    pub fn active_board_mut(&mut self) -> &mut Board {
        self.boards.get_mut(&self.active_board_id).expect("Active board exists")
    }

    pub fn add_board(&mut self, name: impl Into<String>) -> Uuid {
        let board = Board::new(name);
        let id = board.id;
        self.boards.insert(id, board);
        self.manifest.board_order.push(id);
        self.undo_stacks.insert(id, UndoStack::default());
        self.active_board_id = id;
        self.manifest.active_board = id;
        self.mark_edited();
        id
    }

    pub fn remove_board(&mut self, id: Uuid) -> bool {
        if self.boards.len() <= 1 {
            // Do not delete the last board
            return false;
        }

        self.boards.remove(&id);
        self.manifest.board_order.retain(|&b_id| b_id != id);
        self.undo_stacks.remove(&id);

        if self.active_board_id == id {
            self.active_board_id = self.manifest.board_order[0];
            self.manifest.active_board = self.active_board_id;
        }
        self.mark_edited();
        true
    }

    pub fn switch_board(&mut self, id: Uuid) {
        if self.boards.contains_key(&id) {
            self.active_board_id = id;
            self.manifest.active_board = id;
        }
    }

    pub fn rename_board(&mut self, id: Uuid, new_name: impl Into<String>) -> bool {
        if let Some(board) = self.boards.get_mut(&id) {
            let name_str = new_name.into();
            if board.name != name_str {
                board.name = name_str;
                board.updated_at = Utc::now();
                self.manifest.updated_at = Utc::now();
                self.mark_edited();
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn sheet_names(&self) -> Vec<&str> {
        self.manifest
            .board_order
            .iter()
            .filter_map(|id| self.boards.get(id).map(|b| b.name.as_str()))
            .collect()
    }

    pub fn record_undo(&mut self) {
        let board_id = self.active_board_id;
        let elements = self.active_board().elements.clone();
        self.undo_stacks
            .entry(board_id)
            .or_default()
            .push_checkpoint(elements);
        self.mark_edited();
    }

    pub fn undo(&mut self) -> bool {
        let board_id = self.active_board_id;
        let current_elements = self.active_board().elements.clone();
        if let Some(stack) = self.undo_stacks.get_mut(&board_id) {
            if let Some(prev) = stack.undo(current_elements) {
                self.active_board_mut().elements = prev;
                self.active_board_mut().sync_connected_wires();
                self.mark_edited();
                return true;
            }
        }
        false
    }

    pub fn redo(&mut self) -> bool {
        let board_id = self.active_board_id;
        let current_elements = self.active_board().elements.clone();
        if let Some(stack) = self.undo_stacks.get_mut(&board_id) {
            if let Some(next) = stack.redo(current_elements) {
                self.active_board_mut().elements = next;
                self.active_board_mut().sync_connected_wires();
                self.mark_edited();
                return true;
            }
        }
        false
    }

    pub fn can_undo(&self) -> bool {
        self.undo_stacks
            .get(&self.active_board_id)
            .map(|s| s.can_undo())
            .unwrap_or(false)
    }

    pub fn can_redo(&self) -> bool {
        self.undo_stacks
            .get(&self.active_board_id)
            .map(|s| s.can_redo())
            .unwrap_or(false)
    }

    /// Authoritative display name for the project.
    /// If saved to disk, returns the file stem of filepath.
    /// Otherwise returns manifest.name.
    pub fn display_name(&self) -> &str {
        if let Some(path) = &self.filepath {
            if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                if !stem.is_empty() {
                    return stem;
                }
            }
        }
        &self.manifest.name
    }
}

/// Pure helper returning whether a project has unsaved edits.
pub fn is_unsaved(project: &Project) -> bool {
    project.is_unsaved()
}

/// Pure helper returning whether a project contains actual content (non-blank).
pub fn has_content(project: &Project) -> bool {
    project.has_content()
}
