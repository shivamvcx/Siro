use crate::model::board::Board;
use egui::Pos2;
use uuid::Uuid;

pub struct EraserTool {
    pub radius: f32,
    pub is_active: bool,
    pub undo_recorded: bool,
}

impl Default for EraserTool {
    fn default() -> Self {
        Self {
            radius: 12.0,
            is_active: false,
            undo_recorded: false,
        }
    }
}

impl EraserTool {
    pub fn on_pointer_down(&mut self, pos: Pos2, board: &mut Board) -> Vec<Uuid> {
        self.is_active = true;
        self.erase_at(pos, board)
    }

    pub fn on_pointer_move(&mut self, pos: Pos2, board: &mut Board) -> Vec<Uuid> {
        if self.is_active {
            self.erase_at(pos, board)
        } else {
            Vec::new()
        }
    }

    pub fn on_pointer_up(&mut self) {
        self.is_active = false;
        self.undo_recorded = false;
    }

    pub fn erase_at(&self, pos: Pos2, board: &mut Board) -> Vec<Uuid> {
        let mut to_remove = Vec::new();
        for elem in &board.elements {
            if elem.contains_point(pos, self.radius) {
                to_remove.push(elem.id());
            }
        }

        for &id in &to_remove {
            board.remove_element(id);
        }

        to_remove
    }
}
