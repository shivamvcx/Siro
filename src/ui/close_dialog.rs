use crate::model::project::Project;
use egui::{Color32, CornerRadius, Rect, Stroke, Ui, Vec2};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClosePlan {
    Allow,
    Ask,
}

/// Pure decision function:
/// If has_unsaved_content is true, returns ClosePlan::Ask.
/// Otherwise returns ClosePlan::Allow.
pub fn close_plan(has_unsaved_content: bool) -> ClosePlan {
    if has_unsaved_content {
        ClosePlan::Ask
    } else {
        ClosePlan::Allow
    }
}

/// App-wide decision helper across all open projects.
/// If any project has content AND is unsaved, returns ClosePlan::Ask.
/// Otherwise returns ClosePlan::Allow.
pub fn app_close_plan<'a>(projects: impl IntoIterator<Item = &'a Project>) -> ClosePlan {
    let has_unsaved_content = projects
        .into_iter()
        .any(|p| p.has_content() && p.is_unsaved());
    close_plan(has_unsaved_content)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CloseDialogTarget {
    App,
    Tab(usize),
}

#[derive(Clone, Debug)]
pub struct CloseDialogState {
    pub target: CloseDialogTarget,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseDialogAction {
    None,
    SaveThenExitOrClose,
    DiscardChanges,
    Cancel,
}

/// Renders the 3-option modal close dialog over the current UI.
/// Handles Escape key for Cancel. Background clicks are intercepted and do not dismiss.
pub fn render_close_dialog(
    ui: &mut Ui,
    state: &CloseDialogState,
    target_project_name: Option<&str>,
    dark_mode: bool,
) -> CloseDialogAction {
    let mut action = CloseDialogAction::None;
    let screen_rect = ui.ctx().screen_rect();

    // 1. Full-screen translucent backdrop capturing all pointer interactions
    let backdrop_alpha = if dark_mode { 160 } else { 120 };
    ui.painter().rect_filled(
        screen_rect,
        CornerRadius::ZERO,
        Color32::from_black_alpha(backdrop_alpha),
    );
    let _ = ui.allocate_rect(screen_rect, egui::Sense::click());

    // 2. Centered Modal Dialog Card
    let dialog_w = 440.0;
    let dialog_h = 190.0;
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
    let text_title_color = if dark_mode {
        Color32::WHITE
    } else {
        Color32::from_rgb(20, 24, 35)
    };
    let text_body_color = if dark_mode {
        Color32::from_rgb(180, 185, 200)
    } else {
        Color32::from_rgb(85, 90, 105)
    };

    // Soft drop shadow
    ui.painter().rect_filled(
        dialog_rect.translate(Vec2::new(0.0, 6.0)),
        CornerRadius::same(12),
        Color32::from_black_alpha(45),
    );
    // Dialog surface
    ui.painter().rect_filled(dialog_rect, CornerRadius::same(12), card_bg);
    ui.painter().rect_stroke(
        dialog_rect,
        CornerRadius::same(12),
        Stroke::new(1.0_f32, card_border),
        egui::StrokeKind::Middle,
    );

    // Dialog layout
    let inner_rect = dialog_rect.shrink(20.0);
    let mut card_ui = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(inner_rect)
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    let (title_text, body_text, save_btn_text) = match state.target {
        CloseDialogTarget::App => (
            "Save Changes Before Exiting?",
            "You have unsaved changes in your open projects. If you exit without saving, those changes will be lost.".to_string(),
            "Save then exit",
        ),
        CloseDialogTarget::Tab(_) => {
            let name = target_project_name.unwrap_or("Project");
            (
                "Save Changes Before Closing?",
                format!("'{}' has unsaved changes. If you close this tab without saving, your recent edits will be lost.", name),
                "Save then close",
            )
        }
    };

    // Title
    card_ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("⚠️")
                .size(16.0)
                .color(Color32::from_rgb(245, 175, 45)),
        );
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new(title_text)
                .size(16.0)
                .strong()
                .color(text_title_color),
        );
    });

    card_ui.add_space(10.0);

    // Body
    card_ui.label(
        egui::RichText::new(body_text)
            .size(12.5)
            .color(text_body_color),
    );

    card_ui.add_space(20.0);

    // Three Action Buttons: [Cancel] [Discard changes] [Save then exit/close]
    card_ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        ui.spacing_mut().button_padding = Vec2::new(14.0, 7.0);

        // 1. Primary Save Button (Accent Blue)
        let save_btn = egui::Button::new(
            egui::RichText::new(save_btn_text)
                .size(12.5)
                .strong()
                .color(Color32::WHITE),
        )
        .fill(Color32::from_rgb(45, 96, 255))
        .corner_radius(CornerRadius::same(6));

        if ui.add(save_btn).clicked() {
            action = CloseDialogAction::SaveThenExitOrClose;
        }

        ui.add_space(6.0);

        // 2. Discard Changes Button (Red tint / destructive warning)
        let discard_btn = egui::Button::new(
            egui::RichText::new("Discard changes")
                .size(12.5)
                .color(if dark_mode {
                    Color32::from_rgb(255, 110, 110)
                } else {
                    Color32::from_rgb(215, 45, 45)
                }),
        )
        .fill(if dark_mode {
            Color32::from_rgba_unmultiplied(255, 60, 60, 20)
        } else {
            Color32::from_rgba_unmultiplied(220, 38, 38, 14)
        })
        .stroke(Stroke::new(
            1.0_f32,
            if dark_mode {
                Color32::from_rgba_unmultiplied(255, 80, 80, 60)
            } else {
                Color32::from_rgba_unmultiplied(220, 38, 38, 60)
            },
        ))
        .corner_radius(CornerRadius::same(6));

        if ui.add(discard_btn).clicked() {
            action = CloseDialogAction::DiscardChanges;
        }

        ui.add_space(6.0);

        // 3. Cancel Button
        let cancel_btn = egui::Button::new(
            egui::RichText::new("Cancel")
                .size(12.5)
                .color(if dark_mode {
                    Color32::from_rgb(180, 185, 200)
                } else {
                    Color32::from_rgb(90, 95, 110)
                }),
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::new(1.0_f32, card_border))
        .corner_radius(CornerRadius::same(6));

        if ui.add(cancel_btn).clicked() {
            action = CloseDialogAction::Cancel;
        }
    });

    // Keyboard Shortcut: Escape = Cancel
    if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
        action = CloseDialogAction::Cancel;
    }

    action
}
