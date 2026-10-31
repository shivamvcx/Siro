use crate::canvas::camera::Camera;
use crate::model::project::Project;
use egui::{epaint::StrokeKind, Color32, CornerRadius, Pos2, Rect, RichText, Stroke, Ui, Vec2};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq)]
pub enum TopBarAction {
    None,
    GoHome,
    NewProject,
    OpenProject,
    CloseProject,
    CloseProjectTab(usize),
    SwitchProjectTab(usize),
    RenameProject(String),
    ClearBoard,
    SaveProject,
    SaveProjectAs,
    ExportPng,
    ExportSvg,
    ExportBomCsv,
    ToggleTheme,
    TogglePlanner,
    Undo,
    Redo,
    ZoomIn,
    ZoomOut,
    ZoomReset,
    ZoomFit,
    SwitchBoard(Uuid),
    OpenCreateSheetModal,
    OpenRenameSheetModal(Uuid),
    CreateSheet(String),
    RenameSheet(Uuid, String),
    AddBoard,
    DeleteBoard(Uuid),
    DeleteCurrentProject,
    ToggleSmartSnapping,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SheetModalMode {
    Create,
    Rename(Uuid),
}

#[derive(Clone, Debug)]
pub struct SheetModalState {
    pub mode: SheetModalMode,
    pub buffer: String,
    pub error: Option<String>,
    pub focus_requested: bool,
}

pub struct TopBarState {
    pub sheet_modal: Option<SheetModalState>,
    pub hamburger_open: bool,
    pub renaming_board_id: Option<Uuid>,
    pub rename_buffer: String,
    pub is_editing_title: bool,
    pub title_buffer: String,
    pub logo_texture: Option<egui::TextureHandle>,
}

impl Default for TopBarState {
    fn default() -> Self {
        Self {
            sheet_modal: None,
            hamburger_open: false,
            renaming_board_id: None,
            rename_buffer: String::new(),
            is_editing_title: false,
            title_buffer: String::new(),
            logo_texture: None,
        }
    }
}

pub struct TopBar;

impl TopBar {
    /// Renders the unified top window tab strip (supporting multiple open board tabs)
    pub fn render_window_tab_strip(
        ui: &mut Ui,
        tabs: &[(&str, bool)], // (name, dirty)
        active_tab: Option<usize>, // None = "Boards" is active (Home Screen); Some(idx) = Tab idx is active
        state: &mut TopBarState,
        dark_mode: bool,
    ) -> TopBarAction {
        let mut action = TopBarAction::None;

        let strip_bg = if dark_mode {
            Color32::from_rgb(30, 32, 38)
        } else {
            Color32::from_rgb(242, 244, 247)
        };
        let strip_border = if dark_mode {
            Color32::from_rgb(45, 48, 58)
        } else {
            Color32::from_rgb(226, 229, 234)
        };
        let tab_active_bg = if dark_mode {
            Color32::from_rgb(42, 45, 56)
        } else {
            Color32::WHITE
        };
        let tab_active_text = if dark_mode {
            Color32::WHITE
        } else {
            Color32::from_rgb(20, 22, 28)
        };

        let available_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(available_rect, CornerRadius::ZERO, strip_bg);
        ui.painter().line_segment(
            [
                Pos2::new(available_rect.min.x, available_rect.max.y),
                Pos2::new(available_rect.max.x, available_rect.max.y),
            ],
            Stroke::new(1.0_f32, strip_border),
        );

        ui.horizontal(|ui| {
            ui.add_space(8.0);

            // 1. "Boards" Tab (Dashboard)
            let is_boards_active = active_tab.is_none();
            let (boards_rect, boards_res) = ui.allocate_exact_size(Vec2::new(88.0, 30.0), egui::Sense::click());
            let boards_hovered = boards_res.hovered();

            if is_boards_active {
                ui.painter().rect_filled(
                    boards_rect,
                    CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 },
                    tab_active_bg,
                );
                if !dark_mode {
                    ui.painter().rect_stroke(
                        boards_rect,
                        CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 },
                        Stroke::new(1.0_f32, strip_border),
                        StrokeKind::Middle,
                    );
                }
                ui.painter().text(
                    boards_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Boards",
                    egui::FontId::proportional(12.0),
                    tab_active_text,
                );
            } else {
                let boards_bg = if dark_mode {
                    if boards_hovered { Color32::from_rgb(48, 52, 62) } else { Color32::from_rgb(38, 41, 49) }
                } else {
                    if boards_hovered { Color32::from_rgb(230, 233, 240) } else { Color32::from_rgb(238, 240, 244) }
                };
                ui.painter().rect_filled(
                    boards_rect,
                    CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 },
                    boards_bg,
                );
                if !dark_mode {
                    ui.painter().rect_stroke(
                        boards_rect,
                        CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 },
                        Stroke::new(1.0_f32, strip_border),
                        StrokeKind::Middle,
                    );
                }
                ui.painter().text(
                    boards_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    "Boards",
                    egui::FontId::proportional(12.0),
                    if dark_mode { Color32::from_rgb(200, 205, 215) } else { Color32::from_rgb(60, 65, 80) },
                );
                if boards_res.clicked() {
                    action = TopBarAction::GoHome;
                }
            }

            ui.add_space(4.0);

            // 2. Open Board / Project Tabs
            for (idx, &(proj_name, is_dirty)) in tabs.iter().enumerate() {
                let is_active = active_tab == Some(idx);
                let dirty_star = if is_dirty { " •" } else { "" };
                let display_title = format!("{}{}", proj_name, dirty_star);

                let tab_w = (display_title.len() as f32 * 7.5 + 44.0).clamp(110.0, 240.0);
                let (tab_rect, tab_res) = ui.allocate_exact_size(Vec2::new(tab_w, 30.0), egui::Sense::click());

                if is_active {
                    // Elevated active tab styling
                    ui.painter().rect_filled(
                        tab_rect,
                        CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 },
                        tab_active_bg,
                    );
                    if !dark_mode {
                        ui.painter().rect_stroke(
                            tab_rect,
                            CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 },
                            Stroke::new(1.0_f32, strip_border),
                            StrokeKind::Middle,
                        );
                    }

