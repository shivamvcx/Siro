use crate::model::board::Board;
use crate::model::element::BoardElement;
use egui::{Pos2, Rect, Vec2};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DragMode {
    None,
    Translate,
    BoxLasso,
    Resize(ResizeHandle),
    Vertex(usize),
    PerspectiveEdge(usize),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResizeHandle {
    TopLeft,
    TopCenter,
    TopRight,
    RightCenter,
    BottomRight,
    BottomCenter,
    BottomLeft,
    LeftCenter,
}

pub struct SelectTool {
    pub selected_ids: HashSet<Uuid>,
    pub vertex_edit_mode: bool,
    pub drag_mode: DragMode,
    pub start_pointer_world: Option<Pos2>,
    pub lasso_rect_world: Option<Rect>,
    pub initial_positions: HashMap<Uuid, Pos2>,
    pub initial_vertices: HashMap<Uuid, Vec<Pos2>>,
    pub initial_aabbs: HashMap<Uuid, Rect>,
    pub initial_combined_aabb: Option<Rect>,
    pub initial_table_col_widths: HashMap<Uuid, Vec<f32>>,
    pub initial_sizes: HashMap<Uuid, Vec2>,
    pub initial_font_sizes: HashMap<Uuid, f32>,
    pub initial_row_heights: HashMap<Uuid, f32>,
    pub zoom: f32,
    pub active_snap_guides: Vec<crate::canvas::snap::SnapGuideLine>,
    pub snap_enabled: bool,
    pub is_shift_down: bool,
    pub grid_spacing: Option<f32>,
    pub viewport_world: Option<Rect>,
}

impl Default for SelectTool {
    fn default() -> Self {
        Self {
            selected_ids: HashSet::new(),
            vertex_edit_mode: false,
            drag_mode: DragMode::None,
            start_pointer_world: None,
            lasso_rect_world: None,
            initial_positions: HashMap::new(),
            initial_vertices: HashMap::new(),
            initial_aabbs: HashMap::new(),
            initial_combined_aabb: None,
            initial_table_col_widths: HashMap::new(),
            initial_sizes: HashMap::new(),
            initial_font_sizes: HashMap::new(),
            initial_row_heights: HashMap::new(),
            zoom: 1.0,
            active_snap_guides: Vec::new(),
            snap_enabled: false,
            is_shift_down: false,
            grid_spacing: None,
            viewport_world: None,
        }
    }
}

impl SelectTool {
    pub fn select_single(&mut self, id: Uuid) {
        self.selected_ids.clear();
        self.selected_ids.insert(id);
    }

    pub fn toggle_select(&mut self, id: Uuid) {
        if self.selected_ids.contains(&id) {
            self.selected_ids.remove(&id);
        } else {
            self.selected_ids.insert(id);
        }
    }

    pub fn clear_selection(&mut self) {
        self.selected_ids.clear();
        self.drag_mode = DragMode::None;
    }

    pub fn is_selection_aspect_locked(&self, board: &Board) -> bool {
        if self.is_shift_down {
            return true;
        }
        if self.selected_ids.len() == 1 {
            let single_id = *self.selected_ids.iter().next().unwrap();
            if let Some(elem) = board.get_element(single_id) {
                match elem {
                    BoardElement::Image(img) => return img.perspective_locked,
                    BoardElement::PdfCard(_) => return true,
                    _ => {}
                }
            }
        }
        false
    }

    pub fn hit_corner_handle(&self, pointer_world: Pos2, board: &Board, zoom: f32) -> Option<ResizeHandle> {
        if self.selected_ids.is_empty() || self.vertex_edit_mode {
            return None;
        }

        // When perspective lock is OFF on a single selected image, corner resize handles are disabled
        // in favor of the 4 perspective corner warp handles
        if self.selected_ids.len() == 1 {
            let single_id = *self.selected_ids.iter().next().unwrap();
            if let Some(BoardElement::Image(img)) = board.get_element(single_id) {
                if !img.perspective_locked {
                    return None;
                }
            }
        }

        let mut combined: Option<Rect> = None;
        for &id in &self.selected_ids {
            if let Some(elem) = board.get_element(id) {
                let aabb = elem.geometric_aabb();
                combined = Some(match combined {
                    Some(cur) => cur.union(aabb),
                    None => aabb,
                });
            }
        }

        let world_aabb = combined?;
        if world_aabb.width() <= 0.0 && world_aabb.height() <= 0.0 {
            return None;
        }

        let mut min_aabb = world_aabb;
        if min_aabb.width() < 2.0 {
            min_aabb = min_aabb.expand2(Vec2::new(1.0, 0.0));
        }
        if min_aabb.height() < 2.0 {
            min_aabb = min_aabb.expand2(Vec2::new(0.0, 1.0));
        }

        let expand_world = 5.0 / zoom.max(0.01);
        let expanded = min_aabb.expand(expand_world);
        let hit_radius = (12.0 / zoom.max(0.01)).max(8.0);

        let corners = [
            (ResizeHandle::TopLeft, expanded.left_top(), min_aabb.left_top()),
            (ResizeHandle::TopRight, expanded.right_top(), min_aabb.right_top()),
            (ResizeHandle::BottomRight, expanded.right_bottom(), min_aabb.right_bottom()),
            (ResizeHandle::BottomLeft, expanded.left_bottom(), min_aabb.left_bottom()),
            (
                ResizeHandle::TopCenter,
                Pos2::new(expanded.center().x, expanded.min.y),
                Pos2::new(min_aabb.center().x, min_aabb.min.y),
            ),
            (
                ResizeHandle::BottomCenter,
                Pos2::new(expanded.center().x, expanded.max.y),
                Pos2::new(min_aabb.center().x, min_aabb.max.y),
            ),
            (
                ResizeHandle::LeftCenter,
                Pos2::new(expanded.min.x, expanded.center().y),
                Pos2::new(min_aabb.min.x, min_aabb.center().y),
            ),
            (
                ResizeHandle::RightCenter,
                Pos2::new(expanded.max.x, expanded.center().y),
                Pos2::new(min_aabb.max.x, min_aabb.center().y),
            ),
        ];

        for (handle, exp_pos, raw_pos) in corners {
            if (pointer_world - exp_pos).length() <= hit_radius || (pointer_world - raw_pos).length() <= hit_radius {
                return Some(handle);
            }
        }

        None
    }

    pub fn on_pointer_down(&mut self, pointer_world: Pos2, board: &Board, is_ctrl_shift: bool) {
        self.on_pointer_down_with_zoom(pointer_world, board, is_ctrl_shift, self.zoom);
    }

    pub fn on_pointer_down_with_zoom(&mut self, pointer_world: Pos2, board: &Board, is_ctrl_shift: bool, zoom: f32) {
        self.zoom = zoom;
        self.start_pointer_world = Some(pointer_world);

        // Check if pointer hit a vertex handle in vertex edit mode
        if self.vertex_edit_mode && self.selected_ids.len() == 1 {
            let single_id = *self.selected_ids.iter().next().unwrap();
            if let Some(BoardElement::Shape(s)) = board.get_element(single_id) {
                for (v_idx, &vert) in s.vertices.iter().enumerate() {
                    if (pointer_world - vert).length() < 10.0 {
                        self.drag_mode = DragMode::Vertex(v_idx);
                        self.initial_vertices.clear();
                        self.initial_vertices.insert(single_id, s.vertices.clone());
                        return;
                    }
                }
            }
        }

        // Check if pointer hit an image corner or edge handle when perspective lock is OFF
        if self.selected_ids.len() == 1 {
            let single_id = *self.selected_ids.iter().next().unwrap();
            if let Some(BoardElement::Image(img)) = board.get_element(single_id) {
                if !img.perspective_locked {
                    let corners_world = img.get_corners_world();
                    let hit_radius = (14.0 / zoom.max(0.01)).max(10.0);
                    // 1. Corner handles
                    for (c_idx, &pt_world) in corners_world.iter().enumerate() {
                        if (pointer_world - pt_world).length() <= hit_radius {
                            self.drag_mode = DragMode::Vertex(c_idx);
                            self.initial_vertices.clear();
                            self.initial_vertices.insert(single_id, img.get_corners_local().to_vec());
                            return;
                        }
                    }
                    // 2. Edge midpoint handles (Figma-style)
                    for e_idx in 0..4 {
                        let next_idx = (e_idx + 1) % 4;
                        let mid_world = Pos2::new(
                            (corners_world[e_idx].x + corners_world[next_idx].x) * 0.5,
                            (corners_world[e_idx].y + corners_world[next_idx].y) * 0.5,
                        );
                        if (pointer_world - mid_world).length() <= hit_radius {
                            self.drag_mode = DragMode::PerspectiveEdge(e_idx);
                            self.initial_vertices.clear();
                            self.initial_vertices.insert(single_id, img.get_corners_local().to_vec());
                            return;
                        }
                    }
                }
            }
        }

        // Check if pointer hit a corner resize handle of currently selected elements
        if !self.selected_ids.is_empty() && !self.vertex_edit_mode {
            if let Some(handle) = self.hit_corner_handle(pointer_world, board, zoom) {
                self.drag_mode = DragMode::Resize(handle);
                self.snapshot_initial_state(board);
                return;
            }
        }

        // Check if pointer hit an existing element
        let mut hit_elem: Option<Uuid> = None;
        for elem in board.elements.iter().rev() {
            if elem.contains_point(pointer_world, 8.0) {
                hit_elem = Some(elem.id());
                break;
            }
        }

        if let Some(hit_id) = hit_elem {
            if is_ctrl_shift {
                self.toggle_select(hit_id);
            } else if !self.selected_ids.contains(&hit_id) {
                self.select_single(hit_id);
            }

            self.drag_mode = DragMode::Translate;
            self.snapshot_initial_state(board);
        } else {
            // Clicked empty canvas -> Start Box Lasso
            if !is_ctrl_shift {
                self.clear_selection();
            }
            self.drag_mode = DragMode::BoxLasso;
            self.lasso_rect_world = Some(Rect::from_two_pos(pointer_world, pointer_world));
        }
    }

    pub fn on_pointer_move(&mut self, pointer_world: Pos2, board: &mut Board) {
        let start_pointer = match self.start_pointer_world {
            Some(p) => p,
            None => return,
        };

        let visible_cull_rect = self.viewport_world.unwrap_or(Rect::EVERYTHING).expand(100.0);
        let mut other_rects = Vec::new();
        for elem in &board.elements {
            if !self.selected_ids.contains(&elem.id()) {
                let aabb = elem.geometric_aabb();
                if aabb.intersects(visible_cull_rect) {
                    other_rects.push(aabb);
                }
            }
        }

        match self.drag_mode {
            DragMode::Translate => {
                // Defense #9: Absolute anchor offset, preventing cumulative float drift
                let mut total_delta = pointer_world - start_pointer;
                self.active_snap_guides.clear();

                if self.snap_enabled {
                    if let Some(init_combined) = self.initial_combined_aabb {
                        let tentative_rect = init_combined.translate(total_delta);
                        let threshold = 8.0 / self.zoom.max(0.01);
                        let snap_res = crate::canvas::snap::compute_snap(
                            tentative_rect,
                            &other_rects,
                            threshold,
                            self.grid_spacing,
                        );
                        total_delta += snap_res.offset;
                        self.active_snap_guides = snap_res.guides;
                    }
                }

                for &id in &self.selected_ids {
                    if let Some(elem) = board.get_element_mut(id) {
                        match elem {
                            BoardElement::Component(comp) => {
                                if let Some(&init_pos) = self.initial_positions.get(&id) {
                                    comp.pos = init_pos + total_delta;
                                }
                            }
                            BoardElement::Text(txt) => {
                                if let Some(&init_pos) = self.initial_positions.get(&id) {
                                    txt.pos = init_pos + total_delta;
                                }
                            }
                            BoardElement::Table(tbl) => {
                                if let Some(&init_pos) = self.initial_positions.get(&id) {
                                    tbl.pos = init_pos + total_delta;
                                }
                            }
                            BoardElement::PdfCard(pdf) => {
                                if let Some(&init_pos) = self.initial_positions.get(&id) {
                                    pdf.pos = init_pos + total_delta;
                                }
                            }
                            BoardElement::Image(img) => {
                                if let Some(&init_pos) = self.initial_positions.get(&id) {
                                    img.pos = init_pos + total_delta;
                                }
                            }
                            BoardElement::DocAttachment(doc) => {
                                if let Some(&init_pos) = self.initial_positions.get(&id) {
                                    doc.pos = init_pos + total_delta;
                                }
                            }
                            BoardElement::Shape(s) => {
                                if let Some(init_verts) = self.initial_vertices.get(&id) {
                                    s.vertices = init_verts.iter().map(|&v| v + total_delta).collect();
                                    s.recompute_aabb();
                                }
                            }
                            BoardElement::Stroke(stroke) => {
                                if let Some(init_verts) = self.initial_vertices.get(&id) {
                                    for (i, p) in stroke.points.iter_mut().enumerate() {
                                        if let Some(&init_p) = init_verts.get(i) {
                                            p.pos = init_p + total_delta;
                                        }
                                    }
                                    stroke.recompute_aabb();
                                }
                            }
                            BoardElement::Wire(w) => {
                                if let Some(init_verts) = self.initial_vertices.get(&id) {
                                    if w.start.element_id.is_none() {
                                        w.start.pos = init_verts[0] + total_delta;
                                    }
                                    if w.end.element_id.is_none() {
                                        w.end.pos = init_verts[1] + total_delta;
                                    }
                                }
                            }
                        }
                    }
                }
                board.sync_connected_wires();
            }
            DragMode::Resize(handle) => {
                let init_combined = match self.initial_combined_aabb {
                    Some(a) if a.width() > 0.0 || a.height() > 0.0 => a,
                    _ => return,
                };

                let is_lock_on = self.is_selection_aspect_locked(board);
                let threshold = if self.snap_enabled {
                    8.0 / self.zoom.max(0.01)
                } else {
                    0.0
                };
                let resize_res = crate::canvas::snap::compute_resize_with_snap(
                    init_combined,
                    handle,
                    pointer_world,
                    start_pointer,
                    is_lock_on,
                    &other_rects,
                    threshold,
                    self.grid_spacing,
                );

                let new_combined = resize_res.rect;
                self.active_snap_guides = if self.snap_enabled {
                    resize_res.snap_result.guides
                } else {
                    Vec::new()
                };

                let anchor = match handle {
                    ResizeHandle::TopLeft => init_combined.right_bottom(),
                    ResizeHandle::TopRight => init_combined.left_bottom(),
                    ResizeHandle::BottomRight => init_combined.left_top(),
                    ResizeHandle::BottomLeft => init_combined.right_top(),
                    ResizeHandle::TopCenter => Pos2::new(init_combined.center().x, init_combined.max.y),
                    ResizeHandle::BottomCenter => Pos2::new(init_combined.center().x, init_combined.min.y),
                    ResizeHandle::LeftCenter => Pos2::new(init_combined.max.x, init_combined.center().y),
                    ResizeHandle::RightCenter => Pos2::new(init_combined.min.x, init_combined.center().y),
                };

                let mut scale_x = (new_combined.width() / init_combined.width().max(1.0)).max(0.01);
                let mut scale_y = (new_combined.height() / init_combined.height().max(1.0)).max(0.01);

                if matches!(handle, ResizeHandle::TopCenter | ResizeHandle::BottomCenter) {
                    scale_x = 1.0;
                }
                if matches!(handle, ResizeHandle::LeftCenter | ResizeHandle::RightCenter) {
                    scale_y = 1.0;
                }

                for &id in &self.selected_ids {
                    if let Some(elem) = board.get_element_mut(id) {
                        match elem {
                            BoardElement::Table(tbl) => {
                                if let Some(&init_pos) = self.initial_positions.get(&id) {
                                    let new_x = anchor.x + (init_pos.x - anchor.x) * scale_x;
                                    let new_y = anchor.y + (init_pos.y - anchor.y) * scale_y;
                                    tbl.pos = Pos2::new(new_x, new_y);
                                }
                                if let Some(init_widths) = self.initial_table_col_widths.get(&id) {
                                    tbl.col_widths = init_widths
                                        .iter()
                                        .map(|&w| (w * scale_x).max(20.0))
                                        .collect();
                                }
                                if let Some(&init_rh) = self.initial_row_heights.get(&id) {
                                    tbl.custom_row_height = Some((init_rh * scale_y).clamp(16.0, 120.0));
                                }
                            }
                            BoardElement::Shape(s) => {
                                if let Some(init_verts) = self.initial_vertices.get(&id) {
                                    s.vertices = init_verts
                                        .iter()
                                        .map(|&v| {
                                            Pos2::new(
                                                anchor.x + (v.x - anchor.x) * scale_x,
                                                anchor.y + (v.y - anchor.y) * scale_y,
                                            )
                                        })
                                        .collect();
                                    s.recompute_aabb();
                                }
                            }
                            BoardElement::Text(txt) => {
                                if txt.is_sticky {
                                    let scale = scale_x.min(scale_y);
                                    if let Some(&init_pos) = self.initial_positions.get(&id) {
                                        let new_x = anchor.x + (init_pos.x - anchor.x) * scale;
                                        let new_y = anchor.y + (init_pos.y - anchor.y) * scale;
                                        txt.pos = Pos2::new(new_x, new_y);
                                    }
                                    if let Some(&init_size) = self.initial_sizes.get(&id) {
                                        txt.width = (init_size.x * scale).max(40.0);
                                    }
                                } else {
                                    if let Some(&init_pos) = self.initial_positions.get(&id) {
                                        let new_x = anchor.x + (init_pos.x - anchor.x) * scale_x;
                                        let new_y = anchor.y + (init_pos.y - anchor.y) * scale_y;
                                        txt.pos = Pos2::new(new_x, new_y);
                                    }
                                    // Figma behavior: resizing text box affects bounding width, not font size
                                    if let Some(&init_size) = self.initial_sizes.get(&id) {
                                        txt.width = (init_size.x * scale_x).max(40.0);
                                    }
                                }
                            }
                            BoardElement::Component(comp) => {
                                if let Some(&init_pos) = self.initial_positions.get(&id) {
                                    let new_x = anchor.x + (init_pos.x - anchor.x) * scale_x;
                                    let new_y = anchor.y + (init_pos.y - anchor.y) * scale_y;
                                    comp.pos = Pos2::new(new_x, new_y);
                                }
                                if let Some(&init_size) = self.initial_sizes.get(&id) {
                                    comp.width = (init_size.x * scale_x).max(30.0);
                                    comp.height = (init_size.y * scale_y).max(20.0);
                                    comp.terminals = comp.kind.generate_terminals(comp.width, comp.height);
                                }
                            }
                            BoardElement::PdfCard(pdf) => {
                                if is_lock_on {
                                    let scale = scale_x.min(scale_y);
                                    if let Some(&init_pos) = self.initial_positions.get(&id) {
                                        let new_x = anchor.x + (init_pos.x - anchor.x) * scale;
                                        let new_y = anchor.y + (init_pos.y - anchor.y) * scale;
                                        pdf.pos = Pos2::new(new_x, new_y);
                                    }
                                    if let Some(&init_size) = self.initial_sizes.get(&id) {
                                        let aspect = init_size.x / init_size.y.max(1.0);
                                        let new_w = (init_size.x * scale).max(40.0);
                                        let new_h = new_w / aspect;
                                        pdf.size = Vec2::new(new_w, new_h.max(30.0));
                                    }
                                } else {
                                    if let Some(&init_pos) = self.initial_positions.get(&id) {
                                        let new_x = anchor.x + (init_pos.x - anchor.x) * scale_x;
                                        let new_y = anchor.y + (init_pos.y - anchor.y) * scale_y;
                                        pdf.pos = Pos2::new(new_x, new_y);
                                    }
                                    if let Some(&init_size) = self.initial_sizes.get(&id) {
                                        pdf.size = Vec2::new(
                                            (init_size.x * scale_x).max(40.0),
                                            (init_size.y * scale_y).max(30.0),
                                        );
                                    }
                                }
                            }
                            BoardElement::Image(img) => {
                                if is_lock_on {
                                    let scale = scale_x.min(scale_y);
                                    if let Some(&init_pos) = self.initial_positions.get(&id) {
                                        let new_x = anchor.x + (init_pos.x - anchor.x) * scale;
                                        let new_y = anchor.y + (init_pos.y - anchor.y) * scale;
                                        img.pos = Pos2::new(new_x, new_y);
                                    }
                                    if let Some(&init_size) = self.initial_sizes.get(&id) {
                                        let aspect = init_size.x / init_size.y.max(1.0);
                                        let new_w = (init_size.x * scale).max(40.0);
                                        let new_h = new_w / aspect;
                                        img.size = Vec2::new(new_w, new_h.max(30.0));
                                    }
                                } else {
                                    if let Some(&init_pos) = self.initial_positions.get(&id) {
                                        let new_x = anchor.x + (init_pos.x - anchor.x) * scale_x;
                                        let new_y = anchor.y + (init_pos.y - anchor.y) * scale_y;
                                        img.pos = Pos2::new(new_x, new_y);
                                    }
                                    if let Some(&init_size) = self.initial_sizes.get(&id) {
                                        img.size = Vec2::new(
                                            (init_size.x * scale_x).max(40.0),
                                            (init_size.y * scale_y).max(30.0),
                                        );
                                    }
                                }
                                if let Some(init_corners) = self.initial_vertices.get(&id) {
                                    if img.corners.is_some() && init_corners.len() == 4 {
                                        let sx = if is_lock_on { scale_x.min(scale_y) } else { scale_x };
                                        let sy = if is_lock_on { scale_x.min(scale_y) } else { scale_y };
                                        img.corners = Some([
                                            Pos2::new(init_corners[0].x * sx, init_corners[0].y * sy),
                                            Pos2::new(init_corners[1].x * sx, init_corners[1].y * sy),
                                            Pos2::new(init_corners[2].x * sx, init_corners[2].y * sy),
                                            Pos2::new(init_corners[3].x * sx, init_corners[3].y * sy),
                                        ]);
                                    }
                                }
                            }
                            BoardElement::DocAttachment(doc) => {
                                if let Some(&init_pos) = self.initial_positions.get(&id) {
                                    let new_x = anchor.x + (init_pos.x - anchor.x) * scale_x;
                                    let new_y = anchor.y + (init_pos.y - anchor.y) * scale_y;
                                    doc.pos = Pos2::new(new_x, new_y);
                                }
                            }
                            BoardElement::Stroke(stroke) => {
                                if let Some(init_verts) = self.initial_vertices.get(&id) {
                                    for (i, p) in stroke.points.iter_mut().enumerate() {
                                        if let Some(&init_p) = init_verts.get(i) {
                                            p.pos = Pos2::new(
                                                anchor.x + (init_p.x - anchor.x) * scale_x,
                                                anchor.y + (init_p.y - anchor.y) * scale_y,
                                            );
                                        }
                                    }
                                    stroke.recompute_aabb();
                                }
                            }
                            BoardElement::Wire(w) => {
                                if let Some(init_verts) = self.initial_vertices.get(&id) {
                                    if w.start.element_id.is_none() {
                                        w.start.pos = Pos2::new(
                                            anchor.x + (init_verts[0].x - anchor.x) * scale_x,
                                            anchor.y + (init_verts[0].y - anchor.y) * scale_y,
                                        );
                                    }
                                    if w.end.element_id.is_none() {
                                        w.end.pos = Pos2::new(
                                            anchor.x + (init_verts[1].x - anchor.x) * scale_x,
                                            anchor.y + (init_verts[1].y - anchor.y) * scale_y,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                board.sync_connected_wires();
            }
            DragMode::Vertex(v_idx) => {
                if let Some(&single_id) = self.selected_ids.iter().next() {
                    if let Some(elem) = board.get_element_mut(single_id) {
                        match elem {
                            BoardElement::Shape(s) => {
                                if let Some(init_verts) = self.initial_vertices.get(&single_id) {
                                    if let Some(&init_v) = init_verts.get(v_idx) {
                                        let delta = pointer_world - start_pointer;
                                        let mut new_pos = init_v + delta;
                                        self.active_snap_guides.clear();
                                        if self.snap_enabled {
                                            let threshold = 8.0 / self.zoom.max(0.01);
                                            let snap_res = crate::canvas::snap::compute_snap_point(
                                                new_pos,
                                                &other_rects,
                                                threshold,
                                                self.grid_spacing,
                                            );
                                            new_pos += snap_res.offset;
                                            self.active_snap_guides = snap_res.guides;
                                        }
                                        if v_idx < s.vertices.len() {
                                            s.vertices[v_idx] = new_pos;
                                            s.recompute_aabb();
                                        }
                                    }
                                }
                            }
                            BoardElement::Wire(w) => {
                                if let Some(init_verts) = self.initial_vertices.get(&single_id) {
                                    let delta = pointer_world - start_pointer;
                                    let mut new_pos = init_verts.get(v_idx).copied().unwrap_or(pointer_world) + delta;
                                    self.active_snap_guides.clear();
                                    if self.snap_enabled {
                                        let threshold = 8.0 / self.zoom.max(0.01);
                                        let snap_res = crate::canvas::snap::compute_snap_point(
                                            new_pos,
                                            &other_rects,
                                            threshold,
                                            self.grid_spacing,
                                        );
                                        new_pos += snap_res.offset;
                                        self.active_snap_guides = snap_res.guides;
                                    }
                                    if v_idx == 0 && w.start.element_id.is_none() {
                                        w.start.pos = new_pos;
                                    } else if v_idx == 1 && w.end.element_id.is_none() {
                                        w.end.pos = new_pos;
                                    }
                                }
                            }
                            BoardElement::Image(img) => {
                                if let Some(init_verts) = self.initial_vertices.get(&single_id) {
                                    if let Some(&init_v) = init_verts.get(v_idx) {
                                        let delta = pointer_world - start_pointer;
                                        let mut world_pt = img.pos + (init_v + delta).to_vec2();
                                        self.active_snap_guides.clear();
                                        if self.snap_enabled {
                                            let threshold = 8.0 / self.zoom.max(0.01);
                                            let snap_res = crate::canvas::snap::compute_snap_point(
                                                world_pt,
                                                &other_rects,
                                                threshold,
                                                self.grid_spacing,
                                            );
                                            world_pt += snap_res.offset;
                                            self.active_snap_guides = snap_res.guides;
                                        }
                                        let mut current_corners = img.get_corners_local();
                                        if v_idx < 4 {
                                            current_corners[v_idx] = Pos2::new(world_pt.x - img.pos.x, world_pt.y - img.pos.y);
                                            if crate::model::element::ImageElement::is_valid_quad(&current_corners) {
                                                img.corners = Some(current_corners);
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            DragMode::PerspectiveEdge(e_idx) => {
                if let Some(&single_id) = self.selected_ids.iter().next() {
                    if let Some(BoardElement::Image(img)) = board.get_element_mut(single_id) {
                        if let Some(init_verts) = self.initial_vertices.get(&single_id) {
                            if init_verts.len() == 4 && e_idx < 4 {
                                let delta = pointer_world - start_pointer;
                                let c1 = e_idx;
                                let c2 = (e_idx + 1) % 4;
                                let init_mid = Pos2::new(
                                    (init_verts[c1].x + init_verts[c2].x) * 0.5,
                                    (init_verts[c1].y + init_verts[c2].y) * 0.5,
                                );
                                let world_mid = img.pos + (init_mid + delta).to_vec2();
                                self.active_snap_guides.clear();
                                let mut snap_offset = Vec2::ZERO;
                                if self.snap_enabled {
                                    let threshold = 8.0 / self.zoom.max(0.01);
                                    let snap_res = crate::canvas::snap::compute_snap_point(
                                        world_mid,
                                        &other_rects,
                                        threshold,
                                        self.grid_spacing,
                                    );
                                    snap_offset = snap_res.offset;
                                    self.active_snap_guides = snap_res.guides;
                                }
                                let effective_delta = delta + snap_offset;
                                let mut current_corners = img.get_corners_local();
                                current_corners[c1] = init_verts[c1] + effective_delta;
                                current_corners[c2] = init_verts[c2] + effective_delta;
                                if crate::model::element::ImageElement::is_valid_quad(&current_corners) {
                                    img.corners = Some(current_corners);
                                }
                            }
                        }
                    }
                }
            }
            DragMode::BoxLasso => {
                self.lasso_rect_world = Some(Rect::from_two_pos(start_pointer, pointer_world));
            }
            _ => {}
        }
    }

    pub fn on_pointer_up(&mut self, board: &Board) -> bool {
        let had_action = self.drag_mode != DragMode::None;

        if self.drag_mode == DragMode::BoxLasso {
            if let Some(lasso) = self.lasso_rect_world.take() {
                if lasso.width().abs() > 4.0 || lasso.height().abs() > 4.0 {
                    for elem in &board.elements {
                        if elem.intersects_rect(lasso) {
                            self.selected_ids.insert(elem.id());
                        }
                    }
                }
            }
        }

        self.drag_mode = DragMode::None;
        self.active_snap_guides.clear();
        self.start_pointer_world = None;
        self.initial_positions.clear();
        self.initial_vertices.clear();
        self.initial_aabbs.clear();
        self.initial_combined_aabb = None;
        self.initial_table_col_widths.clear();
        self.initial_sizes.clear();
        self.initial_font_sizes.clear();
        self.initial_row_heights.clear();

        had_action
    }

    fn snapshot_initial_state(&mut self, board: &Board) {
        self.initial_positions.clear();
        self.initial_vertices.clear();
        self.initial_aabbs.clear();
        self.initial_table_col_widths.clear();
        self.initial_sizes.clear();
        self.initial_font_sizes.clear();
        self.initial_row_heights.clear();

        let mut combined: Option<Rect> = None;
        for &id in &self.selected_ids {
            if let Some(elem) = board.get_element(id) {
                let aabb = elem.geometric_aabb();
                self.initial_aabbs.insert(id, aabb);
                combined = Some(match combined {
                    Some(cur) => cur.union(aabb),
                    None => aabb,
                });

                match elem {
                    BoardElement::Component(comp) => {
                        self.initial_positions.insert(id, comp.pos);
                        self.initial_sizes.insert(id, Vec2::new(comp.width, comp.height));
                    }
                    BoardElement::Text(txt) => {
                        self.initial_positions.insert(id, txt.pos);
                        self.initial_font_sizes.insert(id, txt.font_size);
                        self.initial_sizes.insert(id, Vec2::new(txt.width, txt.width));
                    }
                    BoardElement::Table(tbl) => {
                        self.initial_positions.insert(id, tbl.pos);
                        self.initial_table_col_widths.insert(id, tbl.col_widths.clone());
                        self.initial_row_heights.insert(id, tbl.row_height());
                    }
                    BoardElement::PdfCard(pdf) => {
                        self.initial_positions.insert(id, pdf.pos);
                        self.initial_sizes.insert(id, pdf.size);
                    }
                    BoardElement::Image(img) => {
                        self.initial_positions.insert(id, img.pos);
                        self.initial_sizes.insert(id, img.size);
                        self.initial_vertices.insert(id, img.get_corners_local().to_vec());
                    }
                    BoardElement::DocAttachment(doc) => {
                        self.initial_positions.insert(id, doc.pos);
                    }
                    BoardElement::Shape(s) => {
                        self.initial_vertices.insert(id, s.vertices.clone());
                    }
                    BoardElement::Stroke(stroke) => {
                        self.initial_vertices.insert(
                            id,
                            stroke.points.iter().map(|p| p.pos).collect(),
                        );
                    }
                    BoardElement::Wire(w) => {
                        self.initial_vertices.insert(id, vec![w.start.pos, w.end.pos]);
                    }
                }
            }
        }
        self.initial_combined_aabb = combined;
    }
}
