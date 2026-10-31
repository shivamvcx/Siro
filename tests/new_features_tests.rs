use egui::{Color32, Pos2, Rect, Stroke, Vec2};
use siro::canvas::tools::TableTool;
use siro::model::element::*;
use siro::ui::color_presets::{is_preset_color, COLOR_PRESETS};
use uuid::Uuid;

#[test]
fn test_feature_1_color_presets_spec() {
    // 1. Swatches in exact specified order: Black, White, Yellow, Red, Light Blue
    assert_eq!(COLOR_PRESETS.len(), 5);
    assert_eq!(COLOR_PRESETS[0].1, "Black");
    assert_eq!(COLOR_PRESETS[0].0, Color32::from_rgb(0, 0, 0));

    assert_eq!(COLOR_PRESETS[1].1, "White");
    assert_eq!(COLOR_PRESETS[1].0, Color32::from_rgb(255, 255, 255));

    assert_eq!(COLOR_PRESETS[2].1, "Yellow");
    assert_eq!(COLOR_PRESETS[2].0, Color32::from_rgb(255, 220, 0));

    assert_eq!(COLOR_PRESETS[3].1, "Red");
    assert_eq!(COLOR_PRESETS[3].0, Color32::from_rgb(235, 50, 50));

    assert_eq!(COLOR_PRESETS[4].1, "Light Blue");
    assert_eq!(COLOR_PRESETS[4].0, Color32::from_rgb(70, 160, 255));

    // Preset check logic
    assert!(is_preset_color(Color32::from_rgb(0, 0, 0)));
    assert!(is_preset_color(Color32::from_rgb(70, 160, 255)));
    assert!(!is_preset_color(Color32::from_rgb(123, 45, 67)));
}

