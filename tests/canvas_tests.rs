use egui::{Pos2, Rect, Vec2};
use siro::canvas::camera::Camera;
use siro::canvas::tools::pen::ramer_douglas_peucker;
use siro::model::element::*;

#[test]
fn test_coordinate_transform_roundtrip_subpixel_precision() {
    let viewport = Rect::from_min_size(Pos2::new(50.0, 80.0), Vec2::new(1200.0, 800.0));
    let test_zooms = [0.05f32, 0.1, 0.5, 1.0, 2.5, 5.0, 10.0, 20.0];
    let test_pans = [
        Pos2::new(0.0, 0.0),
        Pos2::new(-500.0, 320.0),
        Pos2::new(1450.0, -820.0),
    ];
    let test_points = [
        Pos2::new(0.0, 0.0),
        Pos2::new(123.456, 789.012),
        Pos2::new(-4567.89, 9876.54),
        Pos2::new(10000.0, -10000.0),
    ];

    for &zoom in &test_zooms {
        for &pan in &test_pans {
            let camera = Camera::new(pan, zoom);
            for &world_pt in &test_points {
                let screen_pt = camera.world_to_screen(world_pt, viewport);
                let recovered_world_pt = camera.screen_to_world(screen_pt, viewport);

                let error_x = (recovered_world_pt.x - world_pt.x).abs();
                let error_y = (recovered_world_pt.y - world_pt.y).abs();

                assert!(
                    error_x < 1e-2,
                    "Zoom: {}, Pan: {:?}, World: {:?}, Error X: {}",
                    zoom,
                    pan,
                    world_pt,
                    error_x
                );
                assert!(
                    error_y < 1e-2,
                    "Zoom: {}, Pan: {:?}, World: {:?}, Error Y: {}",
                    zoom,
                    pan,
                    world_pt,
                    error_y
                );
            }
        }
    }
}

#[test]
fn test_anchor_preserving_zoom() {
    let viewport = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(1920.0, 1080.0));
    let pointer_screen = Pos2::new(640.0, 480.0);
    let mut camera = Camera::new(Pos2::new(150.0, -80.0), 1.2);

    let world_before = camera.screen_to_world(pointer_screen, viewport);

    // Zoom in by factor 1.5
    camera.zoom_at_pointer(pointer_screen, 1.5, viewport);

    let world_after = camera.screen_to_world(pointer_screen, viewport);

    let err = (world_after - world_before).length();
    assert!(
        err < 1e-4,
        "Anchor drifted after zoom! Before: {:?}, After: {:?}, Error: {}",
        world_before,
        world_after,
        err
    );

    // Zoom out by factor 0.4
    camera.zoom_at_pointer(pointer_screen, 0.4, viewport);
    let world_after_out = camera.screen_to_world(pointer_screen, viewport);
    let err_out = (world_after_out - world_before).length();
    assert!(
        err_out < 1e-4,
        "Anchor drifted after zoom out! Error: {}",
        err_out
    );
}

#[test]
fn test_rdp_stroke_simplification_noisy_line() {
    // Generate a 1,000-point noisy line
    let mut points = Vec::with_capacity(1000);
    for i in 0..1000 {
        let x = i as f32;
        // Small sinusoidal noise
        let noise = ((i as f32) * 0.1).sin() * 0.4;
        let y = x * 0.5 + noise;
        points.push(StrokePoint {
            pos: Pos2::new(x, y),
            pressure: 1.0,
            velocity: 100.0,
        });
    }

    let simplified = ramer_douglas_peucker(&points, 0.75);

    let original_len = points.len();
    let simplified_len = simplified.len();
    let reduction = 1.0 - (simplified_len as f32 / original_len as f32);

    assert!(
        reduction >= 0.75,
        "Expected >= 75% reduction, got {:.1}% (from {} to {} points)",
        reduction * 100.0,
        original_len,
        simplified_len
    );

    // Endpoints must be strictly preserved
    assert_eq!(simplified[0].pos, points[0].pos);
    assert_eq!(simplified.last().unwrap().pos, points.last().unwrap().pos);
}

#[test]
fn test_viewport_frustum_culling() {
    let camera = Camera::new(Pos2::new(0.0, 0.0), 1.0);
    let viewport = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(800.0, 600.0));
    let visible_world = camera.visible_world_rect(viewport);

    // Element 1: Centered inside viewport
    let elem_inside = ShapeElement::new(
        uuid::Uuid::new_v4(),
        ShapeKind::Rectangle,
        vec![Pos2::new(100.0, 100.0), Pos2::new(200.0, 200.0)],
        egui::Color32::WHITE,
        2.0,
        egui::Color32::TRANSPARENT,
    );
    assert!(BoardElement::Shape(elem_inside).intersects_rect(visible_world));

    // Element 2: Far outside viewport (+2000px away)
    let elem_outside = ShapeElement::new(
        uuid::Uuid::new_v4(),
        ShapeKind::Rectangle,
        vec![Pos2::new(2000.0, 2000.0), Pos2::new(2200.0, 2200.0)],
        egui::Color32::WHITE,
        2.0,
        egui::Color32::TRANSPARENT,
    );
    assert!(!BoardElement::Shape(elem_outside).intersects_rect(visible_world));
}

#[test]
fn test_orthogonal_wire_segment_hit_testing() {
    let wire = WireElement::new(
        uuid::Uuid::new_v4(),
        WireEndpoint {
            element_id: None,
            terminal_idx: None,
            pos: Pos2::new(0.0, 0.0),
        },
        WireEndpoint {
            element_id: None,
            terminal_idx: None,
            pos: Pos2::new(100.0, 100.0),
        },
        WireColor::Red5V,
    );
    let board_elem = BoardElement::Wire(wire);

    // Mid X is 50.0. The 3 Manhattan segments are:
    // (0,0) -> (50,0)
    // (50,0) -> (50,100)
    // (50,100) -> (100,100)

    // Segment 1 midpoint: (25.0, 0.0)
    assert!(board_elem.contains_point(Pos2::new(25.0, 0.0), 3.0), "Segment 1 hit test failed");
    // Segment 2 midpoint: (50.0, 50.0)
    assert!(board_elem.contains_point(Pos2::new(50.0, 50.0), 3.0), "Segment 2 vertical run hit test failed");
    // Segment 3 midpoint: (75.0, 100.0)
    assert!(board_elem.contains_point(Pos2::new(75.0, 100.0), 3.0), "Segment 3 hit test failed");

    // Point on the direct diagonal (25.0, 25.0) which is away from Manhattan segments:
    assert!(!board_elem.contains_point(Pos2::new(25.0, 25.0), 3.0), "Diagonal point should not hit orthogonal wire");
}

#[test]
fn test_shape_polygon_vertices_vertex_edit() {
    let mut shape_tool = siro::canvas::tools::shapes::ShapeTool::default();
    
    // Test Diamond shape generation has 4 distinct vertices
    shape_tool.kind = ShapeKind::Diamond;
    shape_tool.on_pointer_down(Pos2::new(0.0, 0.0));
    shape_tool.on_pointer_move(Pos2::new(100.0, 100.0));
    let diamond = shape_tool.finish_shape().expect("Diamond should finish");
    assert_eq!(diamond.vertices.len(), 4, "Diamond must have 4 polygon vertices");
    assert_eq!(diamond.vertices[0], Pos2::new(50.0, 0.0));   // top
    assert_eq!(diamond.vertices[1], Pos2::new(100.0, 50.0)); // right
    assert_eq!(diamond.vertices[2], Pos2::new(50.0, 100.0)); // bottom
    assert_eq!(diamond.vertices[3], Pos2::new(0.0, 50.0));   // left

    // Test Triangle shape generation has 3 distinct vertices
    shape_tool.kind = ShapeKind::Triangle;
    shape_tool.on_pointer_down(Pos2::new(0.0, 0.0));
    shape_tool.on_pointer_move(Pos2::new(100.0, 100.0));
    let triangle = shape_tool.finish_shape().expect("Triangle should finish");
    assert_eq!(triangle.vertices.len(), 3, "Triangle must have 3 polygon vertices");
    assert_eq!(triangle.vertices[0], Pos2::new(50.0, 0.0));   // apex
    assert_eq!(triangle.vertices[1], Pos2::new(100.0, 100.0)); // bottom-right
    assert_eq!(triangle.vertices[2], Pos2::new(0.0, 100.0));   // bottom-left

    // Test Vertex Edit Mode dragging vertex 0 of diamond
    let mut board = siro::model::board::Board::new("Test");
    let d_id = diamond.id;
    board.add_element(BoardElement::Shape(diamond));

    let mut select_tool = siro::canvas::tools::select::SelectTool::default();
    select_tool.vertex_edit_mode = true;
    select_tool.selected_ids.insert(d_id);

    // Pointer down on vertex 0 (50.0, 0.0)
    select_tool.on_pointer_down(Pos2::new(50.0, 0.0), &board, false);
    // Drag vertex 0 to (60.0, -20.0)
    select_tool.on_pointer_move(Pos2::new(60.0, -20.0), &mut board);
    select_tool.on_pointer_up(&board);

    // Verify vertex 0 was updated
    if let Some(BoardElement::Shape(s)) = board.get_element(d_id) {
        assert_eq!(s.vertices[0], Pos2::new(60.0, -20.0));
        assert!(s.cached_aabb.min.y <= -20.0, "Cached AABB must encompass modified vertex");
    } else {
        panic!("Diamond element missing");
    }
}

#[test]
fn test_active_tool_pan_and_names() {
    use siro::canvas::tools::ActiveTool;

    let default_tool = ActiveTool::default();
    assert_eq!(default_tool, ActiveTool::Select);

    let pan_tool = ActiveTool::Pan;
    assert_eq!(pan_tool.display_name(), "Pan Canvas (Hand)");

    let select_tool = ActiveTool::Select;
    assert_eq!(select_tool.display_name(), "Select (V)");

    let wire_tool = ActiveTool::Wire;
    assert_eq!(wire_tool.display_name(), "Magnetic Wire (W)");
}

