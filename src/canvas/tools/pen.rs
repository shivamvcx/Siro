use crate::model::element::{StrokeElement, StrokePoint};
use egui::{Color32, Pos2};
use uuid::Uuid;

pub struct PenTool {
    pub current_points: Vec<StrokePoint>,
    pub color: Color32,
    pub width: f32,
    pub is_highlighter: bool,
    pub last_pos: Option<Pos2>,
    pub last_time: Option<f64>,
}

impl Default for PenTool {
    fn default() -> Self {
        Self {
            current_points: Vec::new(),
            color: Color32::from_rgb(0, 210, 255), // Cyan neon
            width: 3.0,
            is_highlighter: false,
            last_pos: None,
            last_time: None,
        }
    }
}

impl PenTool {
    pub fn new(color: Color32, width: f32, is_highlighter: bool) -> Self {
        Self {
            current_points: Vec::new(),
            color,
            width,
            is_highlighter,
            last_pos: None,
            last_time: None,
        }
    }

    /// Online thinning during drawing: records points only when delta distance >= 2.0 units
    pub fn on_pointer_move(&mut self, pos: Pos2, time_seconds: f64) {
        if let (Some(last_pos), Some(last_t)) = (self.last_pos, self.last_time) {
            let dist = (pos - last_pos).length();
            if dist < 2.0 {
                return;
            }

            let dt = (time_seconds - last_t).max(0.001) as f32;
            let velocity = (dist / dt).clamp(0.0, 5000.0);

            self.current_points.push(StrokePoint {
                pos,
                pressure: 1.0,
                velocity,
            });
            self.last_pos = Some(pos);
            self.last_time = Some(time_seconds);
        } else {
            self.current_points.push(StrokePoint {
                pos,
                pressure: 1.0,
                velocity: 0.0,
            });
            self.last_pos = Some(pos);
            self.last_time = Some(time_seconds);
        }
    }

    /// Finishes the active stroke, runs Ramer-Douglas-Peucker simplification, and returns the StrokeElement.
    pub fn finish_stroke(&mut self) -> Option<StrokeElement> {
        if self.current_points.len() < 2 {
            self.current_points.clear();
            self.last_pos = None;
            self.last_time = None;
            return None;
        }

        let raw_points = std::mem::take(&mut self.current_points);
        self.last_pos = None;
        self.last_time = None;

        let epsilon = if self.is_highlighter { 1.5 } else { 0.75 };
        let simplified = ramer_douglas_peucker(&raw_points, epsilon);

        Some(StrokeElement::new(
            Uuid::new_v4(),
            simplified,
            self.color,
            self.width,
            self.is_highlighter,
        ))
    }

    pub fn is_active(&self) -> bool {
        !self.current_points.is_empty()
    }
}

/// Ramer-Douglas-Peucker line simplification algorithm
pub fn ramer_douglas_peucker(points: &[StrokePoint], epsilon: f32) -> Vec<StrokePoint> {
    if points.len() <= 2 {
        return points.to_vec();
    }

    let mut dmax = 0.0f32;
    let mut index = 0;
    let end = points.len() - 1;

    let p1 = points[0].pos;
    let p2 = points[end].pos;

    for i in 1..end {
        let d = perpendicular_distance(points[i].pos, p1, p2);
        if d > dmax {
            index = i;
            dmax = d;
        }
    }

    if dmax > epsilon {
        let mut rec1 = ramer_douglas_peucker(&points[0..=index], epsilon);
        let rec2 = ramer_douglas_peucker(&points[index..=end], epsilon);
        rec1.pop(); // Remove duplicate junction point
        rec1.extend(rec2);
        rec1
    } else {
        vec![points[0].clone(), points[end].clone()]
    }
}

fn perpendicular_distance(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let ab_len = ab.length();
    if ab_len < 1e-6 {
        return ap.length();
    }
    let cross = (ab.x * ap.y - ab.y * ap.x).abs();
    cross / ab_len
}