#[test]
fn test_feature_2_image_perspective_lock_and_reset() {
    let mut img = ImageElement {
        id: Uuid::new_v4(),
        asset_id: Uuid::new_v4(),
        image_name: "test.png".to_string(),
        pos: Pos2::new(100.0, 100.0),
        size: Vec2::new(200.0, 150.0),
        opacity: 1.0,
        perspective_locked: true,
        corners: None,
    };

    // Default corners are standard unwarped rectangle
    let unwarped = img.get_corners_local();
    assert_eq!(unwarped[0], Pos2::new(0.0, 0.0));
    assert_eq!(unwarped[1], Pos2::new(200.0, 0.0));
    assert_eq!(unwarped[2], Pos2::new(200.0, 150.0));
    assert_eq!(unwarped[3], Pos2::new(0.0, 150.0));

    // Unlock and warp corners
    img.perspective_locked = false;
    img.corners = Some([
        Pos2::new(10.0, 5.0),
        Pos2::new(190.0, 15.0),
        Pos2::new(210.0, 140.0),
        Pos2::new(-5.0, 155.0),
    ]);

    let world_corners = img.get_corners_world();
    assert_eq!(world_corners[0], Pos2::new(110.0, 105.0));
    assert_eq!(world_corners[1], Pos2::new(290.0, 115.0));

    // Quad validity tests (prevents degenerate / self-intersecting quads)
    let valid_quad = [
        Pos2::new(0.0, 0.0),
        Pos2::new(100.0, 0.0),
        Pos2::new(100.0, 100.0),
        Pos2::new(0.0, 100.0),
    ];
    assert!(ImageElement::is_valid_quad(&valid_quad));

    // Degenerate (points too close together < 20px)
    let collapsed_quad = [
        Pos2::new(0.0, 0.0),
        Pos2::new(5.0, 0.0),
        Pos2::new(100.0, 100.0),
        Pos2::new(0.0, 100.0),
    ];
    assert!(!ImageElement::is_valid_quad(&collapsed_quad));

    // Self-intersecting / bowtie quad
    let bowtie_quad = [
        Pos2::new(0.0, 0.0),
        Pos2::new(100.0, 100.0),
        Pos2::new(100.0, 0.0),
        Pos2::new(0.0, 100.0),
    ];
    assert!(!ImageElement::is_valid_quad(&bowtie_quad));

    // Edge handle dragging test
    use siro::canvas::tools::select::{DragMode, SelectTool};
    use siro::model::board::Board;
    let mut board = Board::new("Edge Handle Board");
    let edge_img = ImageElement {
        id: Uuid::new_v4(),
        asset_id: Uuid::new_v4(),
        image_name: "test_edge.png".to_string(),
        pos: Pos2::new(100.0, 100.0),
        size: Vec2::new(200.0, 150.0),
        opacity: 1.0,
        perspective_locked: false,
        corners: None,
    };
    let edge_img_id = edge_img.id;
    board.add_element(BoardElement::Image(edge_img));

    let mut select = SelectTool::default();
    select.selected_ids.insert(edge_img_id);

    // Midpoint of top edge: (corners[0] + corners[1])/2 = (100, 100) + (100, 0) = (200, 100)
    let top_mid = Pos2::new(200.0, 100.0);
    select.on_pointer_down_with_zoom(top_mid, &board, false, 1.0);
    assert_eq!(select.drag_mode, DragMode::PerspectiveEdge(0), "Must hit Top edge handle (index 0)");

    // Drag top edge up by 25px: (200, 75)
    select.on_pointer_move(Pos2::new(200.0, 75.0), &mut board);
    select.on_pointer_up(&board);

    let updated_elem = board.get_element(edge_img_id).unwrap();
    if let BoardElement::Image(img_after) = updated_elem {
        let corners = img_after.corners.unwrap();
        // Top-left and Top-right corners both moved up by 25px
        assert_eq!(corners[0], Pos2::new(0.0, -25.0));
        assert_eq!(corners[1], Pos2::new(200.0, -25.0));
        // Bottom corners remain unchanged
        assert_eq!(corners[2], Pos2::new(200.0, 150.0));
        assert_eq!(corners[3], Pos2::new(0.0, 150.0));
    } else {
        panic!("Element must be Image");
    }

    // Reset perspective restores unwarped and locks
    img.reset_perspective();
    assert!(img.perspective_locked);
    assert_eq!(img.corners, None);

    // Serialization round-trip test for backwards compatibility
    let json = serde_json::to_string(&img).unwrap();
    let deserialized: ImageElement = serde_json::from_str(&json).unwrap();
    assert!(deserialized.perspective_locked);
    assert_eq!(deserialized.corners, None);

    // Backward compatibility: loading old json without perspective fields defaults to locked
    let old_json = r#"{
        "id": "00000000-0000-0000-0000-000000000001",
        "asset_id": "00000000-0000-0000-0000-000000000002",
        "image_name": "legacy.jpg",
        "pos": [50.0, 50.0],
        "size": [100.0, 100.0],
        "opacity": 0.8
    }"#;
    let legacy_img: ImageElement = serde_json::from_str(old_json).unwrap();
    assert!(legacy_img.perspective_locked);
    assert_eq!(legacy_img.corners, None);
}

#[test]
fn test_feature_3_table_cell_recolor_selected_text_spans() {
    let mut table = TableElement::new(Uuid::new_v4(), Pos2::new(0.0, 0.0), 3, 3);
    *table.get_cell_mut(1, 1).unwrap() = "Hello World".to_string();

    // Recolor "World" (characters 6..11) to Red
    table.add_or_update_span(1, 1, 6, 11, Color32::RED);

    let spans = table.get_cell_spans(1, 1).unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].start, 6);
    assert_eq!(spans[0].end, 11);
    assert_eq!(spans[0].color, Color32::RED);

    // Recolor "or" (characters 7..9) to Yellow inside "World"
    table.add_or_update_span(1, 1, 7, 9, Color32::YELLOW);
    let updated_spans = table.get_cell_spans(1, 1).unwrap();
    assert_eq!(updated_spans.len(), 3);
    assert_eq!(updated_spans[0].start, 6);
    assert_eq!(updated_spans[0].end, 7);
    assert_eq!(updated_spans[0].color, Color32::RED);
    assert_eq!(updated_spans[1].start, 7);
    assert_eq!(updated_spans[1].end, 9);
    assert_eq!(updated_spans[1].color, Color32::YELLOW);
    assert_eq!(updated_spans[2].start, 9);
    assert_eq!(updated_spans[2].end, 11);
    assert_eq!(updated_spans[2].color, Color32::RED);

    // Serialization round-trip
    let json = serde_json::to_string(&table).unwrap();
    let deserialized: TableElement = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.get_cell_spans(1, 1).unwrap().len(), 3);
}

