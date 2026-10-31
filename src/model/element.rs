use super::electronics::{ComponentKind, TerminalPin};
use egui::{Color32, Pos2, Rect, Vec2};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrokePoint {
    pub pos: Pos2,
    pub pressure: f32,
    pub velocity: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct StrokeElement {
    pub id: Uuid,
    pub points: Vec<StrokePoint>,
    pub color: Color32,
    pub width: f32,
    pub is_highlighter: bool,
    pub cached_aabb: Rect,
}

impl StrokeElement {
    pub fn new(id: Uuid, points: Vec<StrokePoint>, color: Color32, width: f32, is_highlighter: bool) -> Self {
        let cached_aabb = Self::calculate_aabb(&points, width);
        Self {
            id,
            points,
            color,
            width,
            is_highlighter,
            cached_aabb,
        }
    }

    pub fn calculate_aabb(points: &[StrokePoint], width: f32) -> Rect {
        if points.is_empty() {
            return Rect::NOTHING;
        }
        let mut min = points[0].pos;
        let mut max = points[0].pos;
        for p in points.iter().skip(1) {
            min.x = min.x.min(p.pos.x);
            min.y = min.y.min(p.pos.y);
            max.x = max.x.max(p.pos.x);
            max.y = max.y.max(p.pos.y);
        }
        let pad = (width / 2.0).max(4.0);
        Rect::from_min_max(
            Pos2::new(min.x - pad, min.y - pad),
            Pos2::new(max.x + pad, max.y + pad),
        )
    }

    pub fn recompute_aabb(&mut self) {
        self.cached_aabb = Self::calculate_aabb(&self.points, self.width);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShapeKind {
    Rectangle,
    RoundedRectangle,
    Ellipse,
    Diamond,
    Triangle,
    Line,
    Arrow,
}

impl ShapeKind {
    pub fn display_name(&self) -> &'static str {
        match self {
            ShapeKind::Rectangle => "Rectangle",
            ShapeKind::RoundedRectangle => "Rounded Box",
            ShapeKind::Ellipse => "Ellipse / Circle",
            ShapeKind::Diamond => "Diamond (Decision)",
            ShapeKind::Triangle => "Triangle",
            ShapeKind::Line => "Straight Line",
            ShapeKind::Arrow => "Arrow",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShapeElement {
    pub id: Uuid,
    pub kind: ShapeKind,
    pub vertices: Vec<Pos2>,
    pub stroke_color: Color32,
    pub stroke_width: f32,
    pub fill_color: Color32,
    pub dashed: bool,
    pub start_arrow: bool,
    pub end_arrow: bool,
    pub cached_aabb: Rect,
}

impl ShapeElement {
    pub fn new(
        id: Uuid,
        kind: ShapeKind,
        vertices: Vec<Pos2>,
        stroke_color: Color32,
        stroke_width: f32,
        fill_color: Color32,
    ) -> Self {
        let cached_aabb = Self::calculate_aabb(&vertices, stroke_width);
        Self {
            id,
            kind,
            vertices,
            stroke_color,
            stroke_width,
            fill_color,
            dashed: false,
            start_arrow: false,
            end_arrow: kind == ShapeKind::Arrow,
            cached_aabb,
        }
    }

    pub fn calculate_aabb(vertices: &[Pos2], stroke_width: f32) -> Rect {
        if vertices.is_empty() {
            return Rect::NOTHING;
        }
        let mut min = vertices[0];
        let mut max = vertices[0];
        for v in vertices.iter().skip(1) {
            min.x = min.x.min(v.x);
            min.y = min.y.min(v.y);
            max.x = max.x.max(v.x);
            max.y = max.y.max(v.y);
        }
        let pad = (stroke_width / 2.0).max(4.0);
        Rect::from_min_max(
            Pos2::new(min.x - pad, min.y - pad),
            Pos2::new(max.x + pad, max.y + pad),
        )
    }

    pub fn recompute_aabb(&mut self) {
        self.cached_aabb = Self::calculate_aabb(&self.vertices, self.stroke_width);
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComponentElement {
    pub id: Uuid,
    pub kind: ComponentKind,
    pub pos: Pos2,
    pub rotation_quarters: u8, // 0 = 0 deg, 1 = 90 deg, 2 = 180 deg, 3 = 270 deg
    pub designator: String,
    pub value: String,
    pub terminals: Vec<TerminalPin>,
    pub width: f32,
    pub height: f32,
}

impl ComponentElement {
    pub fn new(id: Uuid, kind: ComponentKind, pos: Pos2) -> Self {
        let (width, height) = kind.default_size();
        let terminals = kind.generate_terminals(width, height);
        let designator = kind.default_designator().to_string();
        let value = kind.default_value().to_string();
        Self {
            id,
            kind,
            pos,
            rotation_quarters: 0,
            designator,
            value,
            terminals,
            width,
            height,
        }
    }

    pub fn aabb(&self) -> Rect {
        let (w, h) = if self.rotation_quarters % 2 == 1 {
            (self.height, self.width)
        } else {
            (self.width, self.height)
        };
        Rect::from_min_size(self.pos, Vec2::new(w, h))
    }

    pub fn terminal_world_pos(&self, terminal_idx: usize) -> Option<Pos2> {
        let pin = self.terminals.get(terminal_idx)?;
        let local = pin.local_pos;
        let rotated_local = match self.rotation_quarters % 4 {
            0 => local,
            1 => Pos2::new(self.height - local.y, local.x),
            2 => Pos2::new(self.width - local.x, self.height - local.y),
            3 => Pos2::new(local.y, self.width - local.x),
            _ => local,
        };
        Some(self.pos + Vec2::new(rotated_local.x, rotated_local.y))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WireEndpoint {
    pub element_id: Option<Uuid>,
    pub terminal_idx: Option<usize>,
    pub pos: Pos2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WireColor {
    Red5V,
    BlackGnd,
    YellowSignal,
    GreenSda,
    BlueScl,
    Orange3V3,
    Custom(Color32),
}

impl WireColor {
    pub fn to_color32(&self) -> Color32 {
        match self {
            WireColor::Red5V => Color32::from_rgb(235, 50, 50),
            WireColor::BlackGnd => Color32::from_rgb(45, 45, 50),
            WireColor::YellowSignal => Color32::from_rgb(240, 200, 20),
            WireColor::GreenSda => Color32::from_rgb(40, 190, 70),
            WireColor::BlueScl => Color32::from_rgb(30, 140, 240),
            WireColor::Orange3V3 => Color32::from_rgb(255, 130, 20),
            WireColor::Custom(c) => *c,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            WireColor::Red5V => "5V Power (Red)",
            WireColor::BlackGnd => "GND (Black)",
            WireColor::YellowSignal => "Signal (Yellow)",
            WireColor::GreenSda => "I2C SDA (Green)",
            WireColor::BlueScl => "I2C SCL (Blue)",
            WireColor::Orange3V3 => "3.3V (Orange)",
            WireColor::Custom(_) => "Custom",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoutingMode {
    Orthogonal,
    Straight,
    Curved,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct WireElement {
    pub id: Uuid,
    pub start: WireEndpoint,
    pub end: WireEndpoint,
    pub color: WireColor,
    pub width: f32,
    pub routing: RoutingMode,
}

impl WireElement {
    pub fn new(id: Uuid, start: WireEndpoint, end: WireEndpoint, color: WireColor) -> Self {
        Self {
            id,
            start,
            end,
            color,
            width: 2.5,
            routing: RoutingMode::Orthogonal,
        }
    }

    pub fn aabb(&self) -> Rect {
        let min_x = self.start.pos.x.min(self.end.pos.x) - 10.0;
        let min_y = self.start.pos.y.min(self.end.pos.y) - 10.0;
        let max_x = self.start.pos.x.max(self.end.pos.x) + 10.0;
        let max_y = self.start.pos.y.max(self.end.pos.y) + 10.0;
        Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
    }
}

fn default_text_bg_color() -> Color32 {
    Color32::from_rgb(35, 40, 50)
}

fn default_text_bg_opacity() -> f32 {
    1.0
}

fn default_text_bg_pad_x() -> f32 {
    8.0
}

fn default_text_bg_pad_y() -> f32 {
    5.0
}

fn default_text_bg_corner_radius() -> f32 {
    6.0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextElement {
    pub id: Uuid,
    pub text: String,
    pub pos: Pos2,
    pub font_size: f32,
    pub color: Color32,
    pub is_sticky: bool,
    pub sticky_color: Color32,
    pub width: f32,
    #[serde(default)]
    pub bg_enabled: bool,
    #[serde(default = "default_text_bg_color")]
    pub bg_color: Color32,
    #[serde(default = "default_text_bg_opacity")]
    pub bg_opacity: f32,
    #[serde(default = "default_text_bg_pad_x")]
    pub bg_pad_x: f32,
    #[serde(default = "default_text_bg_pad_y")]
    pub bg_pad_y: f32,
    #[serde(default = "default_text_bg_corner_radius")]
    pub bg_corner_radius: f32,
    #[serde(default)]
    pub bg_match_width: bool,
}

impl TextElement {
    pub fn new_text(id: Uuid, pos: Pos2, text: String) -> Self {
        Self {
            id,
            text,
            pos,
            font_size: 16.0,
            color: Color32::from_rgb(220, 225, 235),
            is_sticky: false,
            sticky_color: Color32::from_rgb(255, 240, 130),
            width: 200.0,
            bg_enabled: false,
            bg_color: Color32::from_rgb(35, 40, 50),
            bg_opacity: 1.0,
            bg_pad_x: 8.0,
            bg_pad_y: 5.0,
            bg_corner_radius: 6.0,
            bg_match_width: false,
        }
    }

    pub fn new_sticky(id: Uuid, pos: Pos2, text: String, sticky_color: Color32) -> Self {
        Self {
            id,
            text,
            pos,
            font_size: 15.0,
            color: Color32::from_rgb(30, 30, 35),
            is_sticky: true,
            sticky_color,
            width: 180.0,
            bg_enabled: false,
            bg_color: Color32::from_rgb(35, 40, 50),
            bg_opacity: 1.0,
            bg_pad_x: 8.0,
            bg_pad_y: 5.0,
            bg_corner_radius: 6.0,
            bg_match_width: false,
        }
    }

    pub fn aabb(&self) -> Rect {
        if self.is_sticky {
            Rect::from_min_size(self.pos, Vec2::new(self.width, self.width))
        } else {
            let height = (self.text.lines().count().max(1) as f32 * (self.font_size * 1.3)).max(30.0);
            let raw_rect = Rect::from_min_size(self.pos, Vec2::new(self.width, height));
            if self.bg_enabled {
                raw_rect.expand2(Vec2::new(self.bg_pad_x, self.bg_pad_y))
            } else {
                raw_rect
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CellTextSpan {
    pub start: usize,
    pub end: usize,
    pub color: Color32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TableElement {
    pub id: Uuid,
    pub pos: Pos2,
    pub rows: usize,
    pub cols: usize,
    pub cells: Vec<Vec<String>>,
    pub col_widths: Vec<f32>,
    pub is_bom_linked: bool,
    #[serde(default)]
    pub custom_row_height: Option<f32>,
    #[serde(default)]
    pub row_colors: Vec<Option<Color32>>,
    #[serde(default)]
    pub cell_spans: std::collections::HashMap<String, Vec<CellTextSpan>>,
}

impl TableElement {
    pub fn new(id: Uuid, pos: Pos2, rows: usize, cols: usize) -> Self {
        let mut cells = Vec::with_capacity(rows);
        for r in 0..rows {
            let mut row = Vec::with_capacity(cols);
            for c in 0..cols {
                if r == 0 {
                    row.push(format!("Header {}", c + 1));
                } else {
                    row.push(String::new());
                }
            }
            cells.push(row);
        }
        let col_widths = vec![110.0; cols];
        Self {
            id,
            pos,
            rows,
            cols,
            cells,
            col_widths,
            is_bom_linked: false,
            custom_row_height: None,
            row_colors: Vec::new(),
            cell_spans: std::collections::HashMap::new(),
        }
    }

    pub fn cell_key(row: usize, col: usize) -> String {
        format!("{},{}", row, col)
    }

    pub fn get_row_color(&self, row: usize) -> Option<Color32> {
        self.row_colors.get(row).copied().flatten()
    }

    pub fn set_row_color(&mut self, row: usize, color: Option<Color32>) {
        if self.row_colors.len() < self.rows {
            self.row_colors.resize(self.rows, None);
        }
        if row < self.row_colors.len() {
            self.row_colors[row] = color;
        }
    }

    pub fn get_cell_spans(&self, row: usize, col: usize) -> Option<&Vec<CellTextSpan>> {
        self.cell_spans.get(&Self::cell_key(row, col))
    }

    pub fn add_or_update_span(&mut self, row: usize, col: usize, start: usize, end: usize, color: Color32) {
        if start >= end {
            return;
        }
        let key = Self::cell_key(row, col);
        let spans = self.cell_spans.entry(key).or_default();
        let mut new_spans = Vec::new();
        for s in spans.drain(..) {
            if s.end <= start || s.start >= end {
                new_spans.push(s);
            } else {
                if s.start < start {
                    new_spans.push(CellTextSpan { start: s.start, end: start, color: s.color });
                }
                if s.end > end {
                    new_spans.push(CellTextSpan { start: end, end: s.end, color: s.color });
                }
            }
        }
        new_spans.push(CellTextSpan { start, end, color });
        new_spans.sort_by_key(|s| s.start);
        *spans = new_spans;
    }

    pub fn swap_row_spans(&mut self, r1: usize, r2: usize) {
        let mut new_spans = std::collections::HashMap::new();
        for (key, spans) in self.cell_spans.drain() {
            if let Some((r_str, c_str)) = key.split_once(',') {
                if let (Ok(r), Ok(c)) = (r_str.parse::<usize>(), c_str.parse::<usize>()) {
                    let target_r = if r == r1 {
                        r2
                    } else if r == r2 {
                        r1
                    } else {
                        r
                    };
                    new_spans.insert(Self::cell_key(target_r, c), spans);
                }
            }
        }
        self.cell_spans = new_spans;
    }

    pub fn reorder_row_spans(&mut self, from_r: usize, to_r: usize) {
        if from_r == to_r {
            return;
        }
        let mut new_spans = std::collections::HashMap::new();
        for (key, spans) in self.cell_spans.drain() {
            if let Some((r_str, c_str)) = key.split_once(',') {
                if let (Ok(r), Ok(c)) = (r_str.parse::<usize>(), c_str.parse::<usize>()) {
                    let target_r = if r == from_r {
                        to_r
                    } else if from_r < to_r && r > from_r && r <= to_r {
                        r - 1
                    } else if to_r < from_r && r >= to_r && r < from_r {
                        r + 1
                    } else {
                        r
                    };
                    new_spans.insert(Self::cell_key(target_r, c), spans);
                }
            }
        }
        self.cell_spans = new_spans;
    }

    pub fn remove_row_spans(&mut self, removed_row: usize) {
        let mut new_spans = std::collections::HashMap::new();
        for (key, spans) in self.cell_spans.drain() {
            if let Some((r_str, c_str)) = key.split_once(',') {
                if let (Ok(r), Ok(c)) = (r_str.parse::<usize>(), c_str.parse::<usize>()) {
                    if r < removed_row {
                        new_spans.insert(key, spans);
                    } else if r > removed_row {
                        new_spans.insert(Self::cell_key(r - 1, c), spans);
                    }
                }
            }
        }
        self.cell_spans = new_spans;
    }

    pub fn row_height(&self) -> f32 {
        self.custom_row_height.unwrap_or(28.0)
    }

    pub fn total_width(&self) -> f32 {
        self.col_widths.iter().sum()
    }

    pub fn total_height(&self) -> f32 {
        (self.rows as f32 * self.row_height()).max(30.0)
    }

    pub fn aabb(&self) -> Rect {
        Rect::from_min_size(self.pos, Vec2::new(self.total_width(), self.total_height()))
    }

    pub fn get_cell(&self, row: usize, col: usize) -> Option<&str> {
        self.cells.get(row).and_then(|r| r.get(col)).map(|s| s.as_str())
    }

    pub fn get_cell_mut(&mut self, row: usize, col: usize) -> Option<&mut String> {
        self.cells.get_mut(row).and_then(|r| r.get_mut(col))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PdfCardElement {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub pdf_name: String,
    pub current_page: usize,
    pub total_pages: usize,
    pub pos: Pos2,
    pub size: Vec2,
}

impl PdfCardElement {
    pub fn aabb(&self) -> Rect {
        Rect::from_min_size(self.pos, self.size)
    }
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageElement {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub image_name: String,
    pub pos: Pos2,
    pub size: Vec2,
    pub opacity: f32,
    #[serde(default = "default_true")]
    pub perspective_locked: bool,
    #[serde(default)]
    pub corners: Option<[Pos2; 4]>,
}

impl ImageElement {
    pub fn aabb(&self) -> Rect {
        if let Some(corners) = self.corners {
            let mut min = self.pos + corners[0].to_vec2();
            let mut max = min;
            for pt in &corners[1..] {
                let w = self.pos + pt.to_vec2();
                min.x = min.x.min(w.x);
                min.y = min.y.min(w.y);
                max.x = max.x.max(w.x);
                max.y = max.y.max(w.y);
            }
            Rect::from_min_max(min, max)
        } else {
            Rect::from_min_size(self.pos, self.size)
        }
    }

    pub fn get_corners_local(&self) -> [Pos2; 4] {
        self.corners.unwrap_or([
            Pos2::new(0.0, 0.0),
            Pos2::new(self.size.x, 0.0),
            Pos2::new(self.size.x, self.size.y),
            Pos2::new(0.0, self.size.y),
        ])
    }

    pub fn get_corners_world(&self) -> [Pos2; 4] {
        let local = self.get_corners_local();
        [
            self.pos + local[0].to_vec2(),
            self.pos + local[1].to_vec2(),
            self.pos + local[2].to_vec2(),
            self.pos + local[3].to_vec2(),
        ]
    }

    pub fn reset_perspective(&mut self) {
        self.corners = None;
        self.perspective_locked = true;
    }

    pub fn is_valid_quad(pts: &[Pos2; 4]) -> bool {
        // Minimum distance between any two corners
        for i in 0..4 {
            for j in (i + 1)..4 {
                if (pts[i] - pts[j]).length() < 20.0 {
                    return false;
                }
            }
        }
        // Strict convexity: cross products of consecutive edges must all have same non-zero sign
        let cross = |a: Pos2, b: Pos2, c: Pos2| -> f32 {
            (b.x - a.x) * (c.y - b.y) - (b.y - a.y) * (c.x - b.x)
        };
        let c0 = cross(pts[0], pts[1], pts[2]);
        let c1 = cross(pts[1], pts[2], pts[3]);
        let c2 = cross(pts[2], pts[3], pts[0]);
        let c3 = cross(pts[3], pts[0], pts[1]);

        let min_area = 100.0_f32;
        (c0 > min_area && c1 > min_area && c2 > min_area && c3 > min_area)
            || (c0 < -min_area && c1 < -min_area && c2 < -min_area && c3 < -min_area)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DocAttachmentElement {
    pub id: Uuid,
    pub asset_id: Uuid,
    pub filename: String,
    pub extension: String,
    pub size_bytes: u64,
    pub pos: Pos2,
}

impl DocAttachmentElement {
    pub fn aabb(&self) -> Rect {
        Rect::from_min_size(self.pos, Vec2::new(180.0, 60.0))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BoardElement {
    Stroke(StrokeElement),
    Shape(ShapeElement),
    Component(ComponentElement),
    Wire(WireElement),
    Text(TextElement),
    Table(TableElement),
    PdfCard(PdfCardElement),
    Image(ImageElement),
    DocAttachment(DocAttachmentElement),
}

impl BoardElement {
    pub fn id(&self) -> Uuid {
        match self {
            BoardElement::Stroke(e) => e.id,
            BoardElement::Shape(e) => e.id,
            BoardElement::Component(e) => e.id,
            BoardElement::Wire(e) => e.id,
            BoardElement::Text(e) => e.id,
            BoardElement::Table(e) => e.id,
            BoardElement::PdfCard(e) => e.id,
            BoardElement::Image(e) => e.id,
            BoardElement::DocAttachment(e) => e.id,
        }
    }

    pub fn aabb(&self) -> Rect {
        match self {
            BoardElement::Stroke(e) => e.cached_aabb,
            BoardElement::Shape(e) => e.cached_aabb,
            BoardElement::Component(e) => e.aabb(),
            BoardElement::Wire(e) => e.aabb(),
            BoardElement::Text(e) => e.aabb(),
            BoardElement::Table(e) => e.aabb(),
            BoardElement::PdfCard(e) => e.aabb(),
            BoardElement::Image(e) => e.aabb(),
            BoardElement::DocAttachment(e) => e.aabb(),
        }
    }

    pub fn geometric_aabb(&self) -> Rect {
        match self {
            BoardElement::Shape(s) => {
                if s.vertices.is_empty() {
                    return Rect::NOTHING;
                }
                let mut min = s.vertices[0];
                let mut max = s.vertices[0];
                for v in s.vertices.iter().skip(1) {
                    min.x = min.x.min(v.x);
                    min.y = min.y.min(v.y);
                    max.x = max.x.max(v.x);
                    max.y = max.y.max(v.y);
                }
                if (max.x - min.x).abs() < 1e-4 {
                    max.x = min.x + 1.0;
                }
                if (max.y - min.y).abs() < 1e-4 {
                    max.y = min.y + 1.0;
                }
                Rect::from_min_max(min, max)
            }
            BoardElement::Stroke(s) => {
                if s.points.is_empty() {
                    return Rect::NOTHING;
                }
                let mut min = s.points[0].pos;
                let mut max = s.points[0].pos;
                for p in s.points.iter().skip(1) {
                    min.x = min.x.min(p.pos.x);
                    min.y = min.y.min(p.pos.y);
                    max.x = max.x.max(p.pos.x);
                    max.y = max.y.max(p.pos.y);
                }
                if (max.x - min.x).abs() < 1e-4 {
                    max.x = min.x + 1.0;
                }
                if (max.y - min.y).abs() < 1e-4 {
                    max.y = min.y + 1.0;
                }
                Rect::from_min_max(min, max)
            }
            BoardElement::Wire(w) => {
                let min_x = w.start.pos.x.min(w.end.pos.x);
                let min_y = w.start.pos.y.min(w.end.pos.y);
                let mut max_x = w.start.pos.x.max(w.end.pos.x);
                let mut max_y = w.start.pos.y.max(w.end.pos.y);
                if (max_x - min_x).abs() < 1e-4 {
                    max_x = min_x + 1.0;
                }
                if (max_y - min_y).abs() < 1e-4 {
                    max_y = min_y + 1.0;
                }
                Rect::from_min_max(Pos2::new(min_x, min_y), Pos2::new(max_x, max_y))
            }
            _ => self.aabb(),
        }
    }

    pub fn translate(&mut self, delta: Vec2) {
        match self {
            BoardElement::Stroke(e) => {
                for p in &mut e.points {
                    p.pos += delta;
                }
                e.cached_aabb = e.cached_aabb.translate(delta);
            }
            BoardElement::Shape(e) => {
                for v in &mut e.vertices {
                    *v += delta;
                }
                e.cached_aabb = e.cached_aabb.translate(delta);
            }
            BoardElement::Component(e) => {
                e.pos += delta;
            }
            BoardElement::Wire(e) => {
                // If endpoint is unattached or translated directly:
                if e.start.element_id.is_none() {
                    e.start.pos += delta;
                }
                if e.end.element_id.is_none() {
                    e.end.pos += delta;
                }
            }
            BoardElement::Text(e) => {
                e.pos += delta;
            }
            BoardElement::Table(e) => {
                e.pos += delta;
            }
            BoardElement::PdfCard(e) => {
                e.pos += delta;
            }
            BoardElement::Image(e) => {
                e.pos += delta;
            }
            BoardElement::DocAttachment(e) => {
                e.pos += delta;
            }
        }
    }

    pub fn assign_new_id(&mut self) -> Uuid {
        let new_id = Uuid::new_v4();
        match self {
            BoardElement::Stroke(e) => e.id = new_id,
            BoardElement::Shape(e) => e.id = new_id,
            BoardElement::Component(e) => e.id = new_id,
            BoardElement::Wire(e) => e.id = new_id,
            BoardElement::Text(e) => e.id = new_id,
            BoardElement::Table(e) => e.id = new_id,
            BoardElement::PdfCard(e) => e.id = new_id,
            BoardElement::Image(e) => e.id = new_id,
            BoardElement::DocAttachment(e) => e.id = new_id,
        }
        new_id
    }

    pub fn duplicate_with_offset(&self, delta: Vec2) -> Self {
        let mut dup = self.clone();
        dup.assign_new_id();
        dup.translate(delta);
        dup
    }

    pub fn contains_point(&self, point: Pos2, tolerance: f32) -> bool {
        let expanded = self.aabb().expand(tolerance);
        if !expanded.contains(point) {
            return false;
        }

        match self {
            BoardElement::Stroke(e) => {
                let tol_sq = (tolerance + e.width / 2.0).powi(2);
                for i in 0..e.points.len().saturating_sub(1) {
                    let p1 = e.points[i].pos;
                    let p2 = e.points[i + 1].pos;
                    if point_to_segment_distance_sq(point, p1, p2) <= tol_sq {
                        return true;
                    }
                }
                false
            }
            BoardElement::Shape(e) => {
                match e.kind {
                    ShapeKind::Line | ShapeKind::Arrow => {
                        if e.vertices.len() >= 2 {
                            let p1 = e.vertices[0];
                            let p2 = e.vertices[1];
                            let tol_sq = (tolerance + e.stroke_width / 2.0).powi(2);
                            return point_to_segment_distance_sq(point, p1, p2) <= tol_sq;
                        }
                        false
                    }
                    _ => expanded.contains(point),
                }
            }
            BoardElement::Wire(e) => {
                let tol_sq = (tolerance + e.width).powi(2);
                match e.routing {
                    RoutingMode::Orthogonal => {
                        let mid_x = (e.start.pos.x + e.end.pos.x) / 2.0;
                        let p1 = e.start.pos;
                        let p2 = Pos2::new(mid_x, e.start.pos.y);
                        let p3 = Pos2::new(mid_x, e.end.pos.y);
                        let p4 = e.end.pos;
                        point_to_segment_distance_sq(point, p1, p2) <= tol_sq
                            || point_to_segment_distance_sq(point, p2, p3) <= tol_sq
                            || point_to_segment_distance_sq(point, p3, p4) <= tol_sq
                    }
                    _ => point_to_segment_distance_sq(point, e.start.pos, e.end.pos) <= tol_sq,
                }
            }
            _ => expanded.contains(point),
        }
    }

    pub fn intersects_rect(&self, rect: Rect) -> bool {
        self.aabb().intersects(rect)
    }
}

pub fn point_to_segment_distance_sq(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let ab = b - a;
    let ap = p - a;
    let ab_len_sq = ab.length_sq();
    if ab_len_sq < 1e-6 {
        return ap.length_sq();
    }
    let t = (ap.dot(ab) / ab_len_sq).clamp(0.0, 1.0);
    let proj = a + ab * t;
    (p - proj).length_sq()
}
