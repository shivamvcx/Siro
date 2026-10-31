use super::camera::Camera;
use egui::{Color32, Painter, Pos2, Rect, Stroke};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GridStyle {
    Dots,
    Lines,
    None,
}

#[derive(Clone, Copy, Debug)]
pub struct GridSettings {
    pub style: GridStyle,
    pub base_spacing: f32,
    pub snap_to_grid: bool,
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            style: GridStyle::Lines,
            base_spacing: 24.0,
            snap_to_grid: true,
        }
    }
}

pub struct GridRenderer;

impl GridRenderer {
    pub fn snap(pos: Pos2, spacing: f32) -> Pos2 {
        if spacing <= 0.0 {
            return pos;
        }
        let snapped_x = (pos.x / spacing).round() * spacing;
        let snapped_y = (pos.y / spacing).round() * spacing;
        Pos2::new(snapped_x, snapped_y)
    }

    pub fn dynamic_spacing(zoom: f32, base_spacing: f32) -> f32 {
        let mut spacing = base_spacing;
        while spacing * zoom < 16.0 {
            spacing *= 2.0;
        }
        while spacing * zoom > 80.0 {
            spacing /= 2.0;
        }
        spacing
    }

    pub fn render(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        settings: &GridSettings,
        dark_mode: bool,
    ) {
        if settings.style == GridStyle::None {
            return;
        }

        let spacing = Self::dynamic_spacing(camera.zoom, settings.base_spacing);
        let world_rect = camera.visible_world_rect(viewport_rect);

        let start_x = (world_rect.min.x / spacing).floor() * spacing;
        let end_x = (world_rect.max.x / spacing).ceil() * spacing;
        let start_y = (world_rect.min.y / spacing).floor() * spacing;
        let end_y = (world_rect.max.y / spacing).ceil() * spacing;

        let alpha = ((spacing * camera.zoom - 12.0) / 35.0).clamp(0.25, 0.85);

        match settings.style {
            GridStyle::Dots => {
                let dot_color = if dark_mode {
                    Color32::from_rgba_unmultiplied(85, 95, 120, (alpha * 140.0) as u8)
                } else {
                    Color32::from_rgba_unmultiplied(170, 180, 200, (alpha * 160.0) as u8)
                };
                let dot_radius = (1.2 * camera.zoom).clamp(1.0, 2.0);
                let mut x = start_x;
                while x <= end_x {
                    let mut y = start_y;
                    while y <= end_y {
                        let screen_pos = camera.world_to_screen(Pos2::new(x, y), viewport_rect);
                        painter.circle_filled(screen_pos, dot_radius, dot_color);
                        y += spacing;
                    }
                    x += spacing;
                }
            }
            GridStyle::Lines => {
                // Miro subtle grid line colors: #e6e8ec in light mode, #252833 in dark mode
                let line_color = if dark_mode {
                    Color32::from_rgba_unmultiplied(38, 42, 54, (alpha * 220.0) as u8)
                } else {
                    Color32::from_rgba_unmultiplied(228, 231, 236, (alpha * 255.0) as u8)
                };
                let line_stroke = Stroke::new(1.0_f32, line_color);

                let mut x = start_x;
                while x <= end_x {
                    let top = camera.world_to_screen(Pos2::new(x, world_rect.min.y), viewport_rect);
                    let bottom = camera.world_to_screen(Pos2::new(x, world_rect.max.y), viewport_rect);
                    painter.line_segment([top, bottom], line_stroke);
                    x += spacing;
                }
                let mut y = start_y;
                while y <= end_y {
                    let left = camera.world_to_screen(Pos2::new(world_rect.min.x, y), viewport_rect);
                    let right = camera.world_to_screen(Pos2::new(world_rect.max.x, y), viewport_rect);
                    painter.line_segment([left, right], line_stroke);
                    y += spacing;
                }
            }
            GridStyle::None => {}
        }
    }
}
