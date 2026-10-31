use crate::model::electronics::ComponentKind;
use crate::model::element::*;
use crate::model::project::Project;
use crate::storage::history::{
    are_paths_equal, clean_windows_path, default_projects_dir, format_file_size, format_system_time,
    get_common_directories, load_recents, load_starred, save_recents, save_starred, truncate_path_string,
};
use egui::{epaint::PathShape, epaint::StrokeKind, Color32, CornerRadius, Pos2, Rect, RichText, Stroke, Ui, Vec2};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub enum HomeScreenAction {
    None,
    NewProject,
    OpenProject(Option<PathBuf>),
    LoadTemplate(TemplateKind),
    OpenRecent(PathBuf),
    RemoveRecent(usize),
    ToggleStar(PathBuf),
    DeleteProject(PathBuf),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateKind {
    ElectronicsBreadboard,
    CircuitSchematic,
    HackClubShip,
    FirmwareStateMachine,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrowseFolderItem {
    pub path: PathBuf,
    pub name: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BrowseBoardItem {
    pub path: PathBuf,
    pub name: String,
    pub size: u64,
    pub modified: std::time::SystemTime,
}

pub struct HomeScreen {
    pub recent_files: Vec<PathBuf>,
    pub starred_files: Vec<PathBuf>,
    pub search_query: String,
    pub is_grid_view: bool,
    pub active_nav: String,
    pub current_browse_dir: PathBuf,
    pub browse_folders: Vec<BrowseFolderItem>,
    pub browse_files: Vec<BrowseBoardItem>,
    pub browse_error: Option<String>,
    pub browse_loaded_dir: Option<PathBuf>,
    pub persist_on_change: bool,
    pub pending_delete_path: Option<PathBuf>,
    pub logo_texture: Option<egui::TextureHandle>,
}

impl Default for HomeScreen {
    fn default() -> Self {
        let starred_files = load_starred();
        let recent_files = load_recents();
        let current_browse_dir = default_projects_dir();

        let mut screen = Self {
            recent_files,
            starred_files,
            search_query: String::new(),
            is_grid_view: false,
            active_nav: "Home".to_string(),
            current_browse_dir,
            browse_folders: Vec::new(),
            browse_files: Vec::new(),
            browse_error: None,
            browse_loaded_dir: None,
            persist_on_change: true,
            pending_delete_path: None,
            logo_texture: None,
        };
        screen.refresh_browse_dir();
        screen
    }
}

impl HomeScreen {
    pub fn new_in_memory() -> Self {
        Self {
            recent_files: Vec::new(),
            starred_files: Vec::new(),
            search_query: String::new(),
            is_grid_view: false,
            active_nav: "Home".to_string(),
            current_browse_dir: std::env::temp_dir(),
            browse_folders: Vec::new(),
            browse_files: Vec::new(),
            browse_error: None,
            browse_loaded_dir: None,
            persist_on_change: false,
            pending_delete_path: None,
            logo_texture: None,
        }
    }

    /// Actually deletes the `.siro` project file from the computer disk,
    /// cleans up any `.siro.bak` backup file, and prunes it from recents and starred lists.
    pub fn delete_board_file(&mut self, path: &Path) -> Result<(), String> {
        let clean = clean_windows_path(path);
        // Delete actual file from computer disk
        if clean.exists() {
            fs::remove_file(&clean).map_err(|e| format!("Failed to delete file {}: {}", clean.display(), e))?;
        }
        // Also remove .siro.bak backup file if present
        let bak = clean.with_extension("siro.bak");
        if bak.exists() {
            let _ = fs::remove_file(bak);
        }

        // Remove from starred
        self.starred_files.retain(|p| !are_paths_equal(p, &clean));
        if self.persist_on_change {
            let _ = save_starred(&self.starred_files);
        }

        // Remove from recents
        self.recent_files.retain(|p| !are_paths_equal(p, &clean));
        if self.persist_on_change {
            let _ = save_recents(&self.recent_files);
        }

        // Refresh browse view so it disappears from file listing immediately
        self.refresh_browse_dir();

        Ok(())
    }
    pub fn is_starred(&self, path: &Path) -> bool {
        self.starred_files.iter().any(|p| are_paths_equal(p, path))
    }

    pub fn toggle_star(&mut self, path: &Path) {
        let clean = clean_windows_path(path);
        if let Some(pos) = self.starred_files.iter().position(|p| are_paths_equal(p, &clean)) {
            self.starred_files.remove(pos);
        } else {
            self.starred_files.insert(0, clean);
        }
        if self.persist_on_change {
            let _ = save_starred(&self.starred_files);
        }
    }

    pub fn add_recent(&mut self, path: &Path) {
        let clean = clean_windows_path(path);
        if let Some(pos) = self.recent_files.iter().position(|p| are_paths_equal(p, &clean)) {
            self.recent_files.remove(pos);
        }
        self.recent_files.insert(0, clean);
        if self.recent_files.len() > 50 {
            self.recent_files.truncate(50);
        }
        if self.persist_on_change {
            let _ = save_recents(&self.recent_files);
        }
    }

    pub fn remove_recent(&mut self, idx: usize) {
        if idx < self.recent_files.len() {
            self.recent_files.remove(idx);
            if self.persist_on_change {
                let _ = save_recents(&self.recent_files);
            }
        }
    }

    pub fn rename_file_reference(&mut self, old_path: &Path, new_path: &Path) {
        let old_clean = clean_windows_path(old_path);
        let new_clean = clean_windows_path(new_path);

        let mut changed_recent = false;
        for p in self.recent_files.iter_mut() {
            if are_paths_equal(p, &old_clean) {
                *p = new_clean.clone();
                changed_recent = true;
            }
        }
        if changed_recent {
            let mut unique: Vec<PathBuf> = Vec::new();
            for p in self.recent_files.drain(..) {
                if !unique.iter().any(|existing| are_paths_equal(existing, &p)) {
                    unique.push(p);
                }
            }
            self.recent_files = unique;
            if self.persist_on_change {
                let _ = save_recents(&self.recent_files);
            }
        }

        let mut changed_starred = false;
        for p in self.starred_files.iter_mut() {
            if are_paths_equal(p, &old_clean) {
                *p = new_clean.clone();
                changed_starred = true;
            }
        }
        if changed_starred {
            let mut unique: Vec<PathBuf> = Vec::new();
            for p in self.starred_files.drain(..) {
                if !unique.iter().any(|existing| are_paths_equal(existing, &p)) {
                    unique.push(p);
                }
            }
            self.starred_files = unique;
            if self.persist_on_change {
                let _ = save_starred(&self.starred_files);
            }
        }

        self.refresh_browse_dir();
    }

    pub fn reload_history(&mut self) {
        self.starred_files = load_starred();
        self.recent_files = load_recents();
    }

    pub fn set_browse_dir(&mut self, dir: PathBuf) {
        let clean = clean_windows_path(&dir);
        if clean != self.current_browse_dir || self.browse_loaded_dir.as_ref() != Some(&clean) {
            self.current_browse_dir = clean;
            self.refresh_browse_dir();
        }
    }

    pub fn refresh_browse_dir(&mut self) {
        self.browse_loaded_dir = Some(self.current_browse_dir.clone());
        self.browse_folders.clear();
        self.browse_files.clear();
        self.browse_error = None;

        match fs::read_dir(&self.current_browse_dir) {
            Err(err) => {
                self.browse_error = Some(err.to_string());
            }
            Ok(entries) => {
                for entry in entries.flatten() {
                    let path = clean_windows_path(&entry.path());
                    if path.is_dir() {
                        let name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                        if !name.is_empty() && !name.starts_with('.') {
                            self.browse_folders.push(BrowseFolderItem { path, name });
                        }
                    } else if let Some(ext) = path.extension() {
                        if ext.to_string_lossy().eq_ignore_ascii_case("siro") {
                            let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
                            let meta = fs::metadata(&path);
                            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                            let modified = meta.as_ref().ok().and_then(|m| m.modified().ok()).unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                            self.browse_files.push(BrowseBoardItem {
                                path,
                                name,
                                size,
                                modified,
                            });
                        }
                    }
                }

                // Sort folders alphabetically (case-insensitive)
                self.browse_folders.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

                // Sort board files alphabetically (case-insensitive)
                self.browse_files.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
            }
        }
    }

    pub fn ui(&mut self, ui: &mut Ui, dark_mode: bool, settings: &mut crate::storage::settings::AppSettings) -> HomeScreenAction {
        let mut action = HomeScreenAction::None;

        let bg_color = if dark_mode {
            Color32::from_rgb(18, 20, 26)
        } else {
            Color32::from_rgb(255, 255, 255)
        };
        let sidebar_bg = if dark_mode {
            Color32::from_rgb(24, 26, 33)
        } else {
            Color32::from_rgb(249, 250, 252)
        };
        let sidebar_border = if dark_mode {
            Color32::from_rgb(40, 44, 56)
        } else {
            Color32::from_rgb(232, 235, 240)
        };

        // Fill background
        let full_rect = ui.available_rect_before_wrap();
        ui.painter().rect_filled(full_rect, CornerRadius::ZERO, bg_color);

        ui.horizontal(|ui| {
            // ==========================================
            // 1. LEFT NAVIGATION SIDEBAR (Miro Layout)
            // ==========================================
            let sidebar_w = 236.0;
            let sidebar_h = full_rect.height();
            let (sidebar_rect, _) = ui.allocate_exact_size(
                Vec2::new(sidebar_w, sidebar_h),
                egui::Sense::hover(),
            );
            let p = ui.painter_at(sidebar_rect);
            p.rect_filled(sidebar_rect, CornerRadius::ZERO, sidebar_bg);
            p.line_segment(
                [
                    Pos2::new(sidebar_rect.max.x, sidebar_rect.min.y),
                    Pos2::new(sidebar_rect.max.x, sidebar_rect.max.y),
                ],
                Stroke::new(1.0_f32, sidebar_border),
            );

            // Pinned Settings button at bottom-left of sidebar
            let settings_btn_w = 208.0;
            let settings_btn_h = 34.0;
            let settings_btn_rect = Rect::from_min_size(
                Pos2::new(sidebar_rect.min.x + 14.0, sidebar_rect.max.y - 18.0 - settings_btn_h),
                Vec2::new(settings_btn_w, settings_btn_h),
            );

            // Inner content for sidebar (top section down to above Settings button)
            let sidebar_content_rect = Rect::from_min_max(
                sidebar_rect.min + Vec2::new(14.0, 18.0),
                Pos2::new(sidebar_rect.max.x - 14.0, settings_btn_rect.min.y - 10.0),
            );
            let mut sidebar_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(sidebar_content_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );

            // App Branding Header: Official Siro Logo Icon + App Name
            sidebar_ui.horizontal(|ui| {
                let (icon_rect, _) = ui.allocate_exact_size(Vec2::new(26.0, 26.0), egui::Sense::hover());
                let tex = self.logo_texture.get_or_insert_with(|| {
                    let img = image::load_from_memory(include_bytes!("../../assets/icon_256.png"))
                        .expect("Embedded icon must load")
                        .to_rgba8();
                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                        [img.width() as usize, img.height() as usize],
                        img.as_raw(),
                    );
                    ui.ctx().load_texture("siro_app_logo", color_image, egui::TextureOptions::LINEAR)
                });
                ui.painter().image(
                    tex.id(),
                    icon_rect,
                    Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                    Color32::WHITE,
                );

                ui.add_space(8.0);
                ui.label(
                    RichText::new(crate::APP_NAME)
                        .size(14.0)
                        .strong()
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
                );
            });

            sidebar_ui.add_space(16.0);

            // Search Bar & "+" button
            sidebar_ui.horizontal(|ui| {
                let search_frame = egui::Frame::NONE
                    .fill(if dark_mode { Color32::from_rgb(32, 35, 45) } else { Color32::WHITE })
                    .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(55, 60, 75) } else { Color32::from_rgb(220, 224, 232) }))
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(egui::Margin::symmetric(6, 3));
                search_frame.show(ui, |ui| {
                    let desired_w = if self.search_query.is_empty() { 155.0 } else { 135.0 };
                    let search_field = egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text(RichText::new("🔍 Search boards...").color(Color32::from_rgb(140, 145, 160)))
                        .text_color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) })
                        .frame(false)
                        .desired_width(desired_w);
                    ui.add(search_field);
                    if !self.search_query.is_empty() {
                        if ui.add(egui::Button::new(RichText::new("✕").size(10.0)).frame(false)).on_hover_text("Clear search").clicked() {
                            self.search_query.clear();
                        }
                    }
                });

                let plus_btn = egui::Button::new(
                    RichText::new("+")
                        .size(14.0)
                        .strong()
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
                )
                .fill(if dark_mode { Color32::from_rgb(38, 42, 54) } else { Color32::WHITE })
                .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(60, 65, 80) } else { Color32::from_rgb(220, 224, 232) }))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(26.0, 24.0));
                if ui.add(plus_btn).on_hover_text("New Board").clicked() {
                    action = HomeScreenAction::NewProject;
                }
            });

            sidebar_ui.add_space(14.0);

            // Navigation items: Home, Starred, Browse
            let nav_items = [
                ("Home", "🏠", "Home"),
                ("Starred", "", "Starred"),
                ("Browse", "📁", "Browse"),
            ];

            for (key, icon, label) in nav_items {
                let is_active = self.active_nav == key;
                let text_color = if is_active {
                    if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) }
                } else {
                    if dark_mode { Color32::from_rgb(160, 165, 180) } else { Color32::from_rgb(85, 90, 105) }
                };

                let btn_bg = if is_active {
                    if dark_mode { Color32::from_rgb(38, 42, 54) } else { Color32::from_rgb(238, 241, 246) }
                } else {
                    Color32::TRANSPARENT
                };

                let (btn_rect, btn_res) = sidebar_ui.allocate_exact_size(Vec2::new(208.0, 34.0), egui::Sense::click());
                if btn_res.hovered() && !is_active {
                    sidebar_ui.painter().rect_filled(btn_rect, CornerRadius::same(8), if dark_mode { Color32::from_rgb(30, 33, 42) } else { Color32::from_rgb(244, 246, 250) });
                } else if is_active {
                    sidebar_ui.painter().rect_filled(btn_rect, CornerRadius::same(8), btn_bg);
                }

                // Draw Icon
                let icon_pos = btn_rect.left_center() + Vec2::new(12.0, 0.0);
                if key == "Starred" {
                    render_vector_star(
                        sidebar_ui.painter(),
                        icon_pos + Vec2::new(7.0, 0.0),
                        6.5,
                        is_active,
                        btn_res.hovered(),
                        dark_mode,
                    );
                } else {
                    sidebar_ui.painter().text(
                        icon_pos,
                        egui::Align2::LEFT_CENTER,
                        icon,
                        egui::FontId::proportional(14.0),
                        text_color,
                    );
                }

                // Draw Title
                let title_pos = icon_pos + Vec2::new(22.0, 0.0);
                sidebar_ui.painter().text(
                    title_pos,
                    egui::Align2::LEFT_CENTER,
                    label,
                    egui::FontId::proportional(13.5),
                    text_color,
                );

                if btn_res.clicked() {
                    self.active_nav = key.to_string();
                }

                sidebar_ui.add_space(4.0);
            }

            // Render Pinned Settings button at bottom-left of sidebar
            let is_settings = self.active_nav == "Settings";
            let text_color = if is_settings {
                if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) }
            } else {
                if dark_mode { Color32::from_rgb(160, 165, 180) } else { Color32::from_rgb(85, 90, 105) }
            };
            let btn_bg = if is_settings {
                if dark_mode { Color32::from_rgb(38, 42, 54) } else { Color32::from_rgb(238, 241, 246) }
            } else {
                Color32::TRANSPARENT
            };

            let btn_res = ui.allocate_rect(settings_btn_rect, egui::Sense::click());
            if btn_res.hovered() && !is_settings {
                ui.painter().rect_filled(
                    settings_btn_rect,
                    CornerRadius::same(8),
                    if dark_mode { Color32::from_rgb(30, 33, 42) } else { Color32::from_rgb(244, 246, 250) },
                );
            } else if is_settings {
                ui.painter().rect_filled(settings_btn_rect, CornerRadius::same(8), btn_bg);
            }

            let icon_pos = settings_btn_rect.left_center() + Vec2::new(12.0, 0.0);
            ui.painter().text(
                icon_pos,
                egui::Align2::LEFT_CENTER,
                "⚙",
                egui::FontId::proportional(14.0),
                text_color,
            );
            let title_pos = icon_pos + Vec2::new(22.0, 0.0);
            ui.painter().text(
                title_pos,
                egui::Align2::LEFT_CENTER,
                "Settings",
                egui::FontId::proportional(13.5),
                text_color,
            );

            if btn_res.clicked() {
                self.active_nav = "Settings".to_string();
            }


            // ==========================================
            // 2. MAIN DASHBOARD CONTENT AREA
            // ==========================================
            let content_padding = Vec2::new(36.0, 24.0);
            let min_pt = Pos2::new(sidebar_rect.max.x + content_padding.x, full_rect.min.y + content_padding.y);
            let raw_max = full_rect.max - content_padding;
            let max_pt = Pos2::new(raw_max.x.max(min_pt.x + 80.0), raw_max.y.max(min_pt.y + 80.0));
            let content_rect = Rect::from_min_max(min_pt, max_pt);

            let mut content_ui = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(content_rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );

            egui::ScrollArea::vertical().show(&mut content_ui, |ui| {
                // Top Header Row: "siro" branding & quick actions
                self.render_top_header(ui, dark_mode, &mut action);
                ui.add_space(20.0);

                // Route main view based on active navigation tab
                match self.active_nav.as_str() {
                    "Starred" => {
                        self.render_starred_view(ui, dark_mode, &mut action);
                    }
                    "Browse" => {
                        self.render_browse_view(ui, dark_mode, &mut action);
                    }
                    "Settings" => {
                        self.render_settings_view(ui, dark_mode, settings);
                    }
                    _ => {
                        // "Home" view
                        self.render_home_view(ui, dark_mode, &mut action);
                    }
                }

                ui.add_space(40.0);
            });
        });

        // Confirmation Modal for Deleting Board from Computer
        if let Some(target_path) = self.pending_delete_path.clone() {
            self.render_delete_confirmation_modal(ui, &target_path, dark_mode, &mut action);
        }

        action
    }

    fn render_delete_confirmation_modal(
        &mut self,
        ui: &mut Ui,
        path: &Path,
        dark_mode: bool,
        action: &mut HomeScreenAction,
    ) {
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Board");
        let path_str = path.to_string_lossy().to_string();

        let screen_rect = ui.ctx().screen_rect();

        // 1. Full-screen translucent backdrop capturing pointer clicks
        ui.painter().rect_filled(
            screen_rect,
            CornerRadius::ZERO,
            Color32::from_black_alpha(if dark_mode { 160 } else { 120 }),
        );
        let _ = ui.allocate_rect(screen_rect, egui::Sense::click());

        // 2. Centered Modal Dialog Card
        let dialog_w = 420.0;
        let dialog_h = 220.0;
        let dialog_rect = Rect::from_center_size(screen_rect.center(), Vec2::new(dialog_w, dialog_h));

        let card_bg = if dark_mode {
            Color32::from_rgb(28, 30, 38)
        } else {
            Color32::WHITE
        };
        let card_border = if dark_mode {
            Color32::from_rgb(60, 64, 78)
        } else {
            Color32::from_rgb(220, 224, 232)
        };

        // Modal shadow & background
        ui.painter().rect_filled(
            dialog_rect.translate(Vec2::new(0.0, 6.0)),
            CornerRadius::same(12),
            Color32::from_black_alpha(45),
        );
        ui.painter().rect_filled(dialog_rect, CornerRadius::same(12), card_bg);
        ui.painter().rect_stroke(
            dialog_rect,
            CornerRadius::same(12),
            Stroke::new(1.0_f32, card_border),
            StrokeKind::Middle,
        );

        let mut modal_ui = ui.new_child(
            egui::UiBuilder::new()
                .max_rect(dialog_rect.shrink(18.0))
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );

        modal_ui.horizontal(|ui| {
            // Red trash badge icon
            let (icon_rect, _) = ui.allocate_exact_size(Vec2::new(32.0, 32.0), egui::Sense::hover());
            ui.painter().circle_filled(
                icon_rect.center(),
                16.0,
                if dark_mode { Color32::from_rgb(65, 25, 25) } else { Color32::from_rgb(254, 226, 226) },
            );
            ui.painter().text(
                icon_rect.center(),
                egui::Align2::CENTER_CENTER,
                "🗑",
                egui::FontId::proportional(16.0),
                Color32::from_rgb(239, 68, 68),
            );

            ui.add_space(8.0);
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Delete Board from Computer?")
                        .size(16.0)
                        .strong()
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) }),
                );
                ui.label(
                    RichText::new("Permanently removes the .siro file from disk")
                        .size(11.5)
                        .color(Color32::from_rgb(140, 145, 160)),
                );
            });
        });

        modal_ui.add_space(10.0);

        modal_ui.label(
            RichText::new(format!("Are you sure you want to permanently delete \"{}\"?", filename))
                .size(13.0)
                .color(if dark_mode { Color32::from_rgb(220, 225, 235) } else { Color32::from_rgb(45, 50, 65) }),
        );

        modal_ui.add_space(4.0);

        // Path indicator in code/monospace style
        let trunc_path = truncate_path_string(&path_str, 50);
        egui::Frame::NONE
            .fill(if dark_mode { Color32::from_rgb(20, 22, 28) } else { Color32::from_rgb(243, 244, 246) })
            .corner_radius(CornerRadius::same(4))
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(&mut modal_ui, |ui| {
                ui.label(RichText::new(trunc_path).size(11.0).monospace().color(Color32::from_rgb(120, 125, 140)));
            });

        modal_ui.add_space(14.0);

        // Action Buttons Row
        modal_ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Delete button (Red danger)
                let del_btn = egui::Button::new(
                    RichText::new("Delete from Computer")
                        .size(12.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(Color32::from_rgb(220, 38, 38))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(148.0, 32.0));

                if ui.add(del_btn).clicked() {
                    let _ = self.delete_board_file(path);
                    *action = HomeScreenAction::DeleteProject(path.to_path_buf());
                    self.pending_delete_path = None;
                }

                ui.add_space(8.0);

                // Cancel button
                let cancel_btn = egui::Button::new(
                    RichText::new("Cancel")
                        .size(12.5)
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(50, 55, 65) }),
                )
                .fill(if dark_mode { Color32::from_rgb(42, 46, 58) } else { Color32::from_rgb(238, 241, 246) })
                .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(60, 65, 80) } else { Color32::from_rgb(215, 220, 230) }))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(76.0, 32.0));

                if ui.add(cancel_btn).clicked() || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.pending_delete_path = None;
                }
            });
        });
    }

    fn render_top_header(&mut self, ui: &mut Ui, dark_mode: bool, action: &mut HomeScreenAction) {
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(crate::APP_NAME.to_lowercase())
                    .size(24.0)
                    .strong()
                    .color(if dark_mode { Color32::WHITE } else { Color32::BLACK }),
            );

            let (beta_rect, _) = ui.allocate_exact_size(Vec2::new(42.0, 18.0), egui::Sense::hover());
            ui.painter().rect_filled(
                beta_rect,
                CornerRadius::same(9),
                if dark_mode { Color32::from_rgb(40, 44, 56) } else { Color32::from_rgb(235, 238, 245) },
            );
            ui.painter().text(
                beta_rect.center(),
                egui::Align2::CENTER_CENTER,
                "Beta",
                egui::FontId::proportional(10.5),
                if dark_mode { Color32::LIGHT_GRAY } else { Color32::DARK_GRAY },
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let open_btn = egui::Button::new(
                    RichText::new("📂  Open .siro File")
                        .size(13.0)
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
                )
                .fill(if dark_mode { Color32::from_rgb(38, 42, 54) } else { Color32::WHITE })
                .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(60, 65, 80) } else { Color32::from_rgb(220, 224, 232) }))
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(135.0, 32.0));
                if ui.add(open_btn).clicked() {
                    *action = HomeScreenAction::OpenProject(None);
                }
            });
        });
    }

    fn render_settings_view(
        &mut self,
        ui: &mut Ui,
        dark_mode: bool,
        settings: &mut crate::storage::settings::AppSettings,
    ) {
        let text_primary = if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 24, 35) };
        let text_secondary = if dark_mode { Color32::from_rgb(160, 165, 180) } else { Color32::from_rgb(90, 95, 110) };
        let card_bg = if dark_mode { Color32::from_rgb(26, 28, 36) } else { Color32::WHITE };
        let card_border = if dark_mode { Color32::from_rgb(46, 50, 64) } else { Color32::from_rgb(225, 228, 236) };

        // Page Header
        ui.horizontal(|ui| {
            ui.label(RichText::new("⚙ Settings").size(22.0).strong().color(text_primary));
        });
        ui.add_space(4.0);
        ui.label(RichText::new("Configure automatic saving and background persistence.").size(13.0).color(text_secondary));
        ui.add_space(20.0);

        // Autosave Card Frame
        egui::Frame::NONE
            .fill(card_bg)
            .stroke(Stroke::new(1.0_f32, card_border))
            .corner_radius(CornerRadius::same(10))
            .inner_margin(egui::Margin::same(20))
            .show(ui, |ui| {
                ui.set_max_width(560.0);

                ui.label(RichText::new("Automatic Background Saving (Autosave)").size(15.0).strong().color(text_primary));
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Continuously writes open projects with unsaved content to disk at a regular interval.")
                        .size(12.5)
                        .color(text_secondary),
                );
                ui.add_space(16.0);

                let mut is_enabled = settings.is_autosave_enabled();
                let mut changed = false;

                if ui.checkbox(&mut is_enabled, RichText::new("Enable automatic background saving").size(13.0).color(text_primary)).changed() {
                    if is_enabled {
                        settings.autosave_interval_secs = Some(crate::storage::settings::DEFAULT_AUTOSAVE_INTERVAL_SECS);
                    } else {
                        settings.autosave_interval_secs = None;
                    }
                    changed = true;
                }

                ui.add_space(14.0);

                if let Some(mut secs) = settings.autosave_interval_secs {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Save interval:").size(13.0).color(text_primary));
                        let slider = egui::Slider::new(&mut secs, crate::storage::settings::MIN_AUTOSAVE_INTERVAL_SECS..=crate::storage::settings::MAX_AUTOSAVE_INTERVAL_SECS)
                            .suffix(" seconds");
                        if ui.add(slider).changed() {
                            settings.autosave_interval_secs = Some(secs);
                            changed = true;
                        }
                    });

                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(format!("Every {} seconds ({} to 600s range)", secs, crate::storage::settings::MIN_AUTOSAVE_INTERVAL_SECS))
                            .size(12.0)
                            .color(if dark_mode { Color32::from_rgb(100, 165, 255) } else { Color32::from_rgb(35, 95, 235) }),
                    );
                } else {
                    ui.label(
                        RichText::new("Autosave is currently Off. No automatic background writes will occur.")
                            .size(12.5)
                            .color(Color32::from_rgb(245, 158, 11)),
                    );
                }

                ui.add_space(18.0);
                ui.separator();
                ui.add_space(12.0);

                ui.label(
                    RichText::new("Guarantees & Safety:")
                        .size(12.5)
                        .strong()
                        .color(text_primary),
                );
                ui.add_space(4.0);
                ui.label(
                    RichText::new("• The modal close dialog ('Save then exit' / 'Discard changes' / 'Cancel') protects all unsaved projects when exiting or closing tabs, even when autosave is Off.")
                        .size(11.5)
                        .color(text_secondary),
                );
                ui.add_space(3.0);
                ui.label(
                    RichText::new("• Never-saved projects are prompted for Save As on close and are never lost.")
                        .size(11.5)
                        .color(text_secondary),
                );
                ui.add_space(3.0);
                ui.label(
                    RichText::new("• Autosave writes atomically and keeps backup sidecars (.siro.bak).")
                        .size(11.5)
                        .color(text_secondary),
                );

                if changed {
                    let _ = crate::storage::settings::save_settings(settings);
                }
            });
    }

    // -------------------------------------------------------------
    // 1. HOME VIEW (Templates + Recents)
    // -------------------------------------------------------------
    fn render_home_view(&mut self, ui: &mut Ui, dark_mode: bool, action: &mut HomeScreenAction) {
        // Hero Title: Clean greeting without fake username persona
        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            let hero_font = egui::FontId::proportional(36.0);
            ui.label(
                RichText::new("Welcome back")
                    .font(hero_font)
                    .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
            );
            ui.add_space(20.0);
        });

        // ==========================================
        // SECTION: "Start something new"
        // ==========================================
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("Start something new")
                        .size(16.5)
                        .strong()
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
                );
                ui.label(
                    RichText::new("Templates for Hardware Engineering")
                        .size(12.0)
                        .color(Color32::from_rgb(130, 135, 150)),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let create_btn = egui::Button::new(
                    RichText::new("+ Create")
                        .size(13.5)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(if dark_mode { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(37, 99, 235) })
                .corner_radius(CornerRadius::same(8))
                .min_size(Vec2::new(96.0, 34.0));

                if ui.add(create_btn).clicked() {
                    *action = HomeScreenAction::NewProject;
                }
            });
        });

        ui.add_space(14.0);

        // Template Cards Row
        ui.horizontal(|ui| {
            let card_w = 158.0;
            let card_h = 108.0;
            let gap = 14.0;

            // 1. Blank Board Card
            if render_blank_board_card(ui, card_w, card_h, dark_mode) {
                *action = HomeScreenAction::NewProject;
            }

            ui.add_space(gap);

            // 2. Breadboard & MCU
            if render_template_tile(
                ui,
                "Breadboard & MCU",
                TemplateKind::ElectronicsBreadboard,
                Color32::from_rgb(22, 120, 75),
                card_w,
                card_h,
                dark_mode,
            ) {
                *action = HomeScreenAction::LoadTemplate(TemplateKind::ElectronicsBreadboard);
            }

            ui.add_space(gap);

            // 3. Circuit Schematic
            if render_template_tile(
                ui,
                "Circuit Schematic",
                TemplateKind::CircuitSchematic,
                Color32::from_rgb(220, 85, 45),
                card_w,
                card_h,
                dark_mode,
            ) {
                *action = HomeScreenAction::LoadTemplate(TemplateKind::CircuitSchematic);
            }

            ui.add_space(gap);

            // 4. Hack Club Ship
            if render_template_tile(
                ui,
                "Hack Club Ship",
                TemplateKind::HackClubShip,
                Color32::from_rgb(235, 55, 75),
                card_w,
                card_h,
                dark_mode,
            ) {
                *action = HomeScreenAction::LoadTemplate(TemplateKind::HackClubShip);
            }

            ui.add_space(gap);

            // 5. Firmware State Machine
            if render_template_tile(
                ui,
                "Firmware State Machine",
                TemplateKind::FirmwareStateMachine,
                Color32::from_rgb(35, 115, 230),
                card_w,
                card_h,
                dark_mode,
            ) {
                *action = HomeScreenAction::LoadTemplate(TemplateKind::FirmwareStateMachine);
            }
        });

        ui.add_space(36.0);

        // ==========================================
        // SECTION: "Jump back to recents"
        // ==========================================
        ui.horizontal(|ui| {
            ui.label(
                RichText::new("Jump back to recents")
                    .size(16.5)
                    .strong()
                    .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let list_active = !self.is_grid_view;
                let grid_active = self.is_grid_view;

                if ui.selectable_label(list_active, "☰").clicked() {
                    self.is_grid_view = false;
                }
                if ui.selectable_label(grid_active, "☷").clicked() {
                    self.is_grid_view = true;
                }
            });
        });

        ui.add_space(14.0);

        // Filter recent files by search query
        let query = self.search_query.trim().to_lowercase();
        let filtered_recents: Vec<(usize, PathBuf)> = self
            .recent_files
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                if query.is_empty() {
                    true
                } else {
                    p.to_string_lossy().to_lowercase().contains(&query)
                }
            })
            .map(|(i, p)| (i, p.clone()))
            .collect();

        if filtered_recents.is_empty() {
            let msg = if query.is_empty() {
                "No recent boards found. Click '+ Create' or select a template above to get started!"
            } else {
                "No recent boards match your search query."
            };
            self.render_empty_state_card(ui, dark_mode, "📁", msg);
        } else if self.is_grid_view {
            // Recents Grid View
            let card_w = 210.0;
            let card_h = 130.0;
            let gap = 16.0;

            let avail_w = ui.available_width();
            let cols = ((avail_w + gap) / (card_w + gap)).floor().max(1.0) as usize;

            for chunk in filtered_recents.chunks(cols) {
                ui.horizontal(|ui| {
                    for &(_orig_idx, ref path) in chunk {
                        let filename = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Untitled Board");

                        let (rect, res) = ui.allocate_exact_size(Vec2::new(card_w, card_h), egui::Sense::click());
                        let is_hovered = ui.rect_contains_pointer(rect) || res.hovered();

                        let bg = if is_hovered {
                            if dark_mode { Color32::from_rgb(36, 40, 52) } else { Color32::from_rgb(240, 245, 255) }
                        } else {
                            if dark_mode { Color32::from_rgb(26, 28, 36) } else { Color32::from_rgb(250, 251, 254) }
                        };

                        ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
                        ui.painter().rect_stroke(
                            rect,
                            CornerRadius::same(8),
                            Stroke::new(1.0_f32, if is_hovered { Color32::from_rgb(45, 96, 255) } else if dark_mode { Color32::from_rgb(45, 48, 60) } else { Color32::from_rgb(228, 231, 238) }),
                            StrokeKind::Middle,
                        );

                        // Preview area
                        let preview_rect = Rect::from_min_size(rect.min + Vec2::new(8.0, 8.0), Vec2::new(card_w - 16.0, 72.0));
                        ui.painter().rect_filled(preview_rect, CornerRadius::same(5), if dark_mode { Color32::from_rgb(20, 22, 28) } else { Color32::from_rgb(242, 244, 248) });
                        ui.painter().text(
                            preview_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "📐",
                            egui::FontId::proportional(26.0),
                            Color32::from_rgb(45, 96, 255),
                        );

                        // Star button in top-right of preview area
                        let star_rect = Rect::from_center_size(
                            Pos2::new(preview_rect.max.x - 14.0, preview_rect.min.y + 14.0),
                            Vec2::new(22.0, 22.0),
                        );
                        let star_res = ui.allocate_rect(star_rect, egui::Sense::click());
                        let is_st = self.is_starred(path);
                        render_vector_star(
                            ui.painter(),
                            star_rect.center(),
                            7.0,
                            is_st,
                            star_res.hovered(),
                            dark_mode,
                        );
                        if star_res.hovered() {
                            star_res.show_tooltip_text(if is_st { "Unstar board" } else { "Star board" });
                        }

                        // Delete button in top-left of preview area on hover
                        let delete_rect = Rect::from_center_size(
                            Pos2::new(preview_rect.min.x + 14.0, preview_rect.min.y + 14.0),
                            Vec2::new(22.0, 22.0),
                        );
                        let mut delete_clicked = false;
                        if is_hovered {
                            let del_res = ui.allocate_rect(delete_rect, egui::Sense::click());
                            if del_res.hovered() {
                                ui.painter().circle_filled(
                                    delete_rect.center(),
                                    10.0,
                                    if dark_mode { Color32::from_rgb(60, 25, 25) } else { Color32::from_rgb(254, 226, 226) },
                                );
                                del_res.show_tooltip_text("Delete board from computer");
                            }
                            ui.painter().text(
                                delete_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🗑",
                                egui::FontId::proportional(11.5),
                                if del_res.hovered() { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(150, 155, 170) },
                            );
                            if del_res.clicked() {
                                delete_clicked = true;
                                self.pending_delete_path = Some(path.clone());
                            }
                        }

                        // Title text
                        ui.painter().text(
                            rect.min + Vec2::new(10.0, 92.0),
                            egui::Align2::LEFT_TOP,
                            filename,
                            egui::FontId::proportional(13.0),
                            if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) },
                        );

                        if star_res.clicked() {
                            self.toggle_star(path);
                            *action = HomeScreenAction::ToggleStar(path.clone());
                        } else if !delete_clicked && !ui.rect_contains_pointer(star_rect) && !ui.rect_contains_pointer(delete_rect) && res.clicked() {
                            *action = HomeScreenAction::OpenRecent(path.clone());
                        }
                    }
                });
                ui.add_space(gap);
            }
        } else {
            // Recents List View
            let row_h = 42.0;

            // Table Header aligned with data columns
            render_list_table_header(ui, "Name", "Size", "Last modified");
            ui.add_space(4.0);

            for &(_orig_idx, ref path) in &filtered_recents {
                let filename = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled Board");
                let path_str = path.to_string_lossy().to_string();

                let (row_rect, row_res) = ui.allocate_exact_size(Vec2::new(ui.available_width(), row_h), egui::Sense::click());
                let is_hovered = ui.rect_contains_pointer(row_rect) || row_res.hovered();

                if is_hovered {
                    ui.painter().rect_filled(
                        row_rect,
                        CornerRadius::same(6),
                        if dark_mode { Color32::from_rgb(32, 35, 46) } else { Color32::from_rgb(244, 246, 250) },
                    );
                }

                // Star button
                let star_rect = Rect::from_center_size(row_rect.left_center() + Vec2::new(16.0, 0.0), Vec2::new(22.0, 22.0));
                let star_res = ui.allocate_rect(star_rect, egui::Sense::click());
                let is_st = self.is_starred(path);
                render_vector_star(
                    ui.painter(),
                    star_rect.center(),
                    7.0,
                    is_st,
                    star_res.hovered(),
                    dark_mode,
                );
                if star_res.hovered() {
                    star_res.show_tooltip_text(if is_st { "Unstar board" } else { "Star board" });
                }

                // Orange folder icon
                let icon_pos = row_rect.left_center() + Vec2::new(38.0, 0.0);
                ui.painter().text(
                    icon_pos,
                    egui::Align2::LEFT_CENTER,
                    "📁",
                    egui::FontId::proportional(16.0),
                    Color32::from_rgb(245, 158, 11),
                );

                // Board name & missing indicator
                let file_exists = path.exists();
                let name_pos = icon_pos + Vec2::new(24.0, -8.0);
                let name_text = if file_exists {
                    filename.to_string()
                } else {
                    format!("{} (missing)", filename)
                };
                let name_color = if !file_exists {
                    Color32::from_rgb(180, 130, 70)
                } else if dark_mode {
                    Color32::WHITE
                } else {
                    Color32::from_rgb(24, 26, 32)
                };
                ui.painter().text(
                    name_pos,
                    egui::Align2::LEFT_CENTER,
                    name_text,
                    egui::FontId::proportional(13.5),
                    name_color,
                );

                // File path subtext (Unicode-safe truncation)
                let sub_pos = icon_pos + Vec2::new(24.0, 9.0);
                let truncated_path = truncate_path_string(&path_str, 50);
                ui.painter().text(
                    sub_pos,
                    egui::Align2::LEFT_CENTER,
                    truncated_path,
                    egui::FontId::proportional(10.5),
                    Color32::from_rgb(140, 145, 160),
                );

                // Size column (aligned with header at 50% width)
                let size_text = if let Ok(meta) = fs::metadata(path) {
                    format_file_size(meta.len())
                } else {
                    "Missing".to_string()
                };
                let size_pos = Pos2::new(row_rect.min.x + row_rect.width() * 0.50, row_rect.center().y);
                ui.painter().text(
                    size_pos,
                    egui::Align2::LEFT_CENTER,
                    size_text,
                    egui::FontId::proportional(12.5),
                    Color32::from_rgb(140, 145, 160),
                );

                // Last modified column (aligned with header at 72% width)
                let time_text = if let Ok(meta) = fs::metadata(path) {
                    if let Ok(m) = meta.modified() {
                        format_system_time(m)
                    } else {
                        "Unknown".to_string()
                    }
                } else {
                    "Not found".to_string()
                };
                let opened_pos = Pos2::new(row_rect.min.x + row_rect.width() * 0.72, row_rect.center().y);
                ui.painter().text(
                    opened_pos,
                    egui::Align2::LEFT_CENTER,
                    time_text,
                    egui::FontId::proportional(12.5),
                    if file_exists { Color32::from_rgb(140, 145, 160) } else { Color32::from_rgb(210, 130, 40) },
                );

                // Action buttons on right (Open + ✕ Remove from recents + 🗑 Delete from computer)
                let mut delete_clicked = false;
                let mut remove_clicked = false;
                let mut open_clicked = false;

                if is_hovered {
                    let delete_btn_rect = Rect::from_center_size(
                        Pos2::new(row_rect.max.x - 18.0, row_rect.center().y),
                        Vec2::new(20.0, 20.0),
                    );
                    let delete_res = ui.allocate_rect(delete_btn_rect, egui::Sense::click());
                    if delete_res.hovered() {
                        ui.painter().circle_filled(delete_btn_rect.center(), 9.0, if dark_mode { Color32::from_rgb(60, 25, 25) } else { Color32::from_rgb(254, 226, 226) });
                        delete_res.show_tooltip_text("Delete board from computer");
                    }
                    ui.painter().text(
                        delete_btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "🗑",
                        egui::FontId::proportional(11.5),
                        if delete_res.hovered() { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(150, 155, 170) },
                    );
                    if delete_res.clicked() {
                        delete_clicked = true;
                        self.pending_delete_path = Some(path.clone());
                    }

                    let remove_btn_rect = Rect::from_center_size(
                        Pos2::new(row_rect.max.x - 42.0, row_rect.center().y),
                        Vec2::new(20.0, 20.0),
                    );
                    let remove_res = ui.allocate_rect(remove_btn_rect, egui::Sense::click());
                    if remove_res.hovered() {
                        ui.painter().circle_filled(remove_btn_rect.center(), 9.0, if dark_mode { Color32::from_rgb(50, 55, 70) } else { Color32::from_rgb(230, 233, 240) });
                        remove_res.show_tooltip_text("Remove from recents");
                    }
                    ui.painter().text(
                        remove_btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "✕",
                        egui::FontId::proportional(11.0),
                        Color32::from_rgb(150, 155, 170),
                    );
                    if remove_res.clicked() {
                        remove_clicked = true;
                        *action = HomeScreenAction::RemoveRecent(_orig_idx);
                    }

                    let open_btn_rect = Rect::from_center_size(
                        Pos2::new(row_rect.max.x - 88.0, row_rect.center().y),
                        Vec2::new(55.0, 24.0),
                    );
                    let open_res = ui.allocate_rect(open_btn_rect, egui::Sense::click());
                    ui.painter().rect_filled(
                        open_btn_rect,
                        CornerRadius::same(4),
                        if open_res.hovered() { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(35, 75, 210) },
                    );
                    ui.painter().text(
                        open_btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Open",
                        egui::FontId::proportional(11.5),
                        Color32::WHITE,
                    );
                    if open_res.clicked() {
                        open_clicked = true;
                        *action = HomeScreenAction::OpenRecent(path.clone());
                    }
                }

                if star_res.clicked() {
                    self.toggle_star(path);
                    *action = HomeScreenAction::ToggleStar(path.clone());
                } else if !delete_clicked && !remove_clicked && !open_clicked && !ui.rect_contains_pointer(star_rect) && row_res.clicked() {
                    *action = HomeScreenAction::OpenRecent(path.clone());
                }

                ui.add_space(2.0);
            }
        }
    }

    // -------------------------------------------------------------
    // 2. STARRED VIEW (User Favorites)
    // -------------------------------------------------------------
    fn render_starred_view(&mut self, ui: &mut Ui, dark_mode: bool, action: &mut HomeScreenAction) {
        // Section Header
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("⭐ Starred Boards")
                        .size(22.0)
                        .strong()
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
                );
                ui.label(
                    RichText::new("Quick access to your pinned and favorite hardware boards")
                        .size(12.5)
                        .color(Color32::from_rgb(130, 135, 150)),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let list_active = !self.is_grid_view;
                let grid_active = self.is_grid_view;

                if ui.selectable_label(list_active, "☰").clicked() {
                    self.is_grid_view = false;
                }
                if ui.selectable_label(grid_active, "☷").clicked() {
                    self.is_grid_view = true;
                }
            });
        });

        ui.add_space(20.0);

        // Filter starred files by search query
        let query = self.search_query.trim().to_lowercase();
        let filtered_starred: Vec<PathBuf> = self
            .starred_files
            .iter()
            .filter(|p| {
                if query.is_empty() {
                    true
                } else {
                    p.to_string_lossy().to_lowercase().contains(&query)
                }
            })
            .cloned()
            .collect();

        if self.starred_files.is_empty() {
            // True empty state: user has not starred any boards yet
            egui::Frame::NONE
                .fill(if dark_mode { Color32::from_rgb(26, 28, 36) } else { Color32::from_rgb(248, 250, 252) })
                .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(45, 48, 60) } else { Color32::from_rgb(228, 231, 238) }))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(24, 32))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("⭐").size(34.0).color(Color32::from_rgb(245, 190, 40)));
                        ui.add_space(10.0);
                        ui.label(
                            RichText::new("No starred boards yet")
                                .size(16.0)
                                .strong()
                                .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Star boards in Home or Browse to quickly access them here!")
                                .size(12.5)
                                .color(Color32::from_rgb(140, 145, 160)),
                        );
                        ui.add_space(16.0);
                        ui.horizontal(|ui| {
                            let recents_btn = egui::Button::new(RichText::new("🏠 Go to Recents").size(12.5))
                                .fill(if dark_mode { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(37, 99, 235) })
                                .corner_radius(CornerRadius::same(6))
                                .min_size(Vec2::new(120.0, 30.0));
                            if ui.add(recents_btn).clicked() {
                                self.active_nav = "Home".to_string();
                            }

                            ui.add_space(8.0);
                            let browse_btn = egui::Button::new(RichText::new("📁 Browse Files").size(12.5))
                                .fill(if dark_mode { Color32::from_rgb(38, 42, 54) } else { Color32::WHITE })
                                .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(60, 65, 80) } else { Color32::from_rgb(220, 224, 232) }))
                                .corner_radius(CornerRadius::same(6))
                                .min_size(Vec2::new(120.0, 30.0));
                            if ui.add(browse_btn).clicked() {
                                self.active_nav = "Browse".to_string();
                            }
                        });
                    });
                });
        } else if filtered_starred.is_empty() {
            // Search query returned 0 results
            self.render_empty_state_card(ui, dark_mode, "🔍", "No starred boards match your search query.");
        } else if self.is_grid_view {
            // Starred Grid View
            let card_w = 210.0;
            let card_h = 130.0;
            let gap = 16.0;

            let avail_w = ui.available_width();
            let cols = ((avail_w + gap) / (card_w + gap)).floor().max(1.0) as usize;

            for chunk in filtered_starred.chunks(cols) {
                ui.horizontal(|ui| {
                    for path in chunk {
                        let filename = path
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("Untitled Board");

                        let (rect, res) = ui.allocate_exact_size(Vec2::new(card_w, card_h), egui::Sense::click());
                        let is_hovered = ui.rect_contains_pointer(rect) || res.hovered();

                        let bg = if is_hovered {
                            if dark_mode { Color32::from_rgb(36, 40, 52) } else { Color32::from_rgb(240, 245, 255) }
                        } else {
                            if dark_mode { Color32::from_rgb(26, 28, 36) } else { Color32::from_rgb(250, 251, 254) }
                        };

                        ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
                        ui.painter().rect_stroke(
                            rect,
                            CornerRadius::same(8),
                            Stroke::new(1.0_f32, if is_hovered { Color32::from_rgb(45, 96, 255) } else if dark_mode { Color32::from_rgb(45, 48, 60) } else { Color32::from_rgb(228, 231, 238) }),
                            StrokeKind::Middle,
                        );

                        // Preview area
                        let preview_rect = Rect::from_min_size(rect.min + Vec2::new(8.0, 8.0), Vec2::new(card_w - 16.0, 72.0));
                        ui.painter().rect_filled(preview_rect, CornerRadius::same(5), if dark_mode { Color32::from_rgb(20, 22, 28) } else { Color32::from_rgb(242, 244, 248) });
                        ui.painter().text(
                            preview_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "📐",
                            egui::FontId::proportional(26.0),
                            Color32::from_rgb(45, 96, 255),
                        );

                        // Star button in top-right of preview area (gold star with hover feedback)
                        let star_rect = Rect::from_center_size(
                            Pos2::new(preview_rect.max.x - 14.0, preview_rect.min.y + 14.0),
                            Vec2::new(22.0, 22.0),
                        );
                        let star_res = ui.allocate_rect(star_rect, egui::Sense::click());
                        let is_st = self.is_starred(path);
                        render_vector_star(
                            ui.painter(),
                            star_rect.center(),
                            7.0,
                            is_st,
                            star_res.hovered(),
                            dark_mode,
                        );
                        if star_res.hovered() {
                            star_res.show_tooltip_text(if is_st { "Unstar board" } else { "Star board" });
                        }

                        // Delete button in top-left of preview area on hover
                        let delete_rect = Rect::from_center_size(
                            Pos2::new(preview_rect.min.x + 14.0, preview_rect.min.y + 14.0),
                            Vec2::new(22.0, 22.0),
                        );
                        let mut delete_clicked = false;
                        if is_hovered {
                            let del_res = ui.allocate_rect(delete_rect, egui::Sense::click());
                            if del_res.hovered() {
                                ui.painter().circle_filled(
                                    delete_rect.center(),
                                    10.0,
                                    if dark_mode { Color32::from_rgb(60, 25, 25) } else { Color32::from_rgb(254, 226, 226) },
                                );
                                del_res.show_tooltip_text("Delete board from computer");
                            }
                            ui.painter().text(
                                delete_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🗑",
                                egui::FontId::proportional(11.5),
                                if del_res.hovered() { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(150, 155, 170) },
                            );
                            if del_res.clicked() {
                                delete_clicked = true;
                                self.pending_delete_path = Some(path.clone());
                            }
                        }

                        // Title text
                        ui.painter().text(
                            rect.min + Vec2::new(10.0, 88.0),
                            egui::Align2::LEFT_TOP,
                            filename,
                            egui::FontId::proportional(13.0),
                            if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) },
                        );

                        // Path / Status subtext
                        let exists = path.exists();
                        let subtext = if exists {
                            "Starred Board"
                        } else {
                            "(File not found)"
                        };
                        let sub_color = if exists {
                            Color32::from_rgb(140, 145, 160)
                        } else {
                            Color32::from_rgb(235, 90, 90)
                        };
                        ui.painter().text(
                            rect.min + Vec2::new(10.0, 108.0),
                            egui::Align2::LEFT_TOP,
                            subtext,
                            egui::FontId::proportional(10.5),
                            sub_color,
                        );

                        if star_res.clicked() {
                            self.toggle_star(path);
                            *action = HomeScreenAction::ToggleStar(path.clone());
                        } else if !delete_clicked && !ui.rect_contains_pointer(star_rect) && !ui.rect_contains_pointer(delete_rect) && res.clicked() {
                            *action = HomeScreenAction::OpenRecent(path.clone());
                        }
                    }
                });
                ui.add_space(gap);
            }
        } else {
            // Starred List View
            let row_h = 42.0;

            // Table Header aligned with data columns
            render_list_table_header(ui, "Name", "Folder", "Size");
            ui.add_space(4.0);

            for path in &filtered_starred {
                let filename = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Untitled Board");
                let path_str = path.to_string_lossy().to_string();

                let (row_rect, row_res) = ui.allocate_exact_size(Vec2::new(ui.available_width(), row_h), egui::Sense::click());
                let is_hovered = ui.rect_contains_pointer(row_rect) || row_res.hovered();

                if is_hovered {
                    ui.painter().rect_filled(
                        row_rect,
                        CornerRadius::same(6),
                        if dark_mode { Color32::from_rgb(32, 35, 46) } else { Color32::from_rgb(244, 246, 250) },
                    );
                }

                // Star button (clean vector star with hover feedback)
                let star_rect = Rect::from_center_size(row_rect.left_center() + Vec2::new(16.0, 0.0), Vec2::new(22.0, 22.0));
                let star_res = ui.allocate_rect(star_rect, egui::Sense::click());
                let is_st = self.is_starred(path);
                render_vector_star(
                    ui.painter(),
                    star_rect.center(),
                    7.0,
                    is_st,
                    star_res.hovered(),
                    dark_mode,
                );
                if star_res.hovered() {
                    star_res.show_tooltip_text(if is_st { "Unstar board" } else { "Star board" });
                }

                // Icon
                let icon_pos = row_rect.left_center() + Vec2::new(38.0, 0.0);
                ui.painter().text(
                    icon_pos,
                    egui::Align2::LEFT_CENTER,
                    "📐",
                    egui::FontId::proportional(16.0),
                    Color32::from_rgb(45, 96, 255),
                );

                // Board name
                let name_pos = icon_pos + Vec2::new(24.0, -8.0);
                ui.painter().text(
                    name_pos,
                    egui::Align2::LEFT_CENTER,
                    filename,
                    egui::FontId::proportional(13.5),
                    if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) },
                );

                // File path subtext (Unicode-safe truncation)
                let sub_pos = icon_pos + Vec2::new(24.0, 9.0);
                let truncated_path = truncate_path_string(&path_str, 50);
                ui.painter().text(
                    sub_pos,
                    egui::Align2::LEFT_CENTER,
                    truncated_path,
                    egui::FontId::proportional(10.5),
                    Color32::from_rgb(140, 145, 160),
                );

                // Folder name (aligned with header at 50% width)
                let folder_name = path
                    .parent()
                    .and_then(|p| p.file_name())
                    .and_then(|s| s.to_str())
                    .unwrap_or("Projects");
                let folder_pos = Pos2::new(row_rect.min.x + row_rect.width() * 0.50, row_rect.center().y);
                ui.painter().text(
                    folder_pos,
                    egui::Align2::LEFT_CENTER,
                    folder_name,
                    egui::FontId::proportional(12.5),
                    Color32::from_rgb(140, 145, 160),
                );

                // Size column (aligned with header at 72% width)
                let size_text = if let Ok(meta) = fs::metadata(path) {
                    format_file_size(meta.len())
                } else {
                    "Missing".to_string()
                };
                let size_pos = Pos2::new(row_rect.min.x + row_rect.width() * 0.72, row_rect.center().y);
                ui.painter().text(
                    size_pos,
                    egui::Align2::LEFT_CENTER,
                    size_text,
                    egui::FontId::proportional(12.5),
                    Color32::from_rgb(140, 145, 160),
                );

                // Action button on right (Open + 🗑 Delete from computer)
                let mut delete_clicked = false;
                let mut open_clicked = false;
                if is_hovered {
                    let delete_btn_rect = Rect::from_center_size(
                        Pos2::new(row_rect.max.x - 18.0, row_rect.center().y),
                        Vec2::new(20.0, 20.0),
                    );
                    let delete_res = ui.allocate_rect(delete_btn_rect, egui::Sense::click());
                    if delete_res.hovered() {
                        ui.painter().circle_filled(delete_btn_rect.center(), 9.0, if dark_mode { Color32::from_rgb(60, 25, 25) } else { Color32::from_rgb(254, 226, 226) });
                        delete_res.show_tooltip_text("Delete board from computer");
                    }
                    ui.painter().text(
                        delete_btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "🗑",
                        egui::FontId::proportional(11.5),
                        if delete_res.hovered() { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(150, 155, 170) },
                    );
                    if delete_res.clicked() {
                        delete_clicked = true;
                        self.pending_delete_path = Some(path.clone());
                    }

                    let open_btn_rect = Rect::from_center_size(
                        Pos2::new(row_rect.max.x - 64.0, row_rect.center().y),
                        Vec2::new(55.0, 24.0),
                    );
                    let open_res = ui.allocate_rect(open_btn_rect, egui::Sense::click());
                    ui.painter().rect_filled(
                        open_btn_rect,
                        CornerRadius::same(4),
                        if open_res.hovered() { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(35, 75, 210) },
                    );
                    ui.painter().text(
                        open_btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Open",
                        egui::FontId::proportional(11.5),
                        Color32::WHITE,
                    );
                    if open_res.clicked() {
                        open_clicked = true;
                        *action = HomeScreenAction::OpenRecent(path.clone());
                    }
                }

                if star_res.clicked() {
                    self.toggle_star(path);
                    *action = HomeScreenAction::ToggleStar(path.clone());
                } else if !delete_clicked && !open_clicked && !ui.rect_contains_pointer(star_rect) && row_res.clicked() {
                    *action = HomeScreenAction::OpenRecent(path.clone());
                }

                ui.add_space(2.0);
            }
        }
    }

    // -------------------------------------------------------------
    // 3. BROWSE VIEW (Filesystem Board Explorer)
    // -------------------------------------------------------------
    fn render_browse_view(&mut self, ui: &mut Ui, dark_mode: bool, action: &mut HomeScreenAction) {
        if self.browse_loaded_dir.as_ref() != Some(&self.current_browse_dir) {
            self.refresh_browse_dir();
        }

        // Browse Header
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new("📁 Browse Boards")
                        .size(22.0)
                        .strong()
                        .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
                );
                ui.label(
                    RichText::new("Explore .siro board files across your folders and drives")
                        .size(12.5)
                        .color(Color32::from_rgb(130, 135, 150)),
                );
            });

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let picker_btn = egui::Button::new(
                    RichText::new("📂  Browse with File Picker...")
                        .size(13.0)
                        .strong()
                        .color(Color32::WHITE),
                )
                .fill(if dark_mode { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(37, 99, 235) })
                .corner_radius(CornerRadius::same(6))
                .min_size(Vec2::new(210.0, 32.0));

                if ui.add(picker_btn).clicked() {
                    *action = HomeScreenAction::OpenProject(None);
                }
            });
        });

        ui.add_space(14.0);

        // Quick Locations Shortcut Pills
        ui.horizontal_wrapped(|ui| {
            ui.label(
                RichText::new("Locations:")
                    .size(12.0)
                    .strong()
                    .color(Color32::from_rgb(130, 135, 150)),
            );
            ui.add_space(4.0);

            let shortcuts = get_common_directories();
            for (name, path) in shortcuts {
                let is_selected = are_paths_equal(&self.current_browse_dir, &path);
                let pill_bg = if is_selected {
                    if dark_mode { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(37, 99, 235) }
                } else {
                    if dark_mode { Color32::from_rgb(32, 35, 45) } else { Color32::from_rgb(238, 241, 247) }
                };
                let pill_text_color = if is_selected {
                    Color32::WHITE
                } else {
                    if dark_mode { Color32::from_rgb(200, 205, 220) } else { Color32::from_rgb(50, 55, 65) }
                };

                let pill_btn = egui::Button::new(RichText::new(name).size(12.0).color(pill_text_color))
                    .fill(pill_bg)
                    .corner_radius(CornerRadius::same(12))
                    .min_size(Vec2::new(60.0, 24.0));
                if ui.add(pill_btn).clicked() {
                    self.set_browse_dir(path);
                }
                ui.add_space(4.0);
            }

            let choose_btn = egui::Button::new(
                RichText::new("📂 Choose Folder...")
                    .size(12.0)
                    .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
            )
            .fill(if dark_mode { Color32::from_rgb(34, 38, 48) } else { Color32::from_rgb(238, 241, 247) })
            .corner_radius(CornerRadius::same(12))
            .min_size(Vec2::new(110.0, 24.0));

            if ui.add(choose_btn).clicked() {
                if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                    self.set_browse_dir(folder);
                }
            }
        });

        ui.add_space(10.0);

        // Breadcrumb & Path Bar
        egui::Frame::NONE
            .fill(if dark_mode { Color32::from_rgb(28, 31, 40) } else { Color32::from_rgb(245, 247, 250) })
            .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(45, 48, 60) } else { Color32::from_rgb(225, 228, 235) }))
            .corner_radius(CornerRadius::same(6))
            .inner_margin(egui::Margin::symmetric(10, 6))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    let has_parent = self.current_browse_dir.parent().is_some();
                    let up_btn = egui::Button::new(RichText::new("⬆ Up").size(12.0))
                        .min_size(Vec2::new(45.0, 22.0));
                    if ui.add_enabled(has_parent, up_btn).on_hover_text("Navigate to parent directory").clicked() {
                        if let Some(parent) = self.current_browse_dir.parent() {
                            self.set_browse_dir(parent.to_path_buf());
                        }
                    }

                    if ui.add(egui::Button::new(RichText::new("🔄").size(12.0)).min_size(Vec2::new(26.0, 22.0))).on_hover_text("Refresh folder").clicked() {
                        self.refresh_browse_dir();
                    }

                    ui.separator();

                    // Clickable breadcrumb path ancestors
                    let ancestors = get_path_ancestors(&self.current_browse_dir);
                    let total = ancestors.len();
                    for (idx, anc) in ancestors.iter().enumerate() {
                        let name = if anc.parent().is_none() {
                            anc.to_string_lossy().to_string()
                        } else {
                            anc.file_name().and_then(|s| s.to_str()).unwrap_or("dir").to_string()
                        };
                        let is_current = idx == total - 1;
                        let text = if is_current {
                            RichText::new(&name).size(12.5).strong().color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) })
                        } else {
                            RichText::new(&name).size(12.0).color(if dark_mode { Color32::from_rgb(160, 165, 180) } else { Color32::from_rgb(90, 95, 110) })
                        };

                        if ui.link(text).clicked() {
                            self.set_browse_dir(anc.clone());
                        }

                        if !is_current {
                            ui.label(RichText::new("›").size(11.0).color(Color32::from_rgb(160, 165, 180)));
                        }
                    }
                });
            });

        ui.add_space(16.0);

        // Display contents
        if let Some(err) = self.browse_error.clone() {
            egui::Frame::NONE
                .fill(if dark_mode { Color32::from_rgb(45, 20, 20) } else { Color32::from_rgb(254, 242, 242) })
                .stroke(Stroke::new(1.0_f32, Color32::from_rgb(239, 68, 68)))
                .corner_radius(CornerRadius::same(8))
                .inner_margin(egui::Margin::symmetric(16, 14))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new("⚠").size(16.0).color(Color32::from_rgb(239, 68, 68)));
                        ui.label(RichText::new(format!("Cannot open directory: {}", err)).size(13.0).color(Color32::from_rgb(185, 28, 28)));
                        if ui.button("Return to Projects").clicked() {
                            self.set_browse_dir(default_projects_dir());
                        }
                    });
                });
            return;
        }

        let query = self.search_query.trim().to_lowercase();
        let filtered_subdirs: Vec<BrowseFolderItem> = self
            .browse_folders
            .iter()
            .filter(|dir| query.is_empty() || dir.name.to_lowercase().contains(&query))
            .cloned()
            .collect();

        let filtered_boards: Vec<BrowseBoardItem> = self
            .browse_files
            .iter()
            .filter(|b| query.is_empty() || b.name.to_lowercase().contains(&query))
            .cloned()
            .collect();

        // Subdirectories Section
        if !filtered_subdirs.is_empty() {
            ui.label(
                RichText::new("Folders")
                    .size(13.0)
                    .strong()
                    .color(Color32::from_rgb(130, 135, 150)),
            );
            ui.add_space(6.0);
            ui.horizontal_wrapped(|ui| {
                for dir in &filtered_subdirs {
                    let folder_btn = egui::Button::new(
                        RichText::new(format!("📁 {}", dir.name))
                            .size(12.5)
                            .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
                    )
                    .fill(if dark_mode { Color32::from_rgb(32, 36, 46) } else { Color32::from_rgb(244, 246, 250) })
                    .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(50, 55, 70) } else { Color32::from_rgb(225, 228, 235) }))
                    .corner_radius(CornerRadius::same(6))
                    .min_size(Vec2::new(100.0, 28.0));
                    if ui.add(folder_btn).clicked() {
                        self.set_browse_dir(dir.path.clone());
                    }
                    ui.add_space(6.0);
                }
            });
            ui.add_space(18.0);
        }

        // Boards Section Header
        ui.horizontal(|ui| {
            ui.label(
                RichText::new(format!("Boards ({})", filtered_boards.len()))
                    .size(14.0)
                    .strong()
                    .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let list_active = !self.is_grid_view;
                let grid_active = self.is_grid_view;

                if ui.selectable_label(list_active, "☰").clicked() {
                    self.is_grid_view = false;
                }
                if ui.selectable_label(grid_active, "☷").clicked() {
                    self.is_grid_view = true;
                }
            });
        });

        ui.add_space(8.0);

        if self.browse_files.is_empty() {
            // Truly empty directory
            egui::Frame::NONE
                .fill(if dark_mode { Color32::from_rgb(26, 28, 36) } else { Color32::from_rgb(248, 250, 252) })
                .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(45, 48, 60) } else { Color32::from_rgb(228, 231, 238) }))
                .corner_radius(CornerRadius::same(10))
                .inner_margin(egui::Margin::symmetric(24, 28))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(RichText::new("📁").size(32.0).color(Color32::from_rgb(245, 158, 11)));
                        ui.add_space(8.0);
                        ui.label(
                            RichText::new("No .siro boards found in this directory")
                                .size(15.5)
                                .strong()
                                .color(if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) }),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            RichText::new("Save your projects here or choose another folder to open existing boards.")
                                .size(12.5)
                                .color(Color32::from_rgb(140, 145, 160)),
                        );
                        ui.add_space(14.0);
                        ui.horizontal(|ui| {
                            let new_btn = egui::Button::new(RichText::new("+ Create New Board").size(12.5))
                                .fill(if dark_mode { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(37, 99, 235) })
                                .corner_radius(CornerRadius::same(6))
                                .min_size(Vec2::new(135.0, 30.0));
                            if ui.add(new_btn).clicked() {
                                *action = HomeScreenAction::NewProject;
                            }

                            ui.add_space(8.0);
                            let pick_btn = egui::Button::new(RichText::new("📂 Browse with File Picker...").size(12.5))
                                .fill(if dark_mode { Color32::from_rgb(38, 42, 54) } else { Color32::WHITE })
                                .stroke(Stroke::new(1.0_f32, if dark_mode { Color32::from_rgb(60, 65, 80) } else { Color32::from_rgb(220, 224, 232) }))
                                .corner_radius(CornerRadius::same(6))
                                .min_size(Vec2::new(180.0, 30.0));
                            if ui.add(pick_btn).clicked() {
                                *action = HomeScreenAction::OpenProject(None);
                            }
                        });
                    });
                });
        } else if filtered_boards.is_empty() {
            // Query returned 0 results
            self.render_empty_state_card(ui, dark_mode, "🔍", "No boards in this directory match your search query.");
        } else if self.is_grid_view {
            // Browse Grid View
            let card_w = 210.0;
            let card_h = 130.0;
            let gap = 16.0;

            let avail_w = ui.available_width();
            let cols = ((avail_w + gap) / (card_w + gap)).floor().max(1.0) as usize;

            for chunk in filtered_boards.chunks(cols) {
                ui.horizontal(|ui| {
                    for b in chunk {
                        let filename = &b.name;
                        let path = &b.path;

                        let (rect, res) = ui.allocate_exact_size(Vec2::new(card_w, card_h), egui::Sense::click());
                        let is_hovered = ui.rect_contains_pointer(rect) || res.hovered();

                        let bg = if is_hovered {
                            if dark_mode { Color32::from_rgb(36, 40, 52) } else { Color32::from_rgb(240, 245, 255) }
                        } else {
                            if dark_mode { Color32::from_rgb(26, 28, 36) } else { Color32::from_rgb(250, 251, 254) }
                        };

                        ui.painter().rect_filled(rect, CornerRadius::same(8), bg);
                        ui.painter().rect_stroke(
                            rect,
                            CornerRadius::same(8),
                            Stroke::new(1.0_f32, if is_hovered { Color32::from_rgb(45, 96, 255) } else if dark_mode { Color32::from_rgb(45, 48, 60) } else { Color32::from_rgb(228, 231, 238) }),
                            StrokeKind::Middle,
                        );

                        // Preview area
                        let preview_rect = Rect::from_min_size(rect.min + Vec2::new(8.0, 8.0), Vec2::new(card_w - 16.0, 72.0));
                        ui.painter().rect_filled(preview_rect, CornerRadius::same(5), if dark_mode { Color32::from_rgb(20, 22, 28) } else { Color32::from_rgb(242, 244, 248) });
                        ui.painter().text(
                            preview_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "📐",
                            egui::FontId::proportional(26.0),
                            Color32::from_rgb(45, 96, 255),
                        );

                        // Star button in top-right of preview area
                        let star_rect = Rect::from_center_size(
                            Pos2::new(preview_rect.max.x - 14.0, preview_rect.min.y + 14.0),
                            Vec2::new(22.0, 22.0),
                        );
                        let star_res = ui.allocate_rect(star_rect, egui::Sense::click());
                        let is_st = self.is_starred(path);
                        render_vector_star(
                            ui.painter(),
                            star_rect.center(),
                            7.0,
                            is_st,
                            star_res.hovered(),
                            dark_mode,
                        );
                        if star_res.hovered() {
                            star_res.show_tooltip_text(if is_st { "Unstar board" } else { "Star board" });
                        }

                        // Delete button in top-left of preview area on hover
                        let delete_rect = Rect::from_center_size(
                            Pos2::new(preview_rect.min.x + 14.0, preview_rect.min.y + 14.0),
                            Vec2::new(22.0, 22.0),
                        );
                        let mut delete_clicked = false;
                        if is_hovered {
                            let del_res = ui.allocate_rect(delete_rect, egui::Sense::click());
                            if del_res.hovered() {
                                ui.painter().circle_filled(
                                    delete_rect.center(),
                                    10.0,
                                    if dark_mode { Color32::from_rgb(60, 25, 25) } else { Color32::from_rgb(254, 226, 226) },
                                );
                                del_res.show_tooltip_text("Delete board from computer");
                            }
                            ui.painter().text(
                                delete_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "🗑",
                                egui::FontId::proportional(11.5),
                                if del_res.hovered() { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(150, 155, 170) },
                            );
                            if del_res.clicked() {
                                delete_clicked = true;
                                self.pending_delete_path = Some(path.clone());
                            }
                        }

                        // Title text
                        ui.painter().text(
                            rect.min + Vec2::new(10.0, 88.0),
                            egui::Align2::LEFT_TOP,
                            filename,
                            egui::FontId::proportional(13.0),
                            if dark_mode { Color32::WHITE } else { Color32::from_rgb(20, 22, 28) },
                        );

                        // Size & date subtext
                        let sub_info = format!("{} • {}", format_file_size(b.size), format_system_time(b.modified));
                        ui.painter().text(
                            rect.min + Vec2::new(10.0, 108.0),
                            egui::Align2::LEFT_TOP,
                            sub_info,
                            egui::FontId::proportional(10.5),
                            Color32::from_rgb(140, 145, 160),
                        );

                        if star_res.clicked() {
                            self.toggle_star(path);
                            *action = HomeScreenAction::ToggleStar(path.clone());
                        } else if !delete_clicked && !ui.rect_contains_pointer(star_rect) && !ui.rect_contains_pointer(delete_rect) && res.clicked() {
                            *action = HomeScreenAction::OpenRecent(path.clone());
                        }
                    }
                });
                ui.add_space(gap);
            }
        } else {
            // Browse List View
            let row_h = 42.0;

            // Table Header aligned with data columns
            render_list_table_header(ui, "Name", "Size", "Last modified");
            ui.add_space(4.0);

            for b in &filtered_boards {
                let filename = &b.name;
                let path = &b.path;
                let path_str = path.to_string_lossy().to_string();

                let (row_rect, row_res) = ui.allocate_exact_size(Vec2::new(ui.available_width(), row_h), egui::Sense::click());
                let is_hovered = ui.rect_contains_pointer(row_rect) || row_res.hovered();

                if is_hovered {
                    ui.painter().rect_filled(
                        row_rect,
                        CornerRadius::same(6),
                        if dark_mode { Color32::from_rgb(32, 35, 46) } else { Color32::from_rgb(244, 246, 250) },
                    );
                }

                // Star button (clean vector star with hover feedback)
                let star_rect = Rect::from_center_size(row_rect.left_center() + Vec2::new(16.0, 0.0), Vec2::new(22.0, 22.0));
                let star_res = ui.allocate_rect(star_rect, egui::Sense::click());
                let is_st = self.is_starred(path);
                render_vector_star(
                    ui.painter(),
                    star_rect.center(),
                    7.0,
                    is_st,
                    star_res.hovered(),
                    dark_mode,
                );
                if star_res.hovered() {
                    star_res.show_tooltip_text(if is_st { "Unstar board" } else { "Star board" });
                }

                // Icon
                let icon_pos = row_rect.left_center() + Vec2::new(38.0, 0.0);
                ui.painter().text(
                    icon_pos,
                    egui::Align2::LEFT_CENTER,
                    "📐",
                    egui::FontId::proportional(16.0),
                    Color32::from_rgb(45, 96, 255),
                );

                // Board name
                let name_pos = icon_pos + Vec2::new(24.0, -8.0);
                ui.painter().text(
                    name_pos,
                    egui::Align2::LEFT_CENTER,
                    filename,
                    egui::FontId::proportional(13.5),
                    if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) },
                );

                // File path subtext (Unicode-safe truncation)
                let sub_pos = icon_pos + Vec2::new(24.0, 9.0);
                let truncated_path = truncate_path_string(&path_str, 50);
                ui.painter().text(
                    sub_pos,
                    egui::Align2::LEFT_CENTER,
                    truncated_path,
                    egui::FontId::proportional(10.5),
                    Color32::from_rgb(140, 145, 160),
                );

                // Size column (aligned with header at 50% width)
                let size_pos = Pos2::new(row_rect.min.x + row_rect.width() * 0.50, row_rect.center().y);
                ui.painter().text(
                    size_pos,
                    egui::Align2::LEFT_CENTER,
                    format_file_size(b.size),
                    egui::FontId::proportional(12.5),
                    Color32::from_rgb(140, 145, 160),
                );

                // Last modified column (aligned with header at 72% width)
                let time_pos = Pos2::new(row_rect.min.x + row_rect.width() * 0.72, row_rect.center().y);
                ui.painter().text(
                    time_pos,
                    egui::Align2::LEFT_CENTER,
                    format_system_time(b.modified),
                    egui::FontId::proportional(12.5),
                    Color32::from_rgb(140, 145, 160),
                );

                // Action button on right (Open + 🗑 Delete from computer)
                let mut delete_clicked = false;
                let mut open_clicked = false;
                if is_hovered {
                    let delete_btn_rect = Rect::from_center_size(
                        Pos2::new(row_rect.max.x - 18.0, row_rect.center().y),
                        Vec2::new(20.0, 20.0),
                    );
                    let delete_res = ui.allocate_rect(delete_btn_rect, egui::Sense::click());
                    if delete_res.hovered() {
                        ui.painter().circle_filled(delete_btn_rect.center(), 9.0, if dark_mode { Color32::from_rgb(60, 25, 25) } else { Color32::from_rgb(254, 226, 226) });
                        delete_res.show_tooltip_text("Delete board from computer");
                    }
                    ui.painter().text(
                        delete_btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "🗑",
                        egui::FontId::proportional(11.5),
                        if delete_res.hovered() { Color32::from_rgb(239, 68, 68) } else { Color32::from_rgb(150, 155, 170) },
                    );
                    if delete_res.clicked() {
                        delete_clicked = true;
                        self.pending_delete_path = Some(path.clone());
                    }

                    let open_btn_rect = Rect::from_center_size(
                        Pos2::new(row_rect.max.x - 64.0, row_rect.center().y),
                        Vec2::new(55.0, 24.0),
                    );
                    let open_res = ui.allocate_rect(open_btn_rect, egui::Sense::click());
                    ui.painter().rect_filled(
                        open_btn_rect,
                        CornerRadius::same(4),
                        if open_res.hovered() { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(35, 75, 210) },
                    );
                    ui.painter().text(
                        open_btn_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "Open",
                        egui::FontId::proportional(11.5),
                        Color32::WHITE,
                    );
                    if open_res.clicked() {
                        open_clicked = true;
                        *action = HomeScreenAction::OpenRecent(path.clone());
                    }
                }

                if star_res.clicked() {
                    self.toggle_star(path);
                    *action = HomeScreenAction::ToggleStar(path.clone());
                } else if !delete_clicked && !open_clicked && !ui.rect_contains_pointer(star_rect) && row_res.clicked() {
                    *action = HomeScreenAction::OpenRecent(path.clone());
                }

                ui.add_space(2.0);
            }
        }
    }

    fn render_empty_state_card(&self, ui: &mut Ui, dark_mode: bool, icon: &str, message: &str) {
        let (empty_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 100.0), egui::Sense::hover());
        ui.painter().rect_filled(
            empty_rect,
            CornerRadius::same(8),
            if dark_mode { Color32::from_rgb(26, 28, 36) } else { Color32::from_rgb(248, 250, 252) },
        );
        ui.painter().text(
            empty_rect.center() - Vec2::new(0.0, 10.0),
            egui::Align2::CENTER_CENTER,
            icon,
            egui::FontId::proportional(22.0),
            Color32::from_rgb(140, 145, 160),
        );
        ui.painter().text(
            empty_rect.center() + Vec2::new(0.0, 16.0),
            egui::Align2::CENTER_CENTER,
            message,
            egui::FontId::proportional(13.0),
            Color32::from_rgb(140, 145, 160),
        );
    }
}

