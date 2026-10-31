use crate::canvas::camera::Camera;
use crate::canvas::tools::ActiveTool;
use crate::model::board::Board;
use crate::model::element::*;
use egui::{epaint::StrokeKind, Color32, CornerRadius, Pos2, Rect, RichText, Stroke, Ui, Vec2};
use uuid::Uuid;

pub enum PropertiesAction {
    None,
    DeleteSelected,
    DuplicateSelected,
    BringToFront,
    SendToBack,
    BringForward,
    SendBackward,
    RotateComponent(Uuid),
    TableAddRow(Uuid),
    TableRemoveRow(Uuid),
    TableAddCol(Uuid),
    TableRemoveCol(Uuid),
    TableResizeCol(Uuid, f32),
    PdfPrevPage(Uuid),
    PdfNextPage(Uuid),
    LaunchDoc(Uuid),
    ImageResetPerspective(Uuid),
    TableApplyColor(Uuid, Color32),
}

pub struct PropertiesBar;

impl PropertiesBar {
    pub fn calculate_toolbar_width(board: &Board, selected_ids: &[Uuid]) -> f32 {
        if selected_ids.is_empty() {
            return 200.0;
        }

        let single_element = if selected_ids.len() == 1 {
            board.get_element(selected_ids[0])
        } else {
            None
        };

        let has_text = matches!(single_element, Some(BoardElement::Text(_)));
        let is_sticky = match single_element {
            Some(BoardElement::Text(t)) => t.is_sticky,
            _ => false,
        };
        let is_table = matches!(single_element, Some(BoardElement::Table(_)));
        let is_component = matches!(single_element, Some(BoardElement::Component(_)));
        let is_pdf = matches!(single_element, Some(BoardElement::PdfCard(_)));
        let is_doc = matches!(single_element, Some(BoardElement::DocAttachment(_)));
        let is_image = matches!(single_element, Some(BoardElement::Image(_)));

        let has_shape_stroke_wire = if selected_ids.len() == 1 {
            matches!(
                single_element,
                Some(BoardElement::Shape(_)) | Some(BoardElement::Stroke(_)) | Some(BoardElement::Wire(_))
            )
        } else {
            selected_ids.iter().any(|&id| {
                matches!(
                    board.get_element(id),
                    Some(BoardElement::Shape(_)) | Some(BoardElement::Stroke(_)) | Some(BoardElement::Wire(_))
                )
            })
        };

        let mut w = 24.0; // padding left & right

        // 1. Indicator Icon + Separator
        w += 32.0;

        // 2. Context-specific controls
        let mut had_context = false;
        if has_text {
            w += 150.0; // "Aa" + 12 16 22 32 + margins
            if !is_sticky {
                let is_bg_on = match single_element {
                    Some(BoardElement::Text(t)) => t.bg_enabled,
                    _ => false,
                };
                if is_bg_on {
                    w += 65.0; // Background checkbox + style button
                } else {
                    w += 28.0; // Background checkbox
                }
            }
            had_context = true;
        } else if is_table {
            w += 220.0; // + Row, - Row, + Col, - Col
            had_context = true;
        } else if is_component {
            w += 65.0; // ↻ 90°
            had_context = true;
        } else if is_pdf {
            w += 85.0; // P.n, prev, next
            had_context = true;
        } else if is_doc {
            w += 75.0; // 🚀 Open
            had_context = true;
        } else if is_image {
            w += 110.0; // Perspective Lock toggle + Reset perspective
            had_context = true;
        } else if has_shape_stroke_wire {
            w += 190.0; // Line: + 1.5px 2.5px 4px 6px
            had_context = true;
        }

        // 3. Separator between context and colors (if both are shown)
        let show_colors = !is_doc && !is_pdf && !is_image;
        if had_context && show_colors {
            w += 14.0;
        }

        // 4. Color swatches
        if show_colors {
            if is_sticky {
                w += 165.0; // 8 pastel colors (16px circle + 4px spacing)
            } else {
                w += 125.0; // 6 Miro colors (16px circle + 4px spacing)
            }
        }

        // 5. Separator before actions
        w += 14.0;

        // 6. Actions: ⬆ Front, ⬇ Back, ⧉ Duplicate, 🗑 Delete
        w += 130.0;

        // Minimum 20px extra breathing room so nothing collides or wraps
        w + 24.0
    }

