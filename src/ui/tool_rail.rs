use crate::canvas::tools::ActiveTool;
use crate::model::electronics::ComponentKind;
use crate::model::element::ShapeKind;
use egui::{epaint::StrokeKind, Color32, CornerRadius, Pos2, Rect, RichText, Stroke, Ui, Vec2};

pub enum ToolRailAction {
    None,
    SetTool(ActiveTool),
    SetStickyColor(Color32),
    SetStrokeColor(Color32),
    SetStrokeWidth(f32),
    InsertMedia,
    Undo,
    Redo,
    TogglePlanner,
}

pub struct ToolRail {
    pub hub_flyout_open: bool,
    pub sticky_palette_open: bool,
    pub shapes_flyout_open: bool,
    pub pen_flyout_open: bool,
    pub electronics_flyout_open: bool,
    pub templates_flyout_open: bool,
    pub more_flyout_open: bool,
}

impl Default for ToolRail {
    fn default() -> Self {
        Self {
            hub_flyout_open: false,
            sticky_palette_open: false,
            shapes_flyout_open: false,
            pen_flyout_open: false,
            electronics_flyout_open: false,
            templates_flyout_open: false,
            more_flyout_open: false,
        }
    }
}

pub const MIRO_STICKY_COLORS: [(Color32, &str); 16] = [
    (Color32::from_rgb(255, 249, 177), "Light Yellow"),
    (Color32::from_rgb(254, 244, 69), "Warm Yellow"),
    (Color32::from_rgb(250, 197, 90), "Orange"),
    (Color32::from_rgb(240, 138, 138), "Coral / Peach"),
    (Color32::from_rgb(252, 212, 239), "Light Pink"),
    (Color32::from_rgb(246, 147, 229), "Vibrant Pink"),
    (Color32::from_rgb(182, 207, 251), "Lavender"),
    (Color32::from_rgb(179, 157, 250), "Purple"),
    (Color32::from_rgb(146, 229, 252), "Sky Blue"),
    (Color32::from_rgb(126, 181, 250), "Azure Blue"),
    (Color32::from_rgb(115, 229, 216), "Aqua / Mint"),
    (Color32::from_rgb(89, 216, 126), "Green"),
    (Color32::from_rgb(200, 243, 149), "Light Lime"),
    (Color32::from_rgb(164, 236, 72), "Lime"),
    (Color32::from_rgb(248, 249, 250), "White"),
    (Color32::from_rgb(36, 36, 36), "Charcoal Dark"),
];

