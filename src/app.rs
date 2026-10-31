use crate::canvas::camera::Camera;
use crate::canvas::grid::{GridRenderer, GridSettings};
use crate::canvas::render::CanvasRenderer;
use crate::canvas::tools::*;
use crate::model::element::*;
use crate::model::project::Project;
use crate::storage::autosave::AutosaveManager;
use crate::storage::export::ExportManager;
use crate::storage::pdf_worker::PdfWorkerPool;
use crate::storage::siro_file::SiroFileManager;
use crate::ui::home_screen::{create_starter_project, HomeScreen, HomeScreenAction};
use crate::ui::planner_panel::{PlannerAction, PlannerPanel};
use crate::ui::properties_bar::{PropertiesAction, PropertiesBar};
use crate::ui::tool_rail::{ToolRail, ToolRailAction};
use crate::ui::top_bar::{TopBar, TopBarAction, TopBarState};
use eframe::egui::{self, epaint::StrokeKind, Color32, CornerRadius, Pos2, Rect, RichText, Stroke, Vec2};
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppState {
    Home,
    Editor,
}

pub struct SiroApp {
    pub state: AppState,
    pub home: HomeScreen,
    pub camera: Camera,
    pub grid_settings: GridSettings,
    pub current_tool: ActiveTool,
    pub select_tool: SelectTool,
    pub pen_tool: PenTool,
    pub shape_tool: ShapeTool,
    pub wire_tool: WireTool,
    pub eraser_tool: EraserTool,
    pub text_tool: TextTool,
    pub table_tool: TableTool,
    pub tool_rail: ToolRail,
    pub top_bar_state: TopBarState,
    pub planner_panel: PlannerPanel,
    pub planner_open: bool,
    pub dark_mode: bool,
    pub active_stroke_color: Color32,
    pub active_stroke_width: f32,
    pub active_wire_color: WireColor,
    pub active_sticky_color: Color32,
    pub autosave: AutosaveManager,
    pub pdf_worker: PdfWorkerPool,
    pub image_textures: std::collections::HashMap<Uuid, egui::TextureHandle>,
    pub pdf_textures: std::collections::HashMap<(Uuid, usize), egui::TextureHandle>,
    pub editing_text_id: Option<Uuid>,
    pub status_message: Option<(String, std::time::Instant)>,
    pub open_projects: Vec<Project>,
    pub active_project_idx: Option<usize>,
    pub settings: crate::storage::settings::AppSettings,
    pub close_dialog: Option<crate::ui::close_dialog::CloseDialogState>,
    pub allow_close: bool,
    pub popup_manager: crate::ui::PopupManager,
}

impl Default for SiroApp {
    fn default() -> Self {
        Self {
            state: AppState::Home,
            home: HomeScreen::default(),
            camera: Camera::default(),
            grid_settings: GridSettings::default(),
            current_tool: ActiveTool::Select,
            select_tool: SelectTool::default(),
            pen_tool: PenTool::default(),
            shape_tool: ShapeTool::default(),
            wire_tool: WireTool::default(),
            eraser_tool: EraserTool::default(),
            text_tool: TextTool::default(),
            table_tool: TableTool::default(),
            tool_rail: ToolRail::default(),
            top_bar_state: TopBarState::default(),
            planner_panel: PlannerPanel::default(),
            planner_open: false,
            dark_mode: crate::storage::history::load_theme_dark_mode(),
            active_stroke_color: Color32::from_rgb(45, 96, 255),
            active_stroke_width: 2.5,
            active_wire_color: WireColor::Red5V,
            active_sticky_color: Color32::from_rgb(255, 249, 177),
            autosave: AutosaveManager::default(),
            pdf_worker: PdfWorkerPool::new(),
            image_textures: std::collections::HashMap::new(),
            pdf_textures: std::collections::HashMap::new(),
            editing_text_id: None,
            status_message: None,
            open_projects: Vec::new(),
            active_project_idx: None,
            settings: crate::storage::settings::load_settings(),
            close_dialog: None,
            allow_close: false,
            popup_manager: crate::ui::PopupManager::default(),
        }
    }
}

impl SiroApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        setup_custom_fonts(&cc.egui_ctx);
        cc.egui_ctx.style_mut(|s| s.spacing.menu_width = 180.0);
        Self::default()
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), std::time::Instant::now()));
    }

    pub fn open_or_focus_file(&mut self, p: &std::path::Path) {
        if self.state == AppState::Editor {
            if let Some(active_idx) = self.active_project_idx {
                if active_idx < self.open_projects.len() {
                    if self.open_projects[active_idx].is_unsaved() {
                        if let Some(path) = self.open_projects[active_idx].filepath.clone() {
                            if let Err(err) = SiroFileManager::save_project(&mut self.open_projects[active_idx], &path) {
                                self.set_status(format!("Auto-save before open failed: {}", err));
                            }
                        }
                    }
                    self.open_projects[active_idx].active_board_mut().camera_pan = self.camera.pan;
                    self.open_projects[active_idx].active_board_mut().camera_zoom = self.camera.zoom;
                }
            }
        }

        if let Some(idx) = self.open_projects.iter().position(|proj| {
            proj.filepath.as_ref().map_or(false, |fp| crate::storage::history::are_paths_equal(fp, p))
        }) {
            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                if !stem.is_empty() {
                    self.open_projects[idx].manifest.name = stem.to_string();
                }
            }
            self.active_project_idx = Some(idx);
            self.camera.pan = self.open_projects[idx].active_board().camera_pan;
            self.camera.zoom = self.open_projects[idx].active_board().camera_zoom;
            self.state = AppState::Editor;
        } else {
            match SiroFileManager::load_project(p) {
                Ok(mut proj) => {
                    if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                        if !stem.is_empty() {
                            proj.manifest.name = stem.to_string();
                        }
                    }
                    self.home.add_recent(p);
                    self.camera.pan = proj.active_board().camera_pan;
                    self.camera.zoom = proj.active_board().camera_zoom;
                    self.open_projects.push(proj);
                    self.active_project_idx = Some(self.open_projects.len() - 1);
                    self.state = AppState::Editor;
                }
                Err(err) => {
                    self.set_status(format!("Error opening file: {}", err));
                }
            }
        }
    }

    /// Renames the project, synchronizing memory, disk file, backups, and recent/starred history.
    pub fn rename_project(&mut self, project: &mut Project, new_name: &str) {
        let clean = crate::storage::siro_file::sanitize_board_name(new_name);
        if clean.is_empty() {
            return;
        }

        if let Some(old_path) = project.filepath.clone() {
            let extension = old_path.extension().and_then(|e| e.to_str()).unwrap_or("siro");
            let new_filename = format!("{}.{}", clean, extension);
            let new_path = if let Some(parent) = old_path.parent() {
                parent.join(new_filename)
            } else {
                std::path::PathBuf::from(new_filename)
            };

            if new_path != old_path {
                let is_same_file = crate::storage::history::are_paths_equal(&old_path, &new_path);
                if new_path.exists() && !is_same_file {
                    self.set_status(format!("Cannot rename: {:?} already exists", new_path));
                    return;
                }

                // If only the case changed on a case-insensitive filesystem, rename via a temporary path
                let rename_res = if is_same_file {
                    let temp_name = format!(".rename_tmp_{}.siro", uuid::Uuid::new_v4());
                    let temp_path = old_path.parent().unwrap_or_else(|| std::path::Path::new(".")).join(temp_name);
                    std::fs::rename(&old_path, &temp_path)
                        .and_then(|_| std::fs::rename(&temp_path, &new_path))
                } else {
                    std::fs::rename(&old_path, &new_path)
                };

                if let Err(e) = rename_res {
                    self.set_status(format!("Failed to rename file: {}", e));
                    return;
                }

                let old_bak = old_path.with_extension("siro.bak");
                let new_bak = new_path.with_extension("siro.bak");
                if old_bak.exists() {
                    let bak_same = crate::storage::history::are_paths_equal(&old_bak, &new_bak);
                    if bak_same {
                        let temp_bak = old_bak.parent().unwrap_or_else(|| std::path::Path::new(".")).join(format!(".bak_tmp_{}.siro", uuid::Uuid::new_v4()));
                        let _ = std::fs::rename(&old_bak, &temp_bak).and_then(|_| std::fs::rename(&temp_bak, &new_bak));
                    } else {
                        let _ = std::fs::rename(&old_bak, &new_bak);
                    }
                }

                self.home.rename_file_reference(&old_path, &new_path);
                project.filepath = Some(new_path.clone());
                project.manifest.name = clean.clone();

                if let Err(e) = SiroFileManager::save_project(project, &new_path) {
                    self.set_status(format!("Saved rename on disk but manifest write failed: {}", e));
                } else {
                    self.set_status(format!("Renamed to {}", clean));
                }
            } else {
                project.manifest.name = clean;
            }
        } else {
            project.manifest.name = clean;
            project.mark_edited();
        }
    }

    /// Creates and opens a new project tab, auto-saving the previous project if unsaved.
    pub fn create_and_open_new_project(&mut self) {
        if self.state == AppState::Editor {
            if let Some(active_idx) = self.active_project_idx {
                if active_idx < self.open_projects.len() {
                    if self.open_projects[active_idx].is_unsaved() {
                        if let Some(path) = self.open_projects[active_idx].filepath.clone() {
                            if let Err(err) = SiroFileManager::save_project(&mut self.open_projects[active_idx], &path) {
                                self.set_status(format!("Auto-save before new project failed: {}", err));
                            }
                        }
                    }
                    self.open_projects[active_idx].active_board_mut().camera_pan = self.camera.pan;
                    self.open_projects[active_idx].active_board_mut().camera_zoom = self.camera.zoom;
                }
            }
        }

        let new_proj = Project::new("Untitled Project");
        self.open_projects.push(new_proj);
        self.active_project_idx = Some(self.open_projects.len() - 1);
        self.camera = Camera::default();
        self.select_tool.clear_selection();
        self.state = AppState::Editor;
        self.set_status("Created new project");
    }

    /// Single unified dispatch path for user actions triggered from menus, shortcuts, and buttons.
    pub fn dispatch_action(&mut self, action: TopBarAction) {
        match action {
            TopBarAction::None => {}
            TopBarAction::NewProject => {
                self.create_and_open_new_project();
            }
            TopBarAction::OpenProject => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter(format!("{} Project (*.siro)", crate::APP_NAME), &["siro"])
                    .pick_file()
                {
                    self.open_or_focus_file(&path);
                }
            }
            TopBarAction::SaveProject => {
                if let Some(active_idx) = self.active_project_idx {
                    if active_idx < self.open_projects.len() {
                        let mut proj = self.open_projects.remove(active_idx);
                        handle_top_bar_action(self, &mut proj, TopBarAction::SaveProject);
                        self.open_projects.insert(active_idx, proj);
                    }
                }
            }
            TopBarAction::SaveProjectAs => {
                if let Some(active_idx) = self.active_project_idx {
                    if active_idx < self.open_projects.len() {
                        let mut proj = self.open_projects.remove(active_idx);
                        handle_top_bar_action(self, &mut proj, TopBarAction::SaveProjectAs);
                        self.open_projects.insert(active_idx, proj);
                    }
                }
            }
            TopBarAction::ToggleTheme => {
                self.dark_mode = !self.dark_mode;
                let _ = crate::storage::history::save_theme_dark_mode(self.dark_mode);
            }
            other => {
                if let Some(active_idx) = self.active_project_idx {
                    if active_idx < self.open_projects.len() {
                        let mut proj = self.open_projects.remove(active_idx);
                        handle_top_bar_action(self, &mut proj, other);
                        self.open_projects.insert(active_idx, proj);
                    }
                }
            }
        }
    }

    /// Requests closing a project tab. Routes through close_plan.
    /// If project has content and is unsaved, opens the 3-option close modal.
    /// Otherwise closes the tab immediately.
    pub fn request_close_tab(&mut self, idx: usize) {
        if idx < self.open_projects.len() {
            let has_unsaved_content = self.open_projects[idx].has_content() && self.open_projects[idx].is_unsaved();
            match crate::ui::close_dialog::close_plan(has_unsaved_content) {
                crate::ui::close_dialog::ClosePlan::Allow => {
                    self.force_close_tab(idx);
                }
                crate::ui::close_dialog::ClosePlan::Ask => {
                    self.close_dialog = Some(crate::ui::close_dialog::CloseDialogState {
                        target: crate::ui::close_dialog::CloseDialogTarget::Tab(idx),
                    });
                }
            }
        }
    }

    /// Unconditionally closes a tab by index without asking.
    pub fn force_close_tab(&mut self, idx: usize) {
        if idx >= self.open_projects.len() {
            return;
        }
        self.open_projects.remove(idx);
        if self.open_projects.is_empty() {
            self.active_project_idx = None;
            self.state = AppState::Home;
        } else if let Some(cur) = self.active_project_idx {
            if cur == idx {
                let next = idx.min(self.open_projects.len() - 1);
                self.active_project_idx = Some(next);
                self.camera.pan = self.open_projects[next].active_board().camera_pan;
                self.camera.zoom = self.open_projects[next].active_board().camera_zoom;
            } else if cur > idx {
                self.active_project_idx = Some(cur - 1);
            }
        }
    }

    /// Saves a single project before closing/exiting.
    /// For named projects, writes to existing filepath.
    /// For never-saved projects, opens the native Save As dialog.
    /// Returns true if saved, false if cancelled or failed.
    pub fn save_single_project(&mut self, idx: usize) -> bool {
        if idx >= self.open_projects.len() {
            return true;
        }
        let project = &mut self.open_projects[idx];
        if !project.has_content() || !project.is_unsaved() {
            return true;
        }

        if let Some(path) = project.filepath.clone() {
            let res = SiroFileManager::save_project(project, &path);
            let name = project.display_name().to_string();
            match res {
                Ok(()) => {
                    self.autosave.reset_timer();
                    self.set_status(format!("Saved '{}'", name));
                    true
                }
                Err(err) => {
                    self.set_status(format!("Save failed: {}", err));
                    false
                }
            }
        } else {
            let suggested_name = format!("{}.siro", crate::storage::siro_file::sanitize_board_name(project.display_name()));
            let target_path = rfd::FileDialog::new()
                .set_file_name(&suggested_name)
                .add_filter(format!("{} Project (*.siro)", crate::APP_NAME), &["siro"])
                .save_file();

            if let Some(path) = target_path {
                let clean_path = if path.extension().is_none() {
                    path.with_extension("siro")
                } else {
                    path
                };
                if let Some(stem) = clean_path.file_stem().and_then(|s| s.to_str()) {
                    if !stem.is_empty() {
                        project.manifest.name = stem.to_string();
                    }
                }
                let res = SiroFileManager::save_project(project, &clean_path);
                let name = project.display_name().to_string();
                match res {
                    Ok(()) => {
                        self.home.add_recent(&clean_path);
                        self.autosave.reset_timer();
                        self.set_status(format!("Saved '{}'", name));
                        true
                    }
                    Err(err) => {
                        self.set_status(format!("Save As failed: {}", err));
                        false
                    }
                }
            } else {
                // User cancelled Save As: abort exit/close
                false
            }
        }
    }

    /// Saves all open projects that have content and are unsaved.
    /// Returns true if all saved successfully; returns false if any save fails or is cancelled.
    pub fn save_all_unsaved_projects(&mut self) -> bool {
        for idx in 0..self.open_projects.len() {
            if self.open_projects[idx].has_content() && self.open_projects[idx].is_unsaved() {
                if !self.save_single_project(idx) {
                    return false;
                }
            }
        }
        true
    }
}