/// Renders a crisp geometric 5-point star polygon using math (sin/cos).
/// Solid filled warm gold (#f59e0b) when starred; clean outline stroke when unstarred.
/// Ensures fixed bounding size across both states to prevent layout/size jumping.
pub fn render_vector_star(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    is_starred: bool,
    is_hovered: bool,
    dark_mode: bool,
) {
    let r_inner = radius * 0.42;
    let mut pts = Vec::with_capacity(10);
    for i in 0..10 {
        let angle = -std::f32::consts::FRAC_PI_2 + (i as f32) * (std::f32::consts::PI / 5.0);
        let r = if i % 2 == 0 { radius } else { r_inner };
        pts.push(Pos2::new(center.x + r * angle.cos(), center.y + r * angle.sin()));
    }

    if is_starred {
        let gold_fill = Color32::from_rgb(245, 158, 11); // #f59e0b
        let gold_stroke = if is_hovered {
            Color32::from_rgb(217, 119, 6)
        } else {
            Color32::from_rgb(235, 150, 10)
        };

        // Fill inner pentagon
        painter.add(PathShape::convex_polygon(
            vec![pts[1], pts[3], pts[5], pts[7], pts[9]],
            gold_fill,
            Stroke::NONE,
        ));
        // Fill 5 tip triangles
        painter.add(PathShape::convex_polygon(
            vec![pts[9], pts[0], pts[1]],
            gold_fill,
            Stroke::NONE,
        ));
        painter.add(PathShape::convex_polygon(
            vec![pts[1], pts[2], pts[3]],
            gold_fill,
            Stroke::NONE,
        ));
        painter.add(PathShape::convex_polygon(
            vec![pts[3], pts[4], pts[5]],
            gold_fill,
            Stroke::NONE,
        ));
        painter.add(PathShape::convex_polygon(
            vec![pts[5], pts[6], pts[7]],
            gold_fill,
            Stroke::NONE,
        ));
        painter.add(PathShape::convex_polygon(
            vec![pts[7], pts[8], pts[9]],
            gold_fill,
            Stroke::NONE,
        ));

        // Crisp outline
        painter.add(PathShape::closed_line(
            pts,
            Stroke::new(1.0_f32, gold_stroke),
        ));
    } else {
        let outline_color = if is_hovered {
            Color32::from_rgb(245, 158, 11) // Gold highlight on hover
        } else if dark_mode {
            Color32::from_rgb(203, 213, 225) // #cbd5e1 in dark mode
        } else {
            Color32::from_rgb(148, 163, 184) // #94a3b8 in light mode
        };

        painter.add(PathShape::closed_line(
            pts,
            Stroke::new(1.2_f32, outline_color),
        ));
    }
}