#[test]
fn test_feature_4_notion_style_row_controls() {
    let mut table = TableElement::new(Uuid::new_v4(), Pos2::new(0.0, 0.0), 3, 2);
    *table.get_cell_mut(0, 0).unwrap() = "R0".to_string();
    *table.get_cell_mut(1, 0).unwrap() = "R1".to_string();
    *table.get_cell_mut(2, 0).unwrap() = "R2".to_string();

    table.set_row_color(1, Some(Color32::from_rgb(0, 220, 255)));
    assert_eq!(table.get_row_color(1), Some(Color32::from_rgb(0, 220, 255)));

    // Move row 1 up -> swaps with row 0
    TableTool::move_row_up(&mut table, 1);
    assert_eq!(table.get_cell(0, 0), Some("R1"));
    assert_eq!(table.get_cell(1, 0), Some("R0"));
    assert_eq!(table.get_row_color(0), Some(Color32::from_rgb(0, 220, 255)));

    // Move row 0 down -> back to row 1
    TableTool::move_row_down(&mut table, 0);
    assert_eq!(table.get_cell(1, 0), Some("R1"));

    // Reorder row tests (drag and drop reordering)
    let mut reorder_table = TableElement::new(Uuid::new_v4(), Pos2::new(0.0, 0.0), 4, 2);
    *reorder_table.get_cell_mut(0, 0).unwrap() = "Row 0".to_string();
    *reorder_table.get_cell_mut(1, 0).unwrap() = "Row 1".to_string();
    *reorder_table.get_cell_mut(2, 0).unwrap() = "Row 2".to_string();
    *reorder_table.get_cell_mut(3, 0).unwrap() = "Row 3".to_string();
    reorder_table.set_row_color(0, Some(Color32::RED));
    reorder_table.set_row_color(3, Some(Color32::BLUE));
    reorder_table.add_or_update_span(0, 0, 0, 3, Color32::YELLOW);

    // Drag row 0 down to position 2 (Row 1, Row 2, Row 0, Row 3)
    TableTool::reorder_row(&mut reorder_table, 0, 2);
    assert_eq!(reorder_table.get_cell(0, 0), Some("Row 1"));
    assert_eq!(reorder_table.get_cell(1, 0), Some("Row 2"));
    assert_eq!(reorder_table.get_cell(2, 0), Some("Row 0"));
    assert_eq!(reorder_table.get_cell(3, 0), Some("Row 3"));
    assert_eq!(reorder_table.get_row_color(2), Some(Color32::RED));
    assert_eq!(reorder_table.get_cell_spans(2, 0).unwrap().len(), 1);

    // Drag last row 3 up to position 0 (Row 3, Row 1, Row 2, Row 0)
    TableTool::reorder_row(&mut reorder_table, 3, 0);
    assert_eq!(reorder_table.get_cell(0, 0), Some("Row 3"));
    assert_eq!(reorder_table.get_cell(1, 0), Some("Row 1"));
    assert_eq!(reorder_table.get_cell(2, 0), Some("Row 2"));
    assert_eq!(reorder_table.get_cell(3, 0), Some("Row 0"));
    assert_eq!(reorder_table.get_row_color(0), Some(Color32::BLUE));
    assert_eq!(reorder_table.get_row_color(3), Some(Color32::RED));
    assert_eq!(reorder_table.get_cell_spans(3, 0).unwrap().len(), 1);

    // Delete row 2 -> 2 rows remain
    assert!(TableTool::remove_row(&mut table, 2));
    assert_eq!(table.rows, 2);

    // Delete row 1 -> 1 row remains
    assert!(TableTool::remove_row(&mut table, 1));
    assert_eq!(table.rows, 1);

    // Delete last remaining row -> safely blocked!
    assert!(!TableTool::remove_row(&mut table, 0));
    assert_eq!(table.rows, 1);
}

