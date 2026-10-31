use super::camera::Camera;
use crate::model::electronics::ComponentKind;
use crate::model::element::*;
use egui::{
    epaint::PathShape, epaint::StrokeKind, Color32, CornerRadius, FontFamily, FontId, Painter, Pos2, Rect, Stroke, Vec2,
};

pub struct CanvasRenderer;

impl CanvasRenderer {
    pub fn render_element(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        elem: &BoardElement,
        is_selected: bool,
        is_single_selection: bool,
        is_vertex_mode: bool,
        hovered_terminal: Option<(uuid::Uuid, usize)>,
        dark_mode: bool,
        image_textures: &std::collections::HashMap<uuid::Uuid, egui::TextureHandle>,
        pdf_textures: &std::collections::HashMap<(uuid::Uuid, usize), egui::TextureHandle>,
        active_cell: Option<(uuid::Uuid, usize, usize)>,
    ) {
        match elem {
            BoardElement::Stroke(stroke) => {
                Self::render_stroke(painter, camera, viewport_rect, stroke);
            }
            BoardElement::Shape(shape) => {
                Self::render_shape(painter, camera, viewport_rect, shape, is_selected, is_vertex_mode);
            }
            BoardElement::Component(comp) => {
                Self::render_component(painter, camera, viewport_rect, comp, is_selected, hovered_terminal, dark_mode);
            }
            BoardElement::Wire(wire) => {
                Self::render_wire(painter, camera, viewport_rect, wire, is_selected);
            }
            BoardElement::Text(text) => {
                Self::render_text(painter, camera, viewport_rect, text, is_selected);
            }
            BoardElement::Table(table) => {
                Self::render_table(painter, camera, viewport_rect, table, is_selected, dark_mode, active_cell);
            }
            BoardElement::PdfCard(pdf) => {
                Self::render_pdf_card(painter, camera, viewport_rect, pdf, is_selected, dark_mode, pdf_textures.get(&(pdf.asset_id, pdf.current_page)));
            }
            BoardElement::Image(img) => {
                Self::render_image(painter, camera, viewport_rect, img, is_selected, image_textures.get(&img.asset_id));
            }
            BoardElement::DocAttachment(doc) => {
                Self::render_doc_attachment(painter, camera, viewport_rect, doc, is_selected, dark_mode);
            }
        }

        let skip_selection_box = match elem {
            BoardElement::Image(img) => !img.perspective_locked,
            _ => false,
        };

        if is_selected && !is_vertex_mode && !skip_selection_box {
            if is_single_selection {
                Self::render_selection_box(painter, camera, viewport_rect, elem.geometric_aabb());
            } else {
                Self::render_item_selection_outline(painter, camera, viewport_rect, elem.geometric_aabb());
            }
        }
    }

    fn render_stroke(painter: &Painter, camera: &Camera, viewport_rect: Rect, stroke: &StrokeElement) {
        if stroke.points.len() < 2 {
            return;
        }

        let base_width = stroke.width * camera.zoom;
        let color = if stroke.is_highlighter {
            let [r, g, b, _] = stroke.color.to_array();
            Color32::from_rgba_unmultiplied(r, g, b, 60)
        } else {
            stroke.color
        };

        let mut screen_points: Vec<Pos2> = Vec::with_capacity(stroke.points.len());
        for p in &stroke.points {
            screen_points.push(camera.world_to_screen(p.pos, viewport_rect));
        }

        if stroke.is_highlighter {
            let stroke_style = Stroke::new((base_width * 2.2).max(8.0), color);
            painter.add(PathShape::line(screen_points, stroke_style));
        } else {
            // Smooth tapered stroke rendering
            for i in 0..stroke.points.len() - 1 {
                let p1 = screen_points[i];
                let p2 = screen_points[i + 1];
                let vel = stroke.points[i + 1].velocity;
                let taper = (1.0 / (1.0 + vel * 0.04)).clamp(0.5, 1.8);
                let w = (base_width * taper).max(1.0);
                painter.line_segment([p1, p2], Stroke::new(w, color));
            }
        }
    }

    fn render_shape(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        shape: &ShapeElement,
        is_selected: bool,
        is_vertex_mode: bool,
    ) {
        let stroke_w = (shape.stroke_width * camera.zoom).max(1.0);
        let stroke = Stroke::new(stroke_w, shape.stroke_color);
        let screen_verts: Vec<Pos2> = shape
            .vertices
            .iter()
            .map(|v| camera.world_to_screen(*v, viewport_rect))
            .collect();

        if screen_verts.is_empty() {
            return;
        }

        match shape.kind {
            ShapeKind::Rectangle => {
                if screen_verts.len() >= 4 {
                    painter.add(PathShape::convex_polygon(screen_verts.clone(), shape.fill_color, stroke));
                } else if screen_verts.len() >= 2 {
                    let rect = Rect::from_two_pos(screen_verts[0], screen_verts[1]);
                    painter.rect_filled(rect, CornerRadius::ZERO, shape.fill_color);
                    painter.rect_stroke(rect, CornerRadius::ZERO, stroke, StrokeKind::Middle);
                }
            }
            ShapeKind::RoundedRectangle => {
                if screen_verts.len() >= 2 {
                    let rect = Rect::from_two_pos(screen_verts[0], screen_verts[1]);
                    let r = (10.0 * camera.zoom).clamp(2.0, 32.0) as u8;
                    let rounding = CornerRadius::same(r);
                    painter.rect_filled(rect, rounding, shape.fill_color);
                    painter.rect_stroke(rect, rounding, stroke, StrokeKind::Middle);
                }
            }
            ShapeKind::Ellipse => {
                if screen_verts.len() >= 2 {
                    let rect = Rect::from_two_pos(screen_verts[0], screen_verts[1]);
                    let center = rect.center();
                    let radius = (rect.width().abs().min(rect.height().abs())) / 2.0;
                    painter.circle_filled(center, radius, shape.fill_color);
                    painter.circle_stroke(center, radius, stroke);
                }
            }
            ShapeKind::Diamond => {
                if screen_verts.len() >= 4 {
                    painter.add(PathShape::convex_polygon(screen_verts.clone(), shape.fill_color, stroke));
                } else if screen_verts.len() >= 2 {
                    let rect = Rect::from_two_pos(screen_verts[0], screen_verts[1]);
                    let pts = vec![
                        Pos2::new(rect.center().x, rect.min.y),
                        Pos2::new(rect.max.x, rect.center().y),
                        Pos2::new(rect.center().x, rect.max.y),
                        Pos2::new(rect.min.x, rect.center().y),
                    ];
                    painter.add(PathShape::convex_polygon(pts, shape.fill_color, stroke));
                }
            }
            ShapeKind::Triangle => {
                if screen_verts.len() >= 3 {
                    painter.add(PathShape::convex_polygon(screen_verts.clone(), shape.fill_color, stroke));
                } else if screen_verts.len() >= 2 {
                    let rect = Rect::from_two_pos(screen_verts[0], screen_verts[1]);
                    let pts = vec![
                        Pos2::new(rect.center().x, rect.min.y),
                        Pos2::new(rect.max.x, rect.max.y),
                        Pos2::new(rect.min.x, rect.max.y),
                    ];
                    painter.add(PathShape::convex_polygon(pts, shape.fill_color, stroke));
                }
            }
            ShapeKind::Line => {
                if screen_verts.len() >= 2 {
                    painter.line_segment([screen_verts[0], screen_verts[1]], stroke);
                }
            }
            ShapeKind::Arrow => {
                if screen_verts.len() >= 2 {
                    let p1 = screen_verts[0];
                    let p2 = screen_verts[1];
                    painter.line_segment([p1, p2], stroke);
                    Self::draw_arrow_head(painter, p1, p2, shape.stroke_color, stroke_w);
                }
            }
        }

        // Render draggable vertex handles in vertex edit mode
        if is_selected && is_vertex_mode {
            for &vp in &screen_verts {
                painter.circle_filled(vp, 5.0, Color32::from_rgb(0, 210, 255));
                painter.circle_stroke(vp, 5.0, Stroke::new(1.5_f32, Color32::WHITE));
            }
        }
    }

