use egui::{pos2, vec2, Pos2, Rect, Vec2};
use siro::canvas::snap::{
    compute_resize_with_snap, compute_snap, compute_snap_point, compute_snap_resize,
};
use siro::canvas::tools::select::ResizeHandle;
use siro::storage::settings::AppSettings;

#[test]
fn test_snap_edge_to_edge_left_and_right() {
    // Other object B at [100, 100] to [200, 200]
    let other = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));
    let others = vec![other];

    // Dragged object A near B's right edge (B.max.x = 200.0)
    // A.min.x = 204.0, within threshold 8.0
    let dragged = Rect::from_min_max(pos2(204.0, 300.0), pos2(254.0, 350.0));
    let res = compute_snap(dragged, &others, 8.0, None);

    assert!(res.snapped_x);
    // Delta should be 200.0 - 204.0 = -4.0
    assert!((res.offset.x - (-4.0)).abs() < 1e-4);
    assert!(!res.snapped_y);
    assert_eq!(res.offset.y, 0.0);
    assert_eq!(res.guides.len(), 1);
    assert_eq!(res.guides[0].start.x, 200.0);
}

#[test]
fn test_snap_center_to_center() {
    // Other object B center is (150.0, 150.0)
    let other = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));
    let others = vec![other];

    // Dragged object A center is (153.0, 500.0)
    // A center X is 3.0 units away from B center X (150.0)
    let dragged = Rect::from_center_size(pos2(153.0, 500.0), vec2(40.0, 40.0));
    let res = compute_snap(dragged, &others, 8.0, None);

    assert!(res.snapped_x);
    assert!((res.offset.x - (-3.0)).abs() < 1e-4);
    assert_eq!(res.guides[0].start.x, 150.0);
}

#[test]
fn test_snap_nothing_in_range() {
    let other = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));
    let others = vec![other];

    // Dragged is at x=250..300, y=250..300. Distance to nearest edge (200.0) is 50.0 > threshold 8.0
    let dragged = Rect::from_min_max(pos2(250.0, 250.0), pos2(300.0, 300.0));
    let res = compute_snap(dragged, &others, 8.0, None);

    assert!(!res.snapped_x);
    assert!(!res.snapped_y);
    assert_eq!(res.offset, Vec2::ZERO);
    assert!(res.guides.is_empty());
}

#[test]
fn test_snap_multiple_candidates_picks_nearest() {
    // Object 1 at x=100..200 (right edge = 200.0)
    let obj1 = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));
    // Object 2 at x=205..280 (left edge = 205.0)
    let obj2 = Rect::from_min_max(pos2(205.0, 100.0), pos2(280.0, 200.0));
    let others = vec![obj1, obj2];

    // Dragged object at x=204..250
    // Distance to obj1 right edge (200.0) is 4.0
    // Distance to obj2 left edge (205.0) is 1.0 (nearest candidate!)
    let dragged = Rect::from_min_max(pos2(204.0, 300.0), pos2(250.0, 350.0));
    let res = compute_snap(dragged, &others, 8.0, None);

    assert!(res.snapped_x);
    // Should pick 205.0, so delta is +1.0
    assert!((res.offset.x - 1.0).abs() < 1e-4);
    assert_eq!(res.guides[0].start.x, 205.0);
}

#[test]
fn test_snap_independent_axes() {
    let other = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));
    let others = vec![other];

    // X is close (min.x = 202.0 vs 200.0), but Y is 450.0 (far away)
    let dragged = Rect::from_min_max(pos2(202.0, 450.0), pos2(250.0, 500.0));
    let res = compute_snap(dragged, &others, 8.0, None);

    assert!(res.snapped_x);
    assert!(!res.snapped_y);
    assert!((res.offset.x - (-2.0)).abs() < 1e-4);
    assert_eq!(res.offset.y, 0.0);
}