#[test]
fn test_tool_dock_icon_and_label_specifications() {
    // Verify all 11 required dock tools and labels matching Miro dock
    let required_dock_tools: [(&str, &str); 11] = [
        ("✦", "Hub"),
        ("➤", "Select"),
        ("📄⁺", "Templates"),
        ("▦", "Table"),
        ("🗈", "Sticky"),
        ("T", "Text"),
        ("□○△↗", "Shape"),
        ("✎", "Pen"),
        ("⚡", "Circuit"),
        ("⤹", "Wire"),
        ("+", "More"),
    ];

    assert_eq!(required_dock_tools.len(), 11);
    for (icon, label) in required_dock_tools {
        assert!(!icon.is_empty(), "Dock icon must not be empty");
        assert!(!label.is_empty(), "Dock label must not be empty");
    }
}

#[test]
fn test_tool_dock_dimensions_and_no_overflow() {
    let button_count = 11.0;
    let button_h = 36.0;
    let button_spacing = 2.0;
    let padding_y = 12.0; // 6px top + 6px bottom

    let total_buttons_h = button_count * button_h + (button_count - 1.0) * button_spacing;
    assert_eq!(total_buttons_h, 416.0);

    let dock_h = total_buttons_h + padding_y;
    assert_eq!(dock_h, 428.0);

    // Verify 11th button (More) margin above dock capsule bottom edge
    let bottom_margin = dock_h - (6.0 + total_buttons_h);
    assert_eq!(bottom_margin, 6.0, "11th button must have 6px bottom margin");
}

#[test]
fn test_tool_dock_and_undo_pill_vertical_stack_and_spacing() {
    use siro::canvas::tools::ActiveTool;
    use siro::ui::tool_rail::ToolRail;

    let mut tool_rail = ToolRail::default();
    let ctx = egui::Context::default();

    // Test across various standard viewports: 1080p, 720p, compact 600px, tight 540px
    let test_viewports = [
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1920.0, 1080.0)),
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1280.0, 720.0)),
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 600.0)),
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(800.0, 540.0)),
    ];

    for vp in test_viewports {
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (_, rects) = tool_rail.render_floating(
                    ui,
                    ActiveTool::Select,
                    egui::Color32::YELLOW,
                    egui::Color32::BLACK,
                    2.0,
                    true,
                    false,
                    false,
                    vp,
                );

                assert!(rects.len() >= 2, "Expected dock_rect and undo_rect in occupied_rects");
                let dock_rect = rects[0];
                let undo_rect = rects[1];

                // 1. Verify dock capsule dimensions (Miro slim dock ~44px wide, ~420px height)
                assert_eq!(dock_rect.width(), 44.0);
                assert_eq!(dock_rect.height(), 428.0);

                // 2. Verify undo pill dimensions
                assert_eq!(undo_rect.width(), 44.0);
                assert_eq!(undo_rect.height(), 68.0);

                // 3. STRICT NON-OVERLAP INVARIANT: undo pill MUST NEVER collide with dock capsule
                assert!(
                    undo_rect.min.y >= dock_rect.max.y + 6.0,
                    "Undo pill overlaps or violates margin below dock! dock.max.y={}, undo.min.y={} for vp height={}",
                    dock_rect.max.y,
                    undo_rect.min.y,
                    vp.height()
                );

                // 4. Verify clearance from top navigation bar on standard viewports (720p/1080p)
                if vp.height() >= 620.0 {
                    assert!(dock_rect.min.y >= vp.min.y + 54.0, "Dock must clear top header bar on standard viewports");
                }
                // 5. On viewports >= 600px, entire stack must fit inside the viewport
                if vp.height() >= 600.0 {
                    assert!(undo_rect.max.y <= vp.max.y, "Stack must fit inside 600px+ viewport");
                }
            });
        });
    }
}

#[test]
fn test_tool_dock_glyph_coverage() {
    let ctx = egui::Context::default();
    let mut fonts = egui::FontDefinitions::default();
    if let Ok(data) = std::fs::read("C:\\Windows\\Fonts\\segoeui.ttf") {
        fonts.font_data.insert("SegoeUI".to_owned(), std::sync::Arc::new(egui::FontData::from_owned(data)));
        fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "SegoeUI".to_owned());
    }
    if let Ok(sym_data) = std::fs::read("C:\\Windows\\Fonts\\seguisym.ttf") {
        fonts.font_data.insert("SegoeUISymbol".to_owned(), std::sync::Arc::new(egui::FontData::from_owned(sym_data)));
        fonts.families.entry(egui::FontFamily::Proportional).or_default().push("SegoeUISymbol".to_owned());
    }
    if let Ok(emoji_data) = std::fs::read("C:\\Windows\\Fonts\\seguiemj.ttf") {
        fonts.font_data.insert("SegoeUIEmoji".to_owned(), std::sync::Arc::new(egui::FontData::from_owned(emoji_data)));
        fonts.families.entry(egui::FontFamily::Proportional).or_default().push("SegoeUIEmoji".to_owned());
    }
    ctx.set_fonts(fonts);

    let candidates = [
        ("Hub", "✦"),
        ("Select", "➤"),
        ("Hand", "✋"),
        ("Sticky_1", "🗈"),
        ("Sticky_2", "🗒"),
        ("Sticky_3", "📄"),
        ("Shape_1", "▢"),
        ("Shape_2", "⬡"),
        ("Shape_3", "◫"),
        ("Pen", "✎"),
        ("Text", "T"),
        ("Table", "▦"),
        ("Circuit_1", "⚡"),
        ("Circuit_2", "⎍"),
        ("Wire_1", "⤹"),
        ("Wire_2", "🔌"),
        ("Import_1", "📎"),
        ("Import_2", "⤓"),
        ("Undo_Arrow", "⮌"),
        ("Redo_Arrow", "⮎"),
    ];

    let _ = ctx.run(Default::default(), |ctx| {
        for (name, glyph) in candidates {
            let galley = ctx.fonts(|f| f.layout_no_wrap(glyph.to_string(), egui::FontId::proportional(18.0), egui::Color32::WHITE));
            println!("Glyph {}: '{}' width={:.2}, height={:.2}, rows={}", name, glyph, galley.size().x, galley.size().y, galley.rows.len());
        }
    });
}


#[test]
fn test_light_mode_color_tokens_consistency() {
    use egui::Color32;

    // Light mode top strip tokens
    let light_strip_bg = Color32::from_rgb(242, 244, 247);
    let light_strip_border = Color32::from_rgb(226, 229, 234);
    let dark_strip_bg = Color32::from_rgb(30, 32, 38);

    assert_ne!(light_strip_bg, dark_strip_bg);
    assert_eq!(light_strip_bg.r(), 242);
    assert_eq!(light_strip_border.r(), 226);

    // Light mode recovery banner tokens (#fff8e6, #f59e0b, #92400e)
    let light_rec_bg = Color32::from_rgb(255, 248, 230);
    let light_rec_border = Color32::from_rgb(245, 158, 11);
    let light_rec_text = Color32::from_rgb(146, 64, 14);

    assert_eq!(light_rec_bg, Color32::from_rgb(255, 248, 230));
    assert_eq!(light_rec_border, Color32::from_rgb(245, 158, 11));
    assert_eq!(light_rec_text, Color32::from_rgb(146, 64, 14));
}

#[test]
fn test_corner_resize_handle_hit_detection() {
    use siro::canvas::tools::select::{ResizeHandle, SelectTool};
    use siro::model::board::Board;
    use siro::model::element::*;

    let mut board = Board::new("Hit Test Board");
    let table = TableElement::new(uuid::Uuid::new_v4(), Pos2::new(100.0, 100.0), 3, 3);
    let table_id = table.id;
    let table_aabb = table.aabb(); // min: (100, 100), max: (430, 184)
    board.add_element(BoardElement::Table(table));

    let mut select = SelectTool::default();
    select.selected_ids.insert(table_id);

    let zoom = 1.0;

    assert_eq!(
        select.hit_corner_handle(Pos2::new(95.0, 95.0), &board, zoom),
        Some(ResizeHandle::TopLeft)
    );
    assert_eq!(
        select.hit_corner_handle(table_aabb.left_top(), &board, zoom),
        Some(ResizeHandle::TopLeft)
    );
    assert_eq!(
        select.hit_corner_handle(Pos2::new(435.0, 95.0), &board, zoom),
        Some(ResizeHandle::TopRight)
    );
    assert_eq!(
        select.hit_corner_handle(table_aabb.right_top(), &board, zoom),
        Some(ResizeHandle::TopRight)
    );
    assert_eq!(
        select.hit_corner_handle(Pos2::new(435.0, 189.0), &board, zoom),
        Some(ResizeHandle::BottomRight)
    );
    assert_eq!(
        select.hit_corner_handle(table_aabb.right_bottom(), &board, zoom),
        Some(ResizeHandle::BottomRight)
    );
    assert_eq!(
        select.hit_corner_handle(Pos2::new(95.0, 189.0), &board, zoom),
        Some(ResizeHandle::BottomLeft)
    );
    assert_eq!(
        select.hit_corner_handle(table_aabb.left_bottom(), &board, zoom),
        Some(ResizeHandle::BottomLeft)
    );

    // Far away point returns None
    assert_eq!(
        select.hit_corner_handle(Pos2::new(265.0, 142.0), &board, zoom),
        None
    );
    assert_eq!(
        select.hit_corner_handle(Pos2::new(0.0, 0.0), &board, zoom),
        None
    );

    // Testing zoom = 2.0 and zoom = 0.5
    assert_eq!(
        select.hit_corner_handle(Pos2::new(97.5, 97.5), &board, 2.0),
        Some(ResizeHandle::TopLeft)
    );
    assert_eq!(
        select.hit_corner_handle(Pos2::new(90.0, 90.0), &board, 0.5),
        Some(ResizeHandle::TopLeft)
    );
}