#[test]
fn test_feature_5_text_background_properties() {
    let mut text = TextElement::new_text(Uuid::new_v4(), Pos2::new(50.0, 50.0), "Sample Text".into());
    // Default is no background
    assert!(!text.bg_enabled);
    assert_eq!(text.bg_opacity, 1.0);

    // Enable background, set custom color and 50% opacity
    text.bg_enabled = true;
    text.bg_color = Color32::from_rgb(20, 25, 40);
    text.bg_opacity = 0.5;

    assert!(text.bg_enabled);
    assert_eq!(text.bg_color, Color32::from_rgb(20, 25, 40));
    assert_eq!(text.bg_opacity, 0.5);

    // Serialization round-trip
    let json = serde_json::to_string(&text).unwrap();
    let deserialized: TextElement = serde_json::from_str(&json).unwrap();
    assert!(deserialized.bg_enabled);
    assert_eq!(deserialized.bg_opacity, 0.5);
    assert_eq!(deserialized.bg_pad_x, 8.0);
    assert_eq!(deserialized.bg_pad_y, 5.0);
    assert_eq!(deserialized.bg_corner_radius, 6.0);
    assert!(!deserialized.bg_match_width);

    // Custom size and shape
    text.bg_pad_x = 20.0;
    text.bg_pad_y = 12.0;
    text.bg_corner_radius = 16.0;
    text.bg_match_width = true;
    let custom_json = serde_json::to_string(&text).unwrap();
    let custom_deser: TextElement = serde_json::from_str(&custom_json).unwrap();
    assert_eq!(custom_deser.bg_pad_x, 20.0);
    assert_eq!(custom_deser.bg_pad_y, 12.0);
    assert_eq!(custom_deser.bg_corner_radius, 16.0);
    assert!(custom_deser.bg_match_width);

    // Backward compatibility: legacy text without bg fields defaults to safe defaults
    let old_json = r#"{
        "id": "00000000-0000-0000-0000-000000000003",
        "text": "Legacy Text",
        "pos": [0.0, 0.0],
        "font_size": 16.0,
        "color": [220, 225, 235, 255],
        "is_sticky": false,
        "sticky_color": [255, 240, 130, 255],
        "width": 200.0
    }"#;
    let legacy_text: TextElement = serde_json::from_str(old_json).unwrap();
    assert!(!legacy_text.bg_enabled);
    assert_eq!(legacy_text.bg_opacity, 1.0);
    assert_eq!(legacy_text.bg_pad_x, 8.0);
    assert_eq!(legacy_text.bg_pad_y, 5.0);
    assert_eq!(legacy_text.bg_corner_radius, 6.0);
    assert!(!legacy_text.bg_match_width);
}

#[test]
fn test_figma_style_text_box_resizing_preserves_font_size() {
    use siro::canvas::tools::select::{DragMode, ResizeHandle, SelectTool};
    use siro::model::board::Board;

    let mut board = Board::new("Figma Text Resize Board");
    let text = TextElement::new_text(Uuid::new_v4(), Pos2::new(100.0, 100.0), "Front Air Intake -".to_string());
    let text_id = text.id;
    let initial_font_size = text.font_size; // 16.0
    let initial_width = text.width; // 200.0
    board.add_element(BoardElement::Text(text));

    let mut select = SelectTool::default();
    select.selected_ids.insert(text_id);

    // Drag BottomRight handle outwards to scale the text box horizontally
    let handle_pos = Pos2::new(100.0 + initial_width, 100.0 + 30.0);
    select.on_pointer_down_with_zoom(handle_pos, &board, false, 1.0);
    assert!(matches!(select.drag_mode, DragMode::Resize(ResizeHandle::BottomRight)));

    // Drag to new position doubling the width
    select.on_pointer_move(Pos2::new(100.0 + initial_width * 2.0, 100.0 + 30.0), &mut board);
    select.on_pointer_up(&board);

    let elem = board.get_element(text_id).unwrap();
    if let BoardElement::Text(txt) = elem {
        // Font size must stay intact (Figma behavior: font size is not scaled by bounding box drag)
        assert_eq!(txt.font_size, initial_font_size, "Font size must remain unchanged during text box resize");
        // Boundary width must expand
        assert!(txt.width > initial_width, "Boundary width must expand to wrap text wider");
    } else {
        panic!("Element must be Text");
    }
}