impl ToolRail {
    /// Renders Miro's floating left tool dock, separate undo/redo pill, and flyouts.
    /// Returns any action and a list of occupied Screen Rects for canvas hit prevention.
    pub fn render_floating(
        &mut self,
        ui: &mut Ui,
        current_tool: ActiveTool,
        active_sticky_color: Color32,
        active_stroke_color: Color32,
        active_stroke_width: f32,
        can_undo: bool,
        can_redo: bool,
        dark_mode: bool,
        canvas_rect: Rect,
    ) -> (ToolRailAction, Vec<Rect>) {
        let mut action = ToolRailAction::None;
        let mut occupied_rects = Vec::new();

        let dock_w = 44.0;
        let button_count = 11.0;
        let button_h = 36.0;
        let button_spacing = 2.0;
        let padding_y = 12.0; // 6px top + 6px bottom
        let dock_h = button_count * button_h + (button_count - 1.0) * button_spacing + padding_y; // 428.0px
        let dock_x = canvas_rect.min.x + 16.0;

        let undo_w = 44.0;
        let undo_h = 68.0;
        let stack_gap = 8.0;
        let total_stack_h = dock_h + stack_gap + undo_h; // 504.0px

        // Top clearance for header bar (44px + 10px margin = 54px)
        let top_clearance = 54.0;
        let _bottom_clearance = 12.0;

        // Position dock near top-left below floating header (~64px from top), matching Miro layout
        let natural_dock_y = canvas_rect.min.y + 64.0;
        let dock_y = if canvas_rect.height() >= total_stack_h + 70.0 {
            natural_dock_y
        } else {
            // Constrained viewport: clamp dock below header, never pushing off-screen
            (canvas_rect.min.y + top_clearance).min((canvas_rect.max.y - total_stack_h - 4.0).max(canvas_rect.min.y + 8.0))
        };

        let dock_rect = Rect::from_min_size(Pos2::new(dock_x, dock_y), Vec2::new(dock_w, dock_h));
        occupied_rects.push(dock_rect);

        // Undo/Redo pill is placed strictly below tool dock capsule with stack_gap, NEVER overlapping
        let undo_y = dock_rect.max.y + stack_gap;
        let undo_rect = Rect::from_min_size(Pos2::new(dock_x, undo_y), Vec2::new(undo_w, undo_h));
        occupied_rects.push(undo_rect);

        let bg_color = if dark_mode {
            Color32::from_rgb(30, 33, 42)
        } else {
            Color32::WHITE
        };
        let border_color = if dark_mode {
            Color32::from_rgb(52, 56, 70)
        } else {
            Color32::from_rgb(224, 227, 232)
        };

        // Render Drop Shadow & Dock Body
        ui.painter().rect_filled(
            dock_rect.translate(Vec2::new(1.0, 3.0)),
            CornerRadius::same(16),
            Color32::from_rgba_unmultiplied(0, 0, 0, if dark_mode { 26 } else { 14 }),
        );
        ui.painter().rect_filled(
            dock_rect.translate(Vec2::new(2.0, 6.0)),
            CornerRadius::same(16),
            Color32::from_rgba_unmultiplied(0, 0, 0, if dark_mode { 16 } else { 8 }),
        );
        ui.painter().rect_filled(dock_rect, CornerRadius::same(16), bg_color);
        ui.painter().rect_stroke(dock_rect, CornerRadius::same(16), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

        let templates_rect;
        let sticky_rect;
        let shape_rect;
        let pen_rect;
        let comp_rect;
        let more_rect;

        // Tool buttons list inside dock
        {
            let mut child_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(dock_rect.shrink2(Vec2::new(4.0, 6.0)))
                    .layout(egui::Layout::top_down(egui::Align::Center)),
            );
            child_ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);

            // 0. Miro Signature Hub Button (golden yellow badge with sparkle)
            let (_hub_rect, hub_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "✦",
                "Hub",
                "Quick Action Hub",
                self.hub_flyout_open,
                None,
                dark_mode,
            );
            if hub_clicked {
                let was_open = self.hub_flyout_open;
                self.close_all_flyouts();
                self.hub_flyout_open = !was_open;
            }

            // 1. Select Tool (V)
            let is_sel = current_tool == ActiveTool::Select;
            let (_, sel_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "➤",
                "Select",
                "Select (V)",
                is_sel,
                None,
                dark_mode,
            );
            if sel_clicked {
                self.close_all_flyouts();
                action = ToolRailAction::SetTool(ActiveTool::Select);
            }

            // 2. Templates / Formats (📄⁺)
            let (tr, templates_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "📄⁺",
                "Templates",
                "Templates & Flows",
                self.templates_flyout_open,
                None,
                dark_mode,
            );
            templates_rect = tr;
            if templates_clicked {
                let was_open = self.templates_flyout_open;
                self.close_all_flyouts();
                self.templates_flyout_open = !was_open;
            }

            // 3. Table Grid Tool (▦)
            let is_table = current_tool == ActiveTool::Table;
            let (_, table_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "▦",
                "Table",
                "Table Grid",
                is_table,
                None,
                dark_mode,
            );
            if table_clicked {
                self.close_all_flyouts();
                action = ToolRailAction::SetTool(ActiveTool::Table);
            }

            // 4. Sticky Note Tool (N) - Folded note with active color swatch indicator
            let is_sticky = current_tool == ActiveTool::StickyNote;
            let (sr, sticky_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "🗈",
                "Sticky",
                "Sticky note (N)",
                is_sticky || self.sticky_palette_open,
                Some(active_sticky_color),
                dark_mode,
            );
            sticky_rect = sr;
            if sticky_clicked {
                let was_open = self.sticky_palette_open;
                self.close_all_flyouts();
                self.sticky_palette_open = !was_open;
                action = ToolRailAction::SetTool(ActiveTool::StickyNote);
            }

            // 5. Text Tool (T) - Bold serif T
            let is_text = current_tool == ActiveTool::Text;
            let (_, text_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "T",
                "Text",
                "Text (T)",
                is_text,
                None,
                dark_mode,
            );
            if text_clicked {
                self.close_all_flyouts();
                action = ToolRailAction::SetTool(ActiveTool::Text);
            }

            // 6. Shapes & Connectors (□ ○ △ ↗)
            let is_shape = matches!(current_tool, ActiveTool::Shape(_));
            let (shr, shape_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "□○△↗",
                "Shape",
                "Shapes & Lines (R)",
                is_shape || self.shapes_flyout_open,
                None,
                dark_mode,
            );
            shape_rect = shr;
            if shape_clicked {
                let was_open = self.shapes_flyout_open;
                self.close_all_flyouts();
                self.shapes_flyout_open = !was_open;
                if !is_shape {
                    action = ToolRailAction::SetTool(ActiveTool::Shape(ShapeKind::Rectangle));
                }
            }

            // 7. Pen & Highlighter (P) - Stylus nib at 45°
            let is_pen_any = matches!(current_tool, ActiveTool::Pen | ActiveTool::Highlighter | ActiveTool::Eraser);
            let (pr, pen_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "✎",
                "Pen",
                "Pen & Highlighter (P)",
                is_pen_any || self.pen_flyout_open,
                Some(active_stroke_color),
                dark_mode,
            );
            pen_rect = pr;
            if pen_clicked {
                let was_open = self.pen_flyout_open;
                self.close_all_flyouts();
                self.pen_flyout_open = !was_open;
                if !is_pen_any {
                    action = ToolRailAction::SetTool(ActiveTool::Pen);
                }
            }

            // 8. Circuit & Electronics (⚡)
            let is_comp = matches!(current_tool, ActiveTool::Component(_));
            let (cr, comp_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "⚡",
                "Circuit",
                "Circuit & Components (C)",
                is_comp || self.electronics_flyout_open,
                None,
                dark_mode,
            );
            comp_rect = cr;
            if comp_clicked {
                let was_open = self.electronics_flyout_open;
                self.close_all_flyouts();
                self.electronics_flyout_open = !was_open;
            }

            // 9. Magnetic Wire Tool (⤹)
            let is_wire = current_tool == ActiveTool::Wire;
            let (_, wire_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "⤹",
                "Wire",
                "Magnetic Wire (W)",
                is_wire,
                None,
                dark_mode,
            );
            if wire_clicked {
                self.close_all_flyouts();
                action = ToolRailAction::SetTool(ActiveTool::Wire);
            }

            // 10. More tools & Import (+)
            let (mr, more_clicked) = render_dock_tool_btn(
                &mut child_ui,
                "+",
                "More",
                "More tools & Import (+)",
                self.more_flyout_open,
                None,
                dark_mode,
            );
            more_rect = mr;
            if more_clicked {
                let was_open = self.more_flyout_open;
                self.close_all_flyouts();
                self.more_flyout_open = !was_open;
            }
        }

        // ==========================================
        // SEPARATE FLOATING UNDO / REDO PILL
        // ==========================================
        ui.painter().rect_filled(
            undo_rect.translate(Vec2::new(1.0, 3.0)),
            CornerRadius::same(14),
            Color32::from_rgba_unmultiplied(0, 0, 0, if dark_mode { 20 } else { 10 }),
        );
        ui.painter().rect_filled(undo_rect, CornerRadius::same(14), bg_color);
        ui.painter().rect_stroke(undo_rect, CornerRadius::same(14), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

        {
            let mut undo_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(undo_rect.shrink2(Vec2::new(4.0, 5.0)))
                    .layout(egui::Layout::top_down(egui::Align::Center)),
            );
            undo_ui.spacing_mut().item_spacing = Vec2::new(0.0, 2.0);

            // Undo Button (⮌)
            let undo_tt = format!("Undo ({})", crate::shortcuts::AppShortcut::undo_label());
            if render_dock_undo_btn(&mut undo_ui, "⮌", &undo_tt, can_undo, dark_mode) {
                action = ToolRailAction::Undo;
            }

            // Redo Button (⮎)
            let redo_tt = format!("Redo ({})", crate::shortcuts::AppShortcut::redo_label());
            if render_dock_undo_btn(&mut undo_ui, "⮎", &redo_tt, can_redo, dark_mode) {
                action = ToolRailAction::Redo;
            }
        }