#[test]
fn test_snap_zoom_invariance_screen_threshold() {
    let other = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));
    let others = vec![other];

    let screen_threshold = 8.0;

    // 1. At 25% zoom (zoom = 0.25):
    // 8 screen pixels = 8.0 / 0.25 = 32.0 world units threshold
    let zoom_25 = 0.25;
    let world_thresh_25 = screen_threshold / zoom_25;
    assert_eq!(world_thresh_25, 32.0);
    // World delta = 20.0. Screen delta = 20.0 * 0.25 = 5.0 screen px (< 8.0 screen px)
    let dragged_25 = Rect::from_min_max(pos2(220.0, 300.0), pos2(270.0, 350.0));
    let res_25 = compute_snap(dragged_25, &others, world_thresh_25, None);
    assert!(res_25.snapped_x, "Should snap at 25% zoom within 8 screen pixels");
    assert!((res_25.offset.x - (-20.0)).abs() < 1e-4);

    // 2. At 100% zoom (zoom = 1.0):
    // 8 screen pixels = 8.0 / 1.0 = 8.0 world units threshold
    let zoom_100 = 1.0;
    let world_thresh_100 = screen_threshold / zoom_100;
    assert_eq!(world_thresh_100, 8.0);
    // World delta = 20.0 is 20.0 screen px (> 8.0 screen px) -> should NOT snap
    let res_100 = compute_snap(dragged_25, &others, world_thresh_100, None);
    assert!(!res_100.snapped_x, "World delta 20 should not snap at 100% zoom with 8px threshold");

    // 3. At 400% zoom (zoom = 4.0):
    // 8 screen pixels = 8.0 / 4.0 = 2.0 world units threshold
    let zoom_400 = 4.0;
    let world_thresh_400 = screen_threshold / zoom_400;
    assert_eq!(world_thresh_400, 2.0);
    // World delta = 1.5. Screen delta = 1.5 * 4.0 = 6.0 screen px (< 8.0 screen px)
    let dragged_400 = Rect::from_min_max(pos2(201.5, 300.0), pos2(250.0, 350.0));
    let res_400 = compute_snap(dragged_400, &others, world_thresh_400, None);
    assert!(res_400.snapped_x, "Should snap at 400% zoom within 8 screen pixels");
    assert!((res_400.offset.x - (-1.5)).abs() < 1e-4);
}

#[test]
fn test_snap_resize_8_handles() {
    let other = Rect::from_min_max(pos2(300.0, 100.0), pos2(400.0, 200.0));
    let others = vec![other];

    // Dragging RightCenter handle of rect at x=100..297
    // Moving edge is max.x = 297.0, target is other.min.x = 300.0
    let current_rect = Rect::from_min_max(pos2(100.0, 100.0), pos2(297.0, 200.0));
    let res_rc = compute_snap_resize(current_rect, ResizeHandle::RightCenter, &others, 8.0, None);
    assert!(res_rc.snapped_x);
    assert!(!res_rc.snapped_y);
    assert!((res_rc.offset.x - 3.0).abs() < 1e-4);

    // Dragging BottomCenter handle: moving edge is max.y = 198.0, target is other.max.y = 200.0
    let rect_bc = Rect::from_min_max(pos2(100.0, 100.0), pos2(297.0, 198.0));
    let res_bc = compute_snap_resize(rect_bc, ResizeHandle::BottomCenter, &others, 8.0, None);
    assert!(!res_bc.snapped_x);
    assert!(res_bc.snapped_y);
    assert!((res_bc.offset.y - 2.0).abs() < 1e-4);

    // Dragging TopLeft corner handle: moving edges are min.x and min.y
    let target2 = Rect::from_min_max(pos2(50.0, 50.0), pos2(98.0, 99.0));
    let others2 = vec![target2];
    let res_tl = compute_snap_resize(current_rect, ResizeHandle::TopLeft, &others2, 8.0, None);
    assert!(res_tl.snapped_x);
    assert!(res_tl.snapped_y);
    assert!((res_tl.offset.x - (-2.0)).abs() < 1e-4); // min.x 100.0 -> 98.0
    assert!((res_tl.offset.y - (-1.0)).abs() < 1e-4); // min.y 100.0 -> 99.0
}