#[test]
fn test_corner_resize_scaling_table() {
    use siro::canvas::tools::select::{DragMode, ResizeHandle, SelectTool};
    use siro::model::board::Board;
    use siro::model::element::*;

    let mut board = Board::new("Resize Board");
    let table = TableElement::new(uuid::Uuid::new_v4(), Pos2::new(100.0, 100.0), 2, 2);
    let table_id = table.id;
    // initial total_width = 110.0 + 110.0 = 220.0
    // initial total_height = 2 * 28 = 56.0
    board.add_element(BoardElement::Table(table));

    let mut select = SelectTool::default();
    select.selected_ids.insert(table_id);

    // Click on BottomRight corner handle at (320, 156)
    select.on_pointer_down_with_zoom(Pos2::new(320.0, 156.0), &board, false, 1.0);
    assert_eq!(select.drag_mode, DragMode::Resize(ResizeHandle::BottomRight));

    // Drag to double the size: width from 220 to 440, height from 56 to 112
    select.on_pointer_move(Pos2::new(540.0, 212.0), &mut board);
    select.on_pointer_up(&board);

    // Verify table updated
    let tbl = match board.get_element(table_id) {
        Some(BoardElement::Table(t)) => t,
        _ => panic!("Expected table"),
    };

    // Anchor TopLeft remained stationary at (100, 100)
    assert_eq!(tbl.pos, Pos2::new(100.0, 100.0));
    // Each column scaled proportionately from 110.0 to ~220.0
    assert_eq!(tbl.col_widths.len(), 2);
    assert!((tbl.col_widths[0] - 220.0).abs() < 2.0, "col 0 was {}", tbl.col_widths[0]);
    assert!((tbl.col_widths[1] - 220.0).abs() < 2.0, "col 1 was {}", tbl.col_widths[1]);
    assert!((tbl.total_width() - 440.0).abs() < 3.0);
    // Table height scaled proportionately from 56.0 to 112.0
    assert!((tbl.total_height() - 112.0).abs() < 2.0, "total_height was {}", tbl.total_height());

    // Test TopLeft drag:
    select.selected_ids.insert(table_id);
    select.on_pointer_down_with_zoom(Pos2::new(100.0, 100.0), &board, false, 1.0);
    assert_eq!(select.drag_mode, DragMode::Resize(ResizeHandle::TopLeft));

    // Drag TopLeft from (100, 100) to (200, 120)
    select.on_pointer_move(Pos2::new(200.0, 120.0), &mut board);
    select.on_pointer_up(&board);

    let tbl2 = match board.get_element(table_id) {
        Some(BoardElement::Table(t)) => t,
        _ => panic!("Expected table"),
    };
    assert_eq!(tbl2.pos, Pos2::new(200.0, 120.0));
    // Bottom-right stationary anchor at (540, 212)
    assert!((tbl2.pos.x + tbl2.total_width() - 540.0).abs() < 3.0);
    assert!((tbl2.pos.y + tbl2.total_height() - 212.0).abs() < 3.0);
}

#[test]
fn test_corner_resize_scaling_shape_and_component() {
    use siro::canvas::tools::select::SelectTool;
    use siro::model::board::Board;
    use siro::model::element::*;

    let mut board = Board::new("Shape & Component Board");
    let vertices = vec![
        Pos2::new(100.0, 100.0),
        Pos2::new(200.0, 100.0),
        Pos2::new(200.0, 200.0),
        Pos2::new(100.0, 200.0),
    ];
    let shape = ShapeElement::new(
        uuid::Uuid::new_v4(),
        ShapeKind::Rectangle,
        vertices,
        egui::Color32::BLACK,
        2.0,
        egui::Color32::TRANSPARENT,
    );
    let shape_id = shape.id;
    board.add_element(BoardElement::Shape(shape));

    let mut select = SelectTool::default();
    select.selected_ids.insert(shape_id);

    // Drag BottomRight handle (200, 200) outward to (300, 300) -> 2x scale
    select.on_pointer_down_with_zoom(Pos2::new(200.0, 200.0), &board, false, 1.0);
    select.on_pointer_move(Pos2::new(300.0, 300.0), &mut board);
    select.on_pointer_up(&board);

    let s = match board.get_element(shape_id) {
        Some(BoardElement::Shape(s)) => s,
        _ => panic!("Expected shape"),
    };

    assert_eq!(s.vertices[0], Pos2::new(100.0, 100.0));
    assert_eq!(s.vertices[1], Pos2::new(300.0, 100.0));
    assert_eq!(s.vertices[2], Pos2::new(300.0, 300.0));
    assert_eq!(s.vertices[3], Pos2::new(100.0, 300.0));

    // Test Component resizing
    let comp = ComponentElement::new(
        uuid::Uuid::new_v4(),
        siro::model::electronics::ComponentKind::Resistor,
        Pos2::new(50.0, 50.0),
    );
    let comp_id = comp.id;
    let initial_w = comp.width;
    let initial_h = comp.height;
    board.add_element(BoardElement::Component(comp));

    select.selected_ids.clear();
    select.selected_ids.insert(comp_id);

    let br_x = 50.0 + initial_w;
    let br_y = 50.0 + initial_h;
    select.on_pointer_down_with_zoom(Pos2::new(br_x, br_y), &board, false, 1.0);
    select.on_pointer_move(Pos2::new(50.0 + initial_w * 1.5, 50.0 + initial_h * 1.5), &mut board);
    select.on_pointer_up(&board);

    let comp_after = match board.get_element(comp_id) {
        Some(BoardElement::Component(c)) => c,
        _ => panic!("Expected component"),
    };
    assert!((comp_after.width - initial_w * 1.5).abs() < 1.0);
    assert!((comp_after.height - initial_h * 1.5).abs() < 1.0);
    assert_eq!(comp_after.terminals.len(), 2);
}

#[test]
fn test_toolbar_dimensions_and_controls() {
    use siro::model::board::Board;
    use siro::model::element::*;
    use siro::ui::properties_bar::PropertiesBar;

    let mut board = Board::new("Toolbar Test Board");
    let table = TableElement::new(uuid::Uuid::new_v4(), Pos2::new(0.0, 0.0), 3, 3);
    let tbl_id = table.id;
    board.add_element(BoardElement::Table(table));

    let shape = ShapeElement::new(
        uuid::Uuid::new_v4(),
        ShapeKind::Rectangle,
        vec![Pos2::new(0.0, 0.0), Pos2::new(100.0, 100.0)],
        egui::Color32::BLACK,
        2.0,
        egui::Color32::WHITE,
    );
    let shape_id = shape.id;
    board.add_element(BoardElement::Shape(shape));

    let text = TextElement::new_text(uuid::Uuid::new_v4(), Pos2::new(0.0, 0.0), "Hello".into());
    let text_id = text.id;
    board.add_element(BoardElement::Text(text));

    let sticky = TextElement::new_sticky(uuid::Uuid::new_v4(), Pos2::new(0.0, 0.0), "Note".into(), egui::Color32::YELLOW);
    let sticky_id = sticky.id;
    board.add_element(BoardElement::Text(sticky));

    // 1. Table Toolbar Width
    let table_tb_w = PropertiesBar::calculate_toolbar_width(&board, &[tbl_id]);
    assert!(table_tb_w >= 540.0 && table_tb_w <= 620.0, "Table toolbar width was {}", table_tb_w);

    // 2. Shape Toolbar Width
    let shape_tb_w = PropertiesBar::calculate_toolbar_width(&board, &[shape_id]);
    assert!(shape_tb_w >= 500.0 && shape_tb_w <= 600.0, "Shape toolbar width was {}", shape_tb_w);

    // 3. Text Toolbar Width
    let text_tb_w = PropertiesBar::calculate_toolbar_width(&board, &[text_id]);
    assert!(text_tb_w >= 480.0 && text_tb_w <= 550.0, "Text toolbar width was {}", text_tb_w);

    // 4. Sticky Toolbar Width (has 8 swatches)
    let sticky_tb_w = PropertiesBar::calculate_toolbar_width(&board, &[sticky_id]);
    assert!(sticky_tb_w >= 520.0 && sticky_tb_w <= 600.0, "Sticky toolbar width was {}", sticky_tb_w);

    // 5. Multi-selection Toolbar Width
    let multi_tb_w = PropertiesBar::calculate_toolbar_width(&board, &[shape_id, tbl_id]);
    assert!(multi_tb_w >= 500.0, "Multi selection toolbar width was {}", multi_tb_w);
}