        // ==========================================
        // FLYOUT 0: Siro Action Hub & Quick Library
        // ==========================================
        if self.hub_flyout_open {
            let flyout_w = 210.0;
            let flyout_h = 240.0;
            let flyout_y = (dock_rect.min.y + 6.0)
                .clamp(canvas_rect.min.y + 10.0, (canvas_rect.max.y - flyout_h - 10.0).max(canvas_rect.min.y + 10.0));
            let flyout_pos = Pos2::new(dock_rect.max.x + 10.0, flyout_y);
            let flyout_rect = Rect::from_min_size(flyout_pos, Vec2::new(flyout_w, flyout_h));
            occupied_rects.push(flyout_rect);

            ui.painter().rect_filled(
                flyout_rect.translate(Vec2::new(2.0, 4.0)),
                CornerRadius::same(16),
                Color32::from_rgba_unmultiplied(0, 0, 0, 24),
            );
            ui.painter().rect_filled(flyout_rect, CornerRadius::same(16), bg_color);
            ui.painter().rect_stroke(flyout_rect, CornerRadius::same(16), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

            let mut fly_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(flyout_rect.shrink(10.0))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );

            fly_ui.horizontal(|ui| {
                let (badge, _) = ui.allocate_exact_size(Vec2::new(20.0, 20.0), egui::Sense::hover());
                ui.painter().rect_filled(badge, CornerRadius::same(6), Color32::from_rgb(255, 208, 47));
                ui.painter().text(badge.center(), egui::Align2::CENTER_CENTER, "✦", egui::FontId::proportional(12.0), Color32::BLACK);
                ui.label(RichText::new("Quick Action Hub").size(13.0).strong().color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) }));
            });
            fly_ui.add_space(6.0);
            fly_ui.separator();

            if fly_ui.button("📄  Insert Sticky Note (N)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::StickyNote);
                self.hub_flyout_open = false;
            }
            if fly_ui.button("⚡  Draw Magnetic Wire (W)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Wire);
                self.hub_flyout_open = false;
            }
            if fly_ui.button("▦  Add BOM / Data Table").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Table);
                self.hub_flyout_open = false;
            }
            if fly_ui.button("📎  Import Media / Datasheet").clicked() {
                action = ToolRailAction::InsertMedia;
                self.hub_flyout_open = false;
            }
            if fly_ui.button("📋  Toggle STEM Planner").clicked() {
                action = ToolRailAction::TogglePlanner;
                self.hub_flyout_open = false;
            }
        }

        // ==========================================
        // FLYOUT 1: Miro Sticky Pastel Palette Flyout
        // ==========================================
        if self.sticky_palette_open {
            let flyout_w = 98.0;
            let flyout_h = 300.0;
            let flyout_y = (sticky_rect.center().y - 130.0)
                .clamp(canvas_rect.min.y + 10.0, (canvas_rect.max.y - flyout_h - 10.0).max(canvas_rect.min.y + 10.0));
            let flyout_pos = Pos2::new(dock_rect.max.x + 10.0, flyout_y);
            let flyout_rect = Rect::from_min_size(flyout_pos, Vec2::new(flyout_w, flyout_h));
            occupied_rects.push(flyout_rect);

            ui.painter().rect_filled(
                flyout_rect.translate(Vec2::new(2.0, 4.0)),
                CornerRadius::same(16),
                Color32::from_rgba_unmultiplied(0, 0, 0, 24),
            );
            ui.painter().rect_filled(flyout_rect, CornerRadius::same(16), bg_color);
            ui.painter().rect_stroke(flyout_rect, CornerRadius::same(16), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

            let mut fly_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(flyout_rect.shrink(10.0))
                    .layout(egui::Layout::top_down(egui::Align::Center)),
            );

            // 2 columns x 8 rows of pastel swatches
            for row in MIRO_STICKY_COLORS.chunks(2) {
                fly_ui.horizontal(|ui| {
                    for &(color, name) in row {
                        let is_active = active_sticky_color == color;
                        let (sw_rect, sw_res) = ui.allocate_exact_size(Vec2::new(34.0, 30.0), egui::Sense::click());
                        ui.painter().rect_filled(sw_rect, CornerRadius::same(4), color);
                        if is_active {
                            ui.painter().rect_stroke(
                                sw_rect,
                                CornerRadius::same(4),
                                Stroke::new(2.0_f32, Color32::from_rgb(45, 96, 255)),
                                StrokeKind::Inside,
                            );
                        } else {
                            ui.painter().rect_stroke(
                                sw_rect,
                                CornerRadius::same(4),
                                Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 30)),
                                StrokeKind::Inside,
                            );
                        }

                        if sw_res.on_hover_text(name).clicked() {
                            action = ToolRailAction::SetStickyColor(color);
                            self.sticky_palette_open = false;
                        }
                    }
                });
                fly_ui.add_space(2.0);
            }
        }

        // ==========================================
        // FLYOUT 2: Miro Shapes Flyout
        // ==========================================
        if self.shapes_flyout_open {
            let flyout_w = 160.0;
            let flyout_h = 290.0;
            let flyout_y = (shape_rect.center().y - 120.0)
                .clamp(canvas_rect.min.y + 10.0, (canvas_rect.max.y - flyout_h - 10.0).max(canvas_rect.min.y + 10.0));
            let flyout_pos = Pos2::new(dock_rect.max.x + 10.0, flyout_y);
            let flyout_rect = Rect::from_min_size(flyout_pos, Vec2::new(flyout_w, flyout_h));
            occupied_rects.push(flyout_rect);

            ui.painter().rect_filled(
                flyout_rect.translate(Vec2::new(2.0, 4.0)),
                CornerRadius::same(14),
                Color32::from_rgba_unmultiplied(0, 0, 0, 24),
            );
            ui.painter().rect_filled(flyout_rect, CornerRadius::same(14), bg_color);
            ui.painter().rect_stroke(flyout_rect, CornerRadius::same(14), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

            let mut fly_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(flyout_rect.shrink(8.0))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );

            fly_ui.label(RichText::new("Lines").size(11.0).color(Color32::from_rgb(130, 135, 150)));
            if fly_ui.button("╱  Straight Line (L)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Shape(ShapeKind::Line));
                self.shapes_flyout_open = false;
            }
            if fly_ui.button("↗  Arrow").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Shape(ShapeKind::Arrow));
                self.shapes_flyout_open = false;
            }
            if fly_ui.button("⚡  Orthogonal Wire").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Wire);
                self.shapes_flyout_open = false;
            }

            fly_ui.separator();
            fly_ui.label(RichText::new("Basic Shapes").size(11.0).color(Color32::from_rgb(130, 135, 150)));
            if fly_ui.button("▢  Rectangle (R)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Shape(ShapeKind::Rectangle));
                self.shapes_flyout_open = false;
            }
            if fly_ui.button("▢  Rounded Box").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Shape(ShapeKind::RoundedRectangle));
                self.shapes_flyout_open = false;
            }
            if fly_ui.button("⬭  Oval / Circle (O)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Shape(ShapeKind::Ellipse));
                self.shapes_flyout_open = false;
            }
            if fly_ui.button("◇  Diamond").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Shape(ShapeKind::Diamond));
                self.shapes_flyout_open = false;
            }
            if fly_ui.button("△  Triangle").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Shape(ShapeKind::Triangle));
                self.shapes_flyout_open = false;
            }
        }

        // ==========================================
        // FLYOUT 3: Miro Pen & Freehand Tools Flyout
        // ==========================================
        if self.pen_flyout_open {
            let flyout_w = 160.0;
            let flyout_h = 240.0;
            let flyout_y = (pen_rect.center().y - 100.0)
                .clamp(canvas_rect.min.y + 10.0, (canvas_rect.max.y - flyout_h - 10.0).max(canvas_rect.min.y + 10.0));
            let flyout_pos = Pos2::new(dock_rect.max.x + 10.0, flyout_y);
            let flyout_rect = Rect::from_min_size(flyout_pos, Vec2::new(flyout_w, flyout_h));
            occupied_rects.push(flyout_rect);

            ui.painter().rect_filled(
                flyout_rect.translate(Vec2::new(2.0, 4.0)),
                CornerRadius::same(14),
                Color32::from_rgba_unmultiplied(0, 0, 0, 24),
            );
            ui.painter().rect_filled(flyout_rect, CornerRadius::same(14), bg_color);
            ui.painter().rect_stroke(flyout_rect, CornerRadius::same(14), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

            let mut fly_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(flyout_rect.shrink(8.0))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );

            if fly_ui.button("✎  Pen (P)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Pen);
                self.pen_flyout_open = false;
            }
            if fly_ui.button("🖍  Highlighter (H)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Highlighter);
                self.pen_flyout_open = false;
            }
            if fly_ui.button("⌫  Object Eraser (E)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Eraser);
                self.pen_flyout_open = false;
            }

            fly_ui.separator();
            fly_ui.label(RichText::new("Stroke Width").size(11.0).color(Color32::from_rgb(130, 135, 150)));

            fly_ui.horizontal(|ui| {
                let widths = [(1.5, "Fine"), (3.0, "Medium"), (5.5, "Bold")];
                for (w, label) in widths {
                    let is_cur = (active_stroke_width - w).abs() < 0.2;
                    let (cr, c_res) = ui.allocate_exact_size(Vec2::new(36.0, 24.0), egui::Sense::click());
                    if is_cur {
                        ui.painter().rect_filled(cr, CornerRadius::same(4), Color32::from_rgb(45, 96, 255));
                        ui.painter().text(cr.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(10.0), Color32::WHITE);
                    } else {
                        ui.painter().rect_filled(cr, CornerRadius::same(4), if dark_mode { Color32::from_rgb(40, 44, 56) } else { Color32::from_rgb(235, 238, 244) });
                        ui.painter().text(cr.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(10.0), if dark_mode { Color32::LIGHT_GRAY } else { Color32::DARK_GRAY });
                    }
                    if c_res.clicked() {
                        action = ToolRailAction::SetStrokeWidth(w);
                    }
                }
            });

            fly_ui.add_space(4.0);
            fly_ui.label(RichText::new("Color Presets").size(11.0).color(Color32::from_rgb(130, 135, 150)));
            if let Some(col) = crate::ui::color_presets::render_color_presets_and_picker(
                &mut fly_ui,
                active_stroke_color,
                "tool_rail_pen",
            ) {
                action = ToolRailAction::SetStrokeColor(col);
            }
        }

        // ==========================================
        // FLYOUT 4: Electronics & Schematics Flyout
        // ==========================================
        if self.electronics_flyout_open {
            let flyout_w = 200.0;
            let flyout_h = 340.0;
            let flyout_y = (comp_rect.center().y - 150.0)
                .clamp(canvas_rect.min.y + 10.0, (canvas_rect.max.y - flyout_h - 10.0).max(canvas_rect.min.y + 10.0));
            let flyout_pos = Pos2::new(dock_rect.max.x + 10.0, flyout_y);
            let flyout_rect = Rect::from_min_size(flyout_pos, Vec2::new(flyout_w, flyout_h));
            occupied_rects.push(flyout_rect);

            ui.painter().rect_filled(
                flyout_rect.translate(Vec2::new(2.0, 4.0)),
                CornerRadius::same(14),
                Color32::from_rgba_unmultiplied(0, 0, 0, 24),
            );
            ui.painter().rect_filled(flyout_rect, CornerRadius::same(14), bg_color);
            ui.painter().rect_stroke(flyout_rect, CornerRadius::same(14), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

            let mut fly_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(flyout_rect.shrink(8.0))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );

            fly_ui.label(RichText::new("Maker Boards & ICs").size(11.0).strong().color(Color32::from_rgb(45, 96, 255)));
            if fly_ui.button("Raspberry Pi Pico").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Component(ComponentKind::RaspberryPiPico));
                self.electronics_flyout_open = false;
            }
            if fly_ui.button("ESP32-WROOM-32").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Component(ComponentKind::Esp32Wroom));
                self.electronics_flyout_open = false;
            }
            if fly_ui.button("Arduino Nano").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Component(ComponentKind::ArduinoNano));
                self.electronics_flyout_open = false;
            }
            if fly_ui.button("Breadboard Section").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Component(ComponentKind::BreadboardSection));
                self.electronics_flyout_open = false;
            }

            fly_ui.separator();
            fly_ui.label(RichText::new("Schematic Symbols").size(11.0).strong().color(Color32::from_rgb(45, 96, 255)));
            if fly_ui.button("Resistor (R)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Component(ComponentKind::Resistor));
                self.electronics_flyout_open = false;
            }
            if fly_ui.button("Capacitor (C)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Component(ComponentKind::Capacitor));
                self.electronics_flyout_open = false;
            }
            if fly_ui.button("LED Indicator").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Component(ComponentKind::Led));
                self.electronics_flyout_open = false;
            }
            if fly_ui.button("DC Battery").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Component(ComponentKind::Battery));
                self.electronics_flyout_open = false;
            }
            if fly_ui.button("Ground (GND)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Component(ComponentKind::Ground));
                self.electronics_flyout_open = false;
            }
            if fly_ui.button("Switch (SW)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Component(ComponentKind::Switch));
                self.electronics_flyout_open = false;
            }
        }

        // ==========================================
        // FLYOUT: Formats and Flows (Templates Flyout)
        // ==========================================
        if self.templates_flyout_open {
            let flyout_w = 210.0;
            let flyout_h = 280.0;
            let flyout_y = (templates_rect.center().y - 60.0)
                .clamp(canvas_rect.min.y + 10.0, (canvas_rect.max.y - flyout_h - 10.0).max(canvas_rect.min.y + 10.0));
            let flyout_pos = Pos2::new(dock_rect.max.x + 10.0, flyout_y);
            let flyout_rect = Rect::from_min_size(flyout_pos, Vec2::new(flyout_w, flyout_h));
            occupied_rects.push(flyout_rect);

            ui.painter().rect_filled(
                flyout_rect.translate(Vec2::new(2.0, 4.0)),
                CornerRadius::same(16),
                Color32::from_rgba_unmultiplied(0, 0, 0, 24),
            );
            ui.painter().rect_filled(flyout_rect, CornerRadius::same(16), bg_color);
            ui.painter().rect_stroke(flyout_rect, CornerRadius::same(16), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

            let mut fly_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(flyout_rect.shrink(10.0))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );

            fly_ui.horizontal(|ui| {
                ui.label(RichText::new("Formats and Flows").size(13.0).strong().color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) }));
            });
            fly_ui.add_space(6.0);
            fly_ui.separator();

            if fly_ui.button("📊  Diagram & Shapes").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Shape(ShapeKind::Rectangle));
                self.templates_flyout_open = false;
            }
            if fly_ui.button("▦  Data / BOM Table").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Table);
                self.templates_flyout_open = false;
            }
            if fly_ui.button("⏱  Timeline Roadmap").clicked() {
                action = ToolRailAction::TogglePlanner;
                self.templates_flyout_open = false;
            }
            if fly_ui.button("📋  Kanban Framework").clicked() {
                action = ToolRailAction::TogglePlanner;
                self.templates_flyout_open = false;
            }
            if fly_ui.button("📄  Doc / Sticky Notes").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::StickyNote);
                self.templates_flyout_open = false;
            }
            if fly_ui.button("⚡  Circuit Schematic").clicked() {
                self.templates_flyout_open = false;
                self.electronics_flyout_open = true;
            }
            if fly_ui.button("✋  Pan Canvas (Hand)").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Pan);
                self.templates_flyout_open = false;
            }
        }

        // ==========================================
        // FLYOUT: More Tools & Import Flyout
        // ==========================================
        if self.more_flyout_open {
            let flyout_w = 210.0;
            let flyout_h = 200.0;
            let flyout_y = (more_rect.center().y - 120.0)
                .clamp(canvas_rect.min.y + 10.0, (canvas_rect.max.y - flyout_h - 10.0).max(canvas_rect.min.y + 10.0));
            let flyout_pos = Pos2::new(dock_rect.max.x + 10.0, flyout_y);
            let flyout_rect = Rect::from_min_size(flyout_pos, Vec2::new(flyout_w, flyout_h));
            occupied_rects.push(flyout_rect);

            ui.painter().rect_filled(
                flyout_rect.translate(Vec2::new(2.0, 4.0)),
                CornerRadius::same(16),
                Color32::from_rgba_unmultiplied(0, 0, 0, 24),
            );
            ui.painter().rect_filled(flyout_rect, CornerRadius::same(16), bg_color);
            ui.painter().rect_stroke(flyout_rect, CornerRadius::same(16), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

            let mut fly_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(flyout_rect.shrink(10.0))
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );

            fly_ui.horizontal(|ui| {
                ui.label(RichText::new("More Tools & Import").size(13.0).strong().color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) }));
            });
            fly_ui.add_space(6.0);
            fly_ui.separator();

            if fly_ui.button("📁  Import Media / PDF / Docs").clicked() {
                action = ToolRailAction::InsertMedia;
                self.more_flyout_open = false;
            }
            if fly_ui.button("▦  Data Table / BOM Grid").clicked() {
                action = ToolRailAction::SetTool(ActiveTool::Table);
                self.more_flyout_open = false;
            }
            if fly_ui.button("📋  Kanban Task Board").clicked() {
                action = ToolRailAction::TogglePlanner;
                self.more_flyout_open = false;
            }
            if fly_ui.button("⏱  Timeline Roadmap").clicked() {
                action = ToolRailAction::TogglePlanner;
                self.more_flyout_open = false;
            }
            if fly_ui.button("⚡  Electronics Library").clicked() {
                self.more_flyout_open = false;
                self.electronics_flyout_open = true;
            }
        }

        (action, occupied_rects)
    }

    pub fn close_all_flyouts(&mut self) {
        self.hub_flyout_open = false;
        self.sticky_palette_open = false;
        self.shapes_flyout_open = false;
        self.pen_flyout_open = false;
        self.electronics_flyout_open = false;
        self.templates_flyout_open = false;
        self.more_flyout_open = false;
    }
}

