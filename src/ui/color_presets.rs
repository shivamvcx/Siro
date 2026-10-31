use egui::{Color32, Context, Pos2, Rect, Stroke, Ui, Vec2};

use crate::ui::popup_manager::PopupManager;

pub const COLOR_PRESETS: [(Color32, &str); 5] = [
    (Color32::from_rgb(0, 0, 0), "Black"),
    (Color32::from_rgb(255, 255, 255), "White"),
    (Color32::from_rgb(255, 220, 0), "Yellow"),
    (Color32::from_rgb(235, 50, 50), "Red"),
    (Color32::from_rgb(70, 160, 255), "Light Blue"),
];

/// Checks if a given color matches one of the 5 presets.
pub fn is_preset_color(color: Color32) -> bool {
    COLOR_PRESETS.iter().any(|(c, _)| *c == color)
}

/// Renders the preset swatches (Black, White, Yellow, Red, Light Blue)
/// and a round rainbow-gradient button.
/// Returns `(Option<chosen_preset>, rainbow_clicked, rainbow_rect)`.
pub fn render_color_presets_swatches(
    ui: &mut Ui,
    current_color: Color32,
) -> (Option<Color32>, bool, Rect) {
    let mut chosen_preset = None;
    let mut rainbow_clicked = false;
    let mut rainbow_rect = Rect::ZERO;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

        // 1. Preset Swatches in required order: Black, White, Yellow, Red, Light Blue
        for &(preset_col, name) in &COLOR_PRESETS {
            let (rect, res) = ui.allocate_exact_size(Vec2::new(18.0, 18.0), egui::Sense::click());
            let center = rect.center();
            let radius = 7.5;
            let is_selected = current_color == preset_col;

            // Fill circle
            ui.painter().circle_filled(center, radius, preset_col);

            // If white, draw a faint outline so it stands out on light backgrounds
            if preset_col == Color32::WHITE {
                ui.painter().circle_stroke(
                    center,
                    radius,
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(180, 185, 195, 200)),
                );
            }

            // Highlight ring around the selected color
            if is_selected {
                ui.painter().circle_stroke(
                    center,
                    radius + 2.0,
                    Stroke::new(2.0_f32, Color32::from_rgb(45, 96, 255)),
                );
            }

            if res.on_hover_text(name).clicked() {
                chosen_preset = Some(preset_col);
            }
        }

        // 2. Round Rainbow-Gradient Button
        let (rb_rect, rb_res) = ui.allocate_exact_size(Vec2::new(18.0, 18.0), egui::Sense::click());
        let center = rb_rect.center();
        let radius = 7.5;
        let is_custom_selected = !is_preset_color(current_color);
        rainbow_rect = rb_rect;

        // Render rainbow wheel using smooth triangle fan
        draw_rainbow_circle(ui, center, radius);

        // Highlight ring if currently selected color is a custom color
        if is_custom_selected {
            ui.painter().circle_stroke(
                center,
                radius + 2.0,
                Stroke::new(2.0_f32, Color32::from_rgb(45, 96, 255)),
            );
        }

        let rb_res = rb_res.on_hover_text("Custom Color");
        if rb_res.clicked() {
            rainbow_clicked = true;
        }
    });

    (chosen_preset, rainbow_clicked, rainbow_rect)
}

/// Renders a custom color picker popup using the shared PopupManager.
/// Returns `Some(Color32)` if the user selected or changed a color.
pub fn render_custom_color_picker_popup(
    popup_manager: &mut PopupManager,
    ctx: &Context,
    is_dark: bool,
    picker_id: &str,
    parent_popup_id: Option<&str>,
    anchor_rect: Rect,
    current_color: Color32,
) -> Option<Color32> {
    let mut updated_color = None;

    if popup_manager.is_open(picker_id) {
        popup_manager.show(
            ctx,
            is_dark,
            picker_id,
            "Custom Color",
            parent_popup_id,
            Some(anchor_rect),
            None,
            |p_ui| {
                p_ui.set_max_width(220.0);
                p_ui.spacing_mut().item_spacing = Vec2::new(0.0, 6.0);

                let mut edit_color = current_color;
                if egui::color_picker::color_picker_color32(
                    p_ui,
                    &mut edit_color,
                    egui::color_picker::Alpha::Opaque,
                ) {
                    updated_color = Some(edit_color);
                }
            },
        );
    }

    updated_color
}

