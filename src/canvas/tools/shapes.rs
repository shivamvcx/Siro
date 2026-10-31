use crate::model::element::{ShapeElement, ShapeKind};
use egui::{Color32, Pos2};
use uuid::Uuid;

pub struct ShapeTool {
    pub kind: ShapeKind,
    pub start_pos: Option<Pos2>,
    pub current_pos: Option<Pos2>,
    pub stroke_color: Color32,
    pub stroke_width: f32,
    pub fill_color: Color32,
}

impl Default for ShapeTool {
    fn default() -> Self {
        Self {
            kind: ShapeKind::Rectangle,
            start_pos: None,
            current_pos: None,
            stroke_color: Color32::from_rgb(0, 210, 255),
            stroke_width: 2.0,
            fill_color: Color32::from_rgba_unmultiplied(0, 210, 255, 25),
        }
    }
}

impl ShapeTool {
    pub fn new(kind: ShapeKind) -> Self {
        Self {
            kind,
            start_pos: None,
            current_pos: None,
            stroke_color: Color32::from_rgb(0, 210, 255),
            stroke_width: 2.0,
            fill_color: Color32::from_rgba_unmultiplied(0, 210, 255, 25),
        }
    }

    pub fn on_pointer_down(&mut self, pos: Pos2) {
        self.start_pos = Some(pos);
        self.current_pos = Some(pos);
    }

    pub fn on_pointer_move(&mut self, pos: Pos2) {
        if self.start_pos.is_some() {
            self.current_pos = Some(pos);
        }
    }

    pub fn finish_shape(&mut self) -> Option<ShapeElement> {
        let start = self.start_pos.take()?;
        let end = self.current_pos.take()?;

        let dist = (end - start).length();
        if dist < 4.0 {
            return None;
        }

        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);
        let mid_x = (min_x + max_x) / 2.0;
        let mid_y = (min_y + max_y) / 2.0;

        let vertices = match self.kind {
            ShapeKind::Line | ShapeKind::Arrow => vec![start, end],
            ShapeKind::Rectangle => vec![
                Pos2::new(min_x, min_y),
                Pos2::new(max_x, min_y),
                Pos2::new(max_x, max_y),
                Pos2::new(min_x, max_y),
            ],
            ShapeKind::RoundedRectangle | ShapeKind::Ellipse => vec![
                Pos2::new(min_x, min_y),
                Pos2::new(max_x, max_y),
            ],
            ShapeKind::Diamond => vec![
                Pos2::new(mid_x, min_y),
                Pos2::new(max_x, mid_y),
                Pos2::new(mid_x, max_y),
                Pos2::new(min_x, mid_y),
            ],
            ShapeKind::Triangle => vec![
                Pos2::new(mid_x, min_y),
                Pos2::new(max_x, max_y),
                Pos2::new(min_x, max_y),
            ],
        };

        Some(ShapeElement::new(
            Uuid::new_v4(),
            self.kind,
            vertices,
            self.stroke_color,
            self.stroke_width,
            self.fill_color,
        ))
    }

    pub fn is_active(&self) -> bool {
        self.start_pos.is_some()
    }
}