#[test]
fn test_snap_point_endpoints() {
    let other = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));
    let others = vec![other];

    // Dragging line endpoint near other.min.x = 100.0 and other.center.y = 150.0
    let pt = Pos2::new(102.0, 149.0);
    let res = compute_snap_point(pt, &others, 8.0, None);

    assert!(res.snapped_x);
    assert!(res.snapped_y);
    assert!((res.offset.x - (-2.0)).abs() < 1e-4);
    assert!((res.offset.y - 1.0).abs() < 1e-4);
}

#[test]
fn test_snap_to_canvas_grid_lines() {
    let others: Vec<Rect> = vec![];
    // Grid spacing = 50.0
    // Dragged object at x=98.0 (near grid line 100.0), y=148.0 (near grid line 150.0)
    let dragged = Rect::from_min_max(pos2(98.0, 148.0), pos2(148.0, 198.0));
    let res = compute_snap(dragged, &others, 8.0, Some(50.0));

    assert!(res.snapped_x);
    assert!(res.snapped_y);
    assert!((res.offset.x - 2.0).abs() < 1e-4);
    assert!((res.offset.y - 2.0).abs() < 1e-4);
}

#[test]
fn test_settings_smart_snapping_backward_compatibility() {
    // Old settings json with only autosave
    let json = r#"{"autosave_interval_secs": 60}"#;
    let settings: AppSettings = serde_json::from_str(json).unwrap();
    assert!(settings.smart_snapping_enabled, "Should default to true for backward compatibility");
}

#[test]
fn test_resize_free_right_edge_snaps_to_other_left_edge() {
    let other = Rect::from_min_max(pos2(300.0, 100.0), pos2(400.0, 200.0));
    let others = vec![other];

    // Initial rect at x=100..200, y=100..150
    let init_rect = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 150.0));

    // Drag RightCenter handle: start at 200.0, drag pointer to 298.0 (2.0 units from other.min.x = 300.0)
    let start_ptr = pos2(200.0, 125.0);
    let curr_ptr = pos2(298.0, 125.0);

    // Free resize: lock_aspect = false
    let res = compute_resize_with_snap(
        init_rect,
        ResizeHandle::RightCenter,
        curr_ptr,
        start_ptr,
        false,
        &others,
        8.0,
        None,
    );

    // Opposite (left) edge MUST NOT MOVE
    assert_eq!(res.rect.min.x, 100.0);
    assert_eq!(res.rect.min.y, 100.0);
    assert_eq!(res.rect.max.y, 150.0);

    // Moving (right) edge must snap to other.min.x (300.0)
    assert!(res.snap_result.snapped_x);
    assert!((res.rect.max.x - 300.0).abs() < 1e-4);
    assert_eq!(res.snap_result.guides.len(), 1);
    assert_eq!(res.snap_result.guides[0].start.x, 300.0);
}