    /// Renders Miro's floating contextual toolbar directly ABOVE the selected element on the canvas.
    /// Returns any action and the Screen Rect occupied by the toolbar (for canvas hit prevention).
    pub fn render_floating(
        ui: &mut Ui,
        camera: &Camera,
        viewport_rect: Rect,
        _current_tool: ActiveTool,
        selected_ids: &[Uuid],
        board: &mut Board,
        active_stroke_color: &mut Color32,
        active_stroke_width: &mut f32,
        _active_wire_color: &mut WireColor,
        active_sticky_color: &mut Color32,
        dark_mode: bool,
        popup_manager: &mut crate::ui::PopupManager,
    ) -> (PropertiesAction, Option<Rect>) {
        if selected_ids.is_empty() {
            popup_manager.close("text_bg_popover");
            return (PropertiesAction::None, None);
        }

        let mut action = PropertiesAction::None;

        // Compute combined bounding box in world space
        let mut combined_aabb: Option<Rect> = None;
        for &id in selected_ids {
            if let Some(elem) = board.get_element(id) {
                let aabb = elem.geometric_aabb();
                combined_aabb = Some(match combined_aabb {
                    Some(cur) => cur.union(aabb),
                    None => aabb,
                });
            }
        }

        let world_aabb = match combined_aabb {
            Some(a) => a,
            None => return (PropertiesAction::None, None),
        };

        let s_rect = camera.world_rect_to_screen(world_aabb, viewport_rect);

        // Dynamically compute toolbar width based on active context
        let tb_w = Self::calculate_toolbar_width(board, selected_ids).min((viewport_rect.width() - 32.0).max(200.0));
        let tb_h = 38.0;

        // Anchor directly ABOVE the selected element on canvas
        let anchor_x = (s_rect.center().x - tb_w / 2.0).clamp(
            viewport_rect.min.x + 16.0,
            (viewport_rect.max.x - tb_w - 16.0).max(viewport_rect.min.x + 16.0),
        );
        let anchor_y = if s_rect.min.y - 50.0 >= viewport_rect.min.y + 60.0 {
            s_rect.min.y - 50.0
        } else {
            // If element is near top, float below it instead
            s_rect.max.y + 14.0
        };
        let anchor_y = anchor_y.clamp(viewport_rect.min.y + 14.0, (viewport_rect.max.y - tb_h - 14.0).max(viewport_rect.min.y + 14.0));

        let tb_rect = Rect::from_min_size(Pos2::new(anchor_x, anchor_y), Vec2::new(tb_w, tb_h));

        let bg_color = if dark_mode {
            Color32::from_rgb(32, 35, 45)
        } else {
            Color32::WHITE
        };
        let border_color = if dark_mode {
            Color32::from_rgb(52, 56, 72)
        } else {
            Color32::from_rgb(222, 226, 235)
        };

        // Render Drop Shadow & Pill Body
        let p = ui.painter();
        p.rect_filled(
            tb_rect.translate(Vec2::new(1.0, 3.0)),
            CornerRadius::same(12),
            Color32::from_rgba_unmultiplied(0, 0, 0, 22),
        );
        p.rect_filled(tb_rect, CornerRadius::same(12), bg_color);
        p.rect_stroke(tb_rect, CornerRadius::same(12), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(tb_rect.shrink2(Vec2::new(10.0, 3.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.spacing_mut().item_spacing = Vec2::new(5.0, 0.0);
        child_ui.spacing_mut().button_padding = Vec2::new(5.0, 3.0);

        let single_element = if selected_ids.len() == 1 {
            board.get_element(selected_ids[0])
        } else {
            None
        };

        let has_text = matches!(single_element, Some(BoardElement::Text(_)));
        let is_sticky = match single_element {
            Some(BoardElement::Text(t)) => t.is_sticky,
            _ => false,
        };
        let is_table = matches!(single_element, Some(BoardElement::Table(_)));
        let is_component = matches!(single_element, Some(BoardElement::Component(_)));
        let is_pdf = matches!(single_element, Some(BoardElement::PdfCard(_)));
        let is_doc = matches!(single_element, Some(BoardElement::DocAttachment(_)));
        let is_image = matches!(single_element, Some(BoardElement::Image(_)));

        let has_shape_stroke_wire = if selected_ids.len() == 1 {
            matches!(
                single_element,
                Some(BoardElement::Shape(_)) | Some(BoardElement::Stroke(_)) | Some(BoardElement::Wire(_))
            )
        } else {
            selected_ids.iter().any(|&id| {
                matches!(
                    board.get_element(id),
                    Some(BoardElement::Shape(_)) | Some(BoardElement::Stroke(_)) | Some(BoardElement::Wire(_))
                )
            })
        };

        // 1. Element Type Indicator Icon
        let icon = if selected_ids.len() == 1 {
            match single_element {
                Some(BoardElement::Text(t)) if t.is_sticky => "📄",
                Some(BoardElement::Text(_)) => "T",
                Some(BoardElement::Shape(_)) => "▢",
                Some(BoardElement::Component(_)) => "🔌",
                Some(BoardElement::Wire(_)) => "⚡",
                Some(BoardElement::Table(_)) => "▦",
                Some(BoardElement::PdfCard(_)) => "📑",
                Some(BoardElement::Image(_)) => "🖼",
                Some(BoardElement::DocAttachment(_)) => "📎",
                _ => "•",
            }
        } else {
            "⧉"
        };
        child_ui.label(RichText::new(icon).size(13.0).color(Color32::from_rgb(45, 96, 255)));
        child_ui.separator();

        // 2. Context-specific controls
        let mut had_context = false;
        if has_text {
            had_context = true;
            if let Some(BoardElement::Text(t)) = board.get_element_mut(selected_ids[0]) {
                child_ui.label(RichText::new("Aa").size(11.0).color(Color32::from_rgb(130, 135, 150)));
                for &sz in &[12.0f32, 16.0f32, 22.0f32, 32.0f32] {
                    let is_cur = (t.font_size - sz).abs() < 1.0;
                    if child_ui.selectable_label(is_cur, format!("{}", sz as u32)).clicked() {
                        t.font_size = sz;
                    }
                }

                if !t.is_sticky {
                    child_ui.separator();
                    child_ui.checkbox(&mut t.bg_enabled, "BG");
                    if t.bg_enabled {
                        let bg_popup_id = "text_bg_popover";
                        let [r, g, b, _] = t.bg_color.to_array();
                        let alpha = (t.bg_opacity.clamp(0.0, 1.0) * 255.0) as u8;
                        let swatch_btn = child_ui.add(
                            egui::Button::new(
                                RichText::new("⚙ Style ▾").size(11.0).color(Color32::from_rgba_unmultiplied(r, g, b, alpha.max(160)))
                            )
                            .fill(if dark_mode { Color32::from_rgb(45, 50, 65) } else { Color32::from_rgb(240, 242, 248) })
                            .corner_radius(CornerRadius::same(4))
                        ).on_hover_text("Background Style & Shape (Color, Opacity, Padding, Rounding)");

                        if swatch_btn.clicked() {
                            popup_manager.toggle(
                                bg_popup_id,
                                "Background Style",
                                None,
                                Some(swatch_btn.rect),
                                Some(s_rect),
                            );
                        }

                        let mut need_toggle_child_picker = None;
                        let child_picker_id = "text_bg_custom_color_picker";

                        if popup_manager.is_open(bg_popup_id) {
                            popup_manager.show(
                                child_ui.ctx(),
                                dark_mode,
                                bg_popup_id,
                                "Background Style",
                                None,
                                Some(swatch_btn.rect),
                                Some(s_rect),
                                |p_ui| {
                                    p_ui.label(RichText::new("Color:").size(11.0).color(Color32::from_rgb(130, 135, 150)));
                                    let (chosen_preset, rb_clicked, rb_rect) = crate::ui::color_presets::render_color_presets_swatches(
                                        p_ui,
                                        t.bg_color,
                                    );
                                    if let Some(col) = chosen_preset {
                                        t.bg_color = col;
                                    }
                                    if rb_clicked {
                                        need_toggle_child_picker = Some(rb_rect);
                                    }

                                    p_ui.separator();
                                    p_ui.label(RichText::new("Opacity:").size(11.0).color(Color32::from_rgb(130, 135, 150)));
                                    let mut percent = (t.bg_opacity * 100.0).round();
                                    if p_ui.add(egui::Slider::new(&mut percent, 0.0..=100.0).suffix("%")).changed() {
                                        t.bg_opacity = (percent / 100.0).clamp(0.0, 1.0);
                                    }

                                    p_ui.separator();
                                    p_ui.label(RichText::new("Size (Padding):").size(11.0).color(Color32::from_rgb(130, 135, 150)));
                                    p_ui.horizontal(|ui| {
                                        ui.label("X:");
                                        ui.add(egui::Slider::new(&mut t.bg_pad_x, 0.0..=40.0).suffix("px"));
                                    });
                                    p_ui.horizontal(|ui| {
                                        ui.label("Y:");
                                        ui.add(egui::Slider::new(&mut t.bg_pad_y, 0.0..=30.0).suffix("px"));
                                    });
                                    p_ui.checkbox(&mut t.bg_match_width, "Fill text box width");

                                    p_ui.separator();
                                    p_ui.label(RichText::new("Shape (Corner Rounding):").size(11.0).color(Color32::from_rgb(130, 135, 150)));
                                    p_ui.add(egui::Slider::new(&mut t.bg_corner_radius, 0.0..=30.0).suffix("px"));
                                },
                            );
                        }

                        if let Some(rb_rect) = need_toggle_child_picker {
                            popup_manager.toggle(
                                child_picker_id,
                                "Custom Color",
                                Some(bg_popup_id),
                                Some(rb_rect),
                                None,
                            );
                        }

                        if popup_manager.is_open(child_picker_id) {
                            if let Some(col) = crate::ui::color_presets::render_custom_color_picker_popup(
                                popup_manager,
                                child_ui.ctx(),
                                dark_mode,
                                child_picker_id,
                                Some(bg_popup_id),
                                swatch_btn.rect,
                                t.bg_color,
                            ) {
                                t.bg_color = col;
                            }
                        }
                    }
                }
            }
        } else if is_table {
            had_context = true;
            let single_id = selected_ids[0];
            if child_ui.button("+ Row").clicked() {
                action = PropertiesAction::TableAddRow(single_id);
            }
            if child_ui.button("- Row").clicked() {
                action = PropertiesAction::TableRemoveRow(single_id);
            }
            if child_ui.button("+ Col").clicked() {
                action = PropertiesAction::TableAddCol(single_id);
            }
            if child_ui.button("- Col").clicked() {
                action = PropertiesAction::TableRemoveCol(single_id);
            }
        } else if is_component {
            had_context = true;
            let single_id = selected_ids[0];
            if child_ui.button("↻ 90°").on_hover_text("Rotate Component (R)").clicked() {
                action = PropertiesAction::RotateComponent(single_id);
            }
        } else if is_pdf {
            had_context = true;
            let single_id = selected_ids[0];
            let cur_page = match board.get_element(single_id) {
                Some(BoardElement::PdfCard(p)) => p.current_page,
                _ => 0,
            };
            child_ui.label(RichText::new(format!("P.{}", cur_page + 1)).size(11.0));
            if child_ui.button("⮜").clicked() {
                action = PropertiesAction::PdfPrevPage(single_id);
            }
            if child_ui.button("⮞").clicked() {
                action = PropertiesAction::PdfNextPage(single_id);
            }
        } else if is_doc {
            had_context = true;
            let single_id = selected_ids[0];
            if child_ui.button("🚀 Open").on_hover_text("Launch Document").clicked() {
                action = PropertiesAction::LaunchDoc(single_id);
            }
        } else if is_image {
            had_context = true;
            let single_id = selected_ids[0];
            if let Some(BoardElement::Image(img)) = board.get_element_mut(single_id) {
                let lock_label = if img.perspective_locked {
                    "🔒 Lock: ON"
                } else {
                    "🔓 Lock: OFF"
                };
                let lock_tt = if img.perspective_locked {
                    "Perspective Lock is ON: resizing scales uniformly. Click to unlock 4-point perspective warp."
                } else {
                    "Perspective Lock is OFF: drag corners to correct perspective. Click to lock shape."
                };
                if child_ui.selectable_label(img.perspective_locked, lock_label).on_hover_text(lock_tt).clicked() {
                    img.perspective_locked = !img.perspective_locked;
                }

                if child_ui.button("Reset").on_hover_text("Reset perspective correction").clicked() {
                    action = PropertiesAction::ImageResetPerspective(single_id);
                }
            }
        } else if has_shape_stroke_wire {
            had_context = true;
            child_ui.label(RichText::new("Line:").size(11.0).color(Color32::from_rgb(130, 135, 150)));
            let widths = [1.5f32, 2.5f32, 4.0f32, 6.0f32];
            for &w in &widths {
                let is_cur = (*active_stroke_width - w).abs() < 0.1;
                let btn = child_ui.selectable_label(is_cur, format!("{}px", w as u32));
                if btn.clicked() {
                    *active_stroke_width = w;
                    for &id in selected_ids {
                        if let Some(BoardElement::Shape(s)) = board.get_element_mut(id) {
                            s.stroke_width = w;
                            s.recompute_aabb();
                        } else if let Some(BoardElement::Stroke(s)) = board.get_element_mut(id) {
                            s.width = w;
                            s.recompute_aabb();
                        } else if let Some(BoardElement::Wire(wire)) = board.get_element_mut(id) {
                            wire.width = w;
                        }
                    }
                }
            }
        }

        // 3. Separator (if had_context && show_colors)
        let show_colors = !is_doc && !is_pdf && !is_image;
        if had_context && show_colors {
            child_ui.separator();
        }

        // 4. Color Swatches Direct Selection (compact 16px circles with clean spacing)
        if show_colors {
            if is_table {
                let cur_c = *active_stroke_color;
                if let Some(col) = crate::ui::color_presets::render_color_presets_with_manager(
                    &mut child_ui,
                    popup_manager,
                    cur_c,
                    "table_text_presets",
                    None,
                ) {
                    *active_stroke_color = col;
                    action = PropertiesAction::TableApplyColor(selected_ids[0], col);
                }
            } else if has_text && !is_sticky {
                // Item 5: Text color presets: Black, White, Yellow, Red, Light Blue + rainbow picker
                // Highlight selected color. Existing text must NOT change color!
                let cur_c = *active_stroke_color;
                if let Some(col) = crate::ui::color_presets::render_color_presets_with_manager(
                    &mut child_ui,
                    popup_manager,
                    cur_c,
                    "text_color_presets",
                    None,
                ) {
                    *active_stroke_color = col;
                }
            } else {
                let current_color = if is_sticky { *active_sticky_color } else { *active_stroke_color };
                let quick_colors: &[(Color32, &str)] = if is_sticky {
                    &[
                        (Color32::from_rgb(255, 249, 177), "Yellow"),
                        (Color32::from_rgb(250, 197, 90), "Orange"),
                        (Color32::from_rgb(240, 138, 138), "Peach"),
                        (Color32::from_rgb(246, 147, 229), "Pink"),
                        (Color32::from_rgb(182, 207, 251), "Lavender"),
                        (Color32::from_rgb(146, 229, 252), "Sky Blue"),
                        (Color32::from_rgb(115, 229, 216), "Mint"),
                        (Color32::from_rgb(248, 249, 250), "White"),
                    ]
                } else {
                    &[
                        (Color32::from_rgb(45, 96, 255), "Brand Blue"),
                        (Color32::from_rgb(235, 55, 55), "Red"),
                        (Color32::from_rgb(40, 195, 85), "Green"),
                        (Color32::from_rgb(250, 190, 40), "Yellow"),
                        (Color32::from_rgb(175, 100, 245), "Purple"),
                        (Color32::from_rgb(24, 26, 32), "Dark Charcoal"),
                    ]
                };

                for &(col, name) in quick_colors {
                    let (sw_rect, sw_res) = child_ui.allocate_exact_size(Vec2::new(16.0, 16.0), egui::Sense::click());
                    let is_cur = current_color == col;
                    child_ui.painter().circle_filled(sw_rect.center(), 6.5, col);
                    if is_cur {
                        child_ui.painter().circle_stroke(
                            sw_rect.center(),
                            7.5,
                            Stroke::new(2.0_f32, Color32::from_rgb(45, 96, 255)),
                        );
                    } else {
                        child_ui.painter().circle_stroke(
                            sw_rect.center(),
                            6.5,
                            Stroke::new(1.0_f32, Color32::from_rgba_unmultiplied(0, 0, 0, 35)),
                        );
                    }

                    if sw_res.on_hover_text(name).clicked() {
                        if is_sticky {
                            *active_sticky_color = col;
                            for &id in selected_ids {
                                if let Some(BoardElement::Text(t)) = board.get_element_mut(id) {
                                    if t.is_sticky {
                                        t.sticky_color = col;
                                    }
                                }
                            }
                        } else {
                            *active_stroke_color = col;
                            for &id in selected_ids {
                                if let Some(BoardElement::Shape(s)) = board.get_element_mut(id) {
                                    s.stroke_color = col;
                                } else if let Some(BoardElement::Stroke(s)) = board.get_element_mut(id) {
                                    s.color = col;
                                } else if let Some(BoardElement::Wire(w)) = board.get_element_mut(id) {
                                    w.color = WireColor::Custom(col);
                                }
                            }
                        }
                    }
                }
            }
        }

        // 5. Separator before actions
        child_ui.separator();

        // 6. Actions: ⬆ Front, ⬇ Back, ⧉ Duplicate, 🗑 Delete
        if child_ui
            .button(RichText::new("⬆").size(12.0))
            .on_hover_text("Bring to Front")
            .clicked()
        {
            action = PropertiesAction::BringToFront;
        }

        if child_ui
            .button(RichText::new("⬇").size(12.0))
            .on_hover_text("Send to Back")
            .clicked()
        {
            action = PropertiesAction::SendToBack;
        }

        let dup_color = if dark_mode {
            Color32::WHITE
        } else {
            Color32::from_rgb(45, 96, 255)
        };
        if child_ui
            .button(RichText::new("⧉").size(12.5).color(dup_color))
            .on_hover_text(format!("Duplicate Selected ({})", crate::shortcuts::AppShortcut::duplicate_label()))
            .clicked()
        {
            action = PropertiesAction::DuplicateSelected;
        }

        let del_btn = child_ui.button(
            RichText::new("🗑")
                .size(12.5)
                .color(Color32::from_rgb(235, 60, 60)),
        );
        if del_btn.on_hover_text("Delete Selected (Del)").clicked() {
            action = PropertiesAction::DeleteSelected;
        }

        (action, Some(tb_rect))
    }
}


