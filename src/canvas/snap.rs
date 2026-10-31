use crate::canvas::tools::select::ResizeHandle;
use egui::{Pos2, Rect, Vec2};

/// A visual guide line indicating alignment with other objects or grid lines.
#[derive(Clone, Debug, PartialEq)]
pub struct SnapGuideLine {
    /// World-space start position of the guide line
    pub start: Pos2,
    /// World-space end position of the guide line
    pub end: Pos2,
    /// Specific matched anchor points along the guide line
    pub matched_points: Vec<Pos2>,
}

/// Result of a snap calculation containing the translation offset and guide lines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SnapResult {
    /// The offset to apply to the dragged geometry: (dx, dy)
    pub offset: Vec2,
    /// Whether X was snapped
    pub snapped_x: bool,
    /// Whether Y was snapped
    pub snapped_y: bool,
    /// Guide lines to render (in world coordinates)
    pub guides: Vec<SnapGuideLine>,
}

/// Computes Figma-style smart snapping for moving an entire bounding box.
/// Tests Left, Center-X, Right and Top, Center-Y, Bottom against all other rects and optional grid.
pub fn compute_snap(
    dragged_rect: Rect,
    other_rects: &[Rect],
    threshold: f32,
    grid_spacing: Option<f32>,
) -> SnapResult {
    let mut result = SnapResult::default();

    if threshold <= 0.0 {
        return result;
    }

    // 1. Compute snap on X axis
    let dragged_x_anchors = [
        dragged_rect.min.x,
        dragged_rect.center().x,
        dragged_rect.max.x,
    ];

    let mut best_snap_x: Option<(f32, f32, f32, Option<Rect>)> = None; // (abs_dist, delta, target_x, source_rect)

    for other in other_rects {
        let other_x_anchors = [other.min.x, other.center().x, other.max.x];
        for &d_x in &dragged_x_anchors {
            for &o_x in &other_x_anchors {
                let delta = o_x - d_x;
                let dist = delta.abs();
                if dist <= threshold {
                    match best_snap_x {
                        Some((best_dist, _, _, _)) if dist < best_dist => {
                            best_snap_x = Some((dist, delta, o_x, Some(*other)));
                        }
                        None => {
                            best_snap_x = Some((dist, delta, o_x, Some(*other)));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // If no object snap on X, check canvas grid lines
    if best_snap_x.is_none() {
        if let Some(spacing) = grid_spacing {
            if spacing > 0.0 {
                for &d_x in &dragged_x_anchors {
                    let grid_x = (d_x / spacing).round() * spacing;
                    let delta = grid_x - d_x;
                    let dist = delta.abs();
                    if dist <= threshold {
                        match best_snap_x {
                            Some((best_dist, _, _, _)) if dist < best_dist => {
                                best_snap_x = Some((dist, delta, grid_x, None));
                            }
                            None => {
                                best_snap_x = Some((dist, delta, grid_x, None));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    if let Some((_, delta_x, target_x, target_rect_opt)) = best_snap_x {
        result.offset.x = delta_x;
        result.snapped_x = true;

        let margin = 12.0;
        let y_min;
        let y_max;
        let mut matched = vec![Pos2::new(target_x, dragged_rect.center().y)];

        if let Some(target_rect) = target_rect_opt {
            y_min = dragged_rect.min.y.min(target_rect.min.y) - margin;
            y_max = dragged_rect.max.y.max(target_rect.max.y) + margin;
            matched.push(Pos2::new(target_x, target_rect.center().y));
        } else {
            y_min = dragged_rect.min.y - margin;
            y_max = dragged_rect.max.y + margin;
        }

        result.guides.push(SnapGuideLine {
            start: Pos2::new(target_x, y_min),
            end: Pos2::new(target_x, y_max),
            matched_points: matched,
        });
    }

    // 2. Compute snap on Y axis
    let dragged_y_anchors = [
        dragged_rect.min.y,
        dragged_rect.center().y,
        dragged_rect.max.y,
    ];

    let mut best_snap_y: Option<(f32, f32, f32, Option<Rect>)> = None;

    for other in other_rects {
        let other_y_anchors = [other.min.y, other.center().y, other.max.y];
        for &d_y in &dragged_y_anchors {
            for &o_y in &other_y_anchors {
                let delta = o_y - d_y;
                let dist = delta.abs();
                if dist <= threshold {
                    match best_snap_y {
                        Some((best_dist, _, _, _)) if dist < best_dist => {
                            best_snap_y = Some((dist, delta, o_y, Some(*other)));
                        }
                        None => {
                            best_snap_y = Some((dist, delta, o_y, Some(*other)));
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // If no object snap on Y, check canvas grid lines
    if best_snap_y.is_none() {
        if let Some(spacing) = grid_spacing {
            if spacing > 0.0 {
                for &d_y in &dragged_y_anchors {
                    let grid_y = (d_y / spacing).round() * spacing;
                    let delta = grid_y - d_y;
                    let dist = delta.abs();
                    if dist <= threshold {
                        match best_snap_y {
                            Some((best_dist, _, _, _)) if dist < best_dist => {
                                best_snap_y = Some((dist, delta, grid_y, None));
                            }
                            None => {
                                best_snap_y = Some((dist, delta, grid_y, None));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }

    if let Some((_, delta_y, target_y, target_rect_opt)) = best_snap_y {
        result.offset.y = delta_y;
        result.snapped_y = true;

        let margin = 12.0;
        let x_min;
        let x_max;
        let mut matched = vec![Pos2::new(dragged_rect.center().x, target_y)];

        if let Some(target_rect) = target_rect_opt {
            x_min = dragged_rect.min.x.min(target_rect.min.x) - margin;
            x_max = dragged_rect.max.x.max(target_rect.max.x) + margin;
            matched.push(Pos2::new(target_rect.center().x, target_y));
        } else {
            x_min = dragged_rect.min.x - margin;
            x_max = dragged_rect.max.x + margin;
        }

        result.guides.push(SnapGuideLine {
            start: Pos2::new(x_min, target_y),
            end: Pos2::new(x_max, target_y),
            matched_points: matched,
        });
    }

    // 3. Equal Spacing Snapping (Optional, if no edge snap occurred on that axis)
    if !result.snapped_x {
        if let Some(eq_snap) = compute_equal_spacing_x(dragged_rect, other_rects, threshold) {
            result.offset.x = eq_snap.offset.x;
            result.snapped_x = true;
            result.guides.extend(eq_snap.guides);
        }
    }

    if !result.snapped_y {
        if let Some(eq_snap) = compute_equal_spacing_y(dragged_rect, other_rects, threshold) {
            result.offset.y = eq_snap.offset.y;
            result.snapped_y = true;
            result.guides.extend(eq_snap.guides);
        }
    }

    result
}

/// Computes snapping for resizing an object using one of the 8 resize handles.
/// Only the moving edge(s) of the handle are snapped to other objects' edges and centers.
pub fn compute_snap_resize(
    current_rect: Rect,
    handle: ResizeHandle,
    other_rects: &[Rect],
    threshold: f32,
    grid_spacing: Option<f32>,
) -> SnapResult {
    let mut result = SnapResult::default();

    if threshold <= 0.0 {
        return result;
    }

    // Determine which X and Y edges are moving
    let moving_x = match handle {
        ResizeHandle::TopLeft | ResizeHandle::BottomLeft | ResizeHandle::LeftCenter => {
            Some(current_rect.min.x)
        }
        ResizeHandle::TopRight | ResizeHandle::BottomRight | ResizeHandle::RightCenter => {
            Some(current_rect.max.x)
        }
        ResizeHandle::TopCenter | ResizeHandle::BottomCenter => None,
    };

    let moving_y = match handle {
        ResizeHandle::TopLeft | ResizeHandle::TopRight | ResizeHandle::TopCenter => {
            Some(current_rect.min.y)
        }
        ResizeHandle::BottomLeft | ResizeHandle::BottomRight | ResizeHandle::BottomCenter => {
            Some(current_rect.max.y)
        }
        ResizeHandle::LeftCenter | ResizeHandle::RightCenter => None,
    };

    // Snap moving X edge
    if let Some(m_x) = moving_x {
        let mut best_snap_x: Option<(f32, f32, f32, Option<Rect>)> = None;

        for other in other_rects {
            for &o_x in &[other.min.x, other.center().x, other.max.x] {
                let delta = o_x - m_x;
                let dist = delta.abs();
                if dist <= threshold {
                    match best_snap_x {
                        Some((best_dist, _, _, _)) if dist < best_dist => {
                            best_snap_x = Some((dist, delta, o_x, Some(*other)));
                        }
                        None => {
                            best_snap_x = Some((dist, delta, o_x, Some(*other)));
                        }
                        _ => {}
                    }
                }
            }
        }

        if best_snap_x.is_none() {
            if let Some(spacing) = grid_spacing {
                if spacing > 0.0 {
                    let grid_x = (m_x / spacing).round() * spacing;
                    let delta = grid_x - m_x;
                    let dist = delta.abs();
                    if dist <= threshold {
                        best_snap_x = Some((dist, delta, grid_x, None));
                    }
                }
            }
        }

        if let Some((_, delta_x, target_x, target_rect_opt)) = best_snap_x {
            result.offset.x = delta_x;
            result.snapped_x = true;

            let margin = 12.0;
            let y_min;
            let y_max;
            let mut matched = vec![Pos2::new(target_x, current_rect.center().y)];

            if let Some(target_rect) = target_rect_opt {
                y_min = current_rect.min.y.min(target_rect.min.y) - margin;
                y_max = current_rect.max.y.max(target_rect.max.y) + margin;
                matched.push(Pos2::new(target_x, target_rect.center().y));
            } else {
                y_min = current_rect.min.y - margin;
                y_max = current_rect.max.y + margin;
            }

            result.guides.push(SnapGuideLine {
                start: Pos2::new(target_x, y_min),
                end: Pos2::new(target_x, y_max),
                matched_points: matched,
            });
        }
    }

    // Snap moving Y edge
    if let Some(m_y) = moving_y {
        let mut best_snap_y: Option<(f32, f32, f32, Option<Rect>)> = None;

        for other in other_rects {
            for &o_y in &[other.min.y, other.center().y, other.max.y] {
                let delta = o_y - m_y;
                let dist = delta.abs();
                if dist <= threshold {
                    match best_snap_y {
                        Some((best_dist, _, _, _)) if dist < best_dist => {
                            best_snap_y = Some((dist, delta, o_y, Some(*other)));
                        }
                        None => {
                            best_snap_y = Some((dist, delta, o_y, Some(*other)));
                        }
                        _ => {}
                    }
                }
            }
        }

        if best_snap_y.is_none() {
            if let Some(spacing) = grid_spacing {
                if spacing > 0.0 {
                    let grid_y = (m_y / spacing).round() * spacing;
                    let delta = grid_y - m_y;
                    let dist = delta.abs();
                    if dist <= threshold {
                        best_snap_y = Some((dist, delta, grid_y, None));
                    }
                }
            }
        }

        if let Some((_, delta_y, target_y, target_rect_opt)) = best_snap_y {
            result.offset.y = delta_y;
            result.snapped_y = true;

            let margin = 12.0;
            let x_min;
            let x_max;
            let mut matched = vec![Pos2::new(current_rect.center().x, target_y)];

            if let Some(target_rect) = target_rect_opt {
                x_min = current_rect.min.x.min(target_rect.min.x) - margin;
                x_max = current_rect.max.x.max(target_rect.max.x) + margin;
                matched.push(Pos2::new(target_rect.center().x, target_y));
            } else {
                x_min = current_rect.min.x - margin;
                x_max = current_rect.max.x + margin;
            }

            result.guides.push(SnapGuideLine {
                start: Pos2::new(x_min, target_y),
                end: Pos2::new(x_max, target_y),
                matched_points: matched,
            });
        }
    }

    result
}

/// Computes snapping for dragging a single point or line endpoint.
pub fn compute_snap_point(
    point: Pos2,
    other_rects: &[Rect],
    threshold: f32,
    grid_spacing: Option<f32>,
) -> SnapResult {
    let mut result = SnapResult::default();

    if threshold <= 0.0 {
        return result;
    }

    // Snap X
    let mut best_snap_x: Option<(f32, f32, f32, Option<Rect>)> = None;
    for other in other_rects {
        for &o_x in &[other.min.x, other.center().x, other.max.x] {
            let delta = o_x - point.x;
            let dist = delta.abs();
            if dist <= threshold {
                match best_snap_x {
                    Some((best_dist, _, _, _)) if dist < best_dist => {
                        best_snap_x = Some((dist, delta, o_x, Some(*other)));
                    }
                    None => {
                        best_snap_x = Some((dist, delta, o_x, Some(*other)));
                    }
                    _ => {}
                }
            }
        }
    }

    if best_snap_x.is_none() {
        if let Some(spacing) = grid_spacing {
            if spacing > 0.0 {
                let grid_x = (point.x / spacing).round() * spacing;
                let delta = grid_x - point.x;
                let dist = delta.abs();
                if dist <= threshold {
                    best_snap_x = Some((dist, delta, grid_x, None));
                }
            }
        }
    }

    if let Some((_, delta_x, target_x, target_rect_opt)) = best_snap_x {
        result.offset.x = delta_x;
        result.snapped_x = true;

        let margin = 12.0;
        let y_min;
        let y_max;
        let mut matched = vec![Pos2::new(target_x, point.y)];

        if let Some(target_rect) = target_rect_opt {
            y_min = point.y.min(target_rect.min.y) - margin;
            y_max = point.y.max(target_rect.max.y) + margin;
            matched.push(Pos2::new(target_x, target_rect.center().y));
        } else {
            y_min = point.y - margin;
            y_max = point.y + margin;
        }

        result.guides.push(SnapGuideLine {
            start: Pos2::new(target_x, y_min),
            end: Pos2::new(target_x, y_max),
            matched_points: matched,
        });
    }

    // Snap Y
    let mut best_snap_y: Option<(f32, f32, f32, Option<Rect>)> = None;
    for other in other_rects {
        for &o_y in &[other.min.y, other.center().y, other.max.y] {
            let delta = o_y - point.y;
            let dist = delta.abs();
            if dist <= threshold {
                match best_snap_y {
                    Some((best_dist, _, _, _)) if dist < best_dist => {
                        best_snap_y = Some((dist, delta, o_y, Some(*other)));
                    }
                    None => {
                        best_snap_y = Some((dist, delta, o_y, Some(*other)));
                    }
                    _ => {}
                }
            }
        }
    }

    if best_snap_y.is_none() {
        if let Some(spacing) = grid_spacing {
            if spacing > 0.0 {
                let grid_y = (point.y / spacing).round() * spacing;
                let delta = grid_y - point.y;
                let dist = delta.abs();
                if dist <= threshold {
                    best_snap_y = Some((dist, delta, grid_y, None));
                }
            }
        }
    }

    if let Some((_, delta_y, target_y, target_rect_opt)) = best_snap_y {
        result.offset.y = delta_y;
        result.snapped_y = true;

        let margin = 12.0;
        let x_min;
        let x_max;
        let mut matched = vec![Pos2::new(point.x, target_y)];

        if let Some(target_rect) = target_rect_opt {
            x_min = point.x.min(target_rect.min.x) - margin;
            x_max = point.x.max(target_rect.max.x) + margin;
            matched.push(Pos2::new(target_rect.center().x, target_y));
        } else {
            x_min = point.x - margin;
            x_max = point.x + margin;
        }

        result.guides.push(SnapGuideLine {
            start: Pos2::new(x_min, target_y),
            end: Pos2::new(x_max, target_y),
            matched_points: matched,
        });
    }

    result
}

/// Computes equal-spacing snapping along the X axis.
/// If object A and object B have a gap G, and dragged object C is placed near gap G from B, snap C.
fn compute_equal_spacing_x(dragged: Rect, others: &[Rect], threshold: f32) -> Option<SnapResult> {
    if others.len() < 2 {
        return None;
    }

    // Sort relevant others by min.x
    let mut sorted: Vec<Rect> = others.to_vec();
    sorted.sort_by(|a, b| a.min.x.partial_cmp(&b.min.x).unwrap_or(std::cmp::Ordering::Equal));

    for i in 0..sorted.len() {
        for j in (i + 1)..sorted.len() {
            let left = sorted[i];
            let right = sorted[j];
            let gap = right.min.x - left.max.x;
            if gap <= 0.0 {
                continue;
            }

            // Case 1: Dragged is to the right of `right`
            let expected_min_x = right.max.x + gap;
            let delta = expected_min_x - dragged.min.x;
            if delta.abs() <= threshold {
                let mut guides = Vec::new();
                let y = dragged.center().y;
                guides.push(SnapGuideLine {
                    start: Pos2::new(left.max.x, y),
                    end: Pos2::new(expected_min_x + dragged.width(), y),
                    matched_points: vec![
                        Pos2::new(left.max.x, y),
                        Pos2::new(right.min.x, y),
                        Pos2::new(right.max.x, y),
                        Pos2::new(expected_min_x, y),
                    ],
                });
                return Some(SnapResult {
                    offset: Vec2::new(delta, 0.0),
                    snapped_x: true,
                    snapped_y: false,
                    guides,
                });
            }

            // Case 2: Dragged is to the left of `left`
            let expected_max_x = left.min.x - gap;
            let delta = expected_max_x - dragged.max.x;
            if delta.abs() <= threshold {
                let mut guides = Vec::new();
                let y = dragged.center().y;
                guides.push(SnapGuideLine {
                    start: Pos2::new(expected_max_x - dragged.width(), y),
                    end: Pos2::new(right.min.x, y),
                    matched_points: vec![
                        Pos2::new(expected_max_x, y),
                        Pos2::new(left.min.x, y),
                        Pos2::new(left.max.x, y),
                        Pos2::new(right.min.x, y),
                    ],
                });
                return Some(SnapResult {
                    offset: Vec2::new(delta, 0.0),
                    snapped_x: true,
                    snapped_y: false,
                    guides,
                });
            }
        }
    }

    None
}

/// Computes equal-spacing snapping along the Y axis.
fn compute_equal_spacing_y(dragged: Rect, others: &[Rect], threshold: f32) -> Option<SnapResult> {
    if others.len() < 2 {
        return None;
    }

    let mut sorted: Vec<Rect> = others.to_vec();
    sorted.sort_by(|a, b| a.min.y.partial_cmp(&b.min.y).unwrap_or(std::cmp::Ordering::Equal));

    for i in 0..sorted.len() {
        for j in (i + 1)..sorted.len() {
            let top = sorted[i];
            let bottom = sorted[j];
            let gap = bottom.min.y - top.max.y;
            if gap <= 0.0 {
                continue;
            }

            // Case 1: Dragged is below `bottom`
            let expected_min_y = bottom.max.y + gap;
            let delta = expected_min_y - dragged.min.y;
            if delta.abs() <= threshold {
                let mut guides = Vec::new();
                let x = dragged.center().x;
                guides.push(SnapGuideLine {
                    start: Pos2::new(x, top.max.y),
                    end: Pos2::new(x, expected_min_y + dragged.height()),
                    matched_points: vec![
                        Pos2::new(x, top.max.y),
                        Pos2::new(x, bottom.min.y),
                        Pos2::new(x, bottom.max.y),
                        Pos2::new(x, expected_min_y),
                    ],
                });
                return Some(SnapResult {
                    offset: Vec2::new(0.0, delta),
                    snapped_x: false,
                    snapped_y: true,
                    guides,
                });
            }

            // Case 2: Dragged is above `top`
            let expected_max_y = top.min.y - gap;
            let delta = expected_max_y - dragged.max.y;
            if delta.abs() <= threshold {
                let mut guides = Vec::new();
                let x = dragged.center().x;
                guides.push(SnapGuideLine {
                    start: Pos2::new(x, expected_max_y - dragged.height()),
                    end: Pos2::new(x, bottom.min.y),
                    matched_points: vec![
                        Pos2::new(x, expected_max_y),
                        Pos2::new(x, top.min.y),
                        Pos2::new(x, top.max.y),
                        Pos2::new(x, bottom.min.y),
                    ],
                });
                return Some(SnapResult {
                    offset: Vec2::new(0.0, delta),
                    snapped_x: false,
                    snapped_y: true,
                    guides,
                });
            }
        }
    }

    None
}

/// Result of resizing a rect with snapping.
#[derive(Clone, Debug, PartialEq)]
pub struct ResizeSnapResult {
    /// The resulting bounding rect after resize and snap
    pub rect: Rect,
    /// Snap details and guide lines
    pub snap_result: SnapResult,
}

/// Unified 5-step resize and snap pipeline:
/// 1. Compute the raw new rect from the mouse position (free / unconstrained)
/// 2. Run compute_snap on the moving edges only
/// 3. Apply the snap offset
/// 4. If Lock is ON, re-apply the aspect ratio after snapping (dominant axis wins, other derived from ratio)
/// 5. Enforce minimum size and flip handling
pub fn compute_resize_with_snap(
    init_rect: Rect,
    handle: ResizeHandle,
    pointer_world: Pos2,
    start_pointer_world: Pos2,
    lock_aspect: bool,
    other_rects: &[Rect],
    threshold: f32,
    grid_spacing: Option<f32>,
) -> ResizeSnapResult {
    let delta = pointer_world - start_pointer_world;

    let min_w = init_rect.width().clamp(1.0, 20.0);
    let min_h = init_rect.height().clamp(1.0, 20.0);

    // 1. Compute raw new rect from mouse position (free / unconstrained)
    let mut raw_min_x = init_rect.min.x;
    let mut raw_max_x = init_rect.max.x;
    let mut raw_min_y = init_rect.min.y;
    let mut raw_max_y = init_rect.max.y;

    match handle {
        ResizeHandle::TopLeft | ResizeHandle::BottomLeft | ResizeHandle::LeftCenter => {
            raw_min_x = (init_rect.min.x + delta.x).min(init_rect.max.x - min_w);
        }
        ResizeHandle::TopRight | ResizeHandle::BottomRight | ResizeHandle::RightCenter => {
            raw_max_x = (init_rect.max.x + delta.x).max(init_rect.min.x + min_w);
        }
        ResizeHandle::TopCenter | ResizeHandle::BottomCenter => {}
    }

    match handle {
        ResizeHandle::TopLeft | ResizeHandle::TopRight | ResizeHandle::TopCenter => {
            raw_min_y = (init_rect.min.y + delta.y).min(init_rect.max.y - min_h);
        }
        ResizeHandle::BottomLeft | ResizeHandle::BottomRight | ResizeHandle::BottomCenter => {
            raw_max_y = (init_rect.max.y + delta.y).max(init_rect.min.y + min_h);
        }
        ResizeHandle::LeftCenter | ResizeHandle::RightCenter => {}
    }

    let raw_rect = Rect::from_min_max(
        Pos2::new(raw_min_x, raw_min_y),
        Pos2::new(raw_max_x, raw_max_y),
    );

    // 2. Run compute_snap on the moving edges only
    let snap_res = compute_snap_resize(raw_rect, handle, other_rects, threshold, grid_spacing);

    // 3. Apply the snap offset
    let mut snapped_min_x = raw_min_x;
    let mut snapped_max_x = raw_max_x;
    let mut snapped_min_y = raw_min_y;
    let mut snapped_max_y = raw_max_y;

    if snap_res.snapped_x {
        match handle {
            ResizeHandle::TopLeft | ResizeHandle::BottomLeft | ResizeHandle::LeftCenter => {
                snapped_min_x = (snapped_min_x + snap_res.offset.x).min(init_rect.max.x - min_w);
            }
            ResizeHandle::TopRight | ResizeHandle::BottomRight | ResizeHandle::RightCenter => {
                snapped_max_x = (snapped_max_x + snap_res.offset.x).max(init_rect.min.x + min_w);
            }
            _ => {}
        }
    }

    if snap_res.snapped_y {
        match handle {
            ResizeHandle::TopLeft | ResizeHandle::TopRight | ResizeHandle::TopCenter => {
                snapped_min_y = (snapped_min_y + snap_res.offset.y).min(init_rect.max.y - min_h);
            }
            ResizeHandle::BottomLeft | ResizeHandle::BottomRight | ResizeHandle::BottomCenter => {
                snapped_max_y = (snapped_max_y + snap_res.offset.y).max(init_rect.min.y + min_h);
            }
            _ => {}
        }
    }

    // 4. If Lock is ON, re-apply the aspect ratio after snapping (dominant axis wins)
    let init_w = init_rect.width().max(1.0);
    let init_h = init_rect.height().max(1.0);
    let init_aspect = (init_w / init_h).max(0.001);

    if lock_aspect {
        let is_corner = matches!(
            handle,
            ResizeHandle::TopLeft
                | ResizeHandle::TopRight
                | ResizeHandle::BottomRight
                | ResizeHandle::BottomLeft
        );

        if is_corner {
            let current_w = (snapped_max_x - snapped_min_x).max(min_w);
            let current_h = (snapped_max_y - snapped_min_y).max(min_h);
            let scale_x = current_w / init_w;
            let scale_y = current_h / init_h;

            let x_dominant = if snap_res.snapped_x && !snap_res.snapped_y {
                true
            } else if snap_res.snapped_y && !snap_res.snapped_x {
                false
            } else {
                (scale_x - 1.0).abs() >= (scale_y - 1.0).abs()
            };

            if x_dominant {
                let target_h = (current_w / init_aspect).max(min_h);
                match handle {
                    ResizeHandle::TopLeft | ResizeHandle::TopRight => {
                        snapped_min_y = snapped_max_y - target_h;
                    }
                    ResizeHandle::BottomLeft | ResizeHandle::BottomRight => {
                        snapped_max_y = snapped_min_y + target_h;
                    }
                    _ => {}
                }
            } else {
                let target_w = (current_h * init_aspect).max(min_w);
                match handle {
                    ResizeHandle::TopLeft | ResizeHandle::BottomLeft => {
                        snapped_min_x = snapped_max_x - target_w;
                    }
                    ResizeHandle::TopRight | ResizeHandle::BottomRight => {
                        snapped_max_x = snapped_min_x + target_w;
                    }
                    _ => {}
                }
            }
        }
    }

    // 5. Enforce minimum size and ensure opposite anchored edge never moves
    match handle {
        ResizeHandle::TopLeft => {
            snapped_min_x = snapped_min_x.min(init_rect.max.x - min_w);
            snapped_min_y = snapped_min_y.min(init_rect.max.y - min_h);
            snapped_max_x = init_rect.max.x;
            snapped_max_y = init_rect.max.y;
        }
        ResizeHandle::TopRight => {
            snapped_min_x = init_rect.min.x;
            snapped_max_x = snapped_max_x.max(init_rect.min.x + min_w);
            snapped_min_y = snapped_min_y.min(init_rect.max.y - min_h);
            snapped_max_y = init_rect.max.y;
        }
        ResizeHandle::BottomRight => {
            snapped_min_x = init_rect.min.x;
            snapped_max_x = snapped_max_x.max(init_rect.min.x + min_w);
            snapped_min_y = init_rect.min.y;
            snapped_max_y = snapped_max_y.max(init_rect.min.y + min_h);
        }
        ResizeHandle::BottomLeft => {
            snapped_min_x = snapped_min_x.min(init_rect.max.x - min_w);
            snapped_max_x = init_rect.max.x;
            snapped_min_y = init_rect.min.y;
            snapped_max_y = snapped_max_y.max(init_rect.min.y + min_h);
        }
        ResizeHandle::TopCenter => {
            snapped_min_x = init_rect.min.x;
            snapped_max_x = init_rect.max.x;
            snapped_min_y = snapped_min_y.min(init_rect.max.y - min_h);
            snapped_max_y = init_rect.max.y;
        }
        ResizeHandle::BottomCenter => {
            snapped_min_x = init_rect.min.x;
            snapped_max_x = init_rect.max.x;
            snapped_min_y = init_rect.min.y;
            snapped_max_y = snapped_max_y.max(init_rect.min.y + min_h);
        }
        ResizeHandle::LeftCenter => {
            snapped_min_x = snapped_min_x.min(init_rect.max.x - min_w);
            snapped_max_x = init_rect.max.x;
            snapped_min_y = init_rect.min.y;
            snapped_max_y = init_rect.max.y;
        }
        ResizeHandle::RightCenter => {
            snapped_min_x = init_rect.min.x;
            snapped_max_x = snapped_max_x.max(init_rect.min.x + min_w);
            snapped_min_y = init_rect.min.y;
            snapped_max_y = init_rect.max.y;
        }
    }

    let final_rect = Rect::from_min_max(
        Pos2::new(snapped_min_x, snapped_min_y),
        Pos2::new(snapped_max_x, snapped_max_y),
    );

    ResizeSnapResult {
        rect: final_rect,
        snap_result: snap_res,
    }
}