    fn draw_arrow_head(painter: &Painter, from: Pos2, to: Pos2, color: Color32, stroke_w: f32) {
        let dir = to - from;
        let len = dir.length();
        if len < 4.0 {
            return;
        }
        let norm = dir / len;
        let perp = Vec2::new(-norm.y, norm.x);
        let arrow_len = (14.0 + stroke_w * 2.0).min(len * 0.5);
        let arrow_w = arrow_len * 0.45;

        let base = to - norm * arrow_len;
        let left = base + perp * arrow_w;
        let right = base - perp * arrow_w;

        painter.add(PathShape::convex_polygon(
            vec![to, left, right],
            color,
            Stroke::new(1.0_f32, color),
        ));
    }

    fn render_component(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        comp: &ComponentElement,
        _is_selected: bool,
        hovered_terminal: Option<(uuid::Uuid, usize)>,
        dark_mode: bool,
    ) {
        let screen_min = camera.world_to_screen(comp.pos, viewport_rect);
        let (comp_w, comp_h) = if comp.rotation_quarters % 2 == 1 {
            (comp.height, comp.width)
        } else {
            (comp.width, comp.height)
        };
        let screen_size = camera.world_vec_to_screen(Vec2::new(comp_w, comp_h));
        let screen_rect = Rect::from_min_size(screen_min, screen_size);

        let is_ic_or_board = matches!(
            comp.kind,
            ComponentKind::RaspberryPiPico
                | ComponentKind::Esp32Wroom
                | ComponentKind::ArduinoNano
                | ComponentKind::GenericIC { .. }
                | ComponentKind::BreadboardSection
        );

        if is_ic_or_board {
            Self::render_maker_board_or_ic(painter, camera, screen_rect, comp, dark_mode);
        } else {
            Self::render_schematic_symbol(painter, camera, screen_rect, comp, dark_mode);
        }

        // Render terminals and magnetic snap rings
        for (idx, _pin) in comp.terminals.iter().enumerate() {
            if let Some(w_pos) = comp.terminal_world_pos(idx) {
                let s_pos = camera.world_to_screen(w_pos, viewport_rect);
                let is_hovered = hovered_terminal == Some((comp.id, idx));

                let pin_radius = if is_hovered { 6.0 } else { 3.5 };
                let pin_color = if is_hovered {
                    Color32::from_rgb(0, 240, 255)
                } else if dark_mode {
                    Color32::from_rgb(220, 180, 50)
                } else {
                    Color32::from_rgb(180, 130, 20)
                };

                // Pin circle
                painter.circle_filled(s_pos, pin_radius, pin_color);
                painter.circle_stroke(s_pos, pin_radius, Stroke::new(1.0_f32, Color32::BLACK));

                // Pulsing magnetic glow if hovered
                if is_hovered {
                    painter.circle_stroke(
                        s_pos,
                        pin_radius + 4.0,
                        Stroke::new(2.0_f32, Color32::from_rgba_unmultiplied(0, 220, 255, 180)),
                    );
                }
            }
        }
    }