/// Unified table header rendering perfectly aligned with list row data columns
fn render_list_table_header(ui: &mut Ui, col1: &str, col2: &str, col3: &str) {
    let (header_rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), 24.0), egui::Sense::hover());
    let p = ui.painter();
    let text_color = Color32::from_rgb(130, 135, 150);
    let font = egui::FontId::proportional(12.0);

    // Col 1: Name (aligned with icon + title at 62.0)
    p.text(
        Pos2::new(header_rect.min.x + 62.0, header_rect.center().y),
        egui::Align2::LEFT_CENTER,
        col1,
        font.clone(),
        text_color,
    );

    // Col 2: (aligned with row col2 at 50% width)
    p.text(
        Pos2::new(header_rect.min.x + header_rect.width() * 0.50, header_rect.center().y),
        egui::Align2::LEFT_CENTER,
        col2,
        font.clone(),
        text_color,
    );

    // Col 3: (aligned with row col3 at 72% width)
    p.text(
        Pos2::new(header_rect.min.x + header_rect.width() * 0.72, header_rect.center().y),
        egui::Align2::LEFT_CENTER,
        col3,
        font,
        text_color,
    );
}

/// Helper to decompose path ancestors for breadcrumb rendering
fn get_path_ancestors(path: &Path) -> Vec<PathBuf> {
    let mut ancestors = Vec::new();
    let mut curr = Some(path);
    while let Some(p) = curr {
        ancestors.push(p.to_path_buf());
        curr = p.parent();
    }
    ancestors.reverse();
    ancestors
}