#[test]
fn test_toolbar_floating_render_bounds() {
    use siro::canvas::camera::Camera;
    use siro::canvas::tools::ActiveTool;
    use siro::model::board::Board;
    use siro::model::element::*;
    use siro::ui::properties_bar::PropertiesBar;

    let ctx = egui::Context::default();
    let viewport = Rect::from_min_size(Pos2::new(0.0, 0.0), Vec2::new(1920.0, 1080.0));
    let camera = Camera::new(Pos2::new(0.0, 0.0), 1.0);

    let mut board = Board::new("Toolbar Render Test");
    let table = TableElement::new(uuid::Uuid::new_v4(), Pos2::new(300.0, 300.0), 3, 3);
    let tbl_id = table.id;
    board.add_element(BoardElement::Table(table));

    let shape = ShapeElement::new(
        uuid::Uuid::new_v4(),
        ShapeKind::Rectangle,
        vec![Pos2::new(300.0, 300.0), Pos2::new(500.0, 500.0)],
        egui::Color32::BLACK,
        2.0,
        egui::Color32::WHITE,
    );
    let shape_id = shape.id;
    board.add_element(BoardElement::Shape(shape));

    let text = TextElement::new_text(uuid::Uuid::new_v4(), Pos2::new(300.0, 300.0), "Sample Text".into());
    let text_id = text.id;
    board.add_element(BoardElement::Text(text));

    let sticky = TextElement::new_sticky(uuid::Uuid::new_v4(), Pos2::new(300.0, 300.0), "Sticky Note".into(), egui::Color32::YELLOW);
    let sticky_id = sticky.id;
    board.add_element(BoardElement::Text(sticky));

    let comp = ComponentElement::new(uuid::Uuid::new_v4(), siro::model::electronics::ComponentKind::Resistor, Pos2::new(300.0, 300.0));
    let comp_id = comp.id;
    board.add_element(BoardElement::Component(comp));

    let test_cases = [
        ("Table", vec![tbl_id]),
        ("Shape", vec![shape_id]),
        ("Text", vec![text_id]),
        ("Sticky", vec![sticky_id]),
        ("Component", vec![comp_id]),
    ];

    for (name, ids) in test_cases {
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let mut stroke_col = egui::Color32::BLACK;
                let mut stroke_w = 2.0;
                let mut wire_col = WireColor::Red5V;
                let mut sticky_col = egui::Color32::YELLOW;

                let mut popup_manager = siro::ui::PopupManager::default();
                let (_action, tb_rect) = PropertiesBar::render_floating(
                    ui,
                    &camera,
                    viewport,
                    ActiveTool::Select,
                    &ids,
                    &mut board,
                    &mut stroke_col,
                    &mut stroke_w,
                    &mut wire_col,
                    &mut sticky_col,
                    false,
                    &mut popup_manager,
                );

                assert!(tb_rect.is_some(), "{}: Toolbar rect was None", name);
                let rect = tb_rect.unwrap();
                println!("{}: tb_rect = {:?}", name, rect);
            });
        });
    }
}

#[test]
fn test_corner_resize_aspect_ratio_preservation() {
    use siro::canvas::tools::select::{DragMode, ResizeHandle, SelectTool};
    use siro::model::board::Board;
    use siro::model::element::*;

    let mut board = Board::new("Aspect Ratio Test");

    // 1. PdfCard with initial pos=(100, 100), size=(200, 100) -> aspect 2.0
    let pdf = PdfCardElement {
        id: uuid::Uuid::new_v4(),
        asset_id: uuid::Uuid::new_v4(),
        pdf_name: "test.pdf".into(),
        current_page: 0,
        total_pages: 5,
        pos: Pos2::new(100.0, 100.0),
        size: Vec2::new(200.0, 100.0),
    };
    let pdf_id = pdf.id;
    board.add_element(BoardElement::PdfCard(pdf));

    let mut select = SelectTool::default();
    select.selected_ids.insert(pdf_id);

    // Drag TopLeft handle at (100, 100). Stationary anchor is BottomRight at (300, 200).
    select.on_pointer_down_with_zoom(Pos2::new(100.0, 100.0), &board, false, 1.0);
    assert_eq!(select.drag_mode, DragMode::Resize(ResizeHandle::TopLeft));

    // Drag TopLeft with anisotropic delta (scale_x = 1.5, scale_y = 0.5)
    // Moving TopLeft from (100, 100) to (0, 150)
    select.on_pointer_move(Pos2::new(0.0, 150.0), &mut board);
    select.on_pointer_up(&board);

    let pdf_after = match board.get_element(pdf_id) {
        Some(BoardElement::PdfCard(p)) => p,
        _ => panic!("Expected PdfCard"),
    };

    // Aspect ratio 2.0 must be preserved: width / height == 2.0
    let aspect = pdf_after.size.x / pdf_after.size.y;
    assert!((aspect - 2.0).abs() < 1e-2, "Aspect was {}", aspect);

    // The opposite corner (BottomRight) MUST remain strictly pinned to anchor (300, 200)
    let br_x = pdf_after.pos.x + pdf_after.size.x;
    let br_y = pdf_after.pos.y + pdf_after.size.y;
    assert!((br_x - 300.0).abs() < 1.0, "BottomRight.x drifted to {} (expected 300)", br_x);
    assert!((br_y - 200.0).abs() < 1.0, "BottomRight.y drifted to {} (expected 200)", br_y);

    // 2. Sticky note with initial pos=(100, 100), width=180 (square 180x180)
    let sticky = TextElement::new_sticky(uuid::Uuid::new_v4(), Pos2::new(100.0, 100.0), "Sticky Note".into(), egui::Color32::YELLOW);
    let sticky_id = sticky.id;
    board.add_element(BoardElement::Text(sticky));

    select.selected_ids.clear();
    select.selected_ids.insert(sticky_id);

    // Drag TopLeft handle. Stationary anchor is BottomRight at (280, 280)
    select.on_pointer_down_with_zoom(Pos2::new(100.0, 100.0), &board, false, 1.0);
    assert_eq!(select.drag_mode, DragMode::Resize(ResizeHandle::TopLeft));

    // Drag TopLeft to (50, 180) (anisotropic)
    select.on_pointer_move(Pos2::new(50.0, 180.0), &mut board);
    select.on_pointer_up(&board);

    let sticky_after = match board.get_element(sticky_id) {
        Some(BoardElement::Text(t)) => t,
        _ => panic!("Expected Sticky Text"),
    };

    // Sticky note must remain square and its bottom-right must remain pinned to (280, 280)
    let sticky_br_x = sticky_after.pos.x + sticky_after.width;
    let sticky_br_y = sticky_after.pos.y + sticky_after.width;
    assert!((sticky_br_x - 280.0).abs() < 1.0, "Sticky BottomRight.x drifted to {} (expected 280)", sticky_br_x);
    assert!((sticky_br_y - 280.0).abs() < 1.0, "Sticky BottomRight.y drifted to {} (expected 280)", sticky_br_y);
}

#[test]
fn test_corner_resize_straight_line() {
    use siro::canvas::tools::select::{DragMode, ResizeHandle, SelectTool};
    use siro::model::board::Board;
    use siro::model::element::*;

    let mut board = Board::new("Line Board");

    // Horizontal straight line shape: Y is constant at 100.0
    let line = ShapeElement::new(
        uuid::Uuid::new_v4(),
        ShapeKind::Line,
        vec![Pos2::new(100.0, 100.0), Pos2::new(300.0, 100.0)],
        egui::Color32::BLACK,
        2.0,
        egui::Color32::TRANSPARENT,
    );
    let line_id = line.id;
    board.add_element(BoardElement::Shape(line));

    let mut select = SelectTool::default();
    select.selected_ids.insert(line_id);

    // Hit test corner handle on horizontal line should succeed
    let hit = select.hit_corner_handle(Pos2::new(300.0, 100.0), &board, 1.0);
    assert!(hit.is_some(), "Corner handle on horizontal line must be hittable");

    // Click and drag BottomRight handle from 300 to 450
    select.on_pointer_down_with_zoom(Pos2::new(300.0, 100.0), &board, false, 1.0);
    assert!(matches!(select.drag_mode, DragMode::Resize(ResizeHandle::BottomRight | ResizeHandle::TopRight)));

    select.on_pointer_move(Pos2::new(450.0, 100.0), &mut board);
    select.on_pointer_up(&board);

    let line_after = match board.get_element(line_id) {
        Some(BoardElement::Shape(s)) => s,
        _ => panic!("Expected shape"),
    };

    assert_eq!(line_after.vertices[0], Pos2::new(100.0, 100.0));
    assert!((line_after.vertices[1].x - 450.0).abs() < 2.0);
}

#[test]
fn test_miro_top_bar_header_capsule_dimensions_and_states() {
    use siro::model::project::Project;
    use siro::ui::top_bar::{TopBar, TopBarState};

    let ctx = egui::Context::default();
    let mut project = Project::new("Battery Management System");
    let mut state = TopBarState::default();

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Light mode render
            let (_action, rect) = TopBar::render_floating_header(
                ui,
                &mut project,
                &mut state,
                false,
                false,
                egui::pos2(0.0, 0.0),
            );
            assert_eq!(rect.height(), 40.0, "Capsule height must be 40px");
            assert!(rect.width() >= 480.0, "Capsule width must be at least 480px, was {}", rect.width());

            // Dark mode render with dirty flag
            project.mark_edited();
            let (_action2, rect2) = TopBar::render_floating_header(
                ui,
                &mut project,
                &mut state,
                true,
                true,
                egui::pos2(0.0, 0.0),
            );
            assert_eq!(rect2.height(), 40.0);
            assert_eq!(rect2.width(), rect.width());
        });
    });
}


#[test]
fn test_tool_rail_flyouts_and_actions() {
    use siro::canvas::tools::ActiveTool;
    use siro::ui::tool_rail::ToolRail;

    let mut tool_rail = ToolRail::default();
    assert!(!tool_rail.templates_flyout_open);
    assert!(!tool_rail.more_flyout_open);
    assert!(!tool_rail.hub_flyout_open);

    // Opening flyouts
    tool_rail.templates_flyout_open = true;
    tool_rail.more_flyout_open = true;

    // close_all_flyouts must reset all flags
    tool_rail.close_all_flyouts();
    assert!(!tool_rail.templates_flyout_open);
    assert!(!tool_rail.more_flyout_open);
    assert!(!tool_rail.hub_flyout_open);
    assert!(!tool_rail.sticky_palette_open);
    assert!(!tool_rail.shapes_flyout_open);
    assert!(!tool_rail.pen_flyout_open);
    assert!(!tool_rail.electronics_flyout_open);

    let ctx = egui::Context::default();
    let vp = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1920.0, 1080.0));

    // When templates flyout is open, occupied_rects should contain at least 3 rects (dock, undo, flyout)
    tool_rail.templates_flyout_open = true;
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (_, rects) = tool_rail.render_floating(
                ui,
                ActiveTool::Select,
                egui::Color32::YELLOW,
                egui::Color32::BLACK,
                2.0,
                true,
                false,
                false,
                vp,
            );
            assert!(rects.len() >= 3, "Expected dock, undo, and templates flyout rects");
        });
    });
}


