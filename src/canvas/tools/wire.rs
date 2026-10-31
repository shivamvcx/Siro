use crate::model::board::Board;
use crate::model::element::{BoardElement, WireColor, WireElement, WireEndpoint};
use egui::Pos2;
use uuid::Uuid;

pub struct WireTool {
    pub active_start: Option<WireEndpoint>,
    pub current_preview_end: Option<Pos2>,
    pub color: WireColor,
    pub width: f32,
    pub hovered_terminal: Option<(Uuid, usize, Pos2)>,
}

impl Default for WireTool {
    fn default() -> Self {
        Self {
            active_start: None,
            current_preview_end: None,
            color: WireColor::Red5V,
            width: 2.5,
            hovered_terminal: None,
        }
    }
}

impl WireTool {
    pub const SNAP_RADIUS: f32 = 20.0;

    pub fn update_hover(&mut self, pointer_world: Pos2, board: &Board) {
        self.hovered_terminal = None;
        let mut min_dist_sq = Self::SNAP_RADIUS * Self::SNAP_RADIUS;

        for elem in &board.elements {
            if let BoardElement::Component(comp) = elem {
                for (idx, _) in comp.terminals.iter().enumerate() {
                    if let Some(pin_world) = comp.terminal_world_pos(idx) {
                        let dist_sq = (pointer_world - pin_world).length_sq();
                        if dist_sq <= min_dist_sq {
                            min_dist_sq = dist_sq;
                            self.hovered_terminal = Some((comp.id, idx, pin_world));
                        }
                    }
                }
            }
        }
    }

    pub fn on_pointer_down(&mut self, pointer_world: Pos2) {
        let (endpoint_pos, elem_id, term_idx) = if let Some((cid, tid, pin_pos)) = self.hovered_terminal {
            (pin_pos, Some(cid), Some(tid))
        } else {
            (pointer_world, None, None)
        };

        self.active_start = Some(WireEndpoint {
            element_id: elem_id,
            terminal_idx: term_idx,
            pos: endpoint_pos,
        });
        self.current_preview_end = Some(endpoint_pos);
    }

    pub fn on_pointer_move(&mut self, pointer_world: Pos2) {
        if self.active_start.is_some() {
            let end_pos = if let Some((_, _, pin_pos)) = self.hovered_terminal {
                pin_pos
            } else {
                pointer_world
            };
            self.current_preview_end = Some(end_pos);
        }
    }

    pub fn finish_wire(&mut self, pointer_world: Pos2) -> Option<WireElement> {
        let start = self.active_start.take()?;
        self.current_preview_end = None;

        let (end_pos, elem_id, term_idx) = if let Some((cid, tid, pin_pos)) = self.hovered_terminal {
            (pin_pos, Some(cid), Some(tid))
        } else {
            (pointer_world, None, None)
        };

        if (end_pos - start.pos).length() < 5.0 {
            return None;
        }

        let end = WireEndpoint {
            element_id: elem_id,
            terminal_idx: term_idx,
            pos: end_pos,
        };

        let mut wire = WireElement::new(Uuid::new_v4(), start, end, self.color);
        wire.width = self.width;
        Some(wire)
    }

    pub fn is_active(&self) -> bool {
        self.active_start.is_some()
    }
}