    fn render_maker_board_or_ic(
        painter: &Painter,
        camera: &Camera,
        rect: Rect,
        comp: &ComponentElement,
        dark_mode: bool,
    ) {
        match comp.kind {
            ComponentKind::RaspberryPiPico => {
                // PCB green body
                let pcb_color = Color32::from_rgb(22, 85, 48);
                painter.rect_filled(rect, CornerRadius::same(6), pcb_color);
                painter.rect_stroke(rect, CornerRadius::same(6), Stroke::new(1.5_f32, Color32::from_rgb(12, 55, 30)), StrokeKind::Middle);

                // RP2040 chip in center
                let chip_rect = Rect::from_center_size(rect.center(), Vec2::new(rect.width() * 0.45, rect.height() * 0.22));
                painter.rect_filled(chip_rect, CornerRadius::same(2), Color32::from_rgb(40, 42, 45));
                painter.text(
                    chip_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "RP2040",
                    FontId::new((10.0 * camera.zoom).clamp(8.0, 14.0), FontFamily::Monospace),
                    Color32::from_rgb(220, 220, 220),
                );

                // Title label
                painter.text(
                    Pos2::new(rect.center().x, rect.min.y + 12.0 * camera.zoom),
                    egui::Align2::CENTER_CENTER,
                    &comp.designator,
                    FontId::new((11.0 * camera.zoom).clamp(9.0, 16.0), FontFamily::Proportional),
                    Color32::WHITE,
                );
            }
            ComponentKind::Esp32Wroom => {
                // Sleek matte black PCB
                let pcb_color = Color32::from_rgb(30, 32, 38);
                painter.rect_filled(rect, CornerRadius::same(4), pcb_color);
                painter.rect_stroke(rect, CornerRadius::same(4), Stroke::new(1.5_f32, Color32::from_rgb(15, 16, 20)), StrokeKind::Middle);

                // Metal RF Shield
                let shield_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + 8.0 * camera.zoom, rect.min.y + 35.0 * camera.zoom),
                    Vec2::new(rect.width() - 16.0 * camera.zoom, rect.height() * 0.48),
                );
                painter.rect_filled(shield_rect, CornerRadius::same(3), Color32::from_rgb(175, 180, 190));
                painter.rect_stroke(shield_rect, CornerRadius::same(3), Stroke::new(1.0_f32, Color32::from_rgb(120, 125, 135)), StrokeKind::Middle);

                painter.text(
                    shield_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "ESP-WROOM-32",
                    FontId::new((9.0 * camera.zoom).clamp(7.0, 13.0), FontFamily::Monospace),
                    Color32::from_rgb(40, 42, 45),
                );

                // Antenna serpentine pattern area at top
                let ant_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + 10.0 * camera.zoom, rect.min.y + 6.0 * camera.zoom),
                    Vec2::new(rect.width() - 20.0 * camera.zoom, 22.0 * camera.zoom),
                );
                painter.rect_filled(ant_rect, CornerRadius::ZERO, Color32::from_rgb(180, 80, 20));

                painter.text(
                    Pos2::new(rect.center().x, rect.max.y - 12.0 * camera.zoom),
                    egui::Align2::CENTER_CENTER,
                    &comp.designator,
                    FontId::new((10.0 * camera.zoom).clamp(8.0, 14.0), FontFamily::Proportional),
                    Color32::WHITE,
                );
            }
            ComponentKind::ArduinoNano => {
                // Arduino blue PCB
                let pcb_color = Color32::from_rgb(0, 115, 140);
                painter.rect_filled(rect, CornerRadius::same(5), pcb_color);
                painter.rect_stroke(rect, CornerRadius::same(5), Stroke::new(1.5_f32, Color32::from_rgb(0, 80, 100)), StrokeKind::Middle);

                // ATmega IC
                let chip_rect = Rect::from_center_size(rect.center(), Vec2::new(rect.width() * 0.4, rect.height() * 0.2));
                painter.rect_filled(chip_rect, CornerRadius::same(2), Color32::from_rgb(30, 30, 30));

                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "NANO",
                    FontId::new((11.0 * camera.zoom).clamp(8.0, 15.0), FontFamily::Monospace),
                    Color32::WHITE,
                );
            }
            ComponentKind::GenericIC { .. } => {
                // Black DIP IC package
                painter.rect_filled(rect, CornerRadius::same(2), Color32::from_rgb(35, 36, 40));
                painter.rect_stroke(rect, CornerRadius::same(2), Stroke::new(1.5_f32, Color32::from_rgb(80, 85, 95)), StrokeKind::Middle);

                // Pin 1 notch semi-circle
                let notch_center = Pos2::new(rect.center().x, rect.min.y);
                painter.circle_filled(notch_center, 6.0 * camera.zoom, Color32::from_rgb(60, 65, 75));

                painter.text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    &comp.value,
                    FontId::new((10.0 * camera.zoom).clamp(8.0, 14.0), FontFamily::Monospace),
                    Color32::from_rgb(220, 225, 230),
                );
            }
            ComponentKind::BreadboardSection => {
                // Off-white breadboard
                let bb_color = if dark_mode { Color32::from_rgb(45, 48, 55) } else { Color32::from_rgb(245, 245, 240) };
                painter.rect_filled(rect, CornerRadius::same(6), bb_color);
                painter.rect_stroke(rect, CornerRadius::same(6), Stroke::new(1.5_f32, Color32::from_rgb(140, 145, 155)), StrokeKind::Middle);

                // Red (+) and Blue (-) power rail stripe lines
                let left_red = rect.min.x + 15.0 * camera.zoom;
                let left_blue = rect.min.x + 30.0 * camera.zoom;
                painter.line_segment([Pos2::new(left_red, rect.min.y + 10.0), Pos2::new(left_red, rect.max.y - 10.0)], Stroke::new(2.0_f32, Color32::from_rgb(220, 50, 50)));
                painter.line_segment([Pos2::new(left_blue, rect.min.y + 10.0), Pos2::new(left_blue, rect.max.y - 10.0)], Stroke::new(2.0_f32, Color32::from_rgb(50, 100, 220)));

                painter.text(
                    Pos2::new(rect.center().x, rect.min.y + 15.0 * camera.zoom),
                    egui::Align2::CENTER_CENTER,
                    "Breadboard",
                    FontId::new((11.0 * camera.zoom).clamp(8.0, 14.0), FontFamily::Proportional),
                    if dark_mode { Color32::WHITE } else { Color32::BLACK },
                );
            }
            _ => {}
        }
    }

    fn render_schematic_symbol(
        painter: &Painter,
        camera: &Camera,
        rect: Rect,
        comp: &ComponentElement,
        dark_mode: bool,
    ) {
        let sym_color = if dark_mode { Color32::from_rgb(230, 235, 245) } else { Color32::from_rgb(25, 28, 35) };
        let stroke = Stroke::new((2.0 * camera.zoom).clamp(1.5, 3.5), sym_color);

        let c = rect.center();
        let min = rect.min;
        let max = rect.max;

        match comp.kind {
            ComponentKind::Resistor => {
                // Zigzag wire symbol
                let y = c.y;
                let w = rect.width();
                let seg = w / 8.0;
                let h = 10.0 * camera.zoom;
                let pts = vec![
                    Pos2::new(min.x, y),
                    Pos2::new(min.x + seg * 2.0, y),
                    Pos2::new(min.x + seg * 2.5, y - h),
                    Pos2::new(min.x + seg * 3.5, y + h),
                    Pos2::new(min.x + seg * 4.5, y - h),
                    Pos2::new(min.x + seg * 5.5, y + h),
                    Pos2::new(min.x + seg * 6.0, y),
                    Pos2::new(max.x, y),
                ];
                for i in 0..pts.len() - 1 {
                    painter.line_segment([pts[i], pts[i + 1]], stroke);
                }
            }
            ComponentKind::Capacitor => {
                let y = c.y;
                let plate_w = 12.0 * camera.zoom;
                let plate_h = 16.0 * camera.zoom;
                // Leads
                painter.line_segment([Pos2::new(min.x, y), Pos2::new(c.x - plate_w / 2.0, y)], stroke);
                painter.line_segment([Pos2::new(c.x + plate_w / 2.0, y), Pos2::new(max.x, y)], stroke);
                // Plates
                painter.line_segment([Pos2::new(c.x - plate_w / 2.0, y - plate_h), Pos2::new(c.x - plate_w / 2.0, y + plate_h)], stroke);
                painter.line_segment([Pos2::new(c.x + plate_w / 2.0, y - plate_h), Pos2::new(c.x + plate_w / 2.0, y + plate_h)], stroke);
            }
            ComponentKind::Inductor => {
                let y = c.y;
                let r = 8.0 * camera.zoom;
                painter.line_segment([Pos2::new(min.x, y), Pos2::new(c.x - r * 2.0, y)], stroke);
                painter.line_segment([Pos2::new(c.x + r * 2.0, y), Pos2::new(max.x, y)], stroke);
                // 3 loops
                for i in -1..=1 {
                    let cx = c.x + (i as f32) * (r * 1.5);
                    painter.circle_stroke(Pos2::new(cx, y), r, stroke);
                }
            }
            ComponentKind::Diode | ComponentKind::Led => {
                let y = c.y;
                let d_size = 12.0 * camera.zoom;
                painter.line_segment([Pos2::new(min.x, y), Pos2::new(c.x - d_size, y)], stroke);
                painter.line_segment([Pos2::new(c.x + d_size, y), Pos2::new(max.x, y)], stroke);
                // Triangle & bar
                let triangle = vec![
                    Pos2::new(c.x - d_size, y - d_size),
                    Pos2::new(c.x + d_size, y),
                    Pos2::new(c.x - d_size, y + d_size),
                ];
                painter.add(PathShape::convex_polygon(triangle, sym_color, stroke));
                painter.line_segment([Pos2::new(c.x + d_size, y - d_size), Pos2::new(c.x + d_size, y + d_size)], stroke);

                // LED emission arrows
                if comp.kind == ComponentKind::Led {
                    let a1_start = Pos2::new(c.x + 2.0, y - d_size - 2.0);
                    let a1_end = Pos2::new(c.x + 10.0 * camera.zoom, y - d_size - 10.0 * camera.zoom);
                    painter.line_segment([a1_start, a1_end], Stroke::new(1.5_f32, Color32::from_rgb(255, 200, 30)));
                }
            }
            ComponentKind::Battery => {
                let x = c.x;
                let plate_gap = 6.0 * camera.zoom;
                // Leads
                painter.line_segment([Pos2::new(x, min.y), Pos2::new(x, c.y - plate_gap)], stroke);
                painter.line_segment([Pos2::new(x, c.y + plate_gap), Pos2::new(x, max.y)], stroke);
                // Plates: top long (+), bottom short thick (-)
                painter.line_segment([Pos2::new(c.x - 18.0 * camera.zoom, c.y - plate_gap), Pos2::new(c.x + 18.0 * camera.zoom, c.y - plate_gap)], stroke);
                painter.line_segment([Pos2::new(c.x - 9.0 * camera.zoom, c.y + plate_gap), Pos2::new(c.x + 9.0 * camera.zoom, c.y + plate_gap)], Stroke::new(stroke.width * 2.0, sym_color));
            }
            ComponentKind::Ground => {
                let x = c.x;
                let top_y = min.y;
                let g_y = c.y;
                painter.line_segment([Pos2::new(x, top_y), Pos2::new(x, g_y)], stroke);
                // 3 decreasing lines
                painter.line_segment([Pos2::new(x - 14.0 * camera.zoom, g_y), Pos2::new(x + 14.0 * camera.zoom, g_y)], stroke);
                painter.line_segment([Pos2::new(x - 9.0 * camera.zoom, g_y + 4.0 * camera.zoom), Pos2::new(x + 9.0 * camera.zoom, g_y + 4.0 * camera.zoom)], stroke);
                painter.line_segment([Pos2::new(x - 4.0 * camera.zoom, g_y + 8.0 * camera.zoom), Pos2::new(x + 4.0 * camera.zoom, g_y + 8.0 * camera.zoom)], stroke);
            }
            ComponentKind::Switch => {
                let y = c.y;
                let gap = 12.0 * camera.zoom;
                painter.line_segment([Pos2::new(min.x, y), Pos2::new(c.x - gap, y)], stroke);
                painter.line_segment([Pos2::new(c.x + gap, y), Pos2::new(max.x, y)], stroke);
                // Switch hinge contacts
                painter.circle_filled(Pos2::new(c.x - gap, y), 3.0, sym_color);
                painter.circle_filled(Pos2::new(c.x + gap, y), 3.0, sym_color);
                // Open contact arm
                painter.line_segment([Pos2::new(c.x - gap, y), Pos2::new(c.x + gap * 0.8, y - gap * 0.9)], stroke);
            }
            ComponentKind::NpnTransistor | ComponentKind::PnpTransistor | ComponentKind::Mosfet => {
                // Circle body
                painter.circle_stroke(c, 24.0 * camera.zoom, stroke);
                // Base / Gate bar
                painter.line_segment([Pos2::new(c.x - 8.0 * camera.zoom, c.y - 14.0 * camera.zoom), Pos2::new(c.x - 8.0 * camera.zoom, c.y + 14.0 * camera.zoom)], Stroke::new(stroke.width * 1.5, sym_color));
                // Base lead
                painter.line_segment([Pos2::new(min.x, c.y), Pos2::new(c.x - 8.0 * camera.zoom, c.y)], stroke);
                // Collector & Emitter leads
                painter.line_segment([Pos2::new(c.x - 8.0 * camera.zoom, c.y - 7.0 * camera.zoom), Pos2::new(max.x, min.y + 15.0)], stroke);
                painter.line_segment([Pos2::new(c.x - 8.0 * camera.zoom, c.y + 7.0 * camera.zoom), Pos2::new(max.x, max.y - 15.0)], stroke);
            }
            _ => {}
        }

        // Draw designator & value labels
        let label_text = format!("{} ({})", comp.designator, comp.value);
        let font_size = (11.0 * camera.zoom).clamp(9.0, 15.0);
        painter.text(
            Pos2::new(c.x, min.y - 6.0 * camera.zoom),
            egui::Align2::CENTER_BOTTOM,
            label_text,
            FontId::new(font_size, FontFamily::Proportional),
            sym_color,
        );
    }

    fn render_wire(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        wire: &WireElement,
        is_selected: bool,
    ) {
        let s1 = camera.world_to_screen(wire.start.pos, viewport_rect);
        let s2 = camera.world_to_screen(wire.end.pos, viewport_rect);

        let color = wire.color.to_color32();
        let stroke_w = (wire.width * camera.zoom).clamp(1.5, 6.0);
        let stroke = Stroke::new(stroke_w, color);

        match wire.routing {
            RoutingMode::Orthogonal => {
                // Bounded 3-segment Manhattan routing
                let mid_x = (s1.x + s2.x) / 2.0;
                let p1 = s1;
                let p2 = Pos2::new(mid_x, s1.y);
                let p3 = Pos2::new(mid_x, s2.y);
                let p4 = s2;

                painter.line_segment([p1, p2], stroke);
                painter.line_segment([p2, p3], stroke);
                painter.line_segment([p3, p4], stroke);
            }
            RoutingMode::Straight => {
                painter.line_segment([s1, s2], stroke);
            }
            RoutingMode::Curved => {
                // Curved bezier wire
                let ctrl1 = Pos2::new((s1.x + s2.x) / 2.0, s1.y);
                let ctrl2 = Pos2::new((s1.x + s2.x) / 2.0, s2.y);
                let steps = 16;
                let mut prev = s1;
                for i in 1..=steps {
                    let t = i as f32 / steps as f32;
                    let u = 1.0 - t;
                    let curr = Pos2::new(
                        u * u * u * s1.x + 3.0 * u * u * t * ctrl1.x + 3.0 * u * t * t * ctrl2.x + t * t * t * s2.x,
                        u * u * u * s1.y + 3.0 * u * u * t * ctrl1.y + 3.0 * u * t * t * ctrl2.y + t * t * t * s2.y,
                    );
                    painter.line_segment([prev, curr], stroke);
                    prev = curr;
                }
            }
        }

        // Terminal solder dots at start and end
        painter.circle_filled(s1, stroke_w + 1.5, color);
        painter.circle_filled(s2, stroke_w + 1.5, color);

        if is_selected {
            painter.circle_stroke(s1, stroke_w + 4.0, Stroke::new(1.5_f32, Color32::from_rgb(0, 210, 255)));
            painter.circle_stroke(s2, stroke_w + 4.0, Stroke::new(1.5_f32, Color32::from_rgb(0, 210, 255)));
        }
    }

    fn render_text(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        text: &TextElement,
        _is_selected: bool,
    ) {
        let s_pos = camera.world_to_screen(text.pos, viewport_rect);
        let font_size = (text.font_size * camera.zoom).clamp(8.0, 48.0);
        let font_id = FontId::new(font_size, FontFamily::Proportional);

        if text.is_sticky {
            let s_width = text.width * camera.zoom;
            let sticky_rect = Rect::from_min_size(s_pos, Vec2::new(s_width, s_width));
            let r = (8.0 * camera.zoom).clamp(4.0, 12.0) as u8;
            let rounding = CornerRadius::same(r);

            // Miro multi-layer soft drop shadow
            painter.rect_filled(
                sticky_rect.translate(Vec2::new(2.0, 5.0)),
                rounding,
                Color32::from_rgba_unmultiplied(0, 0, 0, 22),
            );
            painter.rect_filled(
                sticky_rect.translate(Vec2::new(1.0, 2.0)),
                rounding,
                Color32::from_rgba_unmultiplied(0, 0, 0, 18),
            );

            // Note paper
            painter.rect_filled(sticky_rect, rounding, text.sticky_color);
            painter.rect_stroke(
                sticky_rect,
                rounding,
                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 16)),
                StrokeKind::Inside,
            );

            // Text content inside sticky with generous padding and word wrapping
            let pad = 12.0 * camera.zoom;
            let text_rect = sticky_rect.shrink(pad);
            let text_color = if text.sticky_color.r() as u16 + text.sticky_color.g() as u16 + text.sticky_color.b() as u16 > 400 {
                Color32::from_rgb(24, 25, 30)
            } else {
                Color32::WHITE
            };

            let wrap_width = text_rect.width().max(40.0);
            let mut job = egui::text::LayoutJob::simple(
                text.text.clone(),
                font_id.clone(),
                text_color,
                wrap_width,
            );
            job.wrap.max_width = wrap_width;
            let galley = painter.fonts(|f| f.layout_job(job));
            painter.galley(text_rect.min, galley, text_color);
        } else {
            let wrap_width = (text.width * camera.zoom).max(40.0);
            let mut job = egui::text::LayoutJob::simple(
                text.text.clone(),
                font_id.clone(),
                text.color,
                wrap_width,
            );
            job.wrap.max_width = wrap_width;
            let galley = painter.fonts(|f| f.layout_job(job));
            let text_rect = Rect::from_min_size(s_pos, galley.size());

            if text.bg_enabled {
                let pad = Vec2::new(text.bg_pad_x * camera.zoom, text.bg_pad_y * camera.zoom);
                let base_rect = if text.bg_match_width {
                    Rect::from_min_size(s_pos, Vec2::new(text.width * camera.zoom, galley.size().y))
                } else {
                    text_rect
                };
                let bg_rect = base_rect.expand2(pad);
                let alpha = (text.bg_opacity.clamp(0.0, 1.0) * 255.0) as u8;
                let [r, g, b, _] = text.bg_color.to_array();
                let actual_bg = Color32::from_rgba_unmultiplied(r, g, b, alpha);
                let rounding = CornerRadius::same((text.bg_corner_radius * camera.zoom).clamp(0.0, 32.0) as u8);
                painter.rect_filled(bg_rect, rounding, actual_bg);
            }

            painter.galley(s_pos, galley, text.color);
        }
    }

    fn render_table(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        table: &TableElement,
        _is_selected: bool,
        dark_mode: bool,
        active_cell: Option<(uuid::Uuid, usize, usize)>,
    ) {
        let s_min = camera.world_to_screen(table.pos, viewport_rect);
        let s_total_w = table.total_width() * camera.zoom;
        let s_total_h = table.total_height() * camera.zoom;
        let table_rect = Rect::from_min_size(s_min, Vec2::new(s_total_w, s_total_h));

        let bg_color = if dark_mode { Color32::from_rgb(28, 30, 36) } else { Color32::WHITE };
        let border_color = if dark_mode { Color32::from_rgb(70, 75, 90) } else { Color32::from_rgb(200, 205, 215) };
        let header_bg = if dark_mode { Color32::from_rgb(40, 45, 55) } else { Color32::from_rgb(235, 240, 250) };
        let text_color = if dark_mode { Color32::from_rgb(230, 235, 245) } else { Color32::from_rgb(20, 22, 28) };

        painter.rect_filled(table_rect, CornerRadius::same(3), bg_color);
        painter.rect_stroke(table_rect, CornerRadius::same(3), Stroke::new(1.5_f32, border_color), StrokeKind::Middle);

        let row_h = table.row_height() * camera.zoom;
        let font_size = (13.0 * camera.zoom).clamp(9.0, 22.0);
        let font_id = FontId::new(font_size, FontFamily::Proportional);

        // Header background
        let header_rect = Rect::from_min_size(s_min, Vec2::new(s_total_w, row_h));
        painter.rect_filled(header_rect, CornerRadius::same(2), header_bg);

        // Grid lines & cell text
        let mut curr_y = s_min.y;
        for (r_idx, row) in table.cells.iter().enumerate() {
            let mut curr_x = s_min.x;
            for (c_idx, cell_text) in row.iter().enumerate() {
                let col_w = table.col_widths.get(c_idx).copied().unwrap_or(100.0) * camera.zoom;
                let cell_rect = Rect::from_min_size(Pos2::new(curr_x, curr_y), Vec2::new(col_w, row_h));

                // Active cell highlight
                if active_cell == Some((table.id, r_idx, c_idx)) {
                    painter.rect_stroke(
                        cell_rect,
                        CornerRadius::ZERO,
                        Stroke::new(2.0_f32, Color32::from_rgb(0, 220, 255)),
                        StrokeKind::Inside,
                    );
                }

                // Right border
                painter.line_segment(
                    [Pos2::new(curr_x + col_w, curr_y), Pos2::new(curr_x + col_w, curr_y + row_h)],
                    Stroke::new(1.0_f32, border_color),
                );

                // Text
                let is_header = r_idx == 0;
                let c_text_color = if is_header {
                    if dark_mode { Color32::from_rgb(0, 220, 255) } else { Color32::from_rgb(0, 100, 200) }
                } else if let Some(row_c) = table.get_row_color(r_idx) {
                    row_c
                } else {
                    text_color
                };

                let text_pos = Pos2::new(curr_x + 6.0 * camera.zoom, curr_y + row_h / 2.0);
                if let Some(spans) = table.get_cell_spans(r_idx, c_idx) {
                    if !spans.is_empty() {
                        let mut job = egui::text::LayoutJob::default();
                        let chars: Vec<char> = cell_text.chars().collect();
                        let mut curr_idx = 0;
                        for span in spans {
                            if span.start > curr_idx && curr_idx < chars.len() {
                                let uncolored: String = chars[curr_idx..span.start.min(chars.len())].iter().collect();
                                job.append(&uncolored, 0.0, egui::TextFormat {
                                    font_id: font_id.clone(),
                                    color: c_text_color,
                                    ..Default::default()
                                });
                            }
                            if span.start < chars.len() {
                                let span_end = span.end.min(chars.len());
                                if span_end > span.start {
                                    let colored: String = chars[span.start..span_end].iter().collect();
                                    job.append(&colored, 0.0, egui::TextFormat {
                                        font_id: font_id.clone(),
                                        color: span.color,
                                        ..Default::default()
                                    });
                                }
                            }
                            curr_idx = span.end.max(curr_idx);
                        }
                        if curr_idx < chars.len() {
                            let rest: String = chars[curr_idx..].iter().collect();
                            job.append(&rest, 0.0, egui::TextFormat {
                                font_id: font_id.clone(),
                                color: c_text_color,
                                ..Default::default()
                            });
                        }
                        let galley = painter.fonts(|f| f.layout_job(job));
                        let rect = egui::Align2::LEFT_CENTER.anchor_size(text_pos, galley.size());
                        painter.galley(rect.min, galley, c_text_color);
                    } else {
                        painter.text(text_pos, egui::Align2::LEFT_CENTER, cell_text, font_id.clone(), c_text_color);
                    }
                } else {
                    painter.text(text_pos, egui::Align2::LEFT_CENTER, cell_text, font_id.clone(), c_text_color);
                }

                curr_x += col_w;
            }

            // Bottom row border
            painter.line_segment(
                [Pos2::new(s_min.x, curr_y + row_h), Pos2::new(s_min.x + s_total_w, curr_y + row_h)],
                Stroke::new(1.0_f32, border_color),
            );

            curr_y += row_h;
        }
    }

    fn render_pdf_card(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        pdf: &PdfCardElement,
        is_selected: bool,
        dark_mode: bool,
        pdf_texture: Option<&egui::TextureHandle>,
    ) {
        let s_min = camera.world_to_screen(pdf.pos, viewport_rect);
        let s_size = camera.world_vec_to_screen(pdf.size);
        let card_rect = Rect::from_min_size(s_min, s_size);

        let bg_color = if dark_mode { Color32::from_rgb(32, 34, 40) } else { Color32::WHITE };
        let border_color = if is_selected {
            Color32::from_rgb(0, 210, 255)
        } else {
            Color32::from_rgb(80, 85, 95)
        };

        painter.rect_filled(card_rect, CornerRadius::same(6), bg_color);
        painter.rect_stroke(card_rect, CornerRadius::same(6), Stroke::new(1.5_f32, border_color), StrokeKind::Middle);

        // Header bar
        let header_h = 28.0 * camera.zoom;
        let header_rect = Rect::from_min_size(s_min, Vec2::new(s_size.x, header_h));
        painter.rect_filled(header_rect, CornerRadius::same(5), Color32::from_rgb(220, 50, 50));

        let title = format!("PDF: {}", pdf.pdf_name);
        painter.text(
            header_rect.left_center() + Vec2::new(8.0 * camera.zoom, 0.0),
            egui::Align2::LEFT_CENTER,
            title,
            FontId::new((11.0 * camera.zoom).clamp(8.0, 14.0), FontFamily::Proportional),
            Color32::WHITE,
        );

        // Content area
        let footer_h = 26.0 * camera.zoom;
        let content_rect = Rect::from_min_max(
            Pos2::new(card_rect.min.x + 2.0, card_rect.min.y + header_h),
            Pos2::new(card_rect.max.x - 2.0, card_rect.max.y - footer_h),
        );

        if let Some(tex) = pdf_texture {
            painter.image(
                tex.id(),
                content_rect,
                Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                Color32::WHITE,
            );
        } else {
            // Draw technical datasheet layout placeholder
            let paper_bg = if dark_mode { Color32::from_rgb(22, 24, 30) } else { Color32::from_rgb(250, 252, 255) };
            painter.rect_filled(content_rect, CornerRadius::ZERO, paper_bg);
            let grid_color = if dark_mode { Color32::from_rgb(38, 42, 52) } else { Color32::from_rgb(230, 235, 245) };
            let step = (content_rect.height() / 8.0).max(12.0);
            for i in 1..8 {
                let y = content_rect.min.y + i as f32 * step;
                painter.line_segment(
                    [Pos2::new(content_rect.min.x + 8.0, y), Pos2::new(content_rect.max.x - 8.0, y)],
                    Stroke::new(1.0_f32, grid_color),
                );
            }
            painter.text(
                content_rect.center(),
                egui::Align2::CENTER_CENTER,
                format!("Datasheet Page {}", pdf.current_page + 1),
                FontId::new((12.0 * camera.zoom).clamp(9.0, 16.0), FontFamily::Monospace),
                if dark_mode { Color32::from_rgb(130, 140, 160) } else { Color32::from_rgb(90, 100, 120) },
            );
        }

        // Page flip footer
        let footer_rect = Rect::from_min_max(
            Pos2::new(card_rect.min.x, card_rect.max.y - footer_h),
            card_rect.max,
        );
        painter.rect_filled(
            footer_rect,
            CornerRadius::same(5),
            if dark_mode { Color32::from_rgb(25, 28, 35) } else { Color32::from_rgb(235, 240, 248) },
        );

        // Previous button box (<)
        let btn_w = 22.0 * camera.zoom;
        let prev_btn = Rect::from_center_size(
            Pos2::new(card_rect.min.x + 24.0 * camera.zoom, card_rect.max.y - footer_h / 2.0),
            Vec2::new(btn_w, footer_h * 0.7),
        );
        painter.rect_filled(prev_btn, CornerRadius::same(3), Color32::from_rgb(55, 60, 75));
        painter.text(prev_btn.center(), egui::Align2::CENTER_CENTER, "⮜", FontId::new(10.0, FontFamily::Proportional), Color32::WHITE);

        // Next button box (>)
        let next_btn = Rect::from_center_size(
            Pos2::new(card_rect.max.x - 24.0 * camera.zoom, card_rect.max.y - footer_h / 2.0),
            Vec2::new(btn_w, footer_h * 0.7),
        );
        painter.rect_filled(next_btn, CornerRadius::same(3), Color32::from_rgb(55, 60, 75));
        painter.text(next_btn.center(), egui::Align2::CENTER_CENTER, "⮞", FontId::new(10.0, FontFamily::Proportional), Color32::WHITE);

        let footer_text = format!("Page {} / {}", pdf.current_page + 1, pdf.total_pages.max(1));
        painter.text(
            Pos2::new(card_rect.center().x, card_rect.max.y - footer_h / 2.0),
            egui::Align2::CENTER_CENTER,
            footer_text,
            FontId::new((10.0 * camera.zoom).clamp(8.0, 13.0), FontFamily::Monospace),
            if dark_mode { Color32::LIGHT_GRAY } else { Color32::DARK_GRAY },
        );
    }

    fn render_image(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        img: &ImageElement,
        is_selected: bool,
        image_texture: Option<&egui::TextureHandle>,
    ) {
        let corners_world = img.get_corners_world();
        let pts_screen = [
            camera.world_to_screen(corners_world[0], viewport_rect),
            camera.world_to_screen(corners_world[1], viewport_rect),
            camera.world_to_screen(corners_world[2], viewport_rect),
            camera.world_to_screen(corners_world[3], viewport_rect),
        ];

        let is_warped = img.corners.is_some();
        let alpha = (img.opacity * 255.0).clamp(0.0, 255.0) as u8;

        if let Some(tex) = image_texture {
            if is_warped {
                // Subdivided grid mesh (8x8) for smooth 4-point perspective warp without diagonal artifacts
                let n = 8;
                let mut mesh = egui::Mesh::with_texture(tex.id());
                let v_color = Color32::from_white_alpha(alpha);
                for j in 0..=n {
                    let v = j as f32 / n as f32;
                    for i in 0..=n {
                        let u = i as f32 / n as f32;
                        let p0 = pts_screen[0];
                        let p1 = pts_screen[1];
                        let p2 = pts_screen[2];
                        let p3 = pts_screen[3];
                        let pos = Pos2::new(
                            (1.0 - u) * (1.0 - v) * p0.x + u * (1.0 - v) * p1.x + u * v * p2.x + (1.0 - u) * v * p3.x,
                            (1.0 - u) * (1.0 - v) * p0.y + u * (1.0 - v) * p1.y + u * v * p2.y + (1.0 - u) * v * p3.y,
                        );
                        mesh.vertices.push(egui::epaint::Vertex {
                            pos,
                            uv: Pos2::new(u, v),
                            color: v_color,
                        });
                    }
                }
                for j in 0..n {
                    for i in 0..n {
                        let row1 = (j * (n + 1) + i) as u32;
                        let row2 = ((j + 1) * (n + 1) + i) as u32;
                        mesh.indices.extend_from_slice(&[row1, row1 + 1, row2 + 1]);
                        mesh.indices.extend_from_slice(&[row1, row2 + 1, row2]);
                    }
                }
                painter.add(mesh);
            } else {
                let s_min = camera.world_to_screen(img.pos, viewport_rect);
                let s_size = camera.world_vec_to_screen(img.size);
                let img_rect = Rect::from_min_size(s_min, s_size);
                painter.image(
                    tex.id(),
                    img_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::from_white_alpha(alpha),
                );
            }
        } else {
            if is_warped {
                painter.add(PathShape::convex_polygon(
                    pts_screen.to_vec(),
                    Color32::from_rgb(45, 50, 60),
                    Stroke::NONE,
                ));
            } else {
                let s_min = camera.world_to_screen(img.pos, viewport_rect);
                let s_size = camera.world_vec_to_screen(img.size);
                let img_rect = Rect::from_min_size(s_min, s_size);
                painter.rect_filled(img_rect, CornerRadius::same(4), Color32::from_rgb(45, 50, 60));
            }
            let center = Pos2::new(
                (pts_screen[0].x + pts_screen[1].x + pts_screen[2].x + pts_screen[3].x) * 0.25,
                (pts_screen[0].y + pts_screen[1].y + pts_screen[2].y + pts_screen[3].y) * 0.25,
            );
            painter.text(
                center,
                egui::Align2::CENTER_CENTER,
                format!("Image: {}", img.image_name),
                FontId::new((12.0 * camera.zoom).clamp(9.0, 16.0), FontFamily::Proportional),
                Color32::WHITE,
            );
        }

        // When perspective lock is OFF and selected, show the 4 draggable corner handles, 4 edge midpoint handles, and quad boundary
        if !img.perspective_locked && is_selected {
            let handle_stroke = Stroke::new(1.5_f32, Color32::from_rgb(45, 96, 255));
            for i in 0..4 {
                let next = (i + 1) % 4;
                painter.line_segment([pts_screen[i], pts_screen[next]], handle_stroke);
            }
            // 4 Edge midpoint handles (distinct smaller squares, Figma-style)
            for i in 0..4 {
                let next = (i + 1) % 4;
                let mid_screen = Pos2::new(
                    (pts_screen[i].x + pts_screen[next].x) * 0.5,
                    (pts_screen[i].y + pts_screen[next].y) * 0.5,
                );
                let edge_rect = Rect::from_center_size(mid_screen, Vec2::new(8.0, 8.0));
                painter.rect_filled(edge_rect, CornerRadius::same(1), Color32::WHITE);
                painter.rect_stroke(edge_rect, CornerRadius::same(1), Stroke::new(1.5_f32, Color32::from_rgb(45, 96, 255)), StrokeKind::Middle);
            }
            // 4 Corner handles (circles)
            for &pt in &pts_screen {
                painter.circle_filled(pt, 5.5, Color32::WHITE);
                painter.circle_stroke(pt, 5.5, Stroke::new(2.0_f32, Color32::from_rgb(45, 96, 255)));
            }
        } else {
            let border_color = if is_selected { Color32::from_rgb(0, 210, 255) } else { Color32::from_rgb(80, 85, 95) };
            if is_warped {
                let stroke = Stroke::new(1.5_f32, border_color);
                for i in 0..4 {
                    let next = (i + 1) % 4;
                    painter.line_segment([pts_screen[i], pts_screen[next]], stroke);
                }
            } else {
                let s_min = camera.world_to_screen(img.pos, viewport_rect);
                let s_size = camera.world_vec_to_screen(img.size);
                let img_rect = Rect::from_min_size(s_min, s_size);
                painter.rect_stroke(img_rect, CornerRadius::same(4), Stroke::new(1.5_f32, border_color), StrokeKind::Middle);
            }
        }
    }

    fn render_doc_attachment(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        doc: &DocAttachmentElement,
        is_selected: bool,
        dark_mode: bool,
    ) {
        let s_min = camera.world_to_screen(doc.pos, viewport_rect);
        let s_size = camera.world_vec_to_screen(Vec2::new(180.0, 60.0));
        let chip_rect = Rect::from_min_size(s_min, s_size);

        let bg = if dark_mode { Color32::from_rgb(34, 38, 48) } else { Color32::from_rgb(240, 244, 250) };
        let border = if is_selected { Color32::from_rgb(0, 210, 255) } else { Color32::from_rgb(90, 100, 120) };

        painter.rect_filled(chip_rect, CornerRadius::same(8), bg);
        painter.rect_stroke(chip_rect, CornerRadius::same(8), Stroke::new(1.5_f32, border), StrokeKind::Middle);

        // Icon badge
        let icon_rect = Rect::from_min_size(
            chip_rect.min + Vec2::new(8.0 * camera.zoom, 8.0 * camera.zoom),
            Vec2::new(36.0 * camera.zoom, 44.0 * camera.zoom),
        );
        let icon_color = match doc.extension.to_lowercase().as_str() {
            "docx" | "doc" => Color32::from_rgb(30, 100, 210), // Word blue
            "xlsx" | "xls" | "csv" => Color32::from_rgb(35, 140, 65), // Excel green
            "pdf" => Color32::from_rgb(215, 45, 45), // PDF red
            _ => Color32::from_rgb(120, 125, 135),
        };
        painter.rect_filled(icon_rect, CornerRadius::same(4), icon_color);

        painter.text(
            icon_rect.center(),
            egui::Align2::CENTER_CENTER,
            doc.extension.to_uppercase(),
            FontId::new((9.0 * camera.zoom).clamp(7.0, 12.0), FontFamily::Monospace),
            Color32::WHITE,
        );

        // Filename & size
        let info_x = chip_rect.min.x + 52.0 * camera.zoom;
        painter.text(
            Pos2::new(info_x, chip_rect.min.y + 14.0 * camera.zoom),
            egui::Align2::LEFT_CENTER,
            &doc.filename,
            FontId::new((11.0 * camera.zoom).clamp(8.0, 14.0), FontFamily::Proportional),
            if dark_mode { Color32::WHITE } else { Color32::BLACK },
        );

        let size_kb = (doc.size_bytes as f64 / 1024.0).max(0.1);
        painter.text(
            Pos2::new(info_x, chip_rect.min.y + 34.0 * camera.zoom),
            egui::Align2::LEFT_CENTER,
            format!("{:.1} KB • Launch App", size_kb),
            FontId::new((9.0 * camera.zoom).clamp(7.0, 11.0), FontFamily::Proportional),
            Color32::from_rgb(140, 150, 165),
        );
    }

    pub fn render_item_selection_outline(painter: &Painter, camera: &Camera, viewport_rect: Rect, world_aabb: Rect) {
        let s_rect = camera.world_rect_to_screen(world_aabb, viewport_rect);
        let miro_blue_light = Color32::from_rgba_unmultiplied(45, 96, 255, 140);
        painter.rect_stroke(s_rect.expand(3.0), CornerRadius::ZERO, Stroke::new(1.2_f32, miro_blue_light), StrokeKind::Middle);
    }

    pub fn render_selection_box(painter: &Painter, camera: &Camera, viewport_rect: Rect, world_aabb: Rect) {
        let s_rect = camera.world_rect_to_screen(world_aabb, viewport_rect);
        let box_rect = s_rect.expand(5.0);
        let miro_blue = Color32::from_rgb(45, 96, 255); // Miro active blue #2d60ff
        let border_stroke = Stroke::new(1.5_f32, miro_blue);
        painter.rect_stroke(box_rect, CornerRadius::ZERO, border_stroke, StrokeKind::Middle);

        // 4 Corner circular resize handles (white fill, blue border)
        let handle_radius = 4.5;
        let corners = [
            box_rect.left_top(),
            box_rect.right_top(),
            box_rect.right_bottom(),
            box_rect.left_bottom(),
        ];

        for pos in corners {
            painter.circle_filled(pos, handle_radius, Color32::WHITE);
            painter.circle_stroke(pos, handle_radius, Stroke::new(1.5_f32, miro_blue));
        }

        // 4 Edge midpoint handles (Figma-style smaller distinct squares, white fill, blue border)
        let midpoints = [
            Pos2::new(box_rect.center().x, box_rect.min.y), // TopCenter
            Pos2::new(box_rect.max.x, box_rect.center().y), // RightCenter
            Pos2::new(box_rect.center().x, box_rect.max.y), // BottomCenter
            Pos2::new(box_rect.min.x, box_rect.center().y), // LeftCenter
        ];

        for mid in midpoints {
            let edge_rect = Rect::from_center_size(mid, Vec2::new(7.0, 7.0));
            painter.rect_filled(edge_rect, CornerRadius::same(1), Color32::WHITE);
            painter.rect_stroke(edge_rect, CornerRadius::same(1), Stroke::new(1.5_f32, miro_blue), StrokeKind::Middle);
        }

        // 4 Blue connector snap dots centered outside the 4 edges
        let snap_dot_radius = 3.0;
        let snap_dots = [
            Pos2::new(box_rect.center().x, box_rect.min.y - 12.0), // Top
            Pos2::new(box_rect.center().x, box_rect.max.y + 12.0), // Bottom
            Pos2::new(box_rect.min.x - 12.0, box_rect.center().y), // Left
            Pos2::new(box_rect.max.x + 12.0, box_rect.center().y), // Right
        ];

        for dot in snap_dots {
            painter.circle_filled(dot, snap_dot_radius, miro_blue);
        }

        // Rotation indicator near bottom-left
        let rot_pos = Pos2::new(box_rect.min.x - 14.0, box_rect.max.y + 14.0);
        painter.circle_filled(rot_pos, 7.0, Color32::from_rgb(240, 243, 250));
        painter.circle_stroke(rot_pos, 7.0, Stroke::new(1.0_f32, Color32::from_rgb(200, 205, 220)));
        painter.text(
            rot_pos,
            egui::Align2::CENTER_CENTER,
            "⟲",
            FontId::new(10.0, FontFamily::Proportional),
            miro_blue,
        );
    }

    /// Renders thin Figma-style magenta guide lines and matched point markers
    pub fn render_snap_guides(
        painter: &Painter,
        camera: &Camera,
        viewport_rect: Rect,
        guides: &[crate::canvas::snap::SnapGuideLine],
    ) {
        if guides.is_empty() {
            return;
        }

        let guide_color = Color32::from_rgb(255, 0, 128); // Figma magenta #ff0080
        let stroke = Stroke::new(1.0_f32, guide_color);

        for guide in guides {
            let s_start = camera.world_to_screen(guide.start, viewport_rect);
            let s_end = camera.world_to_screen(guide.end, viewport_rect);
            painter.line_segment([s_start, s_end], stroke);

            for &pt in &guide.matched_points {
                let s_pt = camera.world_to_screen(pt, viewport_rect);
                painter.circle_filled(s_pt, 3.5, guide_color);
                painter.circle_stroke(s_pt, 3.5, Stroke::new(1.0_f32, Color32::WHITE));
            }
        }
    }
}