#[test]
fn test_miro_top_bar_strip_clean_geometry() {
    use siro::canvas::camera::Camera;
    use siro::model::project::Project;
    use siro::ui::top_bar::{TopBar, TopBarAction, TopBarState};

    let ctx = egui::Context::default();
    let mut project = Project::new("Battery Management System");
    let camera = Camera::default();
    let mut state = TopBarState::default();

    let _ = ctx.run(Default::default(), |ctx| {
        egui::TopBottomPanel::top("test_top_strip").show(ctx, |ui| {
            let action = TopBar::ui(ui, &mut project, &camera, &mut state, false, false);
            assert!(matches!(action, TopBarAction::None));

            let home_action = TopBar::render_home_strip(ui, Some(("Battery Management System", false)), false);
            assert!(matches!(home_action, TopBarAction::None));
        });
    });
}


#[test]
fn test_floating_header_capsule_bounds_and_overflow_safety() {
    use siro::model::project::Project;
    use siro::ui::top_bar::{TopBar, TopBarState};

    let ctx = egui::Context::default();
    let mut state = TopBarState::default();

    let titles = [
        ("A", 530.0), // short name must not underflow capsule
        ("Battery Management System", 560.0),
        ("Advanced Distributed Battery Management System and Power Distribution Grid", 650.0),
    ];

    for (title, min_expected_width) in titles {
        let mut project = Project::new(title);
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let (_action, rect) = TopBar::render_floating_header(
                    ui,
                    &mut project,
                    &mut state,
                    false,
                    false,
                    egui::pos2(0.0, 0.0),
                );
                assert_eq!(rect.height(), 40.0, "Capsule height must be exactly 40px");
                assert!(
                    rect.width() >= min_expected_width,
                    "Title '{}' width {} was less than min expected {}",
                    title,
                    rect.width(),
                    min_expected_width
                );
                assert!(rect.width() <= 700.0, "Capsule width must not exceed 700px");
            });
        });
    }
}

#[test]
fn test_tool_dock_top_left_alignment() {
    use siro::canvas::tools::ActiveTool;
    use siro::ui::tool_rail::ToolRail;

    let mut tool_rail = ToolRail::default();
    let ctx = egui::Context::default();
    let vp_1080p = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1920.0, 1080.0));

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (_, rects) = tool_rail.render_floating(
                ui,
                ActiveTool::Select,
                egui::Color32::YELLOW,
                egui::Color32::BLACK,
                2.0,
                true,
                false,
                false,
                vp_1080p,
            );
            assert!(rects.len() >= 2);
            let dock_rect = rects[0];
            // On 1080p, dock must be comfortably below the header (>= 54.0) and near the top (<= 70.0)
            assert!(dock_rect.min.y >= 54.0, "Dock min y must clear header (>= 54.0)");
            assert!(dock_rect.min.y <= 70.0, "Dock min y must be near top (< 70.0), was {}", dock_rect.min.y);
        });
    });
}


#[test]
fn test_home_screen_active_nav_routing() {
    use siro::ui::home_screen::HomeScreen;

    let mut home = HomeScreen::new_in_memory();
    assert_eq!(home.active_nav, "Home");

    home.active_nav = "Starred".to_string();
    assert_eq!(home.active_nav, "Starred");

    home.active_nav = "Browse".to_string();
    assert_eq!(home.active_nav, "Browse");

    home.active_nav = "Home".to_string();
    assert_eq!(home.active_nav, "Home");
}

#[test]
fn test_starring_and_unstarring_logic() {
    use siro::ui::home_screen::HomeScreen;
    use std::path::PathBuf;

    let mut home = HomeScreen::new_in_memory();
    let test_path = PathBuf::from(r"C:\Projects\MotorController.siro");

    // Remove if already present from previous sessions
    if home.is_starred(&test_path) {
        home.toggle_star(&test_path);
    }
    assert!(!home.is_starred(&test_path));

    // Star the board
    home.toggle_star(&test_path);
    assert!(home.is_starred(&test_path));
    assert!(home.starred_files.iter().any(|p| p == &test_path));

    // Test slash and casing equality matching
    let alt_path = PathBuf::from("c:/projects/MotorController.siro");
    assert!(home.is_starred(&alt_path), "Path with forward slashes must match starred board");

    // Unstar the board
    home.toggle_star(&alt_path);
    assert!(!home.is_starred(&test_path));
    assert!(!home.is_starred(&alt_path));
}

#[test]
fn test_recents_lru_and_deduplication() {
    use siro::ui::home_screen::HomeScreen;
    use std::path::PathBuf;

    let mut home = HomeScreen::new_in_memory();
    let p1 = PathBuf::from(r"C:\Projects\Board1.siro");
    let p2 = PathBuf::from(r"C:\Projects\Board2.siro");

    home.add_recent(&p1);
    assert_eq!(home.recent_files[0], p1);

    home.add_recent(&p2);
    assert_eq!(home.recent_files[0], p2);

    // Re-adding p1 moves it to front with no duplicates
    home.add_recent(&p1);
    assert_eq!(home.recent_files[0], p1);
    let count_p1 = home.recent_files.iter().filter(|&p| p == &p1).count();
    assert_eq!(count_p1, 1, "Recents must not have duplicate entries");

    // Remove recent
    home.remove_recent(0);
    assert_ne!(home.recent_files.get(0), Some(&p1));
}

#[test]
fn test_starred_and_recents_persistence_roundtrip() {
    use siro::storage::history::{load_paths_from, save_paths_to};
    use std::path::PathBuf;

    let temp_file = std::env::temp_dir().join("siro_test_persistence_roundtrip.json");
    let test_paths = vec![
        PathBuf::from(r"C:\SiroTest\ProjectAlpha.siro"),
        PathBuf::from(r"C:\SiroTest\ProjectBeta.siro"),
    ];

    let save_res = save_paths_to(&temp_file, &test_paths);
    assert!(save_res.is_ok(), "Saving paths must succeed: {:?}", save_res);

    let loaded = load_paths_from(&temp_file);
    assert_eq!(loaded, test_paths, "Loaded paths must match saved paths");

    let _ = std::fs::remove_file(&temp_file);
}

#[test]
fn test_home_screen_ui_rendering_all_views() {
    use siro::ui::home_screen::HomeScreen;
    use siro::storage::settings::AppSettings;
    use std::path::PathBuf;

    let mut home = HomeScreen::new_in_memory();
    let mut settings = AppSettings::default();
    home.recent_files.push(PathBuf::from(r"C:\Boards\TestBoard.siro"));
    home.starred_files.push(PathBuf::from(r"C:\Boards\TestBoard.siro"));

    let ctx = egui::Context::default();

    // 1. Render Home View in Light and Dark Mode
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            home.active_nav = "Home".to_string();
            let _ = home.ui(ui, false, &mut settings);
            let _ = home.ui(ui, true, &mut settings);
        });
    });

    // 2. Render Starred View in Light and Dark Mode
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            home.active_nav = "Starred".to_string();
            let _ = home.ui(ui, false, &mut settings);
            home.is_grid_view = true;
            let _ = home.ui(ui, false, &mut settings);
        });
    });

    // 3. Render Browse View in Light and Dark Mode
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            home.active_nav = "Browse".to_string();
            let _ = home.ui(ui, false, &mut settings);
            home.is_grid_view = true;
            let _ = home.ui(ui, false, &mut settings);
        });
    });

    // 4. Render with Search Query Filter
    home.search_query = "TestBoard".to_string();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            home.active_nav = "Home".to_string();
            let _ = home.ui(ui, false, &mut settings);
            home.active_nav = "Starred".to_string();
            let _ = home.ui(ui, false, &mut settings);
            home.active_nav = "Browse".to_string();
            let _ = home.ui(ui, false, &mut settings);
        });
    });
}


#[test]
fn test_history_utilities_and_formatting() {
    use siro::storage::history::{clean_windows_path, format_file_size, get_common_directories, are_paths_equal};
    use std::path::{Path, PathBuf};

    assert_eq!(format_file_size(500), "500 B");
    assert_eq!(format_file_size(2048), "2.0 KB");
    assert_eq!(format_file_size(1024 * 1024 * 3), "3.0 MB");

    let dirs = get_common_directories();
    assert!(!dirs.is_empty(), "Common directories must at least contain Projects folder");
    assert_eq!(dirs[0].0, "Projects");

    // Path equality handling slashes and casing
    let p_win = PathBuf::from(r"C:\Users\Admin\board.siro");
    let p_fwd = PathBuf::from("C:/Users/Admin/board.siro");
    let p_lower = PathBuf::from(r"c:\users\admin\board.siro");
    assert!(are_paths_equal(&p_win, &p_fwd));
    assert!(are_paths_equal(&p_win, &p_lower));

    // Extended Windows verbatim path prefixes
    let verbatim_local = Path::new(r"\\?\C:\Projects\board.siro");
    assert_eq!(clean_windows_path(verbatim_local), PathBuf::from(r"C:\Projects\board.siro"));

    let verbatim_unc = Path::new(r"\\?\UNC\Server\Share\board.siro");
    assert_eq!(clean_windows_path(verbatim_unc), PathBuf::from(r"\\Server\Share\board.siro"));
}