impl eframe::App for SiroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let visuals = if self.dark_mode {
            let mut vis = egui::Visuals::dark();
            vis.slider_trailing_fill = true;
            vis.widgets.inactive.bg_fill = Color32::from_rgb(45, 50, 62);
            vis.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(70, 78, 95));
            vis.widgets.hovered.bg_fill = Color32::from_rgb(55, 62, 78);
            vis.widgets.active.bg_fill = Color32::from_rgb(45, 96, 255);
            vis.widgets.inactive.fg_stroke = Stroke::new(1.5_f32, Color32::from_rgb(220, 225, 235));
            vis
        } else {
            let mut vis = egui::Visuals::light();
            vis.window_fill = Color32::WHITE;
            vis.panel_fill = Color32::from_rgb(249, 250, 252);
            vis.extreme_bg_color = Color32::WHITE;
            vis.slider_trailing_fill = true;
            vis.widgets.inactive.bg_fill = Color32::from_rgb(228, 232, 240);
            vis.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(195, 202, 215));
            vis.widgets.hovered.bg_fill = Color32::from_rgb(218, 224, 235);
            vis.widgets.hovered.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(160, 170, 190));
            vis.widgets.active.bg_fill = Color32::from_rgb(45, 96, 255);
            vis.widgets.active.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(25, 75, 230));
            vis.widgets.inactive.fg_stroke = Stroke::new(1.5_f32, Color32::from_rgb(60, 65, 80));
            vis.widgets.open.bg_fill = Color32::from_rgb(244, 246, 250);
            vis.widgets.noninteractive.bg_fill = Color32::from_rgb(249, 250, 252);
            vis.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(226, 229, 236));
            vis
        };
        ctx.set_visuals(visuals);
        ctx.style_mut(|s| {
            s.spacing.menu_width = 180.0;
            s.spacing.slider_width = 110.0;
            s.spacing.slider_rail_height = 5.0;
        });

        // 1. Shared popup system input handling: Esc closes topmost popup first, outside click closes stack
        let _ = self.popup_manager.update_input(ctx);

        // Poll background PDF worker page results
        let completed_pages = self.pdf_worker.poll_completed();
        for res in completed_pages {
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [res.width as usize, res.height as usize],
                &res.rgba_pixels,
            );
            let tex = ctx.load_texture(
                format!("pdf_{}_{}", res.asset_id, res.page_index),
                color_image,
                egui::TextureOptions::LINEAR,
            );
            self.pdf_textures.insert((res.asset_id, res.page_index), tex);
            ctx.request_repaint();
        }

        // Intercept window close requests (Titlebar X, Taskbar Close, Alt+F4)
        if ctx.input(|i| i.viewport().close_requested()) {
            if self.allow_close {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }

            match crate::ui::close_dialog::app_close_plan(&self.open_projects) {
                crate::ui::close_dialog::ClosePlan::Allow => {
                    self.allow_close = true;
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    return;
                }
                crate::ui::close_dialog::ClosePlan::Ask => {
                    ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
                    self.close_dialog = Some(crate::ui::close_dialog::CloseDialogState {
                        target: crate::ui::close_dialog::CloseDialogTarget::App,
                    });
                }
            }
        } else if self.close_dialog.is_some() {
            // Keep window open while close dialog is awaiting user decision
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
        }

        // Global shortcuts processing (works on Home screen and in Editor)
        let is_modal_open = self.close_dialog.is_some()
            || self.home.pending_delete_path.is_some()
            || self.top_bar_state.sheet_modal.is_some();

        let is_typing = is_modal_open
            || ctx.wants_keyboard_input()
            || self.editing_text_id.is_some()
            || self.table_tool.active_cell.is_some()
            || self.top_bar_state.is_editing_title
            || self.top_bar_state.renaming_board_id.is_some();

        if !is_typing {
            if crate::shortcuts::AppShortcut::NewProject.is_triggered(ctx) {
                self.dispatch_action(TopBarAction::NewProject);
            } else if crate::shortcuts::AppShortcut::OpenProject.is_triggered(ctx) {
                self.dispatch_action(TopBarAction::OpenProject);
            } else if crate::shortcuts::AppShortcut::SaveProjectAs.is_triggered(ctx) {
                self.dispatch_action(TopBarAction::SaveProjectAs);
            } else if crate::shortcuts::AppShortcut::SaveProject.is_triggered(ctx) {
                self.dispatch_action(TopBarAction::SaveProject);
            }
        }

        match self.state {
            AppState::Home => {
                // Top Tab Strip (Miro Title Bar)
                let tab_infos: Vec<(&str, bool)> = self.open_projects
                    .iter()
                    .map(|p| (p.display_name(), p.is_unsaved()))
                    .collect();
                let mut home_strip_action = TopBarAction::None;
                let home_strip_fill = if self.dark_mode {
                    Color32::from_rgb(30, 32, 38)
                } else {
                    Color32::from_rgb(242, 244, 247)
                };
                let home_strip_border = if self.dark_mode {
                    Color32::from_rgb(45, 48, 58)
                } else {
                    Color32::from_rgb(226, 229, 234)
                };
                let home_top_frame = egui::Frame::NONE
                    .fill(home_strip_fill)
                    .stroke(Stroke::new(1.0_f32, home_strip_border));

                egui::TopBottomPanel::top("home_top_tab_strip")
                    .frame(home_top_frame)
                    .exact_height(34.0)
                    .show(ctx, |ui| {
                        home_strip_action = TopBar::render_window_tab_strip(
                            ui,
                            &tab_infos,
                            None,
                            &mut self.top_bar_state,
                            self.dark_mode,
                        );
                    });

                match home_strip_action {
                    TopBarAction::SwitchProjectTab(idx) => {
                        if idx < self.open_projects.len() {
                            self.active_project_idx = Some(idx);
                            self.camera.pan = self.open_projects[idx].active_board().camera_pan;
                            self.camera.zoom = self.open_projects[idx].active_board().camera_zoom;
                            self.state = AppState::Editor;
                        }
                    }
                    TopBarAction::SwitchBoard(_) | TopBarAction::OpenProject => {
                        if let Some(cur) = self.active_project_idx {
                            if cur < self.open_projects.len() {
                                self.camera.pan = self.open_projects[cur].active_board().camera_pan;
                                self.camera.zoom = self.open_projects[cur].active_board().camera_zoom;
                                self.state = AppState::Editor;
                            }
                        }
                    }
                    TopBarAction::CloseProjectTab(idx) => {
                        self.request_close_tab(idx);
                    }
                    TopBarAction::DeleteBoard(_) | TopBarAction::CloseProject => {
                        if let Some(cur) = self.active_project_idx {
                            self.request_close_tab(cur);
                        }
                    }
                    TopBarAction::AddBoard | TopBarAction::NewProject => {
                        self.create_and_open_new_project();
                    }
                    TopBarAction::ToggleTheme => {
                        self.dark_mode = !self.dark_mode;
                        let _ = crate::storage::history::save_theme_dark_mode(self.dark_mode);
                    }
                    _ => {}
                }

                egui::CentralPanel::default().show(ctx, |ui| {
                    match self.home.ui(ui, self.dark_mode, &mut self.settings) {
                        HomeScreenAction::None => {}
                        HomeScreenAction::NewProject => {
                            self.create_and_open_new_project();
                        }
                        HomeScreenAction::OpenProject(maybe_path) => {
                            let path = maybe_path.or_else(|| {
                                rfd::FileDialog::new()
                                    .add_filter(format!("{} Project (*.siro)", crate::APP_NAME), &["siro"])
                                    .pick_file()
                            });
                            if let Some(p) = path {
                                self.open_or_focus_file(&p);
                            }
                        }
                        HomeScreenAction::LoadTemplate(kind) => {
                            let proj = create_starter_project(kind);
                            self.camera.pan = proj.active_board().camera_pan;
                            self.camera.zoom = proj.active_board().camera_zoom;
                            self.open_projects.push(proj);
                            self.active_project_idx = Some(self.open_projects.len() - 1);
                            self.state = AppState::Editor;
                        }
                        HomeScreenAction::OpenRecent(p) => {
                            self.open_or_focus_file(&p);
                        }
                        HomeScreenAction::RemoveRecent(idx) => {
                            self.home.remove_recent(idx);
                        }
                        HomeScreenAction::ToggleStar(p) => {
                            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Board");
                            if self.home.is_starred(&p) {
                                self.set_status(format!("⭐ Starred \"{}\"", stem));
                            } else {
                                self.set_status(format!("Unstarred \"{}\"", stem));
                            }
                        }
                        HomeScreenAction::DeleteProject(p) => {
                            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("Board");
                            match self.home.delete_board_file(&p) {
                                Ok(()) => {
                                    if let Some(idx) = self.open_projects.iter().position(|proj| {
                                        proj.filepath.as_ref().map_or(false, |fp| crate::storage::history::are_paths_equal(fp, &p))
                                    }) {
                                        self.open_projects.remove(idx);
                                        if let Some(cur) = self.active_project_idx {
                                            if cur >= self.open_projects.len() {
                                                self.active_project_idx = if self.open_projects.is_empty() {
                                                    None
                                                } else {
                                                    Some(self.open_projects.len() - 1)
                                                };
                                            }
                                        }
                                    }
                                    self.set_status(format!("Permanently deleted \"{}\" from computer.", stem));
                                }
                                Err(err) => {
                                    self.set_status(format!("Error deleting file: {}", err));
                                }
                            }
                        }
                    }
                });
            }
            AppState::Editor => {
                if self.open_projects.is_empty() || self.active_project_idx.is_none() {
                    self.state = AppState::Home;
                    return;
                }
                let active_idx = self.active_project_idx.unwrap().min(self.open_projects.len() - 1);
                self.active_project_idx = Some(active_idx);

                let mut project = std::mem::replace(&mut self.open_projects[active_idx], Project::new(""));

                // Handle dragged and dropped external files (Images, PDFs, Docs)
                let dropped = ctx.input(|i| i.raw.dropped_files.clone());
                for file in dropped {
                    if let Some(path) = file.path {
                        insert_file_from_path(self, &mut project, &path);
                    }
                }

                // Ensure textures for any visible ImageElement are loaded
                for elem in &project.active_board().elements {
                    if let BoardElement::Image(img) = elem {
                        if !self.image_textures.contains_key(&img.asset_id) {
                            if let Some(asset) = project.asset_store.get(&img.asset_id) {
                                if let Ok(dyn_img) = image::load_from_memory(&asset.data) {
                                    let rgba = dyn_img.to_rgba8();
                                    let color_image = egui::ColorImage::from_rgba_unmultiplied(
                                        [rgba.width() as usize, rgba.height() as usize],
                                        rgba.as_raw(),
                                    );
                                    let tex = ctx.load_texture(
                                        format!("img_{}", img.asset_id),
                                        color_image,
                                        egui::TextureOptions::LINEAR,
                                    );
                                    self.image_textures.insert(img.asset_id, tex);
                                }
                            }
                        }
                    } else if let BoardElement::PdfCard(pdf) = elem {
                        if !self.pdf_textures.contains_key(&(pdf.asset_id, pdf.current_page)) {
                            if let Some(asset) = project.asset_store.get(&pdf.asset_id) {
                                self.pdf_worker.request_page(pdf.asset_id, pdf.current_page, asset.data.clone());
                            }
                        }
                    }
                }

                // Handle global shortcuts
                handle_global_shortcuts(self, &mut project, ctx);

                // Top Window Tab Bar Panel (Miro Tab Strip)
                let mut tab_infos: Vec<(String, bool)> = Vec::with_capacity(self.open_projects.len());
                for (i, p) in self.open_projects.iter().enumerate() {
                    if i == active_idx {
                        tab_infos.push((project.display_name().to_string(), project.is_unsaved()));
                    } else {
                        tab_infos.push((p.display_name().to_string(), p.is_unsaved()));
                    }
                }
                let tab_refs: Vec<(&str, bool)> = tab_infos.iter().map(|(n, d)| (n.as_str(), *d)).collect();

                let mut should_go_home = false;
                let mut should_new_board = false;
                let mut should_open_file = false;
                let mut switch_to_tab: Option<usize> = None;
                let mut close_tab_idx: Option<usize> = None;

                let top_strip_fill = if self.dark_mode {
                    Color32::from_rgb(30, 32, 38)
                } else {
                    Color32::from_rgb(242, 244, 247)
                };
                let top_strip_border = if self.dark_mode {
                    Color32::from_rgb(45, 48, 58)
                } else {
                    Color32::from_rgb(226, 229, 234)
                };
                let editor_top_frame = egui::Frame::NONE
                    .fill(top_strip_fill)
                    .stroke(Stroke::new(1.0_f32, top_strip_border));

                let mut editor_tab_action = TopBarAction::None;
                egui::TopBottomPanel::top("top_tab_strip")
                    .frame(editor_top_frame)
                    .exact_height(34.0)
                    .show(ctx, |ui| {
                        editor_tab_action = TopBar::render_window_tab_strip(
                            ui,
                            &tab_refs,
                            Some(active_idx),
                            &mut self.top_bar_state,
                            self.dark_mode,
                        );
                    });

                match editor_tab_action {
                    TopBarAction::GoHome => {
                        should_go_home = true;
                    }
                    TopBarAction::NewProject => {
                        should_new_board = true;
                    }
                    TopBarAction::SwitchProjectTab(idx) => {
                        switch_to_tab = Some(idx);
                    }
                    TopBarAction::CloseProjectTab(idx) => {
                        close_tab_idx = Some(idx);
                    }
                    TopBarAction::CloseProject => {
                        close_tab_idx = Some(active_idx);
                    }
                    TopBarAction::RenameProject(new_name) => {
                        self.rename_project(&mut project, &new_name);
                    }
                    TopBarAction::DeleteCurrentProject => {
                        if let Some(path) = project.filepath.clone() {
                            self.home.pending_delete_path = Some(path);
                            should_go_home = true;
                        }
                    }
                    _ => {
                        handle_top_bar_action(self, &mut project, editor_tab_action);
                    }
                }

                if should_new_board {
                    self.open_projects[active_idx] = project;
                    self.create_and_open_new_project();
                    return;
                }

                if let Some(target) = switch_to_tab {
                    if target != active_idx && target < self.open_projects.len() {
                        if project.is_unsaved() {
                            if let Some(path) = project.filepath.clone() {
                                if let Err(err) = SiroFileManager::save_project(&mut project, &path) {
                                    self.set_status(format!("Auto-save before tab switch failed: {}", err));
                                }
                            }
                        }
                        project.active_board_mut().camera_pan = self.camera.pan;
                        project.active_board_mut().camera_zoom = self.camera.zoom;
                        self.open_projects[active_idx] = project;
                        self.active_project_idx = Some(target);
                        self.camera.pan = self.open_projects[target].active_board().camera_pan;
                        self.camera.zoom = self.open_projects[target].active_board().camera_zoom;
                        self.select_tool.clear_selection();
                        return;
                    }
                }

                if let Some(target) = close_tab_idx {
                    project.active_board_mut().camera_pan = self.camera.pan;
                    project.active_board_mut().camera_zoom = self.camera.zoom;
                    self.open_projects[active_idx] = project;
                    self.request_close_tab(target);
                    return;
                }

                if should_go_home {
                    project.active_board_mut().camera_pan = self.camera.pan;
                    project.active_board_mut().camera_zoom = self.camera.zoom;
                    self.open_projects[active_idx] = project;
                    self.state = AppState::Home;
                    return;
                }

                // Collapsible STEM Planner Drawer on Right
                if self.planner_open {
                    egui::SidePanel::right("planner_panel")
                        .resizable(true)
                        .default_width(320.0)
                        .max_width(450.0)
                        .show(ctx, |ui| {
                            let pl_action = self.planner_panel.ui(ui, &mut project, self.dark_mode);
                            match pl_action {
                                PlannerAction::None => {}
                                PlannerAction::SyncBomToCanvas => {
                                    project.record_undo();
                                    let table_elem = project.planner.to_canvas_table(self.camera.pan + Vec2::new(100.0, 100.0));
                                    project.active_board_mut().add_element(BoardElement::Table(table_elem));
                                    self.set_status("Synced BOM table to canvas board!");
                                }
                                PlannerAction::ExportBomCsv => {
                                    if let Some(path) = rfd::FileDialog::new()
                                        .add_filter("CSV (*.csv)", &["csv"])
                                        .save_file()
                                    {
                                        if let Err(err) = ExportManager::export_bom_to_csv(&project.planner, &path) {
                                            self.set_status(format!("Export CSV error: {}", err));
                                        } else {
                                            self.set_status(format!("Exported BOM to {:?}", path));
                                        }
                                    }
                                }
                            }
                        });
                }

                // Central Infinite Vector Canvas with Floating Miro UI
                egui::CentralPanel::default().show(ctx, |ui| {
                    let viewport_rect = ui.available_rect_before_wrap();
                    let response = ui.allocate_rect(viewport_rect, egui::Sense::click_and_drag());
                    let painter = ui.painter_at(viewport_rect);

                    // 1. Render Background: Miro off-white #f7f8fa in light mode, #16181f in dark mode
                    let bg_color = if self.dark_mode {
                        Color32::from_rgb(22, 24, 31)
                    } else {
                        Color32::from_rgb(247, 248, 250)
                    };
                    painter.rect_filled(viewport_rect, CornerRadius::ZERO, bg_color);

                    // 2. Render Dynamic LOD Grid
                    GridRenderer::render(&painter, &self.camera, viewport_rect, &self.grid_settings, self.dark_mode);

                    let visible_world_rect = self.camera.visible_world_rect(viewport_rect);
                    {
                        let board = project.active_board();
                        for elem in &board.elements {
                            if elem.intersects_rect(visible_world_rect) {
                                let is_selected = self.select_tool.selected_ids.contains(&elem.id());
                                let is_single_selection = self.select_tool.selected_ids.len() == 1;
                                let is_vertex_mode = self.select_tool.vertex_edit_mode;
                                let hovered_term = self.wire_tool.hovered_terminal.map(|(cid, tid, _)| (cid, tid));

                                CanvasRenderer::render_element(
                                    &painter,
                                    &self.camera,
                                    viewport_rect,
                                    elem,
                                    is_selected,
                                    is_single_selection,
                                    is_vertex_mode,
                                    hovered_term,
                                    self.dark_mode,
                                    &self.image_textures,
                                    &self.pdf_textures,
                                    self.table_tool.active_cell,
                                );
                            }
                        }

                        // For multi-element selection, render the unified outer bounding box
                        if self.select_tool.selected_ids.len() > 1 && !self.select_tool.vertex_edit_mode {
                            let mut combined_aabb: Option<Rect> = None;
                            for elem in &board.elements {
                                if self.select_tool.selected_ids.contains(&elem.id()) {
                                    combined_aabb = match combined_aabb {
                                        Some(cur) => Some(cur.union(elem.geometric_aabb())),
                                        None => Some(elem.geometric_aabb()),
                                    };
                                }
                            }
                            if let Some(r) = combined_aabb {
                                CanvasRenderer::render_selection_box(&painter, &self.camera, viewport_rect, r);
                            }
                        }

                        // Render Figma-style smart snap guides
                        CanvasRenderer::render_snap_guides(
                            &painter,
                            &self.camera,
                            viewport_rect,
                            &self.select_tool.active_snap_guides,
                        );
                    }

                    // Inline Text Editor Overlay
                    if let Some(text_id) = self.editing_text_id {
                        if let Some(BoardElement::Text(t)) = project.active_board_mut().get_element_mut(text_id) {
                            let screen_pos = self.camera.world_to_screen(t.pos, viewport_rect);
                            let size = if t.is_sticky {
                                Vec2::new(160.0 * self.camera.zoom, 160.0 * self.camera.zoom)
                            } else {
                                Vec2::new(280.0 * self.camera.zoom, 36.0 * self.camera.zoom)
                            };
                            let edit_rect = Rect::from_min_size(screen_pos, size);
                            let font_size = (t.font_size * self.camera.zoom).clamp(10.0, 32.0);

                            ui.put(
                                edit_rect,
                                egui::TextEdit::multiline(&mut t.text)
                                    .font(egui::FontId::proportional(font_size))
                                    .desired_width(size.x),
                            );
                        }
                    }

                    // Inline Table Cell Editor Overlay
                    if let Some((tbl_id, r, c)) = self.table_tool.active_cell {
                        if let Some(BoardElement::Table(tbl)) = project.active_board_mut().get_element_mut(tbl_id) {
                            if r < tbl.rows && c < tbl.cols {
                                let mut cur_x = tbl.pos.x;
                                for i in 0..c {
                                    cur_x += tbl.col_widths.get(i).copied().unwrap_or(120.0);
                                }
                                let cur_y = tbl.pos.y + r as f32 * tbl.row_height();
                                let col_w = tbl.col_widths.get(c).copied().unwrap_or(120.0);
                                let cell_rect_world = Rect::from_min_size(Pos2::new(cur_x, cur_y), Vec2::new(col_w, tbl.row_height()));
                                let cell_rect_screen = self.camera.world_rect_to_screen(cell_rect_world, viewport_rect);

                                if let Some(cell_text) = tbl.get_cell_mut(r, c) {
                                    let font_sz = (13.0 * self.camera.zoom).clamp(9.0, 22.0);
                                    let resp = ui.put(
                                        cell_rect_screen,
                                        egui::TextEdit::singleline(cell_text)
                                            .font(egui::FontId::proportional(font_sz))
                                            .desired_width(cell_rect_screen.width()),
                                    );
                                    if let Some(state) = egui::TextEdit::load_state(ui.ctx(), resp.id) {
                                        if let Some(range) = state.cursor.char_range() {
                                            if range.primary.index != range.secondary.index {
                                                let start = range.primary.index.min(range.secondary.index);
                                                let end = range.primary.index.max(range.secondary.index);
                                                self.table_tool.selected_text_range = Some((tbl_id, r, c, start, end));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    // Render Active Tool Previews (pen, shape, wire, lasso)
                    render_tool_previews(self, &painter, viewport_rect);

                    // ==========================================
                    // FLOATING OVERLAYS (Miro UI Structure)
                    // ==========================================
                    let mut occupied_ui_rects = Vec::new();

                    // 1. Floating Canvas Top Navigation Header
                    let (tb_float_action, header_rect) = TopBar::render_floating_header(
                        ui,
                        &mut project,
                        &mut self.top_bar_state,
                        self.planner_open,
                        self.dark_mode,
                        viewport_rect.min,
                    );
                    occupied_ui_rects.push(header_rect);
                    match tb_float_action {
                        TopBarAction::None => {}
                        TopBarAction::GoHome => {
                            should_go_home = true;
                        }
                        TopBarAction::NewProject => {
                            should_new_board = true;
                        }
                        TopBarAction::OpenProject => {
                            should_open_file = true;
                        }
                        TopBarAction::DeleteCurrentProject => {
                            if let Some(path) = project.filepath.clone() {
                                self.home.pending_delete_path = Some(path);
                                should_go_home = true;
                            }
                        }
                        other => {
                            handle_top_bar_action(self, &mut project, other);
                        }
                    }

                    // 2. Floating Left Tool Dock & Separate Undo/Redo Pill & Flyouts
                    let cur_tool = self.current_tool.clone();
                    let (dock_action, dock_rects) = self.tool_rail.render_floating(
                        ui,
                        cur_tool.clone(),
                        self.active_sticky_color,
                        self.active_stroke_color,
                        self.active_stroke_width,
                        project.can_undo(),
                        project.can_redo(),
                        self.dark_mode,
                        viewport_rect,
                    );
                    occupied_ui_rects.extend(dock_rects);
                    handle_tool_rail_action(self, &mut project, dock_action);

                    // 3. Floating Contextual Toolbar directly above selected element
                    let sel_ids: Vec<Uuid> = self.select_tool.selected_ids.iter().cloned().collect();
                    let (props_action, ctx_tb_rect) = PropertiesBar::render_floating(
                        ui,
                        &self.camera,
                        viewport_rect,
                        cur_tool,
                        &sel_ids,
                        project.active_board_mut(),
                        &mut self.active_stroke_color,
                        &mut self.active_stroke_width,
                        &mut self.active_wire_color,
                        &mut self.active_sticky_color,
                        self.dark_mode,
                        &mut self.popup_manager,
                    );
                    if let Some(r) = ctx_tb_rect {
                        occupied_ui_rects.push(r);
                    }
                    handle_properties_action(self, &mut project, props_action);

                    // 4. Floating Bottom-Right Zoom Pill
                    let (zoom_action, zoom_rect) = render_bottom_right_zoom_pill(
                        ui,
                        &mut self.camera,
                        &project,
                        self.dark_mode,
                        self.settings.smart_snapping_enabled,
                        viewport_rect,
                    );
                    occupied_ui_rects.push(zoom_rect);
                    if let Some(act) = zoom_action {
                        handle_top_bar_action(self, &mut project, act);
                    }

                    // 5. Floating Bottom-Left Sheet Switcher Bar
                    let (sheet_action, sheet_rect) = TopBar::render_sheet_bar(
                        ui,
                        &mut project,
                        &mut self.top_bar_state,
                        self.dark_mode,
                        viewport_rect,
                    );
                    if let Some(r) = sheet_rect {
                        occupied_ui_rects.push(r);
                    }
                    handle_top_bar_action(self, &mut project, sheet_action);

                    // Shield all open popups from canvas pointer interactions
                    for p in &self.popup_manager.stack {
                        if p.rect != Rect::ZERO {
                            occupied_ui_rects.push(p.rect);
                        }
                    }

                    // 6. Notion-style table row controls
                    enum TableRowAction {
                        MoveUp(Uuid, usize),
                        MoveDown(Uuid, usize),
                        ChangeColor(Uuid, usize, Color32),
                        DeleteRow(Uuid, usize),
                        ReorderRow(Uuid, usize, usize),
                    }
                    let mut pending_table_action = None;

                    let pointer_screen_pos = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(Pos2::ZERO);
                    let pointer_world_pos = self.camera.screen_to_world(pointer_screen_pos, viewport_rect);
                    for elem in &project.active_board().elements {
                        if let BoardElement::Table(tbl) = elem {
                            let row_h = tbl.row_height();
                            let table_world_aabb = tbl.aabb();
                            let expand_x = 40.0 / self.camera.zoom.max(0.01);
                            let trigger_rect = Rect::from_min_max(
                                Pos2::new(table_world_aabb.min.x - expand_x, table_world_aabb.min.y),
                                Pos2::new(table_world_aabb.max.x, table_world_aabb.max.y),
                            );

                            let open_popup_row = (0..tbl.rows).find(|&r_idx| {
                                let r_id = format!("notion_row_menu_{}_{}", tbl.id, r_idx);
                                let color_id = format!("notion_row_{}_{}_color_picker", tbl.id, r_idx);
                                self.popup_manager.is_open(&r_id) || self.popup_manager.is_open(&color_id)
                            });

                            let active_row_opt = if let Some((drag_tbl_id, drag_r, _)) = self.table_tool.dragging_row {
                                if drag_tbl_id == tbl.id && drag_r < tbl.rows {
                                    Some(drag_r)
                                } else {
                                    None
                                }
                            } else if let Some(r_idx) = open_popup_row {
                                Some(r_idx)
                            } else {
                                None
                            };

                            let row_to_render = active_row_opt.or_else(|| {
                                if trigger_rect.contains(pointer_world_pos) || self.select_tool.selected_ids.contains(&tbl.id) {
                                    let hovered_row = ((pointer_world_pos.y - tbl.pos.y) / row_h).floor() as isize;
                                    if hovered_row >= 0 && (hovered_row as usize) < tbl.rows {
                                        Some(hovered_row as usize)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                }
                            });

                            if let Some(r) = row_to_render {
                                let s_tbl_pos = self.camera.world_to_screen(tbl.pos, viewport_rect);
                                let row_screen_y = s_tbl_pos.y + r as f32 * (row_h * self.camera.zoom);
                                let handle_size = Vec2::new(20.0, 20.0);
                                let handle_pos = Pos2::new(s_tbl_pos.x - 24.0, row_screen_y + (row_h * self.camera.zoom - handle_size.y) / 2.0);
                                let handle_rect = Rect::from_min_size(handle_pos, handle_size);
                                occupied_ui_rects.push(handle_rect);

                                let handle_id = ui.make_persistent_id(format!("notion_row_handle_{}_{}", tbl.id, r));
                                let handle_resp = ui.interact(handle_rect, handle_id, egui::Sense::click_and_drag());

                                let is_dragging = match self.table_tool.dragging_row {
                                    Some((d_id, d_r, _)) => d_id == tbl.id && d_r == r,
                                    None => false,
                                };
                                let is_hovered = handle_resp.hovered();
                                let bg_color = if is_dragging {
                                    if self.dark_mode { Color32::from_rgb(40, 70, 110) } else { Color32::from_rgb(205, 230, 255) }
                                } else if is_hovered {
                                    if self.dark_mode { Color32::from_rgb(45, 52, 65) } else { Color32::from_rgb(225, 230, 240) }
                                } else {
                                    if self.dark_mode { Color32::from_rgb(32, 36, 46) } else { Color32::from_rgb(240, 242, 248) }
                                };
                                let border_color = if is_dragging {
                                    Color32::from_rgb(0, 180, 255)
                                } else if is_hovered {
                                    if self.dark_mode { Color32::from_rgb(70, 78, 98) } else { Color32::from_rgb(195, 202, 215) }
                                } else {
                                    if self.dark_mode { Color32::from_rgb(50, 56, 70) } else { Color32::from_rgb(215, 220, 230) }
                                };
                                ui.painter().rect_filled(handle_rect, CornerRadius::same(4), bg_color);
                                ui.painter().rect_stroke(handle_rect, CornerRadius::same(4), Stroke::new(1.0_f32, border_color), StrokeKind::Middle);
                                let glyph_color = if is_dragging {
                                    Color32::from_rgb(0, 180, 255)
                                } else if self.dark_mode {
                                    Color32::LIGHT_GRAY
                                } else {
                                    Color32::DARK_GRAY
                                };
                                ui.painter().text(
                                    handle_rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "⠿",
                                    egui::FontId::proportional(12.0),
                                    glyph_color,
                                );

                                let row_menu_id = format!("notion_row_menu_{}_{}", tbl.id, r);
                                let table_screen_rect = Rect::from_min_size(
                                    s_tbl_pos,
                                    Vec2::new(tbl.total_width() * self.camera.zoom, tbl.rows as f32 * row_h * self.camera.zoom),
                                );

                                // Notion-style Drag and Click behavior:
                                // 1. Dragging: shows drop indicator line between rows; reorders on release
                                if handle_resp.drag_started() {
                                    self.table_tool.dragging_row = Some((tbl.id, r, pointer_world_pos.y));
                                    self.popup_manager.close(&row_menu_id);
                                }
                                if let Some((drag_tbl_id, from_r, _)) = self.table_tool.dragging_row {
                                    if drag_tbl_id == tbl.id {
                                        ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
                                        let rel_y = pointer_world_pos.y - tbl.pos.y;
                                        let insert_slot = (rel_y / row_h).round().clamp(0.0, tbl.rows as f32) as usize;

                                        // Visible drop-position indicator line
                                        let drop_y = s_tbl_pos.y + (insert_slot as f32) * (row_h * self.camera.zoom);
                                        let s_total_w = tbl.total_width() * self.camera.zoom;
                                        let line_color = Color32::from_rgb(0, 180, 255);
                                        ui.painter().line_segment(
                                            [Pos2::new(s_tbl_pos.x - 14.0, drop_y), Pos2::new(s_tbl_pos.x + s_total_w, drop_y)],
                                            Stroke::new(2.5_f32, line_color),
                                        );
                                        ui.painter().circle_filled(Pos2::new(s_tbl_pos.x - 14.0, drop_y), 4.5, line_color);

                                        if handle_resp.drag_stopped() || ctx.input(|i| i.pointer.any_released()) {
                                            let to_r = if insert_slot > from_r { insert_slot - 1 } else { insert_slot };
                                            if to_r != from_r && to_r < tbl.rows {
                                                pending_table_action = Some(TableRowAction::ReorderRow(tbl.id, from_r, to_r));
                                            }
                                            self.table_tool.dragging_row = None;
                                        }
                                    }
                                } else if is_hovered {
                                    ui.ctx().set_cursor_icon(egui::CursorIcon::Grab);
                                }

                                // 2. Click: opens the row options menu using shared PopupManager
                                if handle_resp.clicked() && self.table_tool.dragging_row.is_none() {
                                    let was_open = self.popup_manager.is_open(&row_menu_id);
                                    for other_r in 0..tbl.rows {
                                        self.popup_manager.close(&format!("notion_row_menu_{}_{}", tbl.id, other_r));
                                        self.popup_manager.close(&format!("notion_row_{}_{}_color_picker", tbl.id, other_r));
                                    }
                                    if !was_open {
                                        self.popup_manager.open(
                                            &row_menu_id,
                                            &format!("Row {}", r + 1),
                                            None,
                                            Some(handle_rect),
                                            Some(table_screen_rect),
                                        );
                                    }
                                }

                                let mut need_toggle_row_color = None;
                                let row_color_picker_id = format!("notion_row_{}_{}_color_picker", tbl.id, r);

                                if self.popup_manager.is_open(&row_menu_id) {
                                    let mut close_menu = false;
                                    self.popup_manager.show(
                                        ui.ctx(),
                                        self.dark_mode,
                                        &row_menu_id,
                                        &format!("Row {}", r + 1),
                                        None,
                                        Some(handle_rect),
                                        Some(table_screen_rect),
                                        |p_ui| {
                                            p_ui.set_min_width(170.0);

                                            // Move up (disabled on first row)
                                            let can_move_up = r > 0;
                                            if p_ui.add_enabled(can_move_up, egui::Button::new("⬆  Move up")).clicked() {
                                                pending_table_action = Some(TableRowAction::MoveUp(tbl.id, r));
                                                close_menu = true;
                                            }

                                            // Move down (disabled on last row)
                                            let can_move_down = r + 1 < tbl.rows;
                                            if p_ui.add_enabled(can_move_down, egui::Button::new("⬇  Move down")).clicked() {
                                                pending_table_action = Some(TableRowAction::MoveDown(tbl.id, r));
                                                close_menu = true;
                                            }

                                            p_ui.separator();
                                            p_ui.label(RichText::new("Row Text Color:").size(11.0).color(Color32::from_rgb(130, 135, 150)));
                                            let cur_c = tbl.get_row_color(r).unwrap_or(if self.dark_mode { Color32::WHITE } else { Color32::BLACK });
                                            let (chosen_preset, rb_clicked, rb_rect) = crate::ui::color_presets::render_color_presets_swatches(
                                                p_ui,
                                                cur_c,
                                            );
                                            if let Some(chosen_c) = chosen_preset {
                                                pending_table_action = Some(TableRowAction::ChangeColor(tbl.id, r, chosen_c));
                                            }
                                            if rb_clicked {
                                                need_toggle_row_color = Some(rb_rect);
                                            }

                                            p_ui.separator();
                                            // Delete row (safely handled: blocked if only 1 row remains)
                                            let can_delete = tbl.rows > 1;
                                            let del_btn = p_ui.add_enabled(
                                                can_delete,
                                                egui::Button::new(RichText::new("🗑  Delete row").color(Color32::from_rgb(235, 60, 60))),
                                            );
                                            if del_btn.clicked() {
                                                pending_table_action = Some(TableRowAction::DeleteRow(tbl.id, r));
                                                close_menu = true;
                                            }
                                        },
                                    );

                                    if close_menu {
                                        self.popup_manager.close(&row_menu_id);
                                    }
                                }

                                if let Some(rb_rect) = need_toggle_row_color {
                                    self.popup_manager.toggle(
                                        &row_color_picker_id,
                                        "Custom Color",
                                        Some(&row_menu_id),
                                        Some(rb_rect),
                                        None,
                                    );
                                }

                                if self.popup_manager.is_open(&row_color_picker_id) {
                                    let cur_c = tbl.get_row_color(r).unwrap_or(if self.dark_mode { Color32::WHITE } else { Color32::BLACK });
                                    if let Some(chosen_c) = crate::ui::color_presets::render_custom_color_picker_popup(
                                        &mut self.popup_manager,
                                        ui.ctx(),
                                        self.dark_mode,
                                        &row_color_picker_id,
                                        Some(&row_menu_id),
                                        handle_rect,
                                        cur_c,
                                    ) {
                                        pending_table_action = Some(TableRowAction::ChangeColor(tbl.id, r, chosen_c));
                                    }
                                }
                            }
                        }
                    }

                    if let Some(act) = pending_table_action {
                        project.record_undo();
                        match act {
                            TableRowAction::MoveUp(id, r) => {
                                if let Some(BoardElement::Table(target_tbl)) = project.active_board_mut().get_element_mut(id) {
                                    TableTool::move_row_up(target_tbl, r);
                                }
                            }
                            TableRowAction::MoveDown(id, r) => {
                                if let Some(BoardElement::Table(target_tbl)) = project.active_board_mut().get_element_mut(id) {
                                    TableTool::move_row_down(target_tbl, r);
                                }
                            }
                            TableRowAction::ReorderRow(id, from, to) => {
                                if let Some(BoardElement::Table(target_tbl)) = project.active_board_mut().get_element_mut(id) {
                                    TableTool::reorder_row(target_tbl, from, to);
                                }
                            }
                            TableRowAction::ChangeColor(id, r, col) => {
                                if let Some(BoardElement::Table(target_tbl)) = project.active_board_mut().get_element_mut(id) {
                                    target_tbl.set_row_color(r, Some(col));
                                }
                            }
                            TableRowAction::DeleteRow(id, r) => {
                                if let Some(BoardElement::Table(target_tbl)) = project.active_board_mut().get_element_mut(id) {
                                    TableTool::remove_row(target_tbl, r);
                                }
                            }
                        }
                    }

                    // Check pointer interaction safety
                    let pointer_screen = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(viewport_rect.center());
                    let is_pointer_over_ui = occupied_ui_rects.iter().any(|r| r.contains(pointer_screen));

                    // Dismiss open tool rail flyouts when clicking empty canvas or pressing Escape
                    if ctx.input(|i| i.key_pressed(egui::Key::Escape))
                        || (response.clicked() && !is_pointer_over_ui)
                    {
                        self.tool_rail.close_all_flyouts();
                    }

                    // 3. Handle Canvas Input
                    handle_canvas_input(self, &mut project, ctx, &response, viewport_rect, is_pointer_over_ui);

                    // Status notification toast
                    if let Some((msg, time)) = &self.status_message {
                        if time.elapsed().as_secs() < 4 {
                            let toast_rect = Rect::from_center_size(
                                Pos2::new(viewport_rect.center().x, viewport_rect.max.y - 25.0),
                                Vec2::new(320.0, 32.0),
                            );
                            let toast_bg = if self.dark_mode {
                                Color32::from_rgba_unmultiplied(10, 15, 25, 235)
                            } else {
                                Color32::from_rgba_unmultiplied(255, 255, 255, 245)
                            };
                            let toast_border = if self.dark_mode {
                                Color32::from_rgb(0, 210, 255)
                            } else {
                                Color32::from_rgb(45, 96, 255)
                            };
                            let toast_text_color = if self.dark_mode {
                                Color32::WHITE
                            } else {
                                Color32::from_rgb(20, 24, 32)
                            };
                            painter.rect_filled(toast_rect, CornerRadius::same(6), toast_bg);
                            painter.rect_stroke(toast_rect, CornerRadius::same(6), Stroke::new(1.0_f32, toast_border), StrokeKind::Middle);
                            painter.text(
                                toast_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                msg,
                                egui::FontId::proportional(12.0),
                                toast_text_color,
                            );
                        }
                    }
                });

                project.active_board_mut().camera_pan = self.camera.pan;
                project.active_board_mut().camera_zoom = self.camera.zoom;
                self.open_projects[active_idx] = project;
                if should_new_board {
                    self.create_and_open_new_project();
                    return;
                }
                if should_open_file {
                    self.dispatch_action(TopBarAction::OpenProject);
                    return;
                }
                if should_go_home {
                    self.state = AppState::Home;
                }
            }
        }

        // -------------------------------------------------------------
        // Background Autosave Sweep (every open project, active & inactive)
        // -------------------------------------------------------------
        let is_modal_open = self.close_dialog.is_some()
            || self.home.pending_delete_path.is_some()
            || self.top_bar_state.sheet_modal.is_some();

        if let Some(res) = self.autosave.poll_autosave_all(
            &mut self.open_projects,
            self.settings.autosave_interval_secs,
            is_modal_open,
        ) {
            match res {
                Ok(_count) => {}
                Err(err) => {
                    self.set_status(err);
                }
            }
        }

        // -------------------------------------------------------------
        // Close Guard Modal Dialog (floats over both Home and Editor)
        // -------------------------------------------------------------
        if let Some(dialog_state) = self.close_dialog.clone() {
            let target_name = match dialog_state.target {
                crate::ui::close_dialog::CloseDialogTarget::App => None,
                crate::ui::close_dialog::CloseDialogTarget::Tab(idx) => {
                    self.open_projects.get(idx).map(|p| p.display_name())
                }
            };

            let mut dialog_action = crate::ui::close_dialog::CloseDialogAction::None;
            egui::Area::new(egui::Id::new("siro_close_dialog_area"))
                .order(egui::Order::Foreground)
                .interactable(true)
                .show(ctx, |ui| {
                    dialog_action = crate::ui::close_dialog::render_close_dialog(
                        ui,
                        &dialog_state,
                        target_name,
                        self.dark_mode,
                    );
                });

            match dialog_action {
                crate::ui::close_dialog::CloseDialogAction::None => {}
                crate::ui::close_dialog::CloseDialogAction::Cancel => {
                    self.close_dialog = None;
                }
                crate::ui::close_dialog::CloseDialogAction::DiscardChanges => {
                    match dialog_state.target {
                        crate::ui::close_dialog::CloseDialogTarget::App => {
                            self.close_dialog = None;
                            self.allow_close = true;
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        crate::ui::close_dialog::CloseDialogTarget::Tab(idx) => {
                            self.close_dialog = None;
                            self.force_close_tab(idx);
                        }
                    }
                }
                crate::ui::close_dialog::CloseDialogAction::SaveThenExitOrClose => {
                    match dialog_state.target {
                        crate::ui::close_dialog::CloseDialogTarget::App => {
                            if self.save_all_unsaved_projects() {
                                self.close_dialog = None;
                                self.allow_close = true;
                                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        }
                        crate::ui::close_dialog::CloseDialogTarget::Tab(idx) => {
                            if self.save_single_project(idx) {
                                self.close_dialog = None;
                                self.force_close_tab(idx);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn render_bottom_right_zoom_pill(
    ui: &mut egui::Ui,
    camera: &mut Camera,
    _project: &Project,
    dark_mode: bool,
    smart_snapping_enabled: bool,
    viewport_rect: Rect,
) -> (Option<TopBarAction>, Rect) {
    let zoom_w = 206.0;
    let zoom_h = 36.0;
    let zoom_pos = Pos2::new(
        viewport_rect.max.x - zoom_w - 18.0,
        viewport_rect.max.y - zoom_h - 18.0,
    );
    let zoom_rect = Rect::from_min_size(zoom_pos, Vec2::new(zoom_w, zoom_h));

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

    let p = ui.painter();
    p.rect_filled(
        zoom_rect.translate(Vec2::new(1.0, 3.0)),
        CornerRadius::same(18),
        Color32::from_rgba_unmultiplied(0, 0, 0, 20),
    );
    p.rect_filled(zoom_rect, CornerRadius::same(18), bg_color);
    p.rect_stroke(
        zoom_rect,
        CornerRadius::same(18),
        Stroke::new(1.0_f32, border_color),
        StrokeKind::Middle,
    );

    let mut action = None;
    let mut child = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(zoom_rect.shrink2(Vec2::new(8.0, 4.0)))
            .layout(egui::Layout::left_to_right(egui::Align::Center)),
    );

    let btn_color = if dark_mode {
        Color32::from_rgb(215, 220, 235)
    } else {
        Color32::from_rgb(50, 55, 70)
    };

    let snap_btn_color = if smart_snapping_enabled {
        Color32::from_rgb(45, 96, 255)
    } else {
        btn_color
    };
    if child
        .add(egui::Button::new(egui::RichText::new("🧲").size(12.0).color(snap_btn_color)).frame(false))
        .on_hover_text(if smart_snapping_enabled {
            "Smart Snapping: ON (Click to disable, or hold Alt while dragging)"
        } else {
            "Smart Snapping: OFF (Click to enable)"
        })
        .clicked()
    {
        action = Some(TopBarAction::ToggleSmartSnapping);
    }

    if child
        .add(egui::Button::new(egui::RichText::new("⛶").size(12.0).color(btn_color)).frame(false))
        .on_hover_text("Fit to Content")
        .clicked()
    {
        action = Some(TopBarAction::ZoomFit);
    }

    if child
        .add(egui::Button::new(egui::RichText::new("−").size(14.0).color(btn_color)).frame(false))
        .on_hover_text("Zoom Out")
        .clicked()
    {
        action = Some(TopBarAction::ZoomOut);
    }

    let pct_text = format!("{}%", (camera.zoom * 100.0).round() as i32);
    if child
        .add(egui::Button::new(egui::RichText::new(pct_text).size(11.5).strong().color(btn_color)).frame(false))
        .on_hover_text("Reset Zoom (100%)")
        .clicked()
    {
        action = Some(TopBarAction::ZoomReset);
    }

    if child
        .add(egui::Button::new(egui::RichText::new("+").size(14.0).color(btn_color)).frame(false))
        .on_hover_text("Zoom In")
        .clicked()
    {
        action = Some(TopBarAction::ZoomIn);
    }

    (action, zoom_rect)
}

fn handle_tool_rail_action(app: &mut SiroApp, project: &mut Project, action: ToolRailAction) {
    match action {
        ToolRailAction::None => {}
        ToolRailAction::SetTool(t) => {
            app.select_tool.vertex_edit_mode = t == ActiveTool::VertexEdit;
            app.current_tool = t;
        }
        ToolRailAction::SetStickyColor(c) => {
            app.active_sticky_color = c;
            app.current_tool = ActiveTool::StickyNote;
        }
        ToolRailAction::SetStrokeColor(c) => {
            app.active_stroke_color = c;
        }
        ToolRailAction::SetStrokeWidth(w) => {
            app.active_stroke_width = w;
        }
        ToolRailAction::InsertMedia => {
            handle_insert_media(app, project);
        }
        ToolRailAction::TogglePlanner => {
            app.planner_open = !app.planner_open;
        }
        ToolRailAction::Undo => {
            project.undo();
        }
        ToolRailAction::Redo => {
            project.redo();
        }
    }
}

fn handle_canvas_input(
    app: &mut SiroApp,
    project: &mut Project,
    ctx: &egui::Context,
    response: &egui::Response,
    viewport_rect: Rect,
    is_pointer_over_ui: bool,
) {
    let pointer_screen = ctx.input(|i| i.pointer.hover_pos()).unwrap_or(viewport_rect.center());
    let pointer_world = app.camera.screen_to_world(pointer_screen, viewport_rect);
    let space_down = ctx.input(|i| i.key_down(egui::Key::Space));
    let middle_down = ctx.input(|i| i.pointer.middle_down());

    // Zoom centered strictly on pointer world anchor
    let scroll_delta = ctx.input(|i| i.smooth_scroll_delta.y);
    if scroll_delta != 0.0 && !is_pointer_over_ui {
        let factor = (1.0 + scroll_delta * 0.002).clamp(0.8, 1.25);
        app.camera.zoom_at_pointer(pointer_screen, factor, viewport_rect);
        ctx.request_repaint();
    }

    // Panning canvas with Middle Mouse or Space + Drag
    if middle_down || (space_down && response.dragged()) {
        app.camera.pan_by(response.drag_delta());
        ctx.request_repaint();
        return;
    }

    // If pointer is over floating UI, suppress new clicks/drags from reaching canvas
    if is_pointer_over_ui && (response.clicked() || response.double_clicked() || response.drag_started()) {
        return;
    }

    let is_ctrl_shift = ctx.input(|i| i.modifiers.command || i.modifiers.shift);

    // Active Tool Logic
    match app.current_tool {
        ActiveTool::Select | ActiveTool::VertexEdit => {
            if !is_pointer_over_ui {
                let resize_handle = match app.select_tool.drag_mode {
                    crate::canvas::tools::select::DragMode::Resize(h) => Some(h),
                    _ => app.select_tool.hit_corner_handle(pointer_world, project.active_board(), app.camera.zoom),
                };

                if let Some(handle) = resize_handle {
                    match handle {
                        crate::canvas::tools::select::ResizeHandle::TopLeft
                        | crate::canvas::tools::select::ResizeHandle::BottomRight => {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeNwSe);
                        }
                        crate::canvas::tools::select::ResizeHandle::TopRight
                        | crate::canvas::tools::select::ResizeHandle::BottomLeft => {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeNeSw);
                        }
                        crate::canvas::tools::select::ResizeHandle::TopCenter
                        | crate::canvas::tools::select::ResizeHandle::BottomCenter => {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeVertical);
                        }
                        crate::canvas::tools::select::ResizeHandle::LeftCenter
                        | crate::canvas::tools::select::ResizeHandle::RightCenter => {
                            ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                        }
                    }
                }
            }

            if response.double_clicked() {
                let mut handled_double_click = false;
                for elem in project.active_board().elements.iter().rev() {
                    if let BoardElement::DocAttachment(doc) = elem {
                        if doc.aabb().expand(4.0).contains(pointer_world) {
                            launch_doc_attachment(project, doc.id);
                            app.set_status(format!("Launching attachment {}", doc.filename));
                            handled_double_click = true;
                            break;
                        }
                    } else if let BoardElement::Text(text_elem) = elem {
                        if text_elem.aabb().expand(4.0).contains(pointer_world) {
                            app.editing_text_id = Some(text_elem.id);
                            app.select_tool.selected_ids.clear();
                            app.select_tool.selected_ids.insert(text_elem.id);
                            handled_double_click = true;
                            break;
                        }
                    }
                }
                if !handled_double_click {
                    app.editing_text_id = None;
                }
            } else if response.clicked() {
                let mut pdf_page_change = None;
                for elem in project.active_board().elements.iter().rev() {
                    if let BoardElement::PdfCard(pdf) = elem {
                        let s_min = app.camera.world_to_screen(pdf.pos, viewport_rect);
                        let s_max = app.camera.world_to_screen(pdf.pos + pdf.size, viewport_rect);
                        let screen_rect = Rect::from_min_max(s_min, s_max);
                        let btn_prev = Rect::from_min_size(Pos2::new(screen_rect.max.x - 52.0, screen_rect.min.y + 4.0), Vec2::new(22.0, 22.0));
                        let btn_next = Rect::from_min_size(Pos2::new(screen_rect.max.x - 26.0, screen_rect.min.y + 4.0), Vec2::new(22.0, 22.0));
                        if btn_prev.contains(pointer_screen) {
                            pdf_page_change = Some((pdf.id, -1_i32));
                            break;
                        } else if btn_next.contains(pointer_screen) {
                            pdf_page_change = Some((pdf.id, 1_i32));
                            break;
                        }
                    }
                }

                if let Some((pdf_id, delta)) = pdf_page_change {
                    if let Some(BoardElement::PdfCard(p)) = project.active_board_mut().get_element_mut(pdf_id) {
                        if delta < 0 {
                            if p.current_page > 0 {
                                p.current_page -= 1;
                            }
                        } else {
                            p.current_page += 1;
                        }
                    }
                } else {
                    let is_over_handle = app.select_tool.hit_corner_handle(pointer_world, project.active_board(), app.camera.zoom).is_some();
                    let mut handled_table = false;
                    if !is_over_handle {
                        for elem in project.active_board().elements.iter().rev() {
                            if let BoardElement::Table(tbl) = elem {
                                if tbl.aabb().expand(4.0).contains(pointer_world) {
                                    let local_x = pointer_world.x - tbl.pos.x;
                                    let local_y = pointer_world.y - tbl.pos.y;
                                    let row = (local_y / tbl.row_height()).floor() as usize;
                                    let mut acc_x = 0.0;
                                    let mut found_col = None;
                                    for (c_idx, &w) in tbl.col_widths.iter().enumerate() {
                                        if local_x >= acc_x && local_x < acc_x + w {
                                            found_col = Some(c_idx);
                                            break;
                                        }
                                        acc_x += w;
                                    }
                                    if let Some(col) = found_col {
                                        if row < tbl.rows && col < tbl.cols {
                                            app.table_tool.active_cell = Some((tbl.id, row, col));
                                            app.select_tool.selected_ids.clear();
                                            app.select_tool.selected_ids.insert(tbl.id);
                                            handled_table = true;
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if !handled_table {
                        app.table_tool.active_cell = None;
                        app.editing_text_id = None;
                        app.select_tool.on_pointer_down_with_zoom(pointer_world, project.active_board(), is_ctrl_shift, app.camera.zoom);
                        app.select_tool.on_pointer_up(project.active_board());
                    }
                }
                ctx.request_repaint();
            } else if response.drag_started() {
                project.record_undo();
                let start_pt = ctx.input(|i| i.pointer.press_origin()).unwrap_or(pointer_screen);
                let start_world = app.camera.screen_to_world(start_pt, viewport_rect);
                let alt_held = ctx.input(|i| i.modifiers.alt);
                let shift_held = ctx.input(|i| i.modifiers.shift);
                app.select_tool.snap_enabled = app.settings.smart_snapping_enabled && !alt_held;
                app.select_tool.is_shift_down = shift_held;
                app.select_tool.grid_spacing = if app.grid_settings.style != crate::canvas::grid::GridStyle::None {
                    Some(crate::canvas::grid::GridRenderer::dynamic_spacing(app.camera.zoom, app.grid_settings.base_spacing))
                } else {
                    None
                };
                app.select_tool.viewport_world = Some(app.camera.visible_world_rect(viewport_rect));
                app.select_tool.on_pointer_down_with_zoom(start_world, project.active_board(), is_ctrl_shift, app.camera.zoom);
                ctx.request_repaint();
            } else if response.dragged() {
                let alt_held = ctx.input(|i| i.modifiers.alt);
                let shift_held = ctx.input(|i| i.modifiers.shift);
                app.select_tool.snap_enabled = app.settings.smart_snapping_enabled && !alt_held;
                app.select_tool.is_shift_down = shift_held;
                app.select_tool.grid_spacing = if app.grid_settings.style != crate::canvas::grid::GridStyle::None {
                    Some(crate::canvas::grid::GridRenderer::dynamic_spacing(app.camera.zoom, app.grid_settings.base_spacing))
                } else {
                    None
                };
                app.select_tool.viewport_world = Some(app.camera.visible_world_rect(viewport_rect));
                app.select_tool.on_pointer_move(pointer_world, project.active_board_mut());
                ctx.request_repaint();
            } else if response.drag_stopped() {
                app.select_tool.on_pointer_up(project.active_board());
                ctx.request_repaint();
            }
        }
        ActiveTool::Pan => {
            ctx.set_cursor_icon(if response.dragged() {
                egui::CursorIcon::Grabbing
            } else {
                egui::CursorIcon::Grab
            });
            if response.dragged() {
                app.camera.pan_by(response.drag_delta());
                ctx.request_repaint();
            }
        }
        ActiveTool::Pen | ActiveTool::Highlighter => {
            let is_hl = app.current_tool == ActiveTool::Highlighter;
            app.pen_tool.is_highlighter = is_hl;
            app.pen_tool.color = app.active_stroke_color;
            app.pen_tool.width = app.active_stroke_width;

            if response.clicked() {
                project.record_undo();
                let time_secs = ctx.input(|i| i.time);
                app.pen_tool.on_pointer_move(pointer_world, time_secs);
                app.pen_tool.on_pointer_move(pointer_world + Vec2::new(0.5, 0.5), time_secs + 0.01);
                if let Some(stroke) = app.pen_tool.finish_stroke() {
                    project.active_board_mut().add_element(BoardElement::Stroke(stroke));
                }
                ctx.request_repaint();
            } else if response.drag_started() {
                let time_secs = ctx.input(|i| i.time);
                app.pen_tool.on_pointer_move(pointer_world, time_secs);
                ctx.request_repaint();
            } else if response.dragged() {
                let time_secs = ctx.input(|i| i.time);
                app.pen_tool.on_pointer_move(pointer_world, time_secs);
                ctx.request_repaint();
            } else if response.drag_stopped() {
                if let Some(stroke) = app.pen_tool.finish_stroke() {
                    project.record_undo();
                    project.active_board_mut().add_element(BoardElement::Stroke(stroke));
                }
                ctx.request_repaint();
            }
        }
        ActiveTool::Eraser => {
            if response.clicked() {
                let has_elem = project.active_board().elements.iter().any(|e| e.contains_point(pointer_world, app.eraser_tool.radius));
                if has_elem {
                    project.record_undo();
                    app.eraser_tool.erase_at(pointer_world, project.active_board_mut());
                    ctx.request_repaint();
                }
            } else if response.drag_started() {
                let has_elem = project.active_board().elements.iter().any(|e| e.contains_point(pointer_world, app.eraser_tool.radius));
                if has_elem {
                    project.record_undo();
                    app.eraser_tool.undo_recorded = true;
                } else {
                    app.eraser_tool.undo_recorded = false;
                }
                app.eraser_tool.on_pointer_down(pointer_world, project.active_board_mut());
                ctx.request_repaint();
            } else if response.dragged() {
                let will_erase = project.active_board().elements.iter().any(|e| e.contains_point(pointer_world, app.eraser_tool.radius));
                if will_erase && !app.eraser_tool.undo_recorded {
                    project.record_undo();
                    app.eraser_tool.undo_recorded = true;
                }
                let removed = app.eraser_tool.on_pointer_move(pointer_world, project.active_board_mut());
                if !removed.is_empty() {
                    ctx.request_repaint();
                }
            } else if response.drag_stopped() {
                app.eraser_tool.on_pointer_up();
            }
        }
        ActiveTool::Shape(kind) => {
            app.shape_tool.kind = kind;
            app.shape_tool.stroke_color = app.active_stroke_color;
            app.shape_tool.stroke_width = app.active_stroke_width;
            let [r, g, b, _] = app.active_stroke_color.to_array();
            app.shape_tool.fill_color = Color32::from_rgba_unmultiplied(r, g, b, 30);

            if response.drag_started() {
                app.shape_tool.on_pointer_down(pointer_world);
                ctx.request_repaint();
            } else if response.dragged() {
                app.shape_tool.on_pointer_move(pointer_world);
                ctx.request_repaint();
            } else if response.drag_stopped() {
                if let Some(shape) = app.shape_tool.finish_shape() {
                    project.record_undo();
                    project.active_board_mut().add_element(BoardElement::Shape(shape));
                }
                ctx.request_repaint();
            }
        }
        ActiveTool::Wire => {
            app.wire_tool.color = app.active_wire_color;
            app.wire_tool.width = app.active_stroke_width;
            app.wire_tool.update_hover(pointer_world, project.active_board());

            if response.drag_started() {
                app.wire_tool.on_pointer_down(pointer_world);
                ctx.request_repaint();
            } else if response.dragged() {
                app.wire_tool.on_pointer_move(pointer_world);
                ctx.request_repaint();
            } else if response.drag_stopped() {
                if let Some(wire) = app.wire_tool.finish_wire(pointer_world) {
                    project.record_undo();
                    project.active_board_mut().add_element(BoardElement::Wire(wire));
                }
                ctx.request_repaint();
            }
        }
        ActiveTool::Text => {
            if response.clicked() {
                project.record_undo();
                let id = Uuid::new_v4();
                let text_elem = TextElement::new_text(id, pointer_world, "Click to edit text".to_string());
                project.active_board_mut().add_element(BoardElement::Text(text_elem));
                app.select_tool.clear_selection();
                app.select_tool.selected_ids.insert(id);
                app.editing_text_id = Some(id);
                app.current_tool = ActiveTool::Select;
            }
        }
        ActiveTool::StickyNote => {
            if response.clicked() {
                project.record_undo();
                let id = Uuid::new_v4();
                let sticky = TextElement::new_sticky(
                    id,
                    pointer_world,
                    "Note / Pinout Idea".to_string(),
                    app.active_sticky_color,
                );
                project.active_board_mut().add_element(BoardElement::Text(sticky));
                app.select_tool.clear_selection();
                app.select_tool.selected_ids.insert(id);
                app.editing_text_id = Some(id);
                app.current_tool = ActiveTool::Select;
            }
        }
        ActiveTool::Table => {
            if response.clicked() {
                project.record_undo();
                let table = app.table_tool.create_table(pointer_world);
                let id = table.id;
                project.active_board_mut().add_element(BoardElement::Table(table));
                app.select_tool.clear_selection();
                app.select_tool.selected_ids.insert(id);
                app.table_tool.active_cell = Some((id, 0, 0));
                app.current_tool = ActiveTool::Select;
            }
        }
        ActiveTool::Component(ref comp_kind) => {
            if response.clicked() {
                project.record_undo();
                let comp = ComponentElement::new(Uuid::new_v4(), comp_kind.clone(), pointer_world);
                project.active_board_mut().add_element(BoardElement::Component(comp));
                app.current_tool = ActiveTool::Select;
            }
        }
    }
}

fn render_tool_previews(app: &SiroApp, painter: &egui::Painter, viewport_rect: Rect) {
    // 1. Freehand Pen Stroke preview while drawing
    if app.pen_tool.is_active() && app.pen_tool.current_points.len() >= 2 {
        let screen_points: Vec<Pos2> = app
            .pen_tool
            .current_points
            .iter()
            .map(|p| app.camera.world_to_screen(p.pos, viewport_rect))
            .collect();
        let stroke_color = if app.pen_tool.is_highlighter {
            let [r, g, b, _] = app.pen_tool.color.to_array();
            Color32::from_rgba_unmultiplied(r, g, b, 70)
        } else {
            app.pen_tool.color
        };
        painter.add(egui::epaint::PathShape::line(
            screen_points,
            Stroke::new(app.pen_tool.width * app.camera.zoom, stroke_color),
        ));
    }

    // 2. Shape preview while dragging
    if app.shape_tool.is_active() {
        if let (Some(start), Some(curr)) = (app.shape_tool.start_pos, app.shape_tool.current_pos) {
            let s1 = app.camera.world_to_screen(start, viewport_rect);
            let s2 = app.camera.world_to_screen(curr, viewport_rect);
            let preview_rect = Rect::from_two_pos(s1, s2);
            let stroke = Stroke::new(2.0_f32, Color32::from_rgb(0, 210, 255));
            painter.rect_stroke(preview_rect, CornerRadius::ZERO, stroke, StrokeKind::Middle);
        }
    }

    // 3. Wire preview while dragging
    if app.wire_tool.is_active() {
        if let (Some(start_endpoint), Some(curr_pos)) = (&app.wire_tool.active_start, app.wire_tool.current_preview_end) {
            let s1 = app.camera.world_to_screen(start_endpoint.pos, viewport_rect);
            let s2 = app.camera.world_to_screen(curr_pos, viewport_rect);
            let mid_x = (s1.x + s2.x) / 2.0;
            let stroke = Stroke::new(2.5_f32, app.wire_tool.color.to_color32());
            painter.line_segment([s1, Pos2::new(mid_x, s1.y)], stroke);
            painter.line_segment([Pos2::new(mid_x, s1.y), Pos2::new(mid_x, s2.y)], stroke);
            painter.line_segment([Pos2::new(mid_x, s2.y), s2], stroke);
            painter.circle_filled(s1, 4.0, app.wire_tool.color.to_color32());
            painter.circle_filled(s2, 4.0, app.wire_tool.color.to_color32());
        }
    }

    // 4. Box Lasso Selection Box preview
    if let Some(lasso_world) = app.select_tool.lasso_rect_world {
        let lasso_screen = app.camera.world_rect_to_screen(lasso_world, viewport_rect);
        painter.rect_filled(
            lasso_screen,
            CornerRadius::ZERO,
            Color32::from_rgba_unmultiplied(0, 210, 255, 30),
        );
        painter.rect_stroke(
            lasso_screen,
            CornerRadius::ZERO,
            Stroke::new(1.5_f32, Color32::from_rgb(0, 210, 255)),
            StrokeKind::Middle,
        );
    }
}

pub fn handle_global_shortcuts(app: &mut SiroApp, project: &mut Project, ctx: &egui::Context) {

    let is_typing = ctx.wants_keyboard_input()
        || app.editing_text_id.is_some()
        || app.table_tool.active_cell.is_some()
        || app.top_bar_state.is_editing_title
        || app.top_bar_state.sheet_modal.is_some()
        || app.top_bar_state.renaming_board_id.is_some();

    if !is_typing {
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::Z)) {
            if project.undo() {
                app.set_status("Undo");
            }
        }

        if ctx.input(|i| i.modifiers.command && (i.key_pressed(egui::Key::Y) || (i.modifiers.shift && i.key_pressed(egui::Key::Z)))) {
            if project.redo() {
                app.set_status("Redo");
            }
        }

        // Only Delete key deletes selected elements (Backspace is intentionally excluded so text editing is not disrupted)
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            if !app.select_tool.selected_ids.is_empty() {
                project.record_undo();
                for &id in &app.select_tool.selected_ids {
                    project.active_board_mut().remove_element(id);
                }
                app.select_tool.clear_selection();
                app.set_status("Deleted selected elements");
            }
        }

        // Duplicate shortcut (Ctrl+D / Cmd+D)
        if ctx.input(|i| i.modifiers.command && i.key_pressed(egui::Key::D)) {
            handle_properties_action(app, project, PropertiesAction::DuplicateSelected);
        }

        // Tool switching shortcuts: ONLY when no command or alt modifier is pressed!
        let no_cmd = ctx.input(|i| !i.modifiers.command && !i.modifiers.alt);
        if no_cmd {
            if ctx.input(|i| i.key_pressed(egui::Key::V)) {
                app.current_tool = ActiveTool::Select;
                app.select_tool.vertex_edit_mode = false;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::A)) {
                app.select_tool.vertex_edit_mode = !app.select_tool.vertex_edit_mode;
                app.current_tool = if app.select_tool.vertex_edit_mode {
                    ActiveTool::VertexEdit
                } else {
                    ActiveTool::Select
                };
            }
            if ctx.input(|i| i.key_pressed(egui::Key::P)) {
                app.current_tool = ActiveTool::Pen;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::H)) {
                app.current_tool = ActiveTool::Pan;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::E)) {
                app.current_tool = ActiveTool::Eraser;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::W)) {
                app.current_tool = ActiveTool::Wire;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::T)) {
                app.current_tool = ActiveTool::Text;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::N)) {
                app.current_tool = ActiveTool::StickyNote;
            }
            if ctx.input(|i| i.key_pressed(egui::Key::R)) {
                if !app.select_tool.selected_ids.is_empty() {
                    project.record_undo();
                    for &id in &app.select_tool.selected_ids {
                        if let Some(BoardElement::Component(comp)) = project.active_board_mut().get_element_mut(id) {
                            comp.rotation_quarters = (comp.rotation_quarters + 1) % 4;
                        }
                    }
                    project.active_board_mut().sync_connected_wires();
                    app.set_status("Rotated selected component 90°");
                }
            }
        }
    }

    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
        if app.top_bar_state.sheet_modal.is_some() {
            app.top_bar_state.sheet_modal = None;
        } else if app.top_bar_state.renaming_board_id.is_some() {
            app.top_bar_state.renaming_board_id = None;
        } else if app.top_bar_state.is_editing_title {
            app.top_bar_state.is_editing_title = false;
        } else if app.editing_text_id.is_some() {
            app.editing_text_id = None;
        } else if app.table_tool.active_cell.is_some() {
            app.table_tool.active_cell = None;
        } else if !app.top_bar_state.hamburger_open {
            app.select_tool.clear_selection();
        }
    }
}

fn handle_top_bar_action(app: &mut SiroApp, project: &mut Project, action: TopBarAction) {
    match action {
        TopBarAction::None
        | TopBarAction::GoHome
        | TopBarAction::CloseProject
        | TopBarAction::CloseProjectTab(_)
        | TopBarAction::SwitchProjectTab(_) => {}
        TopBarAction::RenameProject(name) => {
            app.rename_project(project, &name);
        }
        TopBarAction::NewProject => {
            app.create_and_open_new_project();
        }
        TopBarAction::OpenProject => {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(format!("{} Project (*.siro)", crate::APP_NAME), &["siro"])
                .pick_file()
            {
                app.open_or_focus_file(&path);
            }
        }
        TopBarAction::ClearBoard => {
            project.record_undo();
            project.active_board_mut().elements.clear();
            app.select_tool.clear_selection();
            app.set_status("Cleared board elements");
        }
        TopBarAction::SaveProject => {
            if let Some(path) = project.filepath.clone() {
                match SiroFileManager::save_project(project, &path) {
                    Ok(()) => {
                        app.home.add_recent(&path);
                        app.set_status("Project saved!");
                    }
                    Err(err) => {
                        app.set_status(format!("Save error: {}", err));
                    }
                }
            } else if let Some(path) = rfd::FileDialog::new()
                .add_filter(format!("{} Project (*.siro)", crate::APP_NAME), &["siro"])
                .save_file()
            {
                let save_path = if path.extension().is_none() {
                    path.with_extension("siro")
                } else {
                    path
                };
                if let Some(stem) = save_path.file_stem().and_then(|s| s.to_str()) {
                    if !stem.is_empty() {
                        project.manifest.name = stem.to_string();
                    }
                }
                match SiroFileManager::save_project(project, &save_path) {
                    Ok(()) => {
                        app.home.add_recent(&save_path);
                        app.set_status(format!("Saved to {:?}", save_path));
                    }
                    Err(err) => {
                        app.set_status(format!("Save error: {}", err));
                    }
                }
            }
        }
        TopBarAction::SaveProjectAs => {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter(format!("{} Project (*.siro)", crate::APP_NAME), &["siro"])
                .save_file()
            {
                let save_path = if path.extension().is_none() {
                    path.with_extension("siro")
                } else {
                    path
                };
                if let Some(stem) = save_path.file_stem().and_then(|s| s.to_str()) {
                    if !stem.is_empty() {
                        project.manifest.name = stem.to_string();
                    }
                }
                match SiroFileManager::save_project(project, &save_path) {
                    Ok(()) => {
                        app.home.add_recent(&save_path);
                        app.set_status(format!("Saved to {:?}", save_path));
                    }
                    Err(err) => {
                        app.set_status(format!("Save error: {}", err));
                    }
                }
            }
        }
        TopBarAction::ExportPng => {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("PNG Image (*.png)", &["png"])
                .save_file()
            {
                if let Err(err) = ExportManager::export_board_to_png(project.active_board(), &path) {
                    app.set_status(format!("Export PNG error: {}", err));
                } else {
                    app.set_status(format!("Exported PNG to {:?}", path));
                }
            }
        }
        TopBarAction::ExportSvg => {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("SVG Vector (*.svg)", &["svg"])
                .save_file()
            {
                if let Err(err) = ExportManager::export_board_to_svg(project.active_board(), &path) {
                    app.set_status(format!("Export SVG error: {}", err));
                } else {
                    app.set_status(format!("Exported SVG to {:?}", path));
                }
            }
        }
        TopBarAction::ExportBomCsv => {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("CSV (*.csv)", &["csv"])
                .save_file()
            {
                if let Err(err) = ExportManager::export_bom_to_csv(&project.planner, &path) {
                    app.set_status(format!("Export CSV error: {}", err));
                } else {
                    app.set_status(format!("Exported BOM to {:?}", path));
                }
            }
        }
        TopBarAction::ToggleTheme => {
            app.dark_mode = !app.dark_mode;
            let _ = crate::storage::history::save_theme_dark_mode(app.dark_mode);
        }
        TopBarAction::ToggleSmartSnapping => {
            app.settings.smart_snapping_enabled = !app.settings.smart_snapping_enabled;
            let status = if app.settings.smart_snapping_enabled {
                "Smart Snapping: ON"
            } else {
                "Smart Snapping: OFF"
            };
            app.set_status(status.to_string());
            let _ = crate::storage::settings::save_settings_to(
                &crate::storage::settings::settings_file_path(),
                &app.settings,
            );
        }
        TopBarAction::TogglePlanner => {
            app.planner_open = !app.planner_open;
        }
        TopBarAction::Undo => {
            project.undo();
        }
        TopBarAction::Redo => {
            project.redo();
        }
        TopBarAction::ZoomIn => {
            app.camera.zoom = (app.camera.zoom * 1.2).clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);
        }
        TopBarAction::ZoomOut => {
            app.camera.zoom = (app.camera.zoom / 1.2).clamp(Camera::MIN_ZOOM, Camera::MAX_ZOOM);
        }
        TopBarAction::ZoomReset => {
            app.camera.zoom = 1.0;
        }
        TopBarAction::ZoomFit => {
            let bounds = project.active_board().content_bounds();
            let vp = Rect::from_min_size(Pos2::ZERO, Vec2::new(1280.0, 720.0));
            app.camera.fit_to_rect(bounds, vp);
        }
        TopBarAction::SwitchBoard(id) => {
            project.switch_board(id);
            app.camera.pan = project.active_board().camera_pan;
            app.camera.zoom = project.active_board().camera_zoom;
            app.select_tool.clear_selection();
        }
        TopBarAction::OpenCreateSheetModal => {
            let existing = project.sheet_names();
            let suggested = crate::model::sheet_ops::suggest_next_sheet_name(&existing);
            app.top_bar_state.sheet_modal = Some(crate::ui::top_bar::SheetModalState {
                mode: crate::ui::top_bar::SheetModalMode::Create,
                buffer: suggested,
                error: None,
                focus_requested: true,
            });
        }
        TopBarAction::OpenRenameSheetModal(id) => {
            let cur_name = project.boards.get(&id).map(|b| b.name.clone()).unwrap_or_default();
            app.top_bar_state.sheet_modal = Some(crate::ui::top_bar::SheetModalState {
                mode: crate::ui::top_bar::SheetModalMode::Rename(id),
                buffer: cur_name,
                error: None,
                focus_requested: true,
            });
        }
        TopBarAction::CreateSheet(name) => {
            project.add_board(name);
            app.camera.pan = Pos2::new(0.0, 0.0);
            app.camera.zoom = 1.0;
            app.select_tool.clear_selection();
            if let Some(path) = project.filepath.clone() {
                if let Err(e) = SiroFileManager::save_project(project, &path) {
                    app.set_status(format!("Save error: {}", e));
                }
            }
            app.set_status("Created new sheet");
        }
        TopBarAction::RenameSheet(id, name) => {
            if project.rename_board(id, name) {
                if let Some(path) = project.filepath.clone() {
                    if let Err(e) = SiroFileManager::save_project(project, &path) {
                        app.set_status(format!("Save error: {}", e));
                    }
                }
                app.set_status("Renamed sheet");
            }
        }
        TopBarAction::AddBoard => {
            let existing = project.sheet_names();
            let suggested = crate::model::sheet_ops::suggest_next_sheet_name(&existing);
            project.add_board(suggested);
            app.camera.pan = Pos2::new(0.0, 0.0);
            app.camera.zoom = 1.0;
            app.select_tool.clear_selection();
            if let Some(path) = project.filepath.clone() {
                let _ = SiroFileManager::save_project(project, &path);
            }
        }
        TopBarAction::DeleteBoard(id) => {
            project.remove_board(id);
            app.select_tool.clear_selection();
        }
        TopBarAction::DeleteCurrentProject => {
            if let Some(path) = project.filepath.clone() {
                app.home.pending_delete_path = Some(path);
                app.state = AppState::Home;
            }
        }
    }
}

fn handle_properties_action(app: &mut SiroApp, project: &mut Project, action: PropertiesAction) {
    match action {
        PropertiesAction::None => {}
        PropertiesAction::DeleteSelected => {
            project.record_undo();
            for &id in &app.select_tool.selected_ids {
                project.active_board_mut().remove_element(id);
            }
            app.select_tool.clear_selection();
        }
        PropertiesAction::BringToFront => {
            for &id in &app.select_tool.selected_ids {
                project.active_board_mut().bring_to_front(id);
            }
        }
        PropertiesAction::SendToBack => {
            for &id in &app.select_tool.selected_ids {
                project.active_board_mut().send_to_back(id);
            }
        }
        PropertiesAction::BringForward => {
            for &id in &app.select_tool.selected_ids {
                project.active_board_mut().bring_forward(id);
            }
        }
        PropertiesAction::SendBackward => {
            for &id in &app.select_tool.selected_ids {
                project.active_board_mut().send_backward(id);
            }
        }
        PropertiesAction::RotateComponent(id) => {
            project.record_undo();
            if let Some(BoardElement::Component(comp)) = project.active_board_mut().get_element_mut(id) {
                comp.rotation_quarters = (comp.rotation_quarters + 1) % 4;
            }
            project.active_board_mut().sync_connected_wires();
            app.set_status("Rotated component 90°");
        }
        PropertiesAction::TableAddRow(id) => {
            project.record_undo();
            if let Some(BoardElement::Table(tbl)) = project.active_board_mut().get_element_mut(id) {
                TableTool::add_row(tbl);
            }
        }
        PropertiesAction::TableRemoveRow(id) => {
            project.record_undo();
            if let Some(BoardElement::Table(tbl)) = project.active_board_mut().get_element_mut(id) {
                let last = tbl.rows.saturating_sub(1);
                TableTool::remove_row(tbl, last);
            }
        }
        PropertiesAction::TableAddCol(id) => {
            project.record_undo();
            if let Some(BoardElement::Table(tbl)) = project.active_board_mut().get_element_mut(id) {
                TableTool::add_column(tbl);
            }
        }
        PropertiesAction::TableRemoveCol(id) => {
            project.record_undo();
            if let Some(BoardElement::Table(tbl)) = project.active_board_mut().get_element_mut(id) {
                let last = tbl.cols.saturating_sub(1);
                TableTool::remove_column(tbl, last);
            }
        }
        PropertiesAction::TableResizeCol(id, delta) => {
            if let Some(BoardElement::Table(tbl)) = project.active_board_mut().get_element_mut(id) {
                TableTool::resize_columns(tbl, delta);
            }
        }
        PropertiesAction::TableApplyColor(id, col) => {
            project.record_undo();
            if let Some(BoardElement::Table(tbl)) = project.active_board_mut().get_element_mut(id) {
                if let Some((sel_tbl_id, row, col_idx, start, end)) = app.table_tool.selected_text_range {
                    if sel_tbl_id == id && row < tbl.rows && col_idx < tbl.cols {
                        tbl.add_or_update_span(row, col_idx, start, end, col);
                        app.set_status("Recolored selected text");
                    } else if let Some((_, r, _)) = app.table_tool.active_cell {
                        tbl.set_row_color(r, Some(col));
                        app.set_status("Recolored row text");
                    }
                } else if let Some((_, r, _)) = app.table_tool.active_cell {
                    tbl.set_row_color(r, Some(col));
                    app.set_status("Recolored row text");
                }
            }
        }
        PropertiesAction::PdfPrevPage(id) => {
            if let Some(BoardElement::PdfCard(pdf)) = project.active_board_mut().get_element_mut(id) {
                if pdf.current_page > 0 {
                    pdf.current_page -= 1;
                }
            }
        }
        PropertiesAction::PdfNextPage(id) => {
            if let Some(BoardElement::PdfCard(pdf)) = project.active_board_mut().get_element_mut(id) {
                pdf.current_page += 1;
            }
        }
        PropertiesAction::LaunchDoc(id) => {
            launch_doc_attachment(project, id);
        }
        PropertiesAction::ImageResetPerspective(id) => {
            project.record_undo();
            if let Some(BoardElement::Image(img)) = project.active_board_mut().get_element_mut(id) {
                img.reset_perspective();
            }
            app.set_status("Reset image perspective");
        }
        PropertiesAction::DuplicateSelected => {
            if !app.select_tool.selected_ids.is_empty() {
                project.record_undo();
                let delta = Vec2::new(24.0, 24.0);
                let mut new_elements = Vec::new();
                let mut new_ids = std::collections::HashSet::new();
                for elem in &project.active_board().elements {
                    if app.select_tool.selected_ids.contains(&elem.id()) {
                        let dup = elem.duplicate_with_offset(delta);
                        new_ids.insert(dup.id());
                        new_elements.push(dup);
                    }
                }
                for elem in new_elements {
                    project.active_board_mut().add_element(elem);
                }
                app.select_tool.selected_ids = new_ids;
                app.set_status("Duplicated selection");
            }
        }
    }
}

fn handle_insert_media(app: &mut SiroApp, project: &mut Project) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter("Supported Media (*.png, *.jpg, *.pdf, *.docx, *.xlsx)", &["png", "jpg", "jpeg", "webp", "pdf", "docx", "xlsx", "csv"])
        .pick_file()
    {
        insert_file_from_path(app, project, &path);
    }
}

pub fn insert_file_from_path(app: &mut SiroApp, project: &mut Project, path: &std::path::Path) {
    if let Ok(bytes) = std::fs::read(path) {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();

        project.record_undo();
        let asset_id = project.asset_store.insert(&filename, &ext, bytes.clone());
        let place_pos = app.camera.pan + Vec2::new(150.0, 150.0);

        match ext.as_str() {
            "png" | "jpg" | "jpeg" | "webp" => {
                let img = ImageElement {
                    id: Uuid::new_v4(),
                    asset_id,
                    image_name: filename,
                    pos: place_pos,
                    size: Vec2::new(300.0, 200.0),
                    opacity: 1.0,
                    perspective_locked: true,
                    corners: None,
                };
                project.active_board_mut().add_element(BoardElement::Image(img));
                app.set_status("Inserted image onto board!");
            }
            "pdf" => {
                let pdf = PdfCardElement {
                    id: Uuid::new_v4(),
                    asset_id,
                    pdf_name: filename,
                    current_page: 0,
                    total_pages: 1,
                    pos: place_pos,
                    size: Vec2::new(320.0, 420.0),
                };
                project.active_board_mut().add_element(BoardElement::PdfCard(pdf));
                app.set_status("Inserted PDF datasheet card!");
            }
            _ => {
                let doc = DocAttachmentElement {
                    id: Uuid::new_v4(),
                    asset_id,
                    filename,
                    extension: ext,
                    size_bytes: bytes.len() as u64,
                    pos: place_pos,
                };
                project.active_board_mut().add_element(BoardElement::DocAttachment(doc));
                app.set_status("Inserted document attachment chip!");
            }
        }
    }
}

pub fn launch_doc_attachment(project: &Project, id: Uuid) {
    if let Some(BoardElement::DocAttachment(doc)) = project.active_board().get_element(id) {
        if let Some(asset) = project.asset_store.get(&doc.asset_id) {
            let temp_dir = crate::storage::history::siro_data_dir().join("attachments");
            let _ = std::fs::create_dir_all(&temp_dir);
            let target_file = temp_dir.join(&doc.filename);
            if std::fs::write(&target_file, &asset.data).is_ok() {
                let _ = std::process::Command::new("cmd")
                    .args(["/C", "start", "", target_file.to_str().unwrap_or("")])
                    .spawn();
            }
        }
    }
}

fn setup_custom_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(data) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
        fonts.font_data.insert(
            "SegoeUI".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(data)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "SegoeUI".to_owned());
    }
    if let Ok(sym_data) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
        fonts.font_data.insert(
            "SegoeUISymbol".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(sym_data)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("SegoeUISymbol".to_owned());
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .push("SegoeUISymbol".to_owned());
    }
    if let Ok(emoji_data) = std::fs::read("C:\\Windows\\Fonts\\seguiemj.ttf") {
        fonts.font_data.insert(
            "SegoeUIEmoji".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(emoji_data)),
        );
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .push("SegoeUIEmoji".to_owned());
    }
    if let Ok(georgia_data) = std::fs::read("C:\\Windows\\Fonts\\georgia.ttf") {
        fonts.font_data.insert(
            "Georgia".to_owned(),
            std::sync::Arc::new(egui::FontData::from_owned(georgia_data)),
        );
        fonts
            .families
            .insert(egui::FontFamily::Name("serif".into()), vec!["Georgia".to_owned()]);
    }
    ctx.set_fonts(fonts);
}