                    // Inline Tab Rename Editor or Title Text
                    if state.is_editing_title {
                        let edit_rect = tab_rect.shrink2(Vec2::new(8.0, 4.0));
                        let mut child = ui.new_child(
                            egui::UiBuilder::new()
                                .max_rect(edit_rect)
                                .layout(egui::Layout::left_to_right(egui::Align::Center)),
                        );
                        let edit_res = child.add(
                            egui::TextEdit::singleline(&mut state.title_buffer)
                                .desired_width(tab_w - 30.0),
                        );
                        if edit_res.lost_focus() || ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                            if !state.title_buffer.trim().is_empty() {
                                action = TopBarAction::RenameProject(state.title_buffer.trim().to_string());
                            }
                            state.is_editing_title = false;
                        } else if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                            state.is_editing_title = false;
                        }
                    } else {
                        let title_pos = tab_rect.left_center() + Vec2::new(10.0, 0.0);
                        ui.painter().text(
                            title_pos,
                            egui::Align2::LEFT_CENTER,
                            display_title,
                            egui::FontId::proportional(12.0),
                            tab_active_text,
                        );

                        // Close Tab Button (✕)
                        let close_rect = Rect::from_center_size(
                            Pos2::new(tab_rect.max.x - 14.0, tab_rect.center().y),
                            Vec2::new(16.0, 16.0),
                        );
                        let close_res = ui.allocate_rect(close_rect, egui::Sense::click());
                        if close_res.hovered() {
                            let circle_col = if dark_mode {
                                Color32::from_rgba_unmultiplied(200, 200, 200, 60)
                            } else {
                                Color32::from_rgba_unmultiplied(0, 0, 0, 25)
                            };
                            ui.painter().circle_filled(close_rect.center(), 7.0, circle_col);
                        }
                        ui.painter().text(
                            close_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "✕",
                            egui::FontId::proportional(9.5),
                            tab_active_text,
                        );

                        if close_res.clicked() {
                            action = TopBarAction::CloseProjectTab(idx);
                        } else if tab_res.double_clicked() {
                            state.is_editing_title = true;
                            state.title_buffer = proj_name.to_string();
                        }
                    }
                } else {
                    // Inactive tab styling
                    let tab_hovered = tab_res.hovered();
                    let tab_bg = if dark_mode {
                        if tab_hovered { Color32::from_rgb(44, 48, 58) } else { Color32::from_rgb(38, 41, 49) }
                    } else {
                        if tab_hovered { Color32::from_rgb(232, 235, 242) } else { Color32::from_rgb(238, 240, 244) }
                    };
                    ui.painter().rect_filled(
                        tab_rect,
                        CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 },
                        tab_bg,
                    );
                    if !dark_mode {
                        ui.painter().rect_stroke(
                            tab_rect,
                            CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 },
                            Stroke::new(1.0_f32, strip_border),
                            StrokeKind::Middle,
                        );
                    }

                    let text_color = if dark_mode {
                        Color32::from_rgb(175, 180, 195)
                    } else {
                        Color32::from_rgb(90, 95, 110)
                    };

                    let title_pos = tab_rect.left_center() + Vec2::new(10.0, 0.0);
                    ui.painter().text(
                        title_pos,
                        egui::Align2::LEFT_CENTER,
                        display_title,
                        egui::FontId::proportional(12.0),
                        text_color,
                    );

                    // Close Tab Button (✕)
                    let close_rect = Rect::from_center_size(
                        Pos2::new(tab_rect.max.x - 14.0, tab_rect.center().y),
                        Vec2::new(16.0, 16.0),
                    );
                    let close_res = ui.allocate_rect(close_rect, egui::Sense::click());
                    if close_res.hovered() {
                        let circle_col = if dark_mode {
                            Color32::from_rgba_unmultiplied(200, 200, 200, 60)
                        } else {
                            Color32::from_rgba_unmultiplied(0, 0, 0, 25)
                        };
                        ui.painter().circle_filled(close_rect.center(), 7.0, circle_col);
                    }
                    ui.painter().text(
                        close_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "✕",
                        egui::FontId::proportional(9.5),
                        text_color,
                    );

                    if close_res.clicked() {
                        action = TopBarAction::CloseProjectTab(idx);
                    } else if tab_res.clicked() {
                        action = TopBarAction::SwitchProjectTab(idx);
                    }
                }

                ui.add_space(4.0);
            }

            // 3. "+" Create New Board Button (Consistent across both views!)
            let (add_rect, add_res) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), egui::Sense::click());
            if add_res.hovered() {
                ui.painter().rect_filled(
                    add_rect,
                    CornerRadius::same(5),
                    if dark_mode { Color32::from_rgb(50, 54, 65) } else { Color32::from_rgb(230, 233, 240) },
                );
            }
            ui.painter().text(
                add_rect.center(),
                egui::Align2::CENTER_CENTER,
                "+",
                egui::FontId::proportional(15.0),
                if dark_mode { Color32::from_rgb(180, 185, 200) } else { Color32::from_rgb(90, 95, 110) },
            );
            if add_res.on_hover_text("Create New Board").clicked() {
                action = TopBarAction::NewProject;
            }

            // Right-aligned Window Controls (clean minimal theme toggle)
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.add_space(8.0);

                let theme_icon = if dark_mode { "☀ Light" } else { "🌙 Dark" };
                let theme_btn = egui::Button::new(
                    RichText::new(theme_icon)
                        .size(11.5)
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) }),
                )
                .fill(if dark_mode { Color32::from_rgb(44, 48, 58) } else { Color32::WHITE })
                .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(60, 65, 80) } else { Color32::from_rgb(220, 224, 232) }))
                .corner_radius(CornerRadius::same(6));
                if ui.add(theme_btn).on_hover_text(if dark_mode { "Switch to Light Mode" } else { "Switch to Dark Mode" }).clicked() {
                    action = TopBarAction::ToggleTheme;
                }
            });
        });

        action
    }

    /// Convenience wrapper for rendering single board editor top strip
    pub fn ui(
        ui: &mut Ui,
        project: &mut Project,
        _camera: &Camera,
        state: &mut TopBarState,
        _planner_open: bool,
        dark_mode: bool,
    ) -> TopBarAction {
        let tabs = [(project.display_name(), project.is_unsaved())];
        let action = Self::render_window_tab_strip(ui, &tabs, Some(0), state, dark_mode);
        if let TopBarAction::RenameProject(ref new_name) = action {
            project.manifest.name = new_name.clone();
            project.mark_edited();
        }
        action
    }

    /// Renders the floating Miro navigation capsule header on the canvas
    pub fn render_floating_header(
        ui: &mut Ui,
        project: &mut Project,
        state: &mut TopBarState,
        planner_open: bool,
        dark_mode: bool,
        canvas_min: Pos2,
    ) -> (TopBarAction, Rect) {
        let mut action = TopBarAction::None;

        let header_pos = canvas_min + Vec2::new(18.0, 14.0);
        let title_len = project.display_name().len().max(6) as f32;
        let title_w = (title_len * 8.0).clamp(90.0, 240.0);
        let sheet_extra_w = 80.0_f32;
        let header_w = (360.0 + title_w + sheet_extra_w).clamp(530.0, 680.0);
        let header_size = Vec2::new(header_w, 40.0);
        let header_rect = Rect::from_min_size(header_pos, header_size);

        let bg_color = if dark_mode {
            Color32::from_rgb(30, 33, 42)
        } else {
            Color32::WHITE
        };
        let border_color = if dark_mode {
            Color32::from_rgb(52, 56, 70)
        } else {
            Color32::from_rgb(224, 228, 236)
        };

        // Multi-pass soft drop shadow
        let p = ui.painter();
        p.rect_filled(
            header_rect.translate(Vec2::new(0.0, 2.0)),
            CornerRadius::same(20),
            Color32::from_rgba_unmultiplied(0, 0, 0, if dark_mode { 28 } else { 16 }),
        );
        p.rect_filled(
            header_rect.translate(Vec2::new(0.0, 4.0)),
            CornerRadius::same(20),
            Color32::from_rgba_unmultiplied(0, 0, 0, if dark_mode { 16 } else { 8 }),
        );
        p.rect_filled(header_rect, CornerRadius::same(20), bg_color);
        p.rect_stroke(header_rect, CornerRadius::same(20), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

        // Header child layout in exact left-to-right order:
        // 1. ☰ Hamburger Menu Button (vector drawn)
        // 2. siro Logo
        // 3. Project Avatar / Badge (purple 8-bit invader)
        // 4. Board Title (editable inline)
        // 5. Sheet Switcher Pill
        // 6. More Options (⋮)
        // 7. Export / Share (⤒)
        // 8. Status Pill (Local • Saved ▾)
        // 9. STEM Planner Button
        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(header_rect.shrink2(Vec2::new(10.0, 4.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );
        child_ui.ctx().style_mut(|s| s.spacing.menu_width = 180.0);
        child_ui.spacing_mut().item_spacing = Vec2::new(6.0, 0.0);
        child_ui.spacing_mut().button_padding = Vec2::new(4.0, 3.0);
        child_ui.spacing_mut().menu_width = 180.0;
        child_ui.visuals_mut().widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
        child_ui.visuals_mut().widgets.inactive.bg_stroke = Stroke::NONE;
        child_ui.visuals_mut().widgets.hovered.weak_bg_fill = if dark_mode { Color32::from_rgb(46, 52, 66) } else { Color32::from_rgb(240, 243, 249) };
        child_ui.visuals_mut().widgets.hovered.corner_radius = CornerRadius::same(6);

        // 1. Hamburger Menu Button (crisp vector painter icon with 32x32 click target)
        let (btn_rect, btn_res) = child_ui.allocate_exact_size(Vec2::new(32.0, 32.0), egui::Sense::click());
        let bar_id = child_ui.id();
        let mut bar_state = egui::menu::BarState::load(child_ui.ctx(), bar_id);

        let menu_inner = bar_state.bar_menu(&btn_res, |ui| {
            ui.spacing_mut().button_padding = Vec2::new(8.0, 4.0);
            if ui.button(crate::shortcuts::AppShortcut::NewProject.menu_label()).clicked() {
                action = TopBarAction::NewProject;
                ui.close_menu();
            }
            if ui.button(crate::shortcuts::AppShortcut::OpenProject.menu_label()).clicked() {
                action = TopBarAction::OpenProject;
                ui.close_menu();
            }
            if ui.button(crate::shortcuts::AppShortcut::SaveProject.menu_label()).clicked() {
                action = TopBarAction::SaveProject;
                ui.close_menu();
            }
            if ui.button(crate::shortcuts::AppShortcut::SaveProjectAs.menu_label()).clicked() {
                action = TopBarAction::SaveProjectAs;
                ui.close_menu();
            }
            ui.separator();
            ui.menu_button("📤  Export", |ui| {
                if ui.button("PNG Image (.png)").clicked() {
                    action = TopBarAction::ExportPng;
                    ui.close_menu();
                }
                if ui.button("Vector SVG (.svg)").clicked() {
                    action = TopBarAction::ExportSvg;
                    ui.close_menu();
                }
                if ui.button("BOM Spreadsheet (.csv)").clicked() {
                    action = TopBarAction::ExportBomCsv;
                    ui.close_menu();
                }
            });
            ui.separator();
            if ui.button(if dark_mode { "☀  Switch to Light Mode" } else { "🌙  Switch to Dark Mode" }).clicked() {
                action = TopBarAction::ToggleTheme;
                ui.close_menu();
            }
            if ui.button("🧲  Toggle Smart Snapping").clicked() {
                action = TopBarAction::ToggleSmartSnapping;
                ui.close_menu();
            }
            if project.filepath.is_some() {
                ui.separator();
                if ui.button(RichText::new("🗑  Delete Project File...").color(Color32::from_rgb(239, 68, 68))).clicked() {
                    action = TopBarAction::DeleteCurrentProject;
                    ui.close_menu();
                }
            }
        });

        let is_menu_open = menu_inner.is_some();
        state.hamburger_open = is_menu_open;
        bar_state.store(child_ui.ctx(), bar_id);

        // Vector hamburger visual states (inactive, hovered, pressed, menu open)
        let btn_bg = if is_menu_open {
            if dark_mode { Color32::from_rgb(46, 52, 66) } else { Color32::from_rgb(235, 239, 248) }
        } else if btn_res.is_pointer_button_down_on() {
            if dark_mode { Color32::from_rgb(38, 43, 56) } else { Color32::from_rgb(228, 232, 242) }
        } else if btn_res.hovered() {
            if dark_mode { Color32::from_rgb(46, 52, 66) } else { Color32::from_rgb(240, 243, 249) }
        } else {
            Color32::TRANSPARENT
        };

        if btn_bg != Color32::TRANSPARENT {
            child_ui.painter().rect_filled(btn_rect, CornerRadius::same(6), btn_bg);
        }
        if is_menu_open {
            child_ui.painter().rect_stroke(
                btn_rect,
                CornerRadius::same(6),
                Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(70, 75, 90) } else { Color32::from_rgb(200, 210, 228) }),
                StrokeKind::Inside,
            );
        }

        let icon_color = if dark_mode {
            if is_menu_open || btn_res.hovered() {
                Color32::WHITE
            } else {
                Color32::from_rgb(200, 205, 220)
            }
        } else if is_menu_open || btn_res.hovered() {
            Color32::from_rgb(15, 20, 30)
        } else {
            Color32::from_rgb(50, 55, 70)
        };

        // Render 3 crisp vector lines (equal bar length 16px, equal thickness 2.0px, equal spacing 5px)
        let cx = btn_rect.center().x.round();
        let cy = btn_rect.center().y.round();
        let bar_w = 16.0_f32;
        let bar_h = 2.0_f32;
        for offset_y in [-5.0_f32, 0.0_f32, 5.0_f32] {
            let line_rect = Rect::from_center_size(Pos2::new(cx, (cy + offset_y).round()), Vec2::new(bar_w, bar_h));
            child_ui.painter().rect_filled(line_rect, CornerRadius::ZERO, icon_color);
        }
        btn_res.on_hover_text("Main Menu");

        // 2. Logo: official Siro icon + application brand in bold lowercase sans-serif typography
        let logo_color = if dark_mode {
            Color32::WHITE
        } else {
            Color32::from_rgb(5, 0, 56) // #050038
        };
        let tex = state.logo_texture.get_or_insert_with(|| {
            let img = image::load_from_memory(include_bytes!("../../assets/icon_256.png"))
                .expect("Embedded icon must load")
                .to_rgba8();
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [img.width() as usize, img.height() as usize],
                img.as_raw(),
            );
            child_ui.ctx().load_texture("siro_topbar_logo", color_image, egui::TextureOptions::LINEAR)
        });
        let (icon_rect, _) = child_ui.allocate_exact_size(Vec2::new(18.0, 18.0), egui::Sense::hover());
        child_ui.painter().image(
            tex.id(),
            icon_rect,
            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
            Color32::WHITE,
        );
        child_ui.label(RichText::new(crate::APP_NAME.to_lowercase()).size(16.0).strong().color(logo_color));

        // 3. Project Avatar / Badge: purple retro 8-bit invader/chip badge with cute pixel eyes (18x18px)
        let (avatar_rect, avatar_res) = child_ui.allocate_exact_size(Vec2::new(18.0, 18.0), egui::Sense::hover());
        child_ui.painter().rect_filled(avatar_rect, CornerRadius::same(5), Color32::from_rgb(124, 58, 237));
        // Pixel eyes & pupils
        let eye_y = avatar_rect.center().y - 1.0;
        let left_eye = Rect::from_center_size(Pos2::new(avatar_rect.center().x - 3.5, eye_y), Vec2::new(2.8, 3.2));
        let right_eye = Rect::from_center_size(Pos2::new(avatar_rect.center().x + 3.5, eye_y), Vec2::new(2.8, 3.2));
        child_ui.painter().rect_filled(left_eye, CornerRadius::ZERO, Color32::WHITE);
        child_ui.painter().rect_filled(right_eye, CornerRadius::ZERO, Color32::WHITE);
        let left_pupil = Rect::from_center_size(Pos2::new(avatar_rect.center().x - 3.2, eye_y + 0.5), Vec2::new(1.4, 1.6));
        let right_pupil = Rect::from_center_size(Pos2::new(avatar_rect.center().x + 3.8, eye_y + 0.5), Vec2::new(1.4, 1.6));
        child_ui.painter().rect_filled(left_pupil, CornerRadius::ZERO, Color32::BLACK);
        child_ui.painter().rect_filled(right_pupil, CornerRadius::ZERO, Color32::BLACK);
        // Cute invader mouth
        let mouth_rect = Rect::from_center_size(Pos2::new(avatar_rect.center().x, avatar_rect.center().y + 3.5), Vec2::new(4.0, 1.4));
        child_ui.painter().rect_filled(mouth_rect, CornerRadius::ZERO, Color32::from_rgb(220, 205, 255));
        avatar_res.on_hover_text("Project Avatar");

        // 4. Board Title: Clean, bold title clickable to edit in-place
        if state.is_editing_title {
            let edit_res = child_ui.add(
                egui::TextEdit::singleline(&mut state.title_buffer).desired_width(title_w),
            );
            if child_ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                state.is_editing_title = false;
            } else if edit_res.lost_focus() || child_ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let clean_title = state.title_buffer.trim().to_string();
                if !clean_title.is_empty() {
                    action = TopBarAction::RenameProject(clean_title);
                }
                state.is_editing_title = false;
            }
        } else {
            let title_text = RichText::new(project.display_name())
                .size(13.5)
                .strong()
                .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(18, 22, 32) });
            let title_btn = child_ui.add(egui::Button::new(title_text).frame(false));
            if title_btn.on_hover_text("Click to Rename Project").clicked() {
                state.is_editing_title = true;
                state.title_buffer = project.display_name().to_string();
            }
        }

        // 5. Sheet Switcher Pill in Floating Header (always accessible)
        child_ui.add_space(2.0);
        let active_name = project.boards.get(&project.active_board_id).map(|b| b.name.as_str()).unwrap_or("Sheet");
        let truncated_pill_name = crate::model::sheet_ops::truncate_with_ellipsis(active_name, 16);
        let pill_text = RichText::new(format!("📑 {} ▾", truncated_pill_name))
            .size(11.5)
            .color(if dark_mode { Color32::from_rgb(170, 200, 255) } else { Color32::from_rgb(45, 96, 255) });
        let pill_resp = child_ui.menu_button(pill_text, |ui| {
            ui.spacing_mut().button_padding = Vec2::new(8.0, 4.0);
            ui.label(RichText::new("Schematic Sheets:").size(11.0).color(Color32::GRAY));
            ui.add_space(2.0);

            // Responsive popup width: dynamically scales based on sheet names
            // so normal and longer names (e.g. "Main Schematic", "Power Management") fit without wrapping.
            let longest_name_len = project
                .manifest
                .board_order
                .iter()
                .filter_map(|bid| project.boards.get(bid))
                .map(|b| b.name.chars().count())
                .max()
                .unwrap_or(10);
            let target_menu_width = ((longest_name_len as f32) * 7.5 + 64.0).clamp(210.0, 300.0);
            ui.set_min_width(target_menu_width);

            let font_id = egui::FontId::proportional(12.5);
            let gutter_w = 22.0;

            egui::ScrollArea::vertical()
                .max_height(240.0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    let avail_w = ui.available_width();
                    let text_wrap_w = (avail_w - gutter_w - 6.0).max(100.0);

                    for &bid in &project.manifest.board_order {
                        let bname = project.boards.get(&bid).map(|b| b.name.as_str()).unwrap_or("Sheet");
                        let is_active = project.active_board_id == bid;

                        let text_color = if dark_mode {
                            if is_active { Color32::WHITE } else { Color32::from_rgb(205, 210, 225) }
                        } else {
                            if is_active { Color32::from_rgb(20, 25, 35) } else { Color32::from_rgb(60, 65, 80) }
                        };

                        // Wrap text cleanly at text_wrap_w: identical available width whether active or not!
                        let galley = ui.painter().layout(
                            bname.to_string(),
                            font_id.clone(),
                            text_color,
                            text_wrap_w,
                        );

                        let row_h = (galley.size().y + 10.0).max(28.0);
                        let (row_rect, row_resp) = ui.allocate_exact_size(Vec2::new(avail_w, row_h), egui::Sense::click());

                        let is_hovered = row_resp.hovered();
                        if is_hovered {
                            let hover_bg = if dark_mode {
                                Color32::from_rgba_unmultiplied(255, 255, 255, 18)
                            } else {
                                Color32::from_rgba_unmultiplied(0, 0, 0, 12)
                            };
                            ui.painter().rect_filled(row_rect, CornerRadius::same(4), hover_bg);
                        } else if is_active {
                            let active_bg = if dark_mode {
                                Color32::from_rgba_unmultiplied(100, 160, 255, 22)
                            } else {
                                Color32::from_rgba_unmultiplied(35, 95, 235, 14)
                            };
                            ui.painter().rect_filled(row_rect, CornerRadius::same(4), active_bg);
                        }

                        // Left gutter: checkmark icon if active (always same gutter width)
                        let text_y = row_rect.min.y + (row_h - galley.size().y) * 0.5;
                        if is_active {
                            let check_col = if dark_mode {
                                Color32::from_rgb(100, 160, 255)
                            } else {
                                Color32::from_rgb(35, 95, 235)
                            };
                            let check_y = if let Some(first_row) = galley.rows.first() {
                                text_y + first_row.rect.center().y
                            } else {
                                row_rect.center().y
                            };
                            let check_pos = Pos2::new(row_rect.min.x + 6.0, check_y);
                            ui.painter().text(
                                check_pos,
                                egui::Align2::LEFT_CENTER,
                                "✓",
                                egui::FontId::proportional(12.0),
                                check_col,
                            );
                        }

                        // Text position: starts at row_rect.min.x + gutter_w
                        let text_pos = Pos2::new(row_rect.min.x + gutter_w, text_y);
                        ui.painter().galley(text_pos, galley, text_color);

                        let row_resp = row_resp.on_hover_text(format!("{}\n(Double-click to rename)", bname));
                        if row_resp.double_clicked() {
                            action = TopBarAction::OpenRenameSheetModal(bid);
                            ui.close_menu();
                        } else if row_resp.clicked() {
                            action = TopBarAction::SwitchBoard(bid);
                        }
                    }
                });

            ui.add_space(2.0);
            ui.separator();
            ui.add_space(2.0);

            let add_btn = egui::Button::new(
                RichText::new("+ Add New Sheet")
                    .size(12.0)
                    .color(if dark_mode { Color32::from_rgb(170, 200, 255) } else { Color32::from_rgb(35, 95, 235) }),
            );
            if ui.add_sized(Vec2::new(ui.available_width(), 24.0), add_btn).clicked() {
                action = TopBarAction::OpenCreateSheetModal;
                ui.close_menu();
            }
        });
        pill_resp.response.on_hover_text(format!("Active Sheet: {}", active_name));

        // 6. More Options (⋮)
        let opt_color = if dark_mode { Color32::from_rgb(180, 185, 200) } else { Color32::from_rgb(60, 65, 80) };
        child_ui.menu_button(RichText::new("⋮").size(14.0).strong().color(opt_color), |ui| {
            ui.spacing_mut().button_padding = Vec2::new(8.0, 4.0);
            if ui.button("✏  Rename Sheet").clicked() {
                action = TopBarAction::OpenRenameSheetModal(project.active_board_id);
                ui.close_menu();
            }
            if ui.button("✏  Rename Project").clicked() {
                state.is_editing_title = true;
                state.title_buffer = project.display_name().to_string();
                ui.close_menu();
            }
            if ui.button("🗑  Clear Board").clicked() {
                action = TopBarAction::ClearBoard;
                ui.close_menu();
            }
            if project.filepath.is_some() {
                if ui.button(RichText::new("🗑  Delete Project File...").color(Color32::from_rgb(239, 68, 68))).clicked() {
                    action = TopBarAction::DeleteCurrentProject;
                    ui.close_menu();
                }
            }
            ui.separator();
            if ui.button("+  Add New Sheet").clicked() {
                action = TopBarAction::OpenCreateSheetModal;
                ui.close_menu();
            }
        });

        // 7. Export / Share (⤒)
        child_ui.menu_button(RichText::new("⤒").size(14.5).strong().color(opt_color), |ui| {
            if ui.button("PNG Image Snapshot (.png)").clicked() {
                action = TopBarAction::ExportPng;
                ui.close_menu();
            }
            if ui.button("Vector SVG Drawing (.svg)").clicked() {
                action = TopBarAction::ExportSvg;
                ui.close_menu();
            }
            if ui.button("BOM Spreadsheet (.csv)").clicked() {
                action = TopBarAction::ExportBomCsv;
                ui.close_menu();
            }
        });

        // 8. Status Pill: A soft rounded pill "Local • Saved" or "Local • Edited" (soft grey background #f0f2f5)
        let pill_bg = if dark_mode {
            Color32::from_rgb(42, 46, 58)
        } else {
            Color32::from_rgb(240, 242, 245) // #f0f2f5
        };
        let is_unsaved = project.is_unsaved();
        let status_label = if is_unsaved { "Local • Edited" } else { "Local • Saved" };
        let status_color = if is_unsaved {
            if dark_mode { Color32::from_rgb(245, 175, 45) } else { Color32::from_rgb(202, 105, 5) }
        } else {
            if dark_mode { Color32::from_rgb(170, 175, 190) } else { Color32::from_rgb(85, 90, 105) }
        };
        let (pill_rect, pill_res) = child_ui.allocate_exact_size(Vec2::new(88.0, 24.0), egui::Sense::click());
        child_ui.painter().rect_filled(pill_rect, CornerRadius::same(12), pill_bg);
        if pill_res.hovered() {
            child_ui.painter().rect_stroke(
                pill_rect,
                CornerRadius::same(12),
                Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(70, 75, 90) } else { Color32::from_rgb(215, 218, 226) }),
                StrokeKind::Inside,
            );
        }
        child_ui.painter().text(
            pill_rect.center(),
            egui::Align2::CENTER_CENTER,
            status_label,
            egui::FontId::proportional(10.5),
            status_color,
        );
        if pill_res.on_hover_text(format!("Offline Local Project • Click to Save ({})", crate::shortcuts::AppShortcut::SaveProject.label())).clicked() {
            if is_unsaved {
                action = TopBarAction::SaveProject;
            }
        }

        // 9. STEM Planner Button: Sleek toggle button
        let pl_bg = if planner_open {
            if dark_mode { Color32::from_rgb(38, 52, 85) } else { Color32::from_rgb(235, 242, 255) }
        } else if dark_mode {
            Color32::from_rgb(40, 44, 54)
        } else {
            Color32::from_rgb(245, 246, 249)
        };
        let pl_stroke_col = if planner_open {
            Color32::from_rgb(45, 96, 255)
        } else if dark_mode {
            Color32::from_rgb(60, 65, 80)
        } else {
            Color32::from_rgb(220, 224, 232)
        };
        let pl_text_col = if planner_open {
            Color32::from_rgb(45, 96, 255)
        } else if dark_mode {
            Color32::from_rgb(200, 205, 220)
        } else {
            Color32::from_rgb(50, 55, 70)
        };

        let pl_btn = egui::Button::new(
            RichText::new("📋 Planner").size(11.5).color(pl_text_col),
        )
        .fill(pl_bg)
        .stroke(Stroke::new(1.0_f32, pl_stroke_col))
        .corner_radius(CornerRadius::same(12));

        if child_ui.add(pl_btn).on_hover_text("Toggle STEM Planner").clicked() {
            action = TopBarAction::TogglePlanner;
        }


        // Render Sheet Modal dialog (Add New Sheet / Rename Sheet) if active
        let modal_action = Self::render_sheet_modal(ui, project, state, dark_mode);
        if modal_action != TopBarAction::None {
            action = modal_action;
        }

        (action, header_rect)
    }

    /// Renders the sheet create/rename modal dialog
    pub fn render_sheet_modal(
        ui: &mut Ui,
        project: &Project,
        state: &mut TopBarState,
        dark_mode: bool,
    ) -> TopBarAction {
        let mut action = TopBarAction::None;
        let modal = match &mut state.sheet_modal {
            Some(m) => m,
            None => return TopBarAction::None,
        };

        let screen_rect = ui.ctx().screen_rect();

        // Centered dialog card
        let dialog_w = 380.0;
        let dialog_h = if modal.error.is_some() { 165.0 } else { 145.0 };
        let dialog_rect = Rect::from_center_size(screen_rect.center(), Vec2::new(dialog_w, dialog_h));

        // Semi-transparent backdrop capturing clicks to dismiss
        ui.painter().rect_filled(
            screen_rect,
            CornerRadius::ZERO,
            Color32::from_black_alpha(if dark_mode { 150 } else { 100 }),
        );
        let backdrop_res = ui.allocate_rect(screen_rect, egui::Sense::click());
        if backdrop_res.clicked() {
            let pointer_pos = ui.input(|i| i.pointer.interact_pos()).unwrap_or(Pos2::ZERO);
            if !dialog_rect.contains(pointer_pos) {
                state.sheet_modal = None;
                return TopBarAction::None;
            }
        }

        let card_bg = if dark_mode {
            Color32::from_rgb(30, 33, 42)
        } else {
            Color32::WHITE
        };
        let card_border = if dark_mode {
            Color32::from_rgb(52, 56, 70)
        } else {
            Color32::from_rgb(220, 224, 232)
        };

        ui.painter().rect_filled(
            dialog_rect.translate(Vec2::new(0.0, 4.0)),
            CornerRadius::same(10),
            Color32::from_black_alpha(40),
        );
        ui.painter().rect_filled(dialog_rect, CornerRadius::same(10), card_bg);
        ui.painter().rect_stroke(
            dialog_rect,
            CornerRadius::same(10),
            Stroke::new(1.0_f32, card_border),
            StrokeKind::Middle,
        );

        let mut card_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(dialog_rect.shrink(16.0))
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );

        let title_text = match modal.mode {
            SheetModalMode::Create => "Add New Sheet",
            SheetModalMode::Rename(_) => "Rename Sheet",
        };

        card_ui.label(
            RichText::new(title_text)
                .size(15.0)
                .strong()
                .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) }),
        );
        card_ui.add_space(8.0);

        let edit_res = card_ui.add(
            egui::TextEdit::singleline(&mut modal.buffer)
                .desired_width(dialog_w - 32.0)
                .hint_text("Enter sheet name..."),
        );

        if modal.focus_requested {
            edit_res.request_focus();
            let mut text_state = egui::text_edit::TextEditState::load(card_ui.ctx(), edit_res.id).unwrap_or_default();
            text_state.cursor.set_char_range(Some(egui::text::CCursorRange::two(
                egui::text::CCursor::default(),
                egui::text::CCursor::new(modal.buffer.chars().count()),
            )));
            text_state.store(card_ui.ctx(), edit_res.id);
            modal.focus_requested = false;
        }

        if let Some(ref err) = modal.error {
            card_ui.add_space(4.0);
            card_ui.label(RichText::new(err).size(11.5).color(Color32::from_rgb(239, 68, 68)));
        }

        card_ui.add_space(10.0);

        let mut confirmed = false;
        let mut cancelled = false;

        if card_ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            cancelled = true;
        } else if edit_res.lost_focus() && card_ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            confirmed = true;
        } else if card_ui.input(|i| i.key_pressed(egui::Key::Enter)) {
            confirmed = true;
        }

        card_ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let confirm_btn_label = match modal.mode {
                    SheetModalMode::Create => "Create Sheet",
                    SheetModalMode::Rename(_) => "Save Name",
                };
                let confirm_btn = egui::Button::new(
                    RichText::new(confirm_btn_label).size(12.0).color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(45, 96, 255))
                .corner_radius(CornerRadius::same(6));

                if ui.add(confirm_btn).clicked() {
                    confirmed = true;
                }

                ui.add_space(6.0);

                let cancel_btn = egui::Button::new(
                    RichText::new("Cancel").size(12.0).color(if dark_mode { Color32::from_rgb(190, 195, 205) } else { Color32::from_rgb(80, 85, 95) }),
                )
                .fill(if dark_mode { Color32::from_rgb(42, 46, 56) } else { Color32::from_rgb(240, 242, 246) })
                .corner_radius(CornerRadius::same(6));

                if ui.add(cancel_btn).clicked() {
                    cancelled = true;
                }
            });
        });

        if cancelled {
            state.sheet_modal = None;
        } else if confirmed {
            let existing = project.sheet_names();
            let cur_name = match modal.mode {
                SheetModalMode::Create => None,
                SheetModalMode::Rename(id) => project.boards.get(&id).map(|b| b.name.as_str()),
            };

            match crate::model::sheet_ops::validate_sheet_name(&modal.buffer, &existing, cur_name) {
                Ok(valid_name) => {
                    match modal.mode {
                        SheetModalMode::Create => {
                            action = TopBarAction::CreateSheet(valid_name);
                        }
                        SheetModalMode::Rename(id) => {
                            action = TopBarAction::RenameSheet(id, valid_name);
                        }
                    }
                    state.sheet_modal = None;
                }
                Err(err_msg) => {
                    modal.error = Some(err_msg);
                }
            }
        }

        action
    }

    /// Renders the dark top window tab strip on the Miro Home Dashboard
    pub fn render_home_strip(
        ui: &mut Ui,
        open_project_name: Option<(&str, bool)>,
        dark_mode: bool,
    ) -> TopBarAction {
        let tabs: Vec<(&str, bool)> = open_project_name.into_iter().collect();
        let mut dummy_state = TopBarState::default();
        let action = Self::render_window_tab_strip(ui, &tabs, None, &mut dummy_state, dark_mode);
        match action {
            TopBarAction::CloseProjectTab(_) => TopBarAction::CloseProject,
            TopBarAction::SwitchProjectTab(_) => TopBarAction::SwitchBoard(Uuid::nil()),
            other => other,
        }
    }

    /// Renders the floating sheet switcher bar at the bottom-left of the canvas
    pub fn render_sheet_bar(
        ui: &mut Ui,
        project: &mut Project,
        state: &mut TopBarState,
        dark_mode: bool,
        canvas_rect: Rect,
    ) -> (TopBarAction, Option<Rect>) {
        let mut action = TopBarAction::None;
        let board_ids = project.manifest.board_order.clone();

        // Calculate needed width
        let sheet_count = board_ids.len();
        let bar_h = 32.0;
        let mut total_content_w = 40.0; // left label / icon + padding
        for &bid in &board_ids {
            let name = project.boards.get(&bid).map(|b| b.name.as_str()).unwrap_or("Sheet");
            let tab_w = (name.len() as f32 * 7.0 + 36.0).clamp(70.0, 160.0);
            total_content_w += tab_w + 4.0;
        }
        total_content_w += 32.0; // "+" button + padding

        let bar_w = total_content_w.clamp(140.0, 600.0);
        let bar_pos = Pos2::new(
            canvas_rect.min.x + 18.0,
            canvas_rect.max.y - bar_h - 18.0,
        );
        let bar_rect = Rect::from_min_size(bar_pos, Vec2::new(bar_w, bar_h));

        let bg_color = if dark_mode {
            Color32::from_rgb(30, 33, 42)
        } else {
            Color32::WHITE
        };
        let border_color = if dark_mode {
            Color32::from_rgb(52, 56, 70)
        } else {
            Color32::from_rgb(224, 228, 236)
        };

        // Multi-pass drop shadow
        let p = ui.painter();
        p.rect_filled(
            bar_rect.translate(Vec2::new(0.0, 2.0)),
            CornerRadius::same(8),
            if dark_mode { Color32::from_rgba_unmultiplied(0, 0, 0, 70) } else { Color32::from_rgba_unmultiplied(0, 0, 0, 16) },
        );
        p.rect_filled(bar_rect, CornerRadius::same(8), bg_color);
        p.rect_stroke(bar_rect, CornerRadius::same(8), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

        let mut child_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(bar_rect.shrink2(Vec2::new(6.0, 3.0)))
                .layout(egui::Layout::left_to_right(egui::Align::Center)),
        );

        // Icon/Label
        child_ui.label(RichText::new("📑").size(12.0));
        child_ui.add_space(2.0);

        // Sheet tabs
        for &board_id in &board_ids {
            let is_active = project.active_board_id == board_id;
            let board_name = project
                .boards
                .get(&board_id)
                .map(|b| b.name.as_str())
                .unwrap_or("Sheet");

            let tab_w = (board_name.len() as f32 * 7.0 + 36.0).clamp(70.0, 160.0);
            let (tab_rect, tab_res) = child_ui.allocate_exact_size(Vec2::new(tab_w, 24.0), egui::Sense::click());

            let tab_bg = if is_active {
                if dark_mode { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(238, 243, 255) }
            } else if tab_res.hovered() {
                if dark_mode { Color32::from_rgb(44, 48, 58) } else { Color32::from_rgb(240, 242, 246) }
            } else {
                Color32::TRANSPARENT
            };

            child_ui.painter().rect_filled(tab_rect, CornerRadius::same(5), tab_bg);
            if is_active && !dark_mode {
                child_ui.painter().rect_stroke(
                    tab_rect,
                    CornerRadius::same(5),
                    Stroke::new(1.0_f32, Color32::from_rgb(200, 215, 255)),
                    StrokeKind::Middle,
                );
            }

            let text_color = if is_active {
                if dark_mode { Color32::WHITE } else { Color32::from_rgb(35, 75, 220) }
            } else if dark_mode {
                Color32::from_rgb(180, 185, 200)
            } else {
                Color32::from_rgb(70, 75, 90)
            };

            // Inline Rename Editor
            if state.renaming_board_id == Some(board_id) {
                let mut rename_ui = child_ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(tab_rect.shrink2(Vec2::new(4.0, 2.0)))
                        .layout(egui::Layout::left_to_right(egui::Align::Center)),
                );
                let edit_res = rename_ui.add(
                    egui::TextEdit::singleline(&mut state.rename_buffer)
                        .desired_width(tab_w - 10.0),
                );
                if edit_res.lost_focus() || rename_ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    let existing = project.sheet_names();
                    let cur_name = project.boards.get(&board_id).map(|b| b.name.as_str());
                    if let Ok(valid_name) = crate::model::sheet_ops::validate_sheet_name(&state.rename_buffer, &existing, cur_name) {
                        project.rename_board(board_id, valid_name);
                    }
                    state.renaming_board_id = None;
                } else if rename_ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    state.renaming_board_id = None;
                }
            } else {
                let text_pos = tab_rect.left_center() + Vec2::new(8.0, 0.0);
                child_ui.painter().text(
                    text_pos,
                    egui::Align2::LEFT_CENTER,
                    board_name,
                    egui::FontId::proportional(11.5),
                    text_color,
                );

                // Delete Sheet button (if more than 1 sheet)
                if sheet_count > 1 {
                    let del_rect = Rect::from_center_size(
                        Pos2::new(tab_rect.max.x - 10.0, tab_rect.center().y),
                        Vec2::new(14.0, 14.0),
                    );
                    let del_res = child_ui.allocate_rect(del_rect, egui::Sense::click());
                    if del_res.hovered() {
                        let circle_col = if dark_mode {
                            Color32::from_rgba_unmultiplied(200, 200, 200, 60)
                        } else {
                            Color32::from_rgba_unmultiplied(0, 0, 0, 25)
                        };
                        child_ui.painter().circle_filled(del_rect.center(), 6.0, circle_col);
                    }
                    child_ui.painter().text(
                        del_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "✕",
                        egui::FontId::proportional(8.5),
                        text_color,
                    );
                    if del_res.on_hover_text("Delete Sheet").clicked() {
                        action = TopBarAction::DeleteBoard(board_id);
                    }
                }

                if tab_res.double_clicked() {
                    action = TopBarAction::OpenRenameSheetModal(board_id);
                } else if tab_res.clicked() {
                    action = TopBarAction::SwitchBoard(board_id);
                }
            }

            child_ui.add_space(2.0);
        }

        // "+" Add New Sheet Button
        let (add_sheet_rect, add_sheet_res) = child_ui.allocate_exact_size(Vec2::new(22.0, 22.0), egui::Sense::click());
        if add_sheet_res.hovered() {
            child_ui.painter().rect_filled(
                add_sheet_rect,
                CornerRadius::same(4),
                if dark_mode { Color32::from_rgb(50, 54, 65) } else { Color32::from_rgb(230, 233, 240) },
            );
        }
        child_ui.painter().text(
            add_sheet_rect.center(),
            egui::Align2::CENTER_CENTER,
            "+",
            egui::FontId::proportional(14.0),
            if dark_mode { Color32::from_rgb(180, 185, 200) } else { Color32::from_rgb(90, 95, 110) },
        );
        if add_sheet_res.on_hover_text("Add New Sheet to Board").clicked() {
            action = TopBarAction::OpenCreateSheetModal;
        }

        let modal_action = Self::render_sheet_modal(ui, project, state, dark_mode);
        if modal_action != TopBarAction::None {
            action = modal_action;
        }

        (action, Some(bar_rect))
    }
}