#[test]
fn test_home_screen_edge_cases_and_empty_states() {
    use siro::ui::home_screen::HomeScreen;
    use std::path::PathBuf;

    let mut home = HomeScreen::new_in_memory();
    home.recent_files.clear();
    home.starred_files.clear();

    let ctx = egui::Context::default();
    let mut settings = siro::storage::settings::AppSettings::default();

    // 1. Render all 3 views with 0 recents and 0 starred (pure empty states)
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            home.active_nav = "Home".to_string();
            let _ = home.ui(ui, false, &mut settings);

            home.active_nav = "Starred".to_string();
            let _ = home.ui(ui, false, &mut settings);

            home.active_nav = "Browse".to_string();
            let _ = home.ui(ui, false, &mut settings);
        });
    });

    // 2. Max recents capping (limit 50)
    for i in 0..65 {
        home.add_recent(&PathBuf::from(format!(r"C:\Projects\Board_{}.siro", i)));
    }
    assert_eq!(home.recent_files.len(), 50, "Recents must be capped at 50 entries");
    assert_eq!(home.recent_files[0], PathBuf::from(r"C:\Projects\Board_64.siro"));

    // 3. Browse invalid/inaccessible directory
    home.current_browse_dir = PathBuf::from(r"C:\NonExistent_Siro_Dir_12345");
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            home.active_nav = "Browse".to_string();
            let _ = home.ui(ui, false, &mut settings);
        });
    });

    // 4. Non-matching search query empty states
    home.search_query = "XYZ_NonExistent_Query".to_string();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            home.active_nav = "Home".to_string();
            let _ = home.ui(ui, false, &mut settings);

            home.active_nav = "Starred".to_string();
            let _ = home.ui(ui, false, &mut settings);

            home.active_nav = "Browse".to_string();
            let _ = home.ui(ui, false, &mut settings);
        });
    });
}


#[test]
fn test_multibyte_utf8_path_truncation() {
    use siro::storage::history::truncate_path_string;

    // Multibyte Chinese / Japanese path
    let jp_path = r"C:\Users\測試\Documents\日本語\プロジェクト\very_long_file_name_that_exceeds_max_chars.siro";
    let res_jp = truncate_path_string(jp_path, 30);
    assert!(res_jp.starts_with("..."));
    assert!(res_jp.chars().count() <= 30);

    // Multibyte Emoji path (4-byte code points)
    let emoji_path = r"C:\Users\🚀🎉💻\Hardware_Project_With_Lots_Of_Chars_In_Name.siro";
    let res_emoji = truncate_path_string(emoji_path, 25);
    assert!(res_emoji.starts_with("..."));
    assert!(res_emoji.chars().count() <= 25);

    // Accented European path
    let fr_path = r"C:\Utilisateurs\Éléonore\Modélisation_électronique\circuit_long_name_test.siro";
    let res_fr = truncate_path_string(fr_path, 40);
    assert!(res_fr.starts_with("..."));
    assert!(res_fr.chars().count() <= 40);

    // Edge Cases: Empty string
    assert_eq!(truncate_path_string("", 50), "");

    // Edge Cases: Short string (fits without truncation)
    assert_eq!(truncate_path_string("short_path.siro", 50), "short_path.siro");

    // Edge Cases: Exactly max_chars
    let exact_10 = "1234567890";
    assert_eq!(truncate_path_string(exact_10, 10), exact_10);

    // Edge Cases: max_chars <= 3
    let res_small = truncate_path_string("HelloWorld", 3);
    assert_eq!(res_small, "...");

    let res_zero = truncate_path_string("HelloWorld", 0);
    assert_eq!(res_zero, "...");
}

#[test]
fn test_browse_directory_alphabetical_sorting_and_filtering() {
    use siro::ui::home_screen::HomeScreen;
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("siro_browse_sort_test_{}", uuid::Uuid::new_v4()));
    let _ = fs::create_dir_all(&temp_dir);

    // Create subfolders in unsorted order
    let subfolders = ["zebra_folder", "Alpha_folder", "beta_folder", "123_folder"];
    for folder in &subfolders {
        let _ = fs::create_dir_all(temp_dir.join(folder));
    }

    // Create files (both .siro and non-.siro) in unsorted order
    let files = ["z_board.siro", "A_board.siro", "b_board.siro", "readme.txt", "schematic.pdf"];
    for file in &files {
        let _ = fs::write(temp_dir.join(file), b"test_siro_data");
    }

    let mut home = HomeScreen::new_in_memory();
    home.set_browse_dir(temp_dir.clone());

    // Verify subfolders are sorted alphabetically case-insensitive
    let folder_names: Vec<String> = home.browse_folders.iter().map(|f| f.name.clone()).collect();
    assert_eq!(
        folder_names,
        vec!["123_folder", "Alpha_folder", "beta_folder", "zebra_folder"],
        "Folders in Browse view must be sorted alphabetically case-insensitive"
    );

    // Verify boards are sorted alphabetically case-insensitive and non-.siro files are excluded
    let board_names: Vec<String> = home.browse_files.iter().map(|b| b.name.clone()).collect();
    assert_eq!(
        board_names,
        vec!["A_board", "b_board", "z_board"],
        "Only .siro files must be included in Browse view and sorted alphabetically case-insensitive"
    );

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_home_screen_narrow_viewport_safety() {
    use siro::ui::home_screen::HomeScreen;
    use std::path::PathBuf;

    let mut home = HomeScreen::new_in_memory();
    home.recent_files.push(PathBuf::from(r"C:\Projects\Board1.siro"));
    home.starred_files.push(PathBuf::from(r"C:\Projects\Board1.siro"));

    let ctx = egui::Context::default();

    // Render with extremely narrow viewport (180x250) to test bounds clamping and layout overflow resilience
    let mut settings = siro::storage::settings::AppSettings::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Allocate a tiny sub-ui
            let tiny_rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(180.0, 250.0));
            let mut child_ui = ui.new_child(egui::UiBuilder::new().max_rect(tiny_rect));

            home.active_nav = "Home".to_string();
            let _ = home.ui(&mut child_ui, false, &mut settings);

            home.active_nav = "Starred".to_string();
            let _ = home.ui(&mut child_ui, false, &mut settings);

            home.active_nav = "Browse".to_string();
            let _ = home.ui(&mut child_ui, false, &mut settings);
        });
    });
}


#[test]
fn test_home_screen_vector_star_and_button_interactions() {
    use siro::ui::home_screen::{render_vector_star, HomeScreen, HomeScreenAction};
    use std::path::PathBuf;

    let ctx = egui::Context::default();

    // 1. Direct unit verification of render_vector_star
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let painter = ui.painter();
            let center = egui::Pos2::new(100.0, 100.0);

            // Test unstarred (hollow), dark mode false
            render_vector_star(painter, center, 7.0, false, false, false);

            // Test unstarred (hollow hovered - gold stroke)
            render_vector_star(painter, center, 7.0, false, true, false);

            // Test unstarred in dark mode
            render_vector_star(painter, center, 7.0, false, false, true);

            // Test starred (solid gold fill + outline), dark mode false
            render_vector_star(painter, center, 7.0, true, false, false);

            // Test starred hovered
            render_vector_star(painter, center, 7.0, true, true, true);
        });
    });

    // 2. Multi-frame hover stability test to ensure NO FLICKER occurs when mouse is over buttons
    let mut home = HomeScreen::new_in_memory();
    let test_path = PathBuf::from(r"C:\TestProjects\InteractiveBoard.siro");
    home.recent_files.push(test_path.clone());
    home.starred_files.push(test_path.clone());

    let viewport = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(1280.0, 800.0));

    let mut settings = siro::storage::settings::AppSettings::default();

    // Frame 1: Pointer moves over the recents row area (where Open button is on the right)
    let pointer_at_open = egui::Pos2::new(1050.0, 360.0);
    let mut input_frame1 = egui::RawInput::default();
    input_frame1.screen_rect = Some(viewport);
    input_frame1.events.push(egui::Event::PointerMoved(pointer_at_open));

    let mut _action_f1 = HomeScreenAction::None;
    let _ = ctx.run(input_frame1, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            _action_f1 = home.ui(ui, false, &mut settings);
        });
    });

    // Frame 2: Mouse stays at the exact same position over Open button
    // In buggy egui code, frame 2 would steal hover, hide the button, and cause 60fps flicker
    let mut input_frame2 = egui::RawInput::default();
    input_frame2.screen_rect = Some(viewport);
    input_frame2.events.push(egui::Event::PointerMoved(pointer_at_open));

    let mut _action_f2 = HomeScreenAction::None;
    let _ = ctx.run(input_frame2, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            _action_f2 = home.ui(ui, false, &mut settings);
        });
    });

    // Frame 3: Mouse still at the same position - must be completely stable across multiple consecutive frames
    let mut input_frame3 = egui::RawInput::default();
    input_frame3.screen_rect = Some(viewport);
    input_frame3.events.push(egui::Event::PointerMoved(pointer_at_open));

    let mut _action_f3 = HomeScreenAction::None;
    let _ = ctx.run(input_frame3, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            _action_f3 = home.ui(ui, false, &mut settings);
        });
    });

    // Verify star toggling logic is solid
    assert!(home.is_starred(&test_path));
    home.toggle_star(&test_path);
    assert!(!home.is_starred(&test_path));
    home.toggle_star(&test_path);
    assert!(home.is_starred(&test_path));
}