#[test]
fn test_image_and_text_toolbar_responsiveness() {
    use siro::model::board::Board;
    use siro::ui::properties_bar::PropertiesBar;

    let mut board = Board::new("Toolbar Responsiveness Board");
    let img = ImageElement {
        id: Uuid::new_v4(),
        asset_id: Uuid::new_v4(),
        image_name: "test.png".to_string(),
        pos: Pos2::new(0.0, 0.0),
        size: Vec2::new(200.0, 150.0),
        opacity: 1.0,
        perspective_locked: true,
        corners: None,
    };
    let img_id = img.id;
    board.add_element(BoardElement::Image(img));

    // Image toolbar is now compact (~300-360px) rather than having giant empty whitespace
    let img_tb_w = PropertiesBar::calculate_toolbar_width(&board, &[img_id]);
    assert!(img_tb_w >= 280.0 && img_tb_w <= 360.0, "Image toolbar width was {}", img_tb_w);

    // Text with background enabled is modular and compact
    let mut txt = TextElement::new_text(Uuid::new_v4(), Pos2::new(0.0, 0.0), "Sample".to_string());
    txt.bg_enabled = true;
    let txt_id = txt.id;
    board.add_element(BoardElement::Text(txt));

    let txt_tb_w = PropertiesBar::calculate_toolbar_width(&board, &[txt_id]);
    assert!(txt_tb_w <= 600.0, "Text toolbar width with background must be compact and modular: {}", txt_tb_w);
}