#[test]
fn test_resize_corner_drag_only_x_in_range_y_stays_free() {
    let other = Rect::from_min_max(pos2(300.0, 500.0), pos2(400.0, 600.0));
    let others = vec![other];

    // Initial rect at x=100..200, y=100..200
    let init_rect = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));

    // Drag BottomRight handle:
    // X moves from 200.0 to 297.0 (delta +97.0, target 300.0 is 3.0 away <= threshold 8.0)
    // Y moves from 200.0 to 250.0 (delta +50.0, nearest Y target is 500.0 which is 250 away >> threshold 8.0)
    let start_ptr = pos2(200.0, 200.0);
    let curr_ptr = pos2(297.0, 250.0);

    // Free resize: lock_aspect = false
    let res = compute_resize_with_snap(
        init_rect,
        ResizeHandle::BottomRight,
        curr_ptr,
        start_ptr,
        false,
        &others,
        8.0,
        None,
    );

    // Opposite (top-left) anchor MUST NOT MOVE
    assert_eq!(res.rect.min.x, 100.0);
    assert_eq!(res.rect.min.y, 100.0);

    // X snaps to 300.0
    assert!(res.snap_result.snapped_x);
    assert!((res.rect.max.x - 300.0).abs() < 1e-4);

    // Y moves freely to 250.0 without snapping
    assert!(!res.snap_result.snapped_y);
    assert!((res.rect.max.y - 250.0).abs() < 1e-4);
}

#[test]
fn test_resize_nothing_in_range_free() {
    let other = Rect::from_min_max(pos2(500.0, 500.0), pos2(600.0, 600.0));
    let others = vec![other];

    let init_rect = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 200.0));
    let start_ptr = pos2(200.0, 200.0);
    let curr_ptr = pos2(230.0, 240.0);

    // Free resize: lock_aspect = false
    let res = compute_resize_with_snap(
        init_rect,
        ResizeHandle::BottomRight,
        curr_ptr,
        start_ptr,
        false,
        &others,
        8.0,
        None,
    );

    assert_eq!(res.rect.min.x, 100.0);
    assert_eq!(res.rect.min.y, 100.0);
    assert!(!res.snap_result.snapped_x);
    assert!(!res.snap_result.snapped_y);
    assert_eq!(res.snap_result.offset, Vec2::ZERO);
    assert!(res.snap_result.guides.is_empty());
    assert!((res.rect.max.x - 230.0).abs() < 1e-4);
    assert!((res.rect.max.y - 240.0).abs() < 1e-4);
}

#[test]
fn test_resize_lock_on_preserves_ratio_after_snap() {
    let other = Rect::from_min_max(pos2(300.0, 800.0), pos2(400.0, 900.0));
    let others = vec![other];

    // Initial rect: width = 100.0, height = 50.0, ratio = 2.0
    let init_rect = Rect::from_min_max(pos2(100.0, 100.0), pos2(200.0, 150.0));
    let initial_ratio = init_rect.width() / init_rect.height();
    assert_eq!(initial_ratio, 2.0);

    // Drag BottomRight handle:
    // X moves towards 297.0 (delta +97.0, within threshold 8.0 of other.min.x = 300.0)
    // Target snap will snap right edge to 300.0 -> new width = 200.0.
    let start_ptr = pos2(200.0, 150.0);
    let curr_ptr = pos2(297.0, 180.0);

    // Lock is ON: lock_aspect = true
    let res = compute_resize_with_snap(
        init_rect,
        ResizeHandle::BottomRight,
        curr_ptr,
        start_ptr,
        true,
        &others,
        8.0,
        None,
    );

    // Top-left anchor MUST NOT MOVE
    assert_eq!(res.rect.min.x, 100.0);
    assert_eq!(res.rect.min.y, 100.0);

    // X dominant axis snapped to 300.0
    assert!(res.snap_result.snapped_x);
    assert!((res.rect.max.x - 300.0).abs() < 1e-4);

    // New width = 300.0 - 100.0 = 200.0
    let new_width = res.rect.width();
    assert!((new_width - 200.0).abs() < 1e-4);

    // Ratio MUST be preserved: new_height = 200.0 / 2.0 = 100.0 -> max.y = 200.0
    let new_height = res.rect.height();
    let resulting_ratio = new_width / new_height;
    assert!(
        (resulting_ratio - initial_ratio).abs() < 1e-4,
        "Resulting aspect ratio {} must preserve initial ratio {}",
        resulting_ratio,
        initial_ratio
    );
    assert!((res.rect.max.y - 200.0).abs() < 1e-4);
}