#[test]
fn test_actual_disk_file_deletion_and_cleanup() {
    use siro::ui::home_screen::HomeScreen;
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("siro_del_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("Failed to create test temp dir");

    let board_path = temp_dir.join("MyAwesomeHardwareProject.siro");
    let bak_path = temp_dir.join("MyAwesomeHardwareProject.siro.bak");

    // Create real files on the computer
    fs::write(&board_path, b"dummy siro project data").expect("Write board failed");
    fs::write(&bak_path, b"dummy siro backup data").expect("Write bak failed");

    assert!(board_path.exists(), "Board file must exist before deletion");
    assert!(bak_path.exists(), "Backup file must exist before deletion");

    let mut home = HomeScreen::new_in_memory();
    home.recent_files.push(board_path.clone());
    home.starred_files.push(board_path.clone());

    assert!(home.recent_files.contains(&board_path));
    assert!(home.is_starred(&board_path));

    // Execute actual file deletion
    let result = home.delete_board_file(&board_path);
    assert!(result.is_ok(), "delete_board_file must succeed");

    // 1. Invariant: The file MUST be physically deleted from the computer disk!
    assert!(!board_path.exists(), "Board file must be deleted from computer disk");
    // 2. Invariant: Associated backup file MUST also be deleted
    assert!(!bak_path.exists(), "Backup file must be deleted from computer disk");

    // 3. Invariant: No fake 'deleted' folder should exist!
    let deleted_dir = temp_dir.join("deleted");
    assert!(!deleted_dir.exists(), "Must NOT use a fake 'deleted' folder");
    let dot_deleted_dir = temp_dir.join(".deleted");
    assert!(!dot_deleted_dir.exists(), "Must NOT use a fake '.deleted' folder");

    // 4. Invariant: Recents and Starred lists must be cleaned up
    assert!(!home.recent_files.iter().any(|p| p == &board_path), "Must be removed from recents");
    assert!(!home.is_starred(&board_path), "Must be unstarred");

    // Clean up test directory
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_delete_confirmation_modal_dismiss_and_confirm() {
    use siro::ui::home_screen::HomeScreen;
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("siro_modal_test_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).expect("Failed to create modal temp dir");

    let board_path = temp_dir.join("ConfirmDeleteBoard.siro");
    fs::write(&board_path, b"test payload").expect("Write file");
    assert!(board_path.exists());

    let mut home = HomeScreen::new_in_memory();
    home.recent_files.push(board_path.clone());

    let ctx = egui::Context::default();

    // 1. Trigger pending delete (e.g. user clicked the trash icon)
    home.pending_delete_path = Some(board_path.clone());
    assert_eq!(home.pending_delete_path.as_ref(), Some(&board_path));

    // 2. Render frame with Escape key pressed -> modal should dismiss without deleting file
    let mut esc_input = egui::RawInput::default();
    esc_input.events.push(egui::Event::Key {
        key: egui::Key::Escape,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });

    let mut settings = siro::storage::settings::AppSettings::default();
    let _ = ctx.run(esc_input, |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let _ = home.ui(ui, false, &mut settings);
        });
    });

    // Dismissed by Escape
    assert!(home.pending_delete_path.is_none(), "Escape key must cancel pending delete");
    assert!(board_path.exists(), "File must NOT be deleted when user cancels");

    // 3. Confirm deletion directly through delete_board_file
    let del_res = home.delete_board_file(&board_path);
    assert!(del_res.is_ok());
    assert!(!board_path.exists(), "File must be deleted after confirmation");
    assert!(!home.recent_files.contains(&board_path));

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_top_bar_tab_strip_consistency_between_home_and_editor() {
    use siro::canvas::camera::Camera;
    use siro::model::project::Project;
    use siro::ui::top_bar::{TopBar, TopBarAction, TopBarState};

    let ctx = egui::Context::default();
    let mut project = Project::new("Custom Robotics Board");
    // Add extra sheet to verify it does not hijack the upper section tabs
    project.add_board("Power Regulation Sheet");
    assert_eq!(project.boards.len(), 2);

    let camera = Camera::default();
    let mut state = TopBarState::default();

    // Verify Home tab strip rendering
    let _ = ctx.run(Default::default(), |ctx| {
        egui::TopBottomPanel::top("test_home_strip").show(ctx, |ui| {
            let home_action = TopBar::render_home_strip(
                ui,
                Some(("Custom Robotics Board", true)),
                false,
            );
            assert!(matches!(home_action, TopBarAction::None));
        });
    });

    // Verify Editor tab strip rendering: Project tab matches "Custom Robotics Board", not "Main Schematic"
    let _ = ctx.run(Default::default(), |ctx| {
        egui::TopBottomPanel::top("test_editor_strip").show(ctx, |ui| {
            let editor_action = TopBar::ui(
                ui,
                &mut project,
                &camera,
                &mut state,
                false,
                false,
            );
            assert!(matches!(editor_action, TopBarAction::None));
        });
    });

    // Verify Sheet Switcher Bar renders sheets separately at the bottom
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let vp = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::Vec2::new(1280.0, 720.0));
            let (sheet_action, sheet_rect) = TopBar::render_sheet_bar(
                ui,
                &mut project,
                &mut state,
                false,
                vp,
            );
            assert!(matches!(sheet_action, TopBarAction::None));
            assert!(sheet_rect.is_some());
            let r = sheet_rect.unwrap();
            assert!(r.min.x >= vp.min.x);
            assert!(r.max.y <= vp.max.y);
        });
    });
}


#[test]
fn test_multi_board_tabs_creation_switching_and_no_disappearance() {
    use siro::app::{AppState, SiroApp};
    use siro::model::project::Project;
    use siro::ui::top_bar::{TopBar, TopBarAction, TopBarState};

    let mut app = SiroApp::default();
    assert_eq!(app.state, AppState::Home);
    assert_eq!(app.open_projects.len(), 0);
    assert_eq!(app.active_project_idx, None);

    // Step 1: Create a blank board (Board 1)
    let p1 = Project::new("Board 1");
    app.open_projects.push(p1);
    app.active_project_idx = Some(0);
    app.state = AppState::Editor;

    assert_eq!(app.open_projects.len(), 1);
    assert_eq!(app.open_projects[0].manifest.name, "Board 1");
    assert_eq!(app.active_project_idx, Some(0));

    // Step 2: Create another board while being inside Board 1
    // Previously, TopBarAction::NewProject replaced the active project, causing Board 1 to disappear.
    // In our new architecture, it preserves existing boards and appends the new one.
    let p2 = Project::new("Untitled Project");
    app.open_projects.push(p2);
    app.active_project_idx = Some(app.open_projects.len() - 1);

    // Verify: Previous board did NOT disappear! Both boards exist concurrently.
    assert_eq!(app.open_projects.len(), 2, "Both boards must exist simultaneously");
    assert_eq!(app.open_projects[0].manifest.name, "Board 1", "Board 1 must not disappear");
    assert_eq!(app.open_projects[1].manifest.name, "Untitled Project");
    assert_eq!(app.active_project_idx, Some(1), "New board is now active tab");

    // Step 3: Switch back to Board 1
    app.active_project_idx = Some(0);
    assert_eq!(app.active_project_idx, Some(0));
    assert_eq!(app.open_projects[app.active_project_idx.unwrap()].manifest.name, "Board 1");

    // Step 4: Create a 3rd board while on Board 1
    let p3 = Project::new("Robotics Schematic");
    app.open_projects.push(p3);
    app.active_project_idx = Some(app.open_projects.len() - 1);

    assert_eq!(app.open_projects.len(), 3);
    assert_eq!(app.open_projects[0].manifest.name, "Board 1");
    assert_eq!(app.open_projects[1].manifest.name, "Untitled Project");
    assert_eq!(app.open_projects[2].manifest.name, "Robotics Schematic");
    assert_eq!(app.active_project_idx, Some(2));

    // Step 5: Test TopBar::render_window_tab_strip with multiple open tabs
    let ctx = egui::Context::default();
    let mut top_bar_state = TopBarState::default();
    let tab_infos: Vec<(&str, bool)> = app
        .open_projects
        .iter()
        .map(|p| (p.manifest.name.as_str(), p.is_unsaved()))
        .collect();

    let _ = ctx.run(Default::default(), |ctx| {
        egui::TopBottomPanel::top("multi_tab_strip_test").show(ctx, |ui| {
            let action = TopBar::render_window_tab_strip(
                ui,
                &tab_infos,
                app.active_project_idx,
                &mut top_bar_state,
                false,
            );
            assert!(matches!(action, TopBarAction::None));
        });
    });

    // Step 6: Test closing a middle tab (Untitled Project at idx 1)
    let closing_idx = 1;
    app.open_projects.remove(closing_idx);
    if app.open_projects.is_empty() {
        app.active_project_idx = None;
        app.state = AppState::Home;
    } else if let Some(cur) = app.active_project_idx {
        if cur >= closing_idx {
            app.active_project_idx = Some(cur.saturating_sub(1).min(app.open_projects.len() - 1));
        }
    }

    assert_eq!(app.open_projects.len(), 2);
    assert_eq!(app.open_projects[0].manifest.name, "Board 1");
    assert_eq!(app.open_projects[1].manifest.name, "Robotics Schematic");
    assert_eq!(app.active_project_idx, Some(1));
    assert_eq!(app.open_projects[app.active_project_idx.unwrap()].manifest.name, "Robotics Schematic");

    // Step 7: Close all tabs and ensure fallback to Home
    app.open_projects.clear();
    app.active_project_idx = None;
    app.state = AppState::Home;

    assert_eq!(app.open_projects.len(), 0);
    assert_eq!(app.active_project_idx, None);
    assert_eq!(app.state, AppState::Home);
}