// -------------------------------------------------------------
// Helper Dock Button Renderers (Larger Icons + Tool Names Below)
// -------------------------------------------------------------

fn render_dock_tool_btn(
    ui: &mut Ui,
    icon: &str,
    label: &str,
    tooltip: &str,
    is_active: bool,
    color_swatch: Option<Color32>,
    dark_mode: bool,
) -> (Rect, bool) {
    let size = Vec2::new(36.0, 36.0);
    let (rect, res) = ui.allocate_exact_size(size, egui::Sense::click());

    let bg = if is_active {
        if dark_mode {
            Color32::from_rgb(46, 55, 78)
        } else {
            Color32::from_rgb(234, 240, 255)
        }
    } else if res.hovered() {
        if dark_mode {
            Color32::from_rgb(40, 44, 56)
        } else {
            Color32::from_rgb(243, 245, 249)
        }
    } else {
        Color32::TRANSPARENT
    };

    ui.painter().rect_filled(rect, CornerRadius::same(8), bg);

    let icon_color = if is_active {
        if dark_mode {
            Color32::WHITE
        } else {
            Color32::from_rgb(25, 75, 215)
        }
    } else if dark_mode {
        Color32::from_rgb(215, 220, 235)
    } else {
        Color32::from_rgb(30, 35, 45)
    };

    let icon_pos = rect.center();

    // Render cohesive icons matching Miro craftsmanship
    if label == "Hub" {
        // Golden yellow rounded badge (#ffd02f) with crisp black sparkle ✦
        let badge_rect = Rect::from_center_size(icon_pos, Vec2::new(26.0, 26.0));
        ui.painter().rect_filled(badge_rect, CornerRadius::same(8), Color32::from_rgb(255, 208, 47));
        ui.painter().text(
            badge_rect.center(),
            egui::Align2::CENTER_CENTER,
            "✦",
            egui::FontId::proportional(15.0),
            Color32::BLACK,
        );
    } else if label == "Select" {
        // Crisp Miro-style pointer cursor pointing northwest (11 o'clock)
        let tip = icon_pos + Vec2::new(-6.0, -7.5);
        let right_wing = icon_pos + Vec2::new(4.5, -1.0);
        let notch = icon_pos + Vec2::new(-0.5, 1.0);
        let tail_right = icon_pos + Vec2::new(3.0, 7.0);
        let tail_left = icon_pos + Vec2::new(1.0, 7.8);
        let notch_left = icon_pos + Vec2::new(-1.8, 2.2);
        let left_wing = icon_pos + Vec2::new(-6.0, 4.0);
        ui.painter().add(egui::epaint::Shape::convex_polygon(vec![tip, right_wing, notch], icon_color, Stroke::NONE));
        ui.painter().add(egui::epaint::Shape::convex_polygon(vec![tip, notch, left_wing], icon_color, Stroke::NONE));
        ui.painter().add(egui::epaint::Shape::convex_polygon(vec![notch, tail_right, tail_left, notch_left], icon_color, Stroke::NONE));
    } else if label == "Templates" {
        // Document sheet with folded corner and '+' badge
        let sheet_stroke = Stroke::new(1.4_f32, icon_color);
        let top_left = icon_pos + Vec2::new(-7.0, -8.0);
        let top_right = icon_pos + Vec2::new(3.0, -8.0);
        let fold_right = icon_pos + Vec2::new(7.0, -4.0);
        let bottom_right = icon_pos + Vec2::new(7.0, 8.0);
        let bottom_left = icon_pos + Vec2::new(-7.0, 8.0);
        let fold_corner = icon_pos + Vec2::new(3.0, -4.0);

        ui.painter().line_segment([top_left, top_right], sheet_stroke);
        ui.painter().line_segment([fold_right, bottom_right], sheet_stroke);
        ui.painter().line_segment([bottom_right, bottom_left], sheet_stroke);
        ui.painter().line_segment([bottom_left, top_left], sheet_stroke);
        ui.painter().line_segment([top_right, fold_corner], sheet_stroke);
        ui.painter().line_segment([fold_corner, fold_right], sheet_stroke);

        // Document content lines
        ui.painter().line_segment([icon_pos + Vec2::new(-4.0, -2.0), icon_pos + Vec2::new(2.0, -2.0)], sheet_stroke);
        ui.painter().line_segment([icon_pos + Vec2::new(-4.0, 2.0), icon_pos + Vec2::new(4.0, 2.0)], sheet_stroke);

        // Mini plus (+) badge at bottom-right blending cleanly with state
        let plus_bg = if is_active {
            if dark_mode { Color32::from_rgb(46, 55, 78) } else { Color32::from_rgb(234, 240, 255) }
        } else if res.hovered() {
            if dark_mode { Color32::from_rgb(40, 44, 56) } else { Color32::from_rgb(243, 245, 249) }
        } else {
            if dark_mode { Color32::from_rgb(30, 33, 42) } else { Color32::WHITE }
        };
        let plus_badge_center = icon_pos + Vec2::new(6.0, 6.0);
        ui.painter().circle_filled(plus_badge_center, 4.2, plus_bg);
        ui.painter().line_segment(
            [plus_badge_center + Vec2::new(-2.5, 0.0), plus_badge_center + Vec2::new(2.5, 0.0)],
            Stroke::new(1.3_f32, icon_color),
        );
        ui.painter().line_segment(
            [plus_badge_center + Vec2::new(0.0, -2.5), plus_badge_center + Vec2::new(0.0, 2.5)],
            Stroke::new(1.3_f32, icon_color),
        );
    } else if label == "Table" {
        // Clean 2x2 grid rectangle with dividers
        let grid_rect = Rect::from_center_size(icon_pos, Vec2::new(16.0, 14.0));
        ui.painter().rect_stroke(
            grid_rect,
            CornerRadius::same(2),
            Stroke::new(1.4_f32, icon_color),
            StrokeKind::Inside,
        );
        ui.painter().line_segment(
            [Pos2::new(grid_rect.min.x, grid_rect.center().y), Pos2::new(grid_rect.max.x, grid_rect.center().y)],
            Stroke::new(1.2_f32, icon_color),
        );
        ui.painter().line_segment(
            [Pos2::new(grid_rect.center().x, grid_rect.min.y), Pos2::new(grid_rect.center().x, grid_rect.max.y)],
            Stroke::new(1.2_f32, icon_color),
        );
    } else if label == "Sticky" {
        // Crisp folded note matching Miro design
        let hw = 7.5;
        let hh = 7.5;
        let fold = 4.5;
        let top_left = icon_pos + Vec2::new(-hw, -hh);
        let top_right = icon_pos + Vec2::new(hw, -hh);
        let fold_right = icon_pos + Vec2::new(hw, hh - fold);
        let fold_bottom = icon_pos + Vec2::new(hw - fold, hh);
        let bottom_left = icon_pos + Vec2::new(-hw, hh);
        let fold_corner = icon_pos + Vec2::new(hw - fold, hh - fold);

        if let Some(swatch) = color_swatch {
            let poly = vec![top_left, top_right, fold_right, fold_bottom, bottom_left];
            ui.painter().add(egui::epaint::Shape::convex_polygon(poly, swatch, Stroke::NONE));
            let flap = vec![fold_right, fold_corner, fold_bottom];
            ui.painter().add(egui::epaint::Shape::convex_polygon(flap, Color32::from_rgba_unmultiplied(0, 0, 0, 30), Stroke::NONE));
        }

        let note_stroke = Stroke::new(1.4_f32, icon_color);
        ui.painter().line_segment([top_left, top_right], note_stroke);
        ui.painter().line_segment([top_right, fold_right], note_stroke);
        ui.painter().line_segment([fold_right, fold_bottom], note_stroke);
        ui.painter().line_segment([fold_bottom, bottom_left], note_stroke);
        ui.painter().line_segment([bottom_left, top_left], note_stroke);
        ui.painter().line_segment([fold_right, fold_corner], note_stroke);
        ui.painter().line_segment([fold_corner, fold_bottom], note_stroke);
    } else if label == "Text" {
        // Bold serif T with serifs on top crossbar and bottom base
        let bar_rect = Rect::from_center_size(icon_pos + Vec2::new(0.0, -6.0), Vec2::new(14.0, 2.0));
        ui.painter().rect_filled(bar_rect, CornerRadius::ZERO, icon_color);
        ui.painter().rect_filled(Rect::from_min_size(icon_pos + Vec2::new(-7.0, -6.0), Vec2::new(2.0, 3.0)), CornerRadius::ZERO, icon_color);
        ui.painter().rect_filled(Rect::from_min_size(icon_pos + Vec2::new(5.0, -6.0), Vec2::new(2.0, 3.0)), CornerRadius::ZERO, icon_color);
        let stem_rect = Rect::from_center_size(icon_pos + Vec2::new(0.0, 0.5), Vec2::new(2.2, 12.0));
        ui.painter().rect_filled(stem_rect, CornerRadius::ZERO, icon_color);
        let base_rect = Rect::from_center_size(icon_pos + Vec2::new(0.0, 6.0), Vec2::new(10.0, 2.0));
        ui.painter().rect_filled(base_rect, CornerRadius::ZERO, icon_color);
    } else if label == "Shape" {
        // Miro's composite icon: square, circle, triangle, and arrow (□ ○ △ ↗)
        let s_stroke = Stroke::new(1.3_f32, icon_color);
        let sq_rect = Rect::from_center_size(icon_pos + Vec2::new(-4.5, -4.5), Vec2::new(6.0, 6.0));
        ui.painter().rect_stroke(sq_rect, CornerRadius::ZERO, s_stroke, StrokeKind::Middle);
        let c_pos = icon_pos + Vec2::new(4.5, -4.5);
        ui.painter().circle_stroke(c_pos, 3.2, s_stroke);
        let t_top = icon_pos + Vec2::new(-4.5, 1.5);
        let t_br = icon_pos + Vec2::new(-1.5, 7.5);
        let t_bl = icon_pos + Vec2::new(-7.5, 7.5);
        ui.painter().line_segment([t_top, t_br], s_stroke);
        ui.painter().line_segment([t_br, t_bl], s_stroke);
        ui.painter().line_segment([t_bl, t_top], s_stroke);
        let a_start = icon_pos + Vec2::new(2.0, 7.5);
        let a_end = icon_pos + Vec2::new(7.5, 2.0);
        ui.painter().line_segment([a_start, a_end], s_stroke);
        ui.painter().line_segment([a_end, a_end + Vec2::new(-3.5, 0.0)], s_stroke);
        ui.painter().line_segment([a_end, a_end + Vec2::new(0.0, 3.5)], s_stroke);
    } else if label == "Pen" {
        // Fountain pen / stylus nib angled at 45° pointing Northwest (Miro style)
        let nib_stroke = Stroke::new(1.4_f32, icon_color);
        let tip = icon_pos + Vec2::new(-6.0, -6.0);
        let shoulder1 = icon_pos + Vec2::new(-2.0, -7.0);
        let shoulder2 = icon_pos + Vec2::new(-7.0, -2.0);
        let base1 = icon_pos + Vec2::new(5.0, 2.0);
        let base2 = icon_pos + Vec2::new(2.0, 5.0);
        let end_p = icon_pos + Vec2::new(5.5, 5.5);
        ui.painter().line_segment([tip, shoulder1], nib_stroke);
        ui.painter().line_segment([tip, shoulder2], nib_stroke);
        ui.painter().line_segment([shoulder1, base1], nib_stroke);
        ui.painter().line_segment([shoulder2, base2], nib_stroke);
        ui.painter().line_segment([base1, end_p], nib_stroke);
        ui.painter().line_segment([base2, end_p], nib_stroke);
        let slit_end = icon_pos + Vec2::new(-0.5, -0.5);
        ui.painter().line_segment([tip, slit_end], nib_stroke);
        ui.painter().circle_filled(slit_end, 1.3, icon_color);
    } else if label == "Circuit" {
        // Clean IC chip with pins and lightning bolt
        let chip_rect = Rect::from_center_size(icon_pos, Vec2::new(13.0, 13.0));
        ui.painter().rect_filled(chip_rect, CornerRadius::same(2), icon_color);
        let pin_stroke = Stroke::new(1.2_f32, icon_color);
        for dy in &[-3.0, 3.0] {
            ui.painter().line_segment([Pos2::new(chip_rect.min.x - 3.0, chip_rect.center().y + dy), Pos2::new(chip_rect.min.x, chip_rect.center().y + dy)], pin_stroke);
            ui.painter().line_segment([Pos2::new(chip_rect.max.x, chip_rect.center().y + dy), Pos2::new(chip_rect.max.x + 3.0, chip_rect.center().y + dy)], pin_stroke);
        }
        let bolt_col = if is_active {
            if dark_mode { Color32::from_rgb(46, 55, 78) } else { Color32::from_rgb(234, 240, 255) }
        } else {
            if dark_mode { Color32::from_rgb(30, 33, 42) } else { Color32::WHITE }
        };
        ui.painter().text(
            chip_rect.center(),
            egui::Align2::CENTER_CENTER,
            "⚡",
            egui::FontId::proportional(9.5),
            bolt_col,
        );
    } else if label == "Wire" {
        // S-curve connector wire with connection terminal dots
        let p_start = icon_pos + Vec2::new(-6.0, 5.0);
        let p_end = icon_pos + Vec2::new(6.0, -5.0);
        let wire_stroke = Stroke::new(1.5_f32, icon_color);
        ui.painter().line_segment([p_start, Pos2::new(icon_pos.x, p_start.y)], wire_stroke);
        ui.painter().line_segment([Pos2::new(icon_pos.x, p_start.y), Pos2::new(icon_pos.x, p_end.y)], wire_stroke);
        ui.painter().line_segment([Pos2::new(icon_pos.x, p_end.y), p_end], wire_stroke);
        ui.painter().circle_filled(p_start, 2.5, icon_color);
        ui.painter().circle_filled(p_end, 2.5, icon_color);
    } else if label == "More" {
        // Clean + icon at the bottom of the dock
        let plus_stroke = Stroke::new(1.6_f32, icon_color);
        ui.painter().line_segment(
            [icon_pos + Vec2::new(-6.0, 0.0), icon_pos + Vec2::new(6.0, 0.0)],
            plus_stroke,
        );
        ui.painter().line_segment(
            [icon_pos + Vec2::new(0.0, -6.0), icon_pos + Vec2::new(0.0, 6.0)],
            plus_stroke,
        );
    } else {
        ui.painter().text(
            icon_pos,
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(17.5),
            icon_color,
        );
    }

    // Optional active color swatch dot indicator (e.g. for Sticky notes)
    if let Some(swatch_col) = color_swatch {
        let dot_pos = Pos2::new(rect.max.x - 6.0, rect.max.y - 6.0);
        ui.painter().circle_filled(dot_pos, 3.2, swatch_col);
        ui.painter().circle_stroke(
            dot_pos,
            3.2,
            Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(60, 65, 80) } else { Color32::from_rgb(200, 205, 215) }),
        );
    }

    (rect, res.on_hover_text(tooltip).clicked())
}