// -------------------------------------------------------------
// Miro Template Cards Rendering
// -------------------------------------------------------------

fn render_blank_board_card(ui: &mut Ui, width: f32, height: f32, dark_mode: bool) -> bool {
    let (rect, res) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::click());
    let is_hovered = res.hovered();

    let bg_color = if is_hovered {
        if dark_mode { Color32::from_rgb(34, 38, 48) } else { Color32::from_rgb(244, 247, 255) }
    } else {
        if dark_mode { Color32::from_rgb(26, 28, 36) } else { Color32::from_rgb(252, 253, 255) }
    };
    let border_color = if is_hovered {
        Color32::from_rgb(45, 96, 255)
    } else if dark_mode {
        Color32::from_rgb(48, 52, 65)
    } else {
        Color32::from_rgb(220, 224, 232)
    };

    ui.painter().rect_filled(rect, CornerRadius::same(10), bg_color);
    ui.painter().rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

    // Inner dashed/rounded box with "+" icon
    let inner_box = Rect::from_center_size(
        Pos2::new(rect.center().x, rect.min.y + height * 0.42),
        Vec2::new(width * 0.65, height * 0.48),
    );
    ui.painter().rect_stroke(
        inner_box,
        CornerRadius::same(6),
        Stroke::new(1.2_f32, if is_hovered { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(180, 185, 195) }),
        StrokeKind::Middle,
    );
    ui.painter().text(
        inner_box.center(),
        egui::Align2::CENTER_CENTER,
        "+",
        egui::FontId::proportional(22.0),
        if is_hovered { Color32::from_rgb(45, 96, 255) } else { Color32::from_rgb(120, 125, 140) },
    );

    // Label: "Blank board"
    ui.painter().text(
        Pos2::new(rect.center().x, rect.max.y - 14.0),
        egui::Align2::CENTER_CENTER,
        "Blank board",
        egui::FontId::proportional(12.0),
        if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) },
    );

    res.clicked()
}

