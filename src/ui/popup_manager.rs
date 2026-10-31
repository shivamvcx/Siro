use egui::{
    epaint::Shadow, Align2, Color32, Context, CornerRadius, CursorIcon, FontId, Id, Pos2, Rect,
    Sense, Stroke, Ui, Vec2,
};

#[derive(Clone, Debug)]
pub struct PopupItem {
    pub id: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub pos: Pos2,
    pub size: Vec2,
    pub anchor_rect: Option<Rect>,
    pub avoid_rect: Option<Rect>,
    pub initialized: bool,
    pub rect: Rect,
}

#[derive(Default, Clone, Debug)]
pub struct PopupManager {
    pub stack: Vec<PopupItem>,
    pub just_opened_id: Option<String>,
}

impl PopupManager {
    pub fn is_open(&self, id: &str) -> bool {
        self.stack.iter().any(|p| p.id == id)
    }

    pub fn has_open_popups(&self) -> bool {
        !self.stack.is_empty()
    }

    pub fn open(
        &mut self,
        id: &str,
        title: &str,
        parent_id: Option<&str>,
        anchor_rect: Option<Rect>,
        avoid_rect: Option<Rect>,
    ) {
        if let Some(p) = self.stack.iter_mut().find(|p| p.id == id) {
            p.title = title.to_string();
            p.anchor_rect = anchor_rect;
            p.avoid_rect = avoid_rect;
            return;
        }

        self.stack.push(PopupItem {
            id: id.to_string(),
            title: title.to_string(),
            parent_id: parent_id.map(|s| s.to_string()),
            pos: Pos2::ZERO,
            size: Vec2::new(210.0, 180.0),
            anchor_rect,
            avoid_rect,
            initialized: false,
            rect: Rect::ZERO,
        });
        self.just_opened_id = Some(id.to_string());
    }

    pub fn close(&mut self, id: &str) {
        let mut to_remove = vec![id.to_string()];
        let mut added = true;
        while added {
            added = false;
            for p in &self.stack {
                if let Some(ref p_parent) = p.parent_id {
                    if to_remove.contains(p_parent) && !to_remove.contains(&p.id) {
                        to_remove.push(p.id.clone());
                        added = true;
                    }
                }
            }
        }
        self.stack.retain(|p| !to_remove.contains(&p.id));
    }

    pub fn close_all(&mut self) {
        self.stack.clear();
    }

    pub fn toggle(
        &mut self,
        id: &str,
        title: &str,
        parent_id: Option<&str>,
        anchor_rect: Option<Rect>,
        avoid_rect: Option<Rect>,
    ) {
        if self.is_open(id) {
            self.close(id);
        } else {
            self.open(id, title, parent_id, anchor_rect, avoid_rect);
        }
    }

    pub fn bring_to_top(&mut self, id: &str) {
        if let Some(idx) = self.stack.iter().position(|p| p.id == id) {
            if idx + 1 == self.stack.len() {
                return;
            }
            let mut to_move_ids = vec![id.to_string()];
            let mut added = true;
            while added {
                added = false;
                for p in &self.stack {
                    if let Some(ref p_parent) = p.parent_id {
                        if to_move_ids.contains(p_parent) && !to_move_ids.contains(&p.id) {
                            to_move_ids.push(p.id.clone());
                            added = true;
                        }
                    }
                }
            }
            let mut moving = Vec::new();
            let mut remaining = Vec::new();
            for p in self.stack.drain(..) {
                if to_move_ids.contains(&p.id) {
                    moving.push(p);
                } else {
                    remaining.push(p);
                }
            }
            remaining.extend(moving);
            self.stack = remaining;
        }
    }

    /// Handles Escape key (closes topmost popup first) and Outside click (closes entire stack
    /// only if click lands outside the union of ALL open popups).
    /// Returns `true` if an event was handled / consumed (e.g. Esc pressed).
    pub fn update_input(&mut self, ctx: &Context) -> bool {
        // 1. Esc closes only topmost popup first
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            if let Some(top) = self.stack.last().cloned() {
                self.close(&top.id);
                return true;
            }
        }

        // 2. Outside click detection:
        // Counts only if pointer pressed down outside the union of ALL open popup rects.
        if ctx.input(|i| i.pointer.any_pressed()) {
            if let Some(ptr_pos) = ctx.input(|i| i.pointer.interact_pos()) {
                if let Some(ref just_opened) = self.just_opened_id {
                    // Click was likely on the toggle button that opened the popup; keep it open
                    let _ = just_opened;
                } else if !self.stack.is_empty() {
                    let clicked_in_popup = self.stack.iter().position(|p| p.rect.contains(ptr_pos));
                    if let Some(idx) = clicked_in_popup {
                        let clicked_id = self.stack[idx].id.clone();
                        self.bring_to_top(&clicked_id);
                    } else {
                        self.close_all();
                    }
                }
            }
        }