fn render_dock_undo_btn(
    ui: &mut Ui,
    icon: &str,
    tooltip: &str,
    enabled: bool,
    dark_mode: bool,
) -> bool {
    let size = Vec2::new(36.0, 28.0);
    let (rect, res) = ui.allocate_exact_size(size, if enabled { egui::Sense::click() } else { egui::Sense::hover() });

    let bg = if enabled && res.hovered() {
        if dark_mode {
            Color32::from_rgb(42, 46, 58)
        } else {
            Color32::from_rgb(240, 243, 248)
        }
    } else {
        Color32::TRANSPARENT
    };

    ui.painter().rect_filled(rect, CornerRadius::same(6), bg);

    let icon_color = if !enabled {
        if dark_mode {
            Color32::from_rgb(75, 80, 95)
        } else {
            Color32::from_rgb(185, 190, 200)
        }
    } else if dark_mode {
        Color32::from_rgb(215, 220, 235)
    } else {
        Color32::from_rgb(45, 50, 65)
    };

    let c = rect.center();
    let wire_stroke = Stroke::new(1.6_f32, icon_color);
    if icon == "⮌" {
        // Curved vector Undo arrow: arc curving right-to-left with arrowhead pointing down-left
        let arc_pts = [
            c + Vec2::new(5.0, 3.0),
            c + Vec2::new(4.5, -1.5),
            c + Vec2::new(1.5, -3.5),
            c + Vec2::new(-2.5, -3.0),
            c + Vec2::new(-5.0, -0.5),
        ];
        for pair in arc_pts.windows(2) {
            ui.painter().line_segment([pair[0], pair[1]], wire_stroke);
        }
        // Arrowhead tip pointing down-left
        ui.painter().line_segment([c + Vec2::new(-5.0, -4.5), c + Vec2::new(-5.0, -0.5)], wire_stroke);
        ui.painter().line_segment([c + Vec2::new(-1.0, -0.5), c + Vec2::new(-5.0, -0.5)], wire_stroke);
    } else if icon == "⮎" {
        // Curved vector Redo arrow: arc curving left-to-right with arrowhead pointing down-right
        let arc_pts = [
            c + Vec2::new(-5.0, 3.0),
            c + Vec2::new(-4.5, -1.5),
            c + Vec2::new(-1.5, -3.5),
            c + Vec2::new(2.5, -3.0),
            c + Vec2::new(5.0, -0.5),
        ];
        for pair in arc_pts.windows(2) {
            ui.painter().line_segment([pair[0], pair[1]], wire_stroke);
        }
        // Arrowhead tip pointing down-right
        ui.painter().line_segment([c + Vec2::new(5.0, -4.5), c + Vec2::new(5.0, -0.5)], wire_stroke);
        ui.painter().line_segment([c + Vec2::new(1.0, -0.5), c + Vec2::new(5.0, -0.5)], wire_stroke);
    } else {
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(16.0),
            icon_color,
        );
    }

    if enabled {
        res.on_hover_text(tooltip).clicked()
    } else {
        false
    }
}