/// Renders the preset swatches and wires the rainbow button to the shared PopupManager.
/// Used directly for standalone toolbars and properties.
pub fn render_color_presets_with_manager(
    ui: &mut Ui,
    popup_manager: &mut PopupManager,
    current_color: Color32,
    id_source: &str,
    parent_popup_id: Option<&str>,
) -> Option<Color32> {
    let (preset_col, rb_clicked, rb_rect) = render_color_presets_swatches(ui, current_color);
    let picker_id = format!("{}_custom_color_picker", id_source);

    if rb_clicked {
        popup_manager.toggle(
            &picker_id,
            "Custom Color",
            parent_popup_id,
            Some(rb_rect),
            None,
        );
    }

    let is_dark = ui.ctx().style().visuals.dark_mode;
    let custom_col = render_custom_color_picker_popup(
        popup_manager,
        ui.ctx(),
        is_dark,
        &picker_id,
        parent_popup_id,
        rb_rect,
        current_color,
    );

    preset_col.or(custom_col)
}

/// Backwards-compatible standalone version of render_color_presets_and_picker.
pub fn render_color_presets_and_picker(
    ui: &mut Ui,
    current_color: Color32,
    id_source: &str,
) -> Option<Color32> {
    let mut chosen_color = None;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(4.0, 0.0);

        for &(preset_col, name) in &COLOR_PRESETS {
            let (rect, res) = ui.allocate_exact_size(Vec2::new(18.0, 18.0), egui::Sense::click());
            let center = rect.center();
            let radius = 7.5;
            let is_selected = current_color == preset_col;

            ui.painter().circle_filled(center, radius, preset_col);
            if preset_col == Color32::WHITE {
                ui.painter().circle_stroke(
                    center,
                    radius,
                    Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(180, 185, 195, 200)),
                );
            }
            if is_selected {
                ui.painter().circle_stroke(
                    center,
                    radius + 2.0,
                    Stroke::new(2.0_f32, Color32::from_rgb(45, 96, 255)),
                );
            }
            if res.on_hover_text(name).clicked() {
                chosen_color = Some(preset_col);
            }
        }

        let (rb_rect, rb_res) = ui.allocate_exact_size(Vec2::new(18.0, 18.0), egui::Sense::click());
        let center = rb_rect.center();
        let radius = 7.5;
        let is_custom_selected = !is_preset_color(current_color);

        draw_rainbow_circle(ui, center, radius);
        if is_custom_selected {
            ui.painter().circle_stroke(
                center,
                radius + 2.0,
                Stroke::new(2.0_f32, Color32::from_rgb(45, 96, 255)),
            );
        }

        let popup_id = ui.make_persistent_id(format!("{}_color_picker_popup", id_source));
        let rb_res = rb_res.on_hover_text("Custom Color");
        if rb_res.clicked() {
            ui.memory_mut(|mem| mem.toggle_popup(popup_id));
        }

        egui::popup_below_widget(
            ui,
            popup_id,
            &rb_res,
            egui::PopupCloseBehavior::CloseOnClickOutside,
            |popup_ui| {
                popup_ui.spacing_mut().item_spacing = Vec2::new(0.0, 6.0);
                popup_ui.set_max_width(220.0);

                let mut edit_color = current_color;
                popup_ui.label(
                    egui::RichText::new("Custom Color")
                        .size(11.0)
                        .color(Color32::from_rgb(130, 135, 150)),
                );

                if egui::color_picker::color_picker_color32(
                    popup_ui,
                    &mut edit_color,
                    egui::color_picker::Alpha::Opaque,
                ) {
                    chosen_color = Some(edit_color);
                }
            },
        );
    });

    chosen_color
}

/// Draws a smooth circular rainbow gradient using a triangle fan.
fn draw_rainbow_circle(ui: &Ui, center: Pos2, radius: f32) {
    let segments = 12;
    let mut mesh = egui::Mesh::default();

    mesh.vertices.push(egui::epaint::Vertex {
        pos: center,
        uv: Pos2::ZERO,
        color: Color32::WHITE,
    });

    for i in 0..segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let pos = center + Vec2::new(angle.cos(), angle.sin()) * radius;
        let hue = i as f32 / segments as f32;
        let color: Color32 = egui::ecolor::Hsva::new(hue, 1.0, 1.0, 1.0).into();
        mesh.vertices.push(egui::epaint::Vertex {
            pos,
            uv: Pos2::ZERO,
            color,
        });

        let next = (i + 1) % segments;
        mesh.indices.extend_from_slice(&[0, 1 + i as u32, 1 + next as u32]);
    }

    ui.painter().add(mesh);
    ui.painter().circle_stroke(
        center,
        radius,
        Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 40)),
    );
}