fn render_template_tile(
    ui: &mut Ui,
    title: &str,
    template: TemplateKind,
    accent: Color32,
    width: f32,
    height: f32,
    dark_mode: bool,
) -> bool {
    let (rect, res) = ui.allocate_exact_size(Vec2::new(width, height), egui::Sense::click());
    let is_hovered = res.hovered();

    let bg_color = if is_hovered {
        if dark_mode { Color32::from_rgb(34, 38, 48) } else { Color32::from_rgb(244, 247, 255) }
    } else {
        if dark_mode { Color32::from_rgb(26, 28, 36) } else { Color32::from_rgb(252, 253, 255) }
    };
    let border_color = if is_hovered {
        accent
    } else if dark_mode {
        Color32::from_rgb(48, 52, 65)
    } else {
        Color32::from_rgb(220, 224, 232)
    };

    ui.painter().rect_filled(rect, CornerRadius::same(10), bg_color);
    ui.painter().rect_stroke(rect, CornerRadius::same(10), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);

    // Graphical Illustration Preview Area
    let preview_rect = Rect::from_min_size(rect.min + Vec2::new(8.0, 8.0), Vec2::new(width - 16.0, height * 0.58));
    ui.painter().rect_filled(
        preview_rect,
        CornerRadius::same(6),
        if dark_mode { Color32::from_rgb(20, 22, 28) } else { Color32::from_rgb(242, 245, 250) },
    );

    match template {
        TemplateKind::ElectronicsBreadboard => {
            // Draw mini breadboard + chip + LED graphic
            let chip = Rect::from_center_size(preview_rect.center() - Vec2::new(20.0, 0.0), Vec2::new(32.0, 18.0));
            ui.painter().rect_filled(chip, CornerRadius::same(2), Color32::from_rgb(22, 85, 48));
            let led_pos = preview_rect.center() + Vec2::new(24.0, -4.0);
            ui.painter().circle_filled(led_pos, 5.0, Color32::from_rgb(40, 220, 90));
            // Wire line
            ui.painter().line_segment(
                [chip.right_center(), led_pos],
                Stroke::new(1.5_f32, Color32::from_rgb(240, 200, 30)),
            );
        }
        TemplateKind::CircuitSchematic => {
            // Draw mini schematic: Resistor + Ground
            let c = preview_rect.center();
            ui.painter().line_segment([c - Vec2::new(35.0, 0.0), c - Vec2::new(15.0, 0.0)], Stroke::new(1.5_f32, accent));
            ui.painter().rect_stroke(Rect::from_center_size(c, Vec2::new(26.0, 12.0)), CornerRadius::ZERO, Stroke::new(1.5_f32, accent), StrokeKind::Middle);
            ui.painter().line_segment([c + Vec2::new(13.0, 0.0), c + Vec2::new(35.0, 0.0)], Stroke::new(1.5_f32, accent));
        }
        TemplateKind::HackClubShip => {
            // Draw mini Grant checklist & BOM table
            let c = preview_rect.center();
            let b1 = Rect::from_center_size(c - Vec2::new(0.0, 8.0), Vec2::new(60.0, 10.0));
            let b2 = Rect::from_center_size(c + Vec2::new(0.0, 8.0), Vec2::new(60.0, 10.0));
            ui.painter().rect_filled(b1, CornerRadius::same(2), Color32::from_rgb(235, 60, 60));
            ui.painter().rect_filled(b2, CornerRadius::same(2), Color32::from_rgb(70, 150, 240));
        }
        TemplateKind::FirmwareStateMachine => {
            // Draw mini bubbles with arrows
            let c = preview_rect.center();
            let p1 = c - Vec2::new(28.0, 0.0);
            let p2 = c + Vec2::new(28.0, 0.0);
            ui.painter().circle_filled(p1, 9.0, Color32::from_rgb(35, 115, 230));
            ui.painter().circle_filled(p2, 9.0, Color32::from_rgb(30, 190, 140));
            ui.painter().line_segment([p1 + Vec2::new(9.0, 0.0), p2 - Vec2::new(9.0, 0.0)], Stroke::new(1.5_f32, Color32::from_rgb(180, 190, 210)));
        }
    }

    // Title label
    ui.painter().text(
        Pos2::new(rect.center().x, rect.max.y - 14.0),
        egui::Align2::CENTER_CENTER,
        title,
        egui::FontId::proportional(11.5),
        if dark_mode { Color32::WHITE } else { Color32::from_rgb(24, 26, 32) },
    );

    res.clicked()
}