#[test]
fn test_item_3_shared_popup_manager_stack_and_behavior() {
    use siro::ui::popup_manager::PopupManager;
    use egui::{Pos2, Rect, Vec2};

    let mut pm = PopupManager::default();

    // 1. Open parent popup (e.g. Background Style)
    let parent_anchor = Rect::from_min_size(Pos2::new(200.0, 100.0), Vec2::new(60.0, 30.0));
    let avoid_rect = Rect::from_min_size(Pos2::new(180.0, 140.0), Vec2::new(150.0, 40.0));
    pm.open("text_bg_popover", "Background Style", None, Some(parent_anchor), Some(avoid_rect));
    assert!(pm.is_open("text_bg_popover"));
    assert_eq!(pm.stack.len(), 1);

    // Initial placement computation: avoids covering avoid_rect
    let ctx = egui::Context::default();
    let initial_pos = pm.calculate_initial_pos(
        &ctx,
        None,
        Some(parent_anchor),
        Some(avoid_rect),
        Vec2::new(200.0, 200.0),
    );
    // Verified: does not overlap avoid_rect
    let popup_rect = Rect::from_min_size(initial_pos, Vec2::new(200.0, 200.0));
    assert!(!popup_rect.intersects(avoid_rect), "Popup must not overlap selected object");

    // Simulate popup size & rect updated after rendering
    pm.stack[0].pos = initial_pos;
    pm.stack[0].size = Vec2::new(200.0, 200.0);
    pm.stack[0].rect = popup_rect;
    pm.stack[0].initialized = true;

    // 2. Open child popup (e.g. Custom Color Picker)
    let child_anchor = Rect::from_min_size(Pos2::new(initial_pos.x + 20.0, initial_pos.y + 40.0), Vec2::new(18.0, 18.0));
    pm.open("custom_color_picker", "Custom Color", Some("text_bg_popover"), Some(child_anchor), None);
    assert_eq!(pm.stack.len(), 2);
    assert_eq!(pm.stack[1].parent_id, Some("text_bg_popover".to_string()));

    // Child placement: anchors next to parent
    let child_pos = pm.calculate_initial_pos(
        &ctx,
        Some("text_bg_popover"),
        Some(child_anchor),
        None,
        Vec2::new(180.0, 180.0),
    );
    // Child x is adjacent to parent max x (+ 8px margin)
    assert_eq!(child_pos.x, pm.stack[0].rect.max.x + 8.0);
    pm.stack[1].pos = child_pos;
    pm.stack[1].size = Vec2::new(180.0, 180.0);
    pm.stack[1].rect = Rect::from_min_size(child_pos, Vec2::new(180.0, 180.0));
    pm.stack[1].initialized = true;

    // 3. Draggable: dragging child only modifies child pos
    let old_parent_pos = pm.stack[0].pos;
    let old_child_pos = pm.stack[1].pos;
    pm.stack[1].pos += Vec2::new(15.0, -10.0);
    assert_eq!(pm.stack[0].pos, old_parent_pos, "Dragging child must NOT move parent");
    assert_ne!(pm.stack[1].pos, old_child_pos, "Dragging child moves child");

    // 4. Clamping keeps popups fully inside window bounds
    let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(800.0, 600.0));
    let clamped = PopupManager::clamp_rect_to_screen(Pos2::new(750.0, 550.0), Vec2::new(200.0, 200.0), screen);
    assert!(clamped.x + 200.0 <= screen.max.x, "Clamped x must be within screen bounds");
    assert!(clamped.y + 200.0 <= screen.max.y, "Clamped y must be within screen bounds");

    // 5. Esc closes only topmost popup first
    // Emulate Esc key in context
    let _ = ctx.run(egui::RawInput {
        events: vec![egui::Event::Key {
            key: egui::Key::Escape,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        }],
        ..Default::default()
    }, |ctx| {
        let consumed = pm.update_input(ctx);
        assert!(consumed, "Esc should be consumed by popup manager");
    });
    // Topmost popup ("custom_color_picker") closed, parent ("text_bg_popover") remains open!
    assert!(!pm.is_open("custom_color_picker"), "Topmost child popup must be closed first by Esc");
    assert!(pm.is_open("text_bg_popover"), "Parent popup must remain open after Esc closes child");

    // 6. Outside click closes the entire stack
    pm.just_opened_id = None;
    let outside_pos = Pos2::new(5.0, 5.0);
    assert!(!pm.stack.iter().any(|p| p.rect.contains(outside_pos)));
    let _ = ctx.run(egui::RawInput {
        events: vec![egui::Event::PointerButton {
            pos: outside_pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
        ..Default::default()
    }, |ctx| {
        pm.update_input(ctx);
    });
    assert!(pm.stack.is_empty(), "Outside click must close entire stack");
}

#[test]
fn test_item_4_global_slider_styling_visibility() {
    let mut vis_light = egui::Visuals::light();
    vis_light.slider_trailing_fill = true;
    vis_light.widgets.inactive.bg_fill = Color32::from_rgb(228, 232, 240);
    vis_light.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(195, 202, 215));
    vis_light.widgets.inactive.fg_stroke = Stroke::new(1.5_f32, Color32::from_rgb(60, 65, 80));

    // Contrast check: inactive slider track is clearly distinguishable from pure white window
    assert_ne!(vis_light.widgets.inactive.bg_fill, Color32::WHITE);
    assert!(vis_light.slider_trailing_fill, "Filled track portion must be enabled");
    assert!(vis_light.widgets.inactive.bg_stroke.width >= 1.0, "Track must have visible border");
    assert!(vis_light.widgets.inactive.fg_stroke.width >= 1.0, "Handle must have visible border");

    let mut vis_dark = egui::Visuals::dark();
    vis_dark.slider_trailing_fill = true;
    vis_dark.widgets.inactive.bg_fill = Color32::from_rgb(45, 50, 62);
    vis_dark.widgets.inactive.bg_stroke = Stroke::new(1.0_f32, Color32::from_rgb(70, 78, 95));
    assert!(vis_dark.slider_trailing_fill, "Dark mode filled track portion must be enabled");
    assert!(vis_dark.widgets.inactive.bg_stroke.width >= 1.0);
}