        self.just_opened_id = None;
        false
    }

    pub fn calculate_initial_pos(
        &self,
        ctx: &Context,
        parent_id: Option<&str>,
        anchor_rect: Option<Rect>,
        avoid_rect: Option<Rect>,
        popup_size: Vec2,
    ) -> Pos2 {
        let screen = ctx.screen_rect();
        let margin = 8.0_f32;

        if let Some(p_id) = parent_id {
            if let Some(parent) = self.stack.iter().find(|p| p.id == p_id) {
                let mut x = parent.rect.max.x + margin;
                let mut y = parent.rect.min.y;
                if x + popup_size.x > screen.max.x - margin {
                    x = parent.rect.min.x - popup_size.x - margin;
                }
                x = x.clamp(screen.min.x + margin, (screen.max.x - popup_size.x - margin).max(screen.min.x + margin));
                y = y.clamp(screen.min.y + margin, (screen.max.y - popup_size.y - margin).max(screen.min.y + margin));
                return Pos2::new(x, y);
            }
        }

        if let Some(avoid) = avoid_rect {
            let anchor = anchor_rect.unwrap_or(avoid);
            // Placement strategy: avoid covering selected object
            // 1. Place above anchor if there's enough space
            if anchor.min.y - popup_size.y - margin >= screen.min.y + margin {
                let x = anchor.min.x.clamp(screen.min.x + margin, (screen.max.x - popup_size.x - margin).max(screen.min.x + margin));
                let y = anchor.min.y - popup_size.y - margin;
                return Pos2::new(x, y);
            }
            // 2. Place to right of avoid rect
            if avoid.max.x + margin + popup_size.x <= screen.max.x - margin {
                let x = avoid.max.x + margin;
                let y = avoid.min.y.clamp(screen.min.y + margin, (screen.max.y - popup_size.y - margin).max(screen.min.y + margin));
                return Pos2::new(x, y);
            }
            // 3. Place to left of avoid rect
            if avoid.min.x - margin - popup_size.x >= screen.min.x + margin {
                let x = avoid.min.x - margin - popup_size.x;
                let y = avoid.min.y.clamp(screen.min.y + margin, (screen.max.y - popup_size.y - margin).max(screen.min.y + margin));
                return Pos2::new(x, y);
            }
            // 4. Place below avoid rect
            if avoid.max.y + margin + popup_size.y <= screen.max.y - margin {
                let x = anchor.min.x.clamp(screen.min.x + margin, (screen.max.x - popup_size.x - margin).max(screen.min.x + margin));
                let y = avoid.max.y + margin;
                return Pos2::new(x, y);
            }
        }

        if let Some(anchor) = anchor_rect {
            let mut x = anchor.min.x;
            let mut y = anchor.max.y + margin;
            if y + popup_size.y > screen.max.y - margin {
                y = anchor.min.y - popup_size.y - margin;
            }
            x = x.clamp(screen.min.x + margin, (screen.max.x - popup_size.x - margin).max(screen.min.x + margin));
            y = y.clamp(screen.min.y + margin, (screen.max.y - popup_size.y - margin).max(screen.min.y + margin));
            return Pos2::new(x, y);
        }

        Pos2::new(120.0, 120.0)
    }

    pub fn clamp_rect_to_screen(pos: Pos2, size: Vec2, screen: Rect) -> Pos2 {
        let margin = 8.0_f32;
        let max_x = (screen.max.x - size.x - margin).max(screen.min.x + margin);
        let max_y = (screen.max.y - size.y - margin).max(screen.min.y + margin);
        Pos2::new(
            pos.x.clamp(screen.min.x + margin, max_x),
            pos.y.clamp(screen.min.y + margin, max_y),
        )
    }

    /// Renders a popup with Figma-style frame, draggable title bar, close button,
    /// and auto-placement that avoids covering the selected object.
    pub fn show<R>(
        &mut self,
        ctx: &Context,
        is_dark: bool,
        id: &str,
        title: &str,
        parent_id: Option<&str>,
        anchor_rect: Option<Rect>,
        avoid_rect: Option<Rect>,
        content: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        if !self.is_open(id) {
            return None;
        }

        let screen = ctx.screen_rect();
        let popup_idx = self.stack.iter().position(|p| p.id == id)?;

        if !self.stack[popup_idx].initialized {
            let size = if self.stack[popup_idx].size.x > 20.0 {
                self.stack[popup_idx].size
            } else {
                Vec2::new(210.0, 180.0)
            };
            self.stack[popup_idx].pos =
                self.calculate_initial_pos(ctx, parent_id, anchor_rect, avoid_rect, size);
            self.stack[popup_idx].initialized = true;
        }

        let current_pos = self.stack[popup_idx].pos;
        let current_size = self.stack[popup_idx].size;
        let clamped_pos = Self::clamp_rect_to_screen(current_pos, current_size, screen);
        self.stack[popup_idx].pos = clamped_pos;

        let mut should_close = false;
        let mut drag_delta = Vec2::ZERO;
        let area_id = Id::new(format!("popup_area_{}", id));

        let bg_fill = if is_dark {
            Color32::from_rgb(32, 36, 46)
        } else {
            Color32::WHITE
        };
        let border_stroke = Stroke::new(
            1.0_f32,
            if is_dark {
                Color32::from_rgb(60, 68, 85)
            } else {
                Color32::from_rgb(215, 220, 230)
            },
        );
        let title_bg = if is_dark {
            Color32::from_rgb(25, 28, 36)
        } else {
            Color32::from_rgb(245, 247, 250)
        };
        let text_color = if is_dark {
            Color32::from_rgb(230, 235, 245)
        } else {
            Color32::from_rgb(40, 45, 55)
        };

        let mut result = None;

        let area_resp = egui::Area::new(area_id)
            .fixed_pos(clamped_pos)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let frame = egui::Frame::NONE
                    .fill(bg_fill)
                    .stroke(border_stroke)
                    .corner_radius(CornerRadius::same(8))
                    .shadow(Shadow {
                        offset: [0, 4],
                        blur: 16,
                        spread: 0,
                        color: Color32::from_black_alpha(if is_dark { 70 } else { 35 }),
                    });

                frame.show(ui, |ui| {
                    ui.set_min_width(190.0);
                    ui.set_max_width(320.0);

                    // --- 1. Title bar ---
                    let title_h = 28.0;
                    let (title_rect, title_resp) = ui.allocate_exact_size(
                        Vec2::new(ui.available_width(), title_h),
                        Sense::click_and_drag(),
                    );

                    ui.painter().rect_filled(
                        title_rect,
                        CornerRadius {
                            nw: 8,
                            ne: 8,
                            sw: 0,
                            se: 0,
                        },
                        title_bg,
                    );
                    ui.painter().line_segment(
                        [
                            Pos2::new(title_rect.min.x, title_rect.max.y),
                            Pos2::new(title_rect.max.x, title_rect.max.y),
                        ],
                        Stroke::new(1.0_f32, border_stroke.color),
                    );

                    if title_resp.dragged() {
                        drag_delta = title_resp.drag_delta();
                        ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                    } else if title_resp.hovered() {
                        ui.ctx().set_cursor_icon(CursorIcon::Grab);
                    }

                    let title_text_pos = Pos2::new(title_rect.min.x + 10.0, title_rect.center().y);
                    ui.painter().text(
                        title_text_pos,
                        Align2::LEFT_CENTER,
                        title,
                        FontId::proportional(12.0),
                        text_color,
                    );

                    let close_btn_size = 18.0;
                    let close_rect = Rect::from_center_size(
                        Pos2::new(title_rect.max.x - 14.0, title_rect.center().y),
                        Vec2::splat(close_btn_size),
                    );
                    let close_resp = ui.interact(
                        close_rect,
                        Id::new(format!("close_{}", id)),
                        Sense::click(),
                    );
                    let close_hovered = close_resp.hovered();
                    if close_hovered {
                        ui.painter().rect_filled(
                            close_rect,
                            CornerRadius::same(4),
                            if is_dark {
                                Color32::from_rgb(50, 56, 70)
                            } else {
                                Color32::from_rgb(230, 235, 242)
                            },
                        );
                    }
                    ui.painter().text(
                        close_rect.center(),
                        Align2::CENTER_CENTER,
                        "✕",
                        FontId::proportional(11.0),
                        if close_hovered {
                            Color32::from_rgb(235, 60, 60)
                        } else {
                            Color32::from_rgb(140, 145, 160)
                        },
                    );
                    if close_resp.clicked() {
                        should_close = true;
                    }

                    // --- 2. Body content ---
                    let body_frame = egui::Frame::NONE.inner_margin(egui::Margin::same(10));
                    body_frame.show(ui, |ui| {
                        result = Some(content(ui));
                    });
                });
            });

        let final_rect = area_resp.response.rect;
        if let Some(p) = self.stack.iter_mut().find(|p| p.id == id) {
            p.size = final_rect.size();
            p.rect = final_rect;
            if drag_delta != Vec2::ZERO {
                p.pos = Self::clamp_rect_to_screen(p.pos + drag_delta, p.size, screen);
            }
        }

        if should_close {
            self.close(id);
        }

        result
    }
}
