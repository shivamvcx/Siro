use super::element::BoardElement;
use chrono::{DateTime, Utc};
use egui::{Pos2, Rect};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn default_board_name() -> String {
    "Sheet".to_string()
}

fn default_camera_pan() -> Pos2 {
    Pos2::new(0.0, 0.0)
}

fn default_camera_zoom() -> f32 {
    1.0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    #[serde(default = "Uuid::new_v4")]
    pub id: Uuid,
    #[serde(default = "default_board_name")]
    pub name: String,
    #[serde(default)]
    pub elements: Vec<BoardElement>,
    #[serde(default = "default_camera_pan")]
    pub camera_pan: Pos2,
    #[serde(default = "default_camera_zoom")]
    pub camera_zoom: f32,
    #[serde(default = "Utc::now")]
    pub created_at: DateTime<Utc>,
    #[serde(default = "Utc::now")]
    pub updated_at: DateTime<Utc>,
}

impl Board {
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            elements: Vec::new(),
            camera_pan: Pos2::new(0.0, 0.0),
            camera_zoom: 1.0,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn add_element(&mut self, elem: BoardElement) {
        self.elements.push(elem);
        self.updated_at = Utc::now();
    }

    pub fn remove_element(&mut self, id: Uuid) -> Option<BoardElement> {
        if let Some(pos) = self.elements.iter().position(|e| e.id() == id) {
            let removed = self.elements.remove(pos);
            self.updated_at = Utc::now();
            self.handle_element_removed(id);
            Some(removed)
        } else {
            None
        }
    }

    pub fn get_element(&self, id: Uuid) -> Option<&BoardElement> {
        self.elements.iter().find(|e| e.id() == id)
    }

    pub fn get_element_mut(&mut self, id: Uuid) -> Option<&mut BoardElement> {
        self.elements.iter_mut().find(|e| e.id() == id)
    }

    pub fn bring_to_front(&mut self, id: Uuid) {
        if let Some(pos) = self.elements.iter().position(|e| e.id() == id) {
            let elem = self.elements.remove(pos);
            self.elements.push(elem);
            self.updated_at = Utc::now();
        }
    }

    pub fn send_to_back(&mut self, id: Uuid) {
        if let Some(pos) = self.elements.iter().position(|e| e.id() == id) {
            let elem = self.elements.remove(pos);
            self.elements.insert(0, elem);
            self.updated_at = Utc::now();
        }
    }

    pub fn bring_forward(&mut self, id: Uuid) {
        if let Some(pos) = self.elements.iter().position(|e| e.id() == id) {
            if pos + 1 < self.elements.len() {
                self.elements.swap(pos, pos + 1);
                self.updated_at = Utc::now();
            }
        }
    }

    pub fn send_backward(&mut self, id: Uuid) {
        if let Some(pos) = self.elements.iter().position(|e| e.id() == id) {
            if pos > 0 {
                self.elements.swap(pos, pos - 1);
                self.updated_at = Utc::now();
            }
        }
    }

    /// Handles graceful decoupling when an element (e.g. an IC or resistor) is removed
    pub fn handle_element_removed(&mut self, removed_id: Uuid) {
        for elem in &mut self.elements {
            if let BoardElement::Wire(w) = elem {
                if w.start.element_id == Some(removed_id) {
                    w.start.element_id = None;
                    w.start.terminal_idx = None;
                }
                if w.end.element_id == Some(removed_id) {
                    w.end.element_id = None;
                    w.end.terminal_idx = None;
                }
            }
        }
    }

    /// Synchronizes wire endpoints to current component positions
    pub fn sync_connected_wires(&mut self) {
        // Collect current component terminal world positions
        let mut terminal_positions: Vec<(Uuid, usize, Pos2)> = Vec::new();
        for elem in &self.elements {
            if let BoardElement::Component(comp) = elem {
                for (idx, _) in comp.terminals.iter().enumerate() {
                    if let Some(pos) = comp.terminal_world_pos(idx) {
                        terminal_positions.push((comp.id, idx, pos));
                    }
                }
            }
        }

        for elem in &mut self.elements {
            if let BoardElement::Wire(w) = elem {
                if let (Some(elem_id), Some(term_idx)) = (w.start.element_id, w.start.terminal_idx) {
                    if let Some((_, _, pos)) = terminal_positions.iter().find(|(cid, tid, _)| *cid == elem_id && *tid == term_idx) {
                        w.start.pos = *pos;
                    }
                }
                if let (Some(elem_id), Some(term_idx)) = (w.end.element_id, w.end.terminal_idx) {
                    if let Some((_, _, pos)) = terminal_positions.iter().find(|(cid, tid, _)| *cid == elem_id && *tid == term_idx) {
                        w.end.pos = *pos;
                    }
                }
            }
        }
    }

    /// Computes cumulative bounding box of all elements on board
    pub fn content_bounds(&self) -> Rect {
        if self.elements.is_empty() {
            return Rect::from_min_size(Pos2::new(-200.0, -200.0), egui::Vec2::new(400.0, 400.0));
        }
        let mut rect = self.elements[0].aabb();
        for e in self.elements.iter().skip(1) {
            rect = rect.union(e.aabb());
        }
        rect.expand(50.0)
    }
}