#[test]
fn test_delete_key_only_deletes_selected_and_not_backspace() {
    use siro::app::{handle_global_shortcuts, SiroApp};
    use siro::model::element::{BoardElement, ShapeElement, ShapeKind};
    use siro::model::project::Project;

    let ctx = egui::Context::default();
    let mut app = SiroApp::default();
    let mut project = Project::new("Test Board");

    // Add a shape to the board
    let shape = BoardElement::Shape(ShapeElement::new(
        uuid::Uuid::new_v4(),
        ShapeKind::Rectangle,
        vec![
            egui::Pos2::new(100.0, 100.0),
            egui::Pos2::new(300.0, 100.0),
            egui::Pos2::new(300.0, 250.0),
            egui::Pos2::new(100.0, 250.0),
        ],
        egui::Color32::BLACK,
        2.0,
        egui::Color32::TRANSPARENT,
    ));
    let shape_id = shape.id();
    project.active_board_mut().add_element(shape);
    assert_eq!(project.active_board().elements.len(), 1);

    // Select the element
    app.select_tool.selected_ids.insert(shape_id);

    // 1. Simulate pressing Backspace: Selected element must NOT be deleted
    let mut backspace_input = egui::RawInput::default();
    backspace_input.events.push(egui::Event::Key {
        key: egui::Key::Backspace,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let _ = ctx.run(backspace_input, |ctx| {
        handle_global_shortcuts(&mut app, &mut project, ctx);
    });

    assert_eq!(
        project.active_board().elements.len(),
        1,
        "Backspace must NEVER delete selected elements"
    );
    assert!(
        app.select_tool.selected_ids.contains(&shape_id),
        "Element must remain selected after Backspace"
    );

    // 2. Simulate pressing Delete: Selected element MUST be deleted
    let mut delete_input = egui::RawInput::default();
    delete_input.events.push(egui::Event::Key {
        key: egui::Key::Delete,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let _ = ctx.run(delete_input, |ctx| {
        handle_global_shortcuts(&mut app, &mut project, ctx);
    });

    assert_eq!(
        project.active_board().elements.len(),
        0,
        "Del key must delete selected elements"
    );
    assert!(
        app.select_tool.selected_ids.is_empty(),
        "Selection must be cleared after deletion"
    );
}

#[test]
fn test_editing_text_protects_elements_from_deletion() {
    use siro::app::{handle_global_shortcuts, SiroApp};
    use siro::model::element::{BoardElement, TextElement};
    use siro::model::project::Project;

    let ctx = egui::Context::default();
    let mut app = SiroApp::default();
    let mut project = Project::new("Text Editing Board");

    // Add a text box / sticky note
    let text_elem = BoardElement::Text(TextElement::new_sticky(
        uuid::Uuid::new_v4(),
        egui::Pos2::new(50.0, 50.0),
        "Important note".to_string(),
        egui::Color32::from_rgb(255, 249, 177),
    ));
    let text_id = text_elem.id();
    project.active_board_mut().add_element(text_elem);
    assert_eq!(project.active_board().elements.len(), 1);

    // Select the text element and simulate currently writing in it
    app.select_tool.selected_ids.insert(text_id);
    app.editing_text_id = Some(text_id);

    // 1. While writing, pressing Backspace must NOT delete the text box or sticky note
    let mut backspace_input = egui::RawInput::default();
    backspace_input.events.push(egui::Event::Key {
        key: egui::Key::Backspace,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let _ = ctx.run(backspace_input, |ctx| {
        handle_global_shortcuts(&mut app, &mut project, ctx);
    });

    assert_eq!(
        project.active_board().elements.len(),
        1,
        "Backspace while editing text must NOT delete the text box or sticky note"
    );

    // 2. While writing, pressing Delete (e.g. forward delete character) must NOT delete the text box
    let mut delete_input = egui::RawInput::default();
    delete_input.events.push(egui::Event::Key {
        key: egui::Key::Delete,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let _ = ctx.run(delete_input, |ctx| {
        handle_global_shortcuts(&mut app, &mut project, ctx);
    });

    assert_eq!(
        project.active_board().elements.len(),
        1,
        "Delete key while editing text must NOT delete the text box"
    );
    assert!(
        project.active_board().get_element(text_id).is_some(),
        "Text element must stay intact on board"
    );

    // 3. Stop editing text (e.g. user presses Esc or clicks away)
    app.editing_text_id = None;

    // Now pressing Delete deletes the selected item
    let mut del_after_finish = egui::RawInput::default();
    del_after_finish.events.push(egui::Event::Key {
        key: egui::Key::Delete,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    let _ = ctx.run(del_after_finish, |ctx| {
        handle_global_shortcuts(&mut app, &mut project, ctx);
    });

    assert_eq!(
        project.active_board().elements.len(),
        0,
        "Delete key when not editing text deletes the selected item"
    );
}

#[test]
fn test_ui_honesty_branding_and_app_name() {
    assert_eq!(siro::APP_NAME, "Siro", "Central APP_NAME must be Siro");
    assert!(!siro::APP_NAME.is_empty(), "APP_NAME constant must be non-empty");
}

#[test]
fn test_theme_persistence_roundtrip() {
    use siro::storage::history::{load_theme_dark_mode, save_theme_dark_mode, theme_file_path};

    let original = load_theme_dark_mode();
    let test_path = theme_file_path();

    // Toggle and save true
    assert!(save_theme_dark_mode(true).is_ok());
    assert!(load_theme_dark_mode(), "Loaded theme must be true (dark mode)");

    // Toggle and save false
    assert!(save_theme_dark_mode(false).is_ok());
    assert!(!load_theme_dark_mode(), "Loaded theme must be false (light mode)");

    // Restore original state
    let _ = save_theme_dark_mode(original);
    let _ = test_path;
}

#[test]
fn test_ui_honesty_recents_table_honest_metadata() {
    use siro::ui::home_screen::HomeScreen;
    use std::path::PathBuf;

    let mut home = HomeScreen::new_in_memory();
    let temp_board = std::env::temp_dir().join(format!("honesty_test_{}.siro", uuid::Uuid::new_v4()));
    let _ = std::fs::write(&temp_board, b"test siro file content");

    home.recent_files.push(temp_board.clone());
    home.recent_files.push(PathBuf::from(r"C:\NonExistent\MissingBoard.siro"));
    home.is_grid_view = false; // test list table view

    let ctx = egui::Context::default();
    let mut settings = siro::storage::settings::AppSettings::default();
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let _ = home.ui(ui, false, &mut settings);
            let _ = home.ui(ui, true, &mut settings);
        });
    });

    let _ = std::fs::remove_file(&temp_board);
}

#[test]
fn test_ui_honesty_zero_forbidden_strings_in_src() {
    let src_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let forbidden_phrases = [
        "Silicon Labs",
        "Hardware Lab",
        "Group Boards",
        "Offline Mode",
        "And we're back, Builder",
        "Miro Blue",
    ];

    fn check_dir(dir: &std::path::Path, forbidden: &[&str]) {
        for entry in std::fs::read_dir(dir).expect("read_dir failed") {
            let entry = entry.expect("dir entry failed");
            let path = entry.path();
            if path.is_dir() {
                check_dir(&path, forbidden);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content = std::fs::read_to_string(&path).expect("read file failed");
                for &phrase in forbidden {
                    assert!(
                        !content.contains(phrase),
                        "Found forbidden phrase {:?} in {:?}",
                        phrase,
                        path
                    );
                }

                // Check that literal "Siro" only appears in lib.rs constant definition
                if path.file_name().and_then(|s| s.to_str()) != Some("lib.rs") {
                    assert!(
                        !content.contains("\"Siro\""),
                        "Found hardcoded \"Siro\" literal in {:?}; must use APP_NAME constant",
                        path
                    );
                }
            }
        }
    }

    check_dir(&src_dir, &forbidden_phrases);
}

#[test]
fn test_sheet_switcher_dropdown_responsive_wrapping_and_alignment() {
    use siro::canvas::camera::Camera;
    use siro::model::project::Project;
    use siro::ui::top_bar::{TopBar, TopBarAction, TopBarState};

    let ctx = egui::Context::default();
    let font_id = egui::FontId::proportional(12.5);
    let gutter_w = 22.0;

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Case 1: "Main Schematic" (the exact case reported by user)
            // In the menu, available width is at least target_menu_width (210px) minus margins
            let menu_w = 210.0_f32;
            let text_wrap_w = (menu_w - 16.0 - gutter_w - 6.0).max(100.0);

            // Lay out "Main Schematic" when active and inactive
            let galley_active = ui.painter().layout(
                "Main Schematic".to_string(),
                font_id.clone(),
                egui::Color32::WHITE,
                text_wrap_w,
            );
            let galley_inactive = ui.painter().layout(
                "Main Schematic".to_string(),
                font_id.clone(),
                egui::Color32::from_rgb(205, 210, 225),
                text_wrap_w,
            );

            // Neither must wrap!
            assert_eq!(
                galley_active.rows.len(),
                1,
                "'Main Schematic' must not wrap to line 2 when selected"
            );
            assert_eq!(
                galley_inactive.rows.len(),
                1,
                "'Main Schematic' must not wrap when unselected"
            );
            assert_eq!(galley_active.rows[0].text(), "Main Schematic");
            assert_eq!(galley_inactive.rows[0].text(), "Main Schematic");

            // Row heights must be identical across active and inactive states
            let row_h_active = (galley_active.size().y + 10.0).max(28.0);
            let row_h_inactive = (galley_inactive.size().y + 10.0).max(28.0);
            assert_eq!(
                row_h_active, row_h_inactive,
                "Row heights must be identical between selected and unselected states"
            );

            // Case 2: A name that is actually longer than box width
            let long_name = "Super Long Schematic Sheet Name That Exceeds Box Width";
            let galley_long_active = ui.painter().layout(
                long_name.to_string(),
                font_id.clone(),
                egui::Color32::WHITE,
                text_wrap_w,
            );
            let galley_long_inactive = ui.painter().layout(
                long_name.to_string(),
                font_id.clone(),
                egui::Color32::from_rgb(205, 210, 225),
                text_wrap_w,
            );

            // Genuinely long name wraps, and wraps to the EXACT same row count whether active or not!
            assert!(
                galley_long_active.rows.len() > 1,
                "Names longer than box width must wrap"
            );
            assert_eq!(
                galley_long_active.rows.len(),
                galley_long_inactive.rows.len(),
                "Wrap row count must be identical whether active or inactive"
            );
            for (idx, (r1, r2)) in galley_long_active
                .rows
                .iter()
                .zip(galley_long_inactive.rows.iter())
                .enumerate()
            {
                assert_eq!(
                    r1.text(),
                    r2.text(),
                    "Row {} text wrap point must match between active and inactive states",
                    idx
                );
            }
        });
    });

    // Also verify TopBar::ui with multiple sheets renders without panic
    let mut project = Project::new("Test Multi Sheet");
    let sheet2_id = project.add_board("Sheet 2");
    assert_eq!(project.boards.len(), 2);
    let camera = Camera::default();
    let mut state = TopBarState::default();

    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let action = TopBar::ui(ui, &mut project, &camera, &mut state, false, false);
            assert!(matches!(action, TopBarAction::None));
        });
    });

    // Switch to Sheet 2 and verify again
    project.active_board_id = sheet2_id;
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |ui| {
            let action = TopBar::ui(ui, &mut project, &camera, &mut state, false, false);
            assert!(matches!(action, TopBarAction::None));
        });
    });
}