pub fn create_starter_project(kind: TemplateKind) -> Project {
    match kind {
        TemplateKind::ElectronicsBreadboard => {
            let mut project = Project::new("Breadboard & MCU Prototype");
            let board = project.active_board_mut();

            // Place Pico MCU
            let pico = ComponentElement::new(Uuid::new_v4(), ComponentKind::RaspberryPiPico, Pos2::new(100.0, 100.0));
            let pico_id = pico.id;
            board.add_element(BoardElement::Component(pico));

            // Place Breadboard
            let breadboard = ComponentElement::new(Uuid::new_v4(), ComponentKind::BreadboardSection, Pos2::new(420.0, 100.0));
            board.add_element(BoardElement::Component(breadboard));

            // Place LED
            let led = ComponentElement::new(Uuid::new_v4(), ComponentKind::Led, Pos2::new(460.0, 140.0));
            let led_id = led.id;
            board.add_element(BoardElement::Component(led));

            // Place 220 Ohm Resistor
            let resistor = ComponentElement::new(Uuid::new_v4(), ComponentKind::Resistor, Pos2::new(460.0, 220.0));
            let res_id = resistor.id;
            board.add_element(BoardElement::Component(resistor));

            // Wire connections
            let w1 = WireElement::new(
                Uuid::new_v4(),
                WireEndpoint { element_id: Some(pico_id), terminal_idx: Some(0), pos: Pos2::new(260.0, 130.0) },
                WireEndpoint { element_id: Some(led_id), terminal_idx: Some(0), pos: Pos2::new(460.0, 140.0) },
                WireColor::Red5V,
            );
            board.add_element(BoardElement::Wire(w1));

            let w2 = WireElement::new(
                Uuid::new_v4(),
                WireEndpoint { element_id: Some(led_id), terminal_idx: Some(1), pos: Pos2::new(490.0, 140.0) },
                WireEndpoint { element_id: Some(res_id), terminal_idx: Some(0), pos: Pos2::new(460.0, 220.0) },
                WireColor::YellowSignal,
            );
            board.add_element(BoardElement::Wire(w2));

            let w3 = WireElement::new(
                Uuid::new_v4(),
                WireEndpoint { element_id: Some(res_id), terminal_idx: Some(1), pos: Pos2::new(510.0, 220.0) },
                WireEndpoint { element_id: Some(pico_id), terminal_idx: Some(2), pos: Pos2::new(260.0, 190.0) },
                WireColor::BlackGnd,
            );
            board.add_element(BoardElement::Wire(w3));

            board.sync_connected_wires();
            project
        }
        TemplateKind::CircuitSchematic => {
            let mut project = Project::new("Power Supply Circuit");
            let board = project.active_board_mut();

            let vreg = ComponentElement::new(Uuid::new_v4(), ComponentKind::Esp32Wroom, Pos2::new(140.0, 160.0));
            let vreg_id = vreg.id;
            board.add_element(BoardElement::Component(vreg));

            let cap = ComponentElement::new(Uuid::new_v4(), ComponentKind::Capacitor, Pos2::new(400.0, 180.0));
            let cap_id = cap.id;
            board.add_element(BoardElement::Component(cap));

            let gnd = ComponentElement::new(Uuid::new_v4(), ComponentKind::Ground, Pos2::new(300.0, 320.0));
            let gnd_id = gnd.id;
            board.add_element(BoardElement::Component(gnd));

            let w1 = WireElement::new(
                Uuid::new_v4(),
                WireEndpoint { element_id: Some(vreg_id), terminal_idx: Some(0), pos: Pos2::new(260.0, 180.0) },
                WireEndpoint { element_id: Some(cap_id), terminal_idx: Some(0), pos: Pos2::new(400.0, 180.0) },
                WireColor::Red5V,
            );
            board.add_element(BoardElement::Wire(w1));

            let w2 = WireElement::new(
                Uuid::new_v4(),
                WireEndpoint { element_id: Some(vreg_id), terminal_idx: Some(1), pos: Pos2::new(260.0, 240.0) },
                WireEndpoint { element_id: Some(gnd_id), terminal_idx: Some(0), pos: Pos2::new(300.0, 320.0) },
                WireColor::BlackGnd,
            );
            board.add_element(BoardElement::Wire(w2));

            let w3 = WireElement::new(
                Uuid::new_v4(),
                WireEndpoint { element_id: Some(cap_id), terminal_idx: Some(1), pos: Pos2::new(440.0, 260.0) },
                WireEndpoint { element_id: Some(gnd_id), terminal_idx: Some(0), pos: Pos2::new(300.0, 320.0) },
                WireColor::BlackGnd,
            );
            board.add_element(BoardElement::Wire(w3));

            board.sync_connected_wires();
            project
        }
        TemplateKind::HackClubShip => {
            let mut project = Project::new("Hack Club Grant Ship");
            let bom_table = project.planner.to_canvas_table(Pos2::new(120.0, 120.0));
            project.active_board_mut().add_element(BoardElement::Table(bom_table));

            let note = TextElement::new_sticky(
                Uuid::new_v4(),
                Pos2::new(120.0, 360.0),
                "HACK CLUB SHIP CHECKLIST:\n- Video demo recorded\n- Code pushed to GitHub\n- Post on #scrapbook\n- Sourcing grant budget under $100".to_string(),
                Color32::from_rgb(255, 180, 180),
            );
            project.active_board_mut().add_element(BoardElement::Text(note));

            project
        }
        TemplateKind::FirmwareStateMachine => {
            let mut project = Project::new("Firmware State Machine");
            let board = project.active_board_mut();

            let states = [
                ("INIT", Pos2::new(100.0, 180.0)),
                ("IDLE_LISTEN", Pos2::new(320.0, 180.0)),
                ("READ_SENSORS", Pos2::new(540.0, 180.0)),
                ("TRANSMIT_PACKET", Pos2::new(540.0, 340.0)),
                ("DEEP_SLEEP", Pos2::new(320.0, 340.0)),
            ];

            for (name, pos) in states {
                let shape = ShapeElement::new(
                    Uuid::new_v4(),
                    ShapeKind::RoundedRectangle,
                    vec![pos, pos + Vec2::new(140.0, 70.0)],
                    Color32::from_rgb(0, 210, 255),
                    2.0,
                    Color32::from_rgba_unmultiplied(0, 210, 255, 25),
                );
                board.add_element(BoardElement::Shape(shape));

                let label = TextElement::new_text(
                    Uuid::new_v4(),
                    pos + Vec2::new(20.0, 25.0),
                    name.to_string(),
                );
                board.add_element(BoardElement::Text(label));
            }

            // Arrow from INIT to IDLE
            let arrow = ShapeElement::new(
                Uuid::new_v4(),
                ShapeKind::Arrow,
                vec![Pos2::new(240.0, 215.0), Pos2::new(320.0, 215.0)],
                Color32::from_rgb(0, 210, 255),
                2.0,
                Color32::TRANSPARENT,
            );
            board.add_element(BoardElement::Shape(arrow));

            project
        }
    }
}