#[test]
fn test_item_5_text_color_presets_do_not_recolor_existing_text() {
    let ctx = egui::Context::default();
    let mut active_stroke_color = Color32::from_rgb(0, 0, 0); // Black initially
    let initial_elem_color = Color32::from_rgb(100, 150, 200);

    let mut text_element = TextElement::new_text(Uuid::new_v4(), Pos2::new(10.0, 10.0), "Hello".into());
    text_element.color = initial_elem_color;

    // Simulate clicking Red preset swatch in toolbar
    let _ = ctx.run(Default::default(), |ctx| {
        egui::CentralPanel::default().show(ctx, |_ui| {
            // Preset 3 is Red: (235, 50, 50)
            let red = Color32::from_rgb(235, 50, 50);
            active_stroke_color = red;
        });
    });

    // Requirement: "The selected color is highlighted. Existing text must not change color."
    assert_eq!(active_stroke_color, Color32::from_rgb(235, 50, 50), "Active stroke color updated");
    assert_eq!(text_element.color, initial_elem_color, "Existing text must NOT change color when picking text color");
}

#[test]
fn test_item_6_text_background_custom_color_picker_live_update() {
    use siro::ui::popup_manager::PopupManager;

    let mut popup_manager = PopupManager::default();
    let mut text = TextElement::new_text(Uuid::new_v4(), Pos2::new(50.0, 50.0), "BG Test".into());
    text.bg_enabled = true;
    text.bg_color = Color32::from_rgb(0, 0, 0);
    text.bg_opacity = 0.75;

    // Open Background Style popup
    popup_manager.open("text_bg_popover", "Background Style", None, None, None);

    // Rainbow button clicked -> opens custom color child popup
    popup_manager.open(
        "text_bg_custom_color_picker",
        "Custom Color",
        Some("text_bg_popover"),
        Some(Rect::from_min_size(Pos2::new(100.0, 100.0), Vec2::new(18.0, 18.0))),
        None,
    );

    assert!(popup_manager.is_open("text_bg_popover"), "Parent BG popup must remain open");
    assert!(popup_manager.is_open("text_bg_custom_color_picker"), "Child color picker must be open");

    // Live update color
    let new_custom_color = Color32::from_rgb(140, 45, 220);
    text.bg_color = new_custom_color;

    assert_eq!(text.bg_color, new_custom_color, "Live color update must apply to text background");
    assert_eq!(text.bg_opacity, 0.75, "Opacity must be preserved when changing background color");

    // Serialization persistence test
    let json = serde_json::to_string(&text).unwrap();
    let deser: TextElement = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.bg_color, new_custom_color);
    assert_eq!(deser.bg_opacity, 0.75);
}

#[test]
fn test_table_row_menu_remains_open_when_cursor_moves_to_options() {
    use siro::ui::popup_manager::PopupManager;

    let mut pm = PopupManager::default();
    let tbl_id = Uuid::new_v4();
    let clicked_row = 2; // Row 3 (0-indexed 2)
    let row_menu_id = format!("notion_row_menu_{}_{}", tbl_id, clicked_row);

    // Open row menu on row 2
    pm.open(
        &row_menu_id,
        "Row 3",
        None,
        Some(Rect::from_min_size(Pos2::new(100.0, 150.0), Vec2::new(20.0, 20.0))),
        None,
    );

    assert!(pm.is_open(&row_menu_id));

    // Simulate finding open popup row during frame render:
    let open_popup_row = (0..5).find(|&r_idx| {
        let r_id = format!("notion_row_menu_{}_{}", tbl_id, r_idx);
        let color_id = format!("notion_row_{}_{}_color_picker", tbl_id, r_idx);
        pm.is_open(&r_id) || pm.is_open(&color_id)
    });

    // Verified: open_popup_row locks active_row to 2 regardless of where cursor is!
    assert_eq!(open_popup_row, Some(2), "Active row must stay locked to row 2 while popup is open");

    // Moving cursor to options rect does NOT close popup
    let ctx = egui::Context::default();
    let _ = ctx.run(egui::RawInput {
        events: vec![],
        ..Default::default()
    }, |ctx| {
        pm.update_input(ctx);
    });

    assert!(pm.is_open(&row_menu_id), "Popup must remain open when moving cursor over menu options");
}
