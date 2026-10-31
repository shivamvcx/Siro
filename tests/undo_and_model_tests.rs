use egui::{Color32, Pos2, Vec2};
use siro::model::board::Board;
use siro::model::electronics::ComponentKind;
use siro::model::element::*;
use siro::model::planner::PlannerData;
use siro::model::project::Project;
use siro::model::undo::UndoStack;
use siro::storage::siro_file::SiroFileManager;
use uuid::Uuid;

#[test]
fn test_transactional_undo_stack_bounded_depth() {
    let mut undo_stack = UndoStack::new(100);

    // Push 150 states
    for i in 0..150 {
        let dummy_element = BoardElement::Text(TextElement::new_text(
            Uuid::new_v4(),
            Pos2::new(i as f32, 0.0),
            format!("Text {}", i),
        ));
        undo_stack.push_checkpoint(vec![dummy_element]);
    }

    // Depth must be strictly bounded at 100
    assert_eq!(undo_stack.depth(), 100);

    // Undo 50 times
    let mut current = vec![];
    for _ in 0..50 {
        if let Some(prev) = undo_stack.undo(current.clone()) {
            current = prev;
        }
    }

    assert_eq!(undo_stack.depth(), 50);
    assert!(undo_stack.can_undo());
    assert!(undo_stack.can_redo());

    // Introducing a new mutation must invalidate redo
    undo_stack.push_checkpoint(vec![]);
    assert!(!undo_stack.can_redo());
}

#[test]
fn test_magnetic_wire_latching_and_soft_anchor_fallback() {
    let mut board = Board::new("Circuit Test");

    // Place ESP32
    let esp = ComponentElement::new(Uuid::new_v4(), ComponentKind::Esp32Wroom, Pos2::new(100.0, 100.0));
    let esp_id = esp.id;
    let esp_pin_world = esp.terminal_world_pos(0).unwrap();
    board.add_element(BoardElement::Component(esp));

    // Place Resistor
    let resistor = ComponentElement::new(Uuid::new_v4(), ComponentKind::Resistor, Pos2::new(350.0, 100.0));
    let r_id = resistor.id;
    let r_pin_world = resistor.terminal_world_pos(0).unwrap();
    board.add_element(BoardElement::Component(resistor));

    // Connect wire
    let wire = WireElement::new(
        Uuid::new_v4(),
        WireEndpoint {
            element_id: Some(esp_id),
            terminal_idx: Some(0),
            pos: esp_pin_world,
        },
        WireEndpoint {
            element_id: Some(r_id),
            terminal_idx: Some(0),
            pos: r_pin_world,
        },
        WireColor::Red5V,
    );
    let wire_id = wire.id;
    board.add_element(BoardElement::Wire(wire));

    // Move ESP32 by +50px X, +30px Y
    if let Some(BoardElement::Component(comp)) = board.get_element_mut(esp_id) {
        comp.pos += Vec2::new(50.0, 30.0);
    }

    // Synchronize wire endpoints
    board.sync_connected_wires();

    // Verify wire start endpoint moved along with the ESP32
    if let Some(BoardElement::Wire(w)) = board.get_element(wire_id) {
        assert_eq!(w.start.element_id, Some(esp_id));
        assert_eq!(w.start.pos, esp_pin_world + Vec2::new(50.0, 30.0));
    } else {
        panic!("Wire not found");
    }

    // Now delete the ESP32 (Defense #6: Soft-anchor fallback)
    board.remove_element(esp_id);

    // Verify wire still exists without panic, decoupled to a floating terminal at the exact previous position
    if let Some(BoardElement::Wire(w)) = board.get_element(wire_id) {
        assert_eq!(w.start.element_id, None, "Expected decoupled start endpoint");
        assert_eq!(w.start.terminal_idx, None);
        assert_eq!(w.start.pos, esp_pin_world + Vec2::new(50.0, 30.0));
        assert_eq!(w.end.element_id, Some(r_id), "End endpoint should still be attached to resistor");
    } else {
        panic!("Wire deleted unexpectedly");
    }
}

#[test]
fn test_bom_calculation_and_canvas_table_sync() {
    let mut planner = PlannerData::default();
    planner.bom.clear();

    planner.bom.push(siro::model::planner::BomItem::new("RP2040 Pico", "MCU", 2, 5.00));
    planner.bom.push(siro::model::planner::BomItem::new("OLED", "Display", 1, 8.50));
    planner.bom.push(siro::model::planner::BomItem::new("Resistor", "10k 0805", 10, 0.15));

    // Mark OLED as acquired
    planner.bom[1].acquired = true;

    let total = planner.total_cost();
    let expected_total = 2.0 * 5.00 + 1.0 * 8.50 + 10.0 * 0.15; // 10.0 + 8.50 + 1.50 = 20.00
    assert!((total - expected_total).abs() < 1e-4);

    let acquired = planner.acquired_cost();
    assert!((acquired - 8.50).abs() < 1e-4);

    let remaining = planner.remaining_cost();
    assert!((remaining - 11.50).abs() < 1e-4);

    // Sync to canvas table
    let table = planner.to_canvas_table(Pos2::new(100.0, 100.0));
    assert_eq!(table.rows, 4); // 1 header + 3 items
    assert_eq!(table.cols, 6);
    assert_eq!(table.cells[0][0], "Item / Part");
    assert_eq!(table.cells[1][0], "RP2040 Pico");
    assert_eq!(table.cells[2][5], "Acquired ✓");
    assert_eq!(table.cells[3][5], "Needed");
}

#[test]
fn test_atomic_siro_serialization_roundtrip() {
    let temp_dir = std::env::temp_dir().join(format!("siro_test_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let test_file = temp_dir.join("test_project.siro");

    let mut project = Project::new("Robotics Project");
    let board1_id = project.active_board_id;

    // Add board 2 and board 3
    let board2_id = project.add_board("Power Subsystem");
    let board3_id = project.add_board("Sensors & Telemetry");

    // Populate board 1 with components and wires
    let pico = ComponentElement::new(Uuid::new_v4(), ComponentKind::RaspberryPiPico, Pos2::new(50.0, 50.0));
    let pico_id = pico.id;
    let r1 = ComponentElement::new(Uuid::new_v4(), ComponentKind::Resistor, Pos2::new(250.0, 80.0));
    let r1_id = r1.id;
    let wire = WireElement::new(
        Uuid::new_v4(),
        WireEndpoint { element_id: Some(pico_id), terminal_idx: Some(0), pos: Pos2::new(50.0, 65.0) },
        WireEndpoint { element_id: Some(r1_id), terminal_idx: Some(0), pos: Pos2::new(250.0, 95.0) },
        WireColor::YellowSignal,
    );
    let note = TextElement::new_sticky(Uuid::new_v4(), Pos2::new(400.0, 50.0), "Check 3.3V pin".into(), Color32::YELLOW);

    let b1 = project.boards.get_mut(&board1_id).unwrap();
    b1.add_element(BoardElement::Component(pico));
    b1.add_element(BoardElement::Component(r1));
    b1.add_element(BoardElement::Wire(wire));
    b1.add_element(BoardElement::Text(note));

    // Save project
    SiroFileManager::save_project(&mut project, &test_file).expect("Save failed");
    assert!(test_file.exists());

    // Load project back
    let loaded = SiroFileManager::load_project(&test_file).expect("Load failed");

    assert_eq!(loaded.manifest.name, "Robotics Project");
    assert_eq!(loaded.manifest.board_order.len(), 3);
    assert!(loaded.boards.contains_key(&board1_id));
    assert!(loaded.boards.contains_key(&board2_id));
    assert!(loaded.boards.contains_key(&board3_id));

    let loaded_b1 = loaded.boards.get(&board1_id).unwrap();
    assert_eq!(loaded_b1.elements.len(), 4);

    // Clean up
    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_component_rotation_dimensions_and_pin_mapping() {
    let mut comp = ComponentElement::new(Uuid::new_v4(), ComponentKind::Resistor, Pos2::new(100.0, 100.0));
    // Default resistor: width = 100.0, height = 30.0
    assert_eq!(comp.width, 100.0);
    assert_eq!(comp.height, 30.0);

    // Rotation 0
    let aabb0 = comp.aabb();
    assert_eq!(aabb0.width(), 100.0);
    assert_eq!(aabb0.height(), 30.0);
    let term0_r0 = comp.terminal_world_pos(0).unwrap();
    let term1_r0 = comp.terminal_world_pos(1).unwrap();
    assert_eq!(term0_r0, Pos2::new(100.0, 115.0));
    assert_eq!(term1_r0, Pos2::new(200.0, 115.0));

    // Rotation 1 (90 degrees) - Width & height MUST swap!
    comp.rotation_quarters = 1;
    let aabb1 = comp.aabb();
    assert_eq!(aabb1.width(), 30.0);
    assert_eq!(aabb1.height(), 100.0);
    let term0_r1 = comp.terminal_world_pos(0).unwrap();
    let term1_r1 = comp.terminal_world_pos(1).unwrap();
    assert_eq!(term0_r1, Pos2::new(115.0, 100.0));
    assert_eq!(term1_r1, Pos2::new(115.0, 200.0));

    // Rotation 2 (180 degrees)
    comp.rotation_quarters = 2;
    let aabb2 = comp.aabb();
    assert_eq!(aabb2.width(), 100.0);
    assert_eq!(aabb2.height(), 30.0);
    let term0_r2 = comp.terminal_world_pos(0).unwrap();
    let term1_r2 = comp.terminal_world_pos(1).unwrap();
    assert_eq!(term0_r2, Pos2::new(200.0, 115.0));
    assert_eq!(term1_r2, Pos2::new(100.0, 115.0));

    // Rotation 3 (270 degrees)
    comp.rotation_quarters = 3;
    let aabb3 = comp.aabb();
    assert_eq!(aabb3.width(), 30.0);
    assert_eq!(aabb3.height(), 100.0);
    let term0_r3 = comp.terminal_world_pos(0).unwrap();
    let term1_r3 = comp.terminal_world_pos(1).unwrap();
    assert_eq!(term0_r3, Pos2::new(115.0, 200.0));
    assert_eq!(term1_r3, Pos2::new(115.0, 100.0));
}

#[test]
fn test_autosave_preserves_project_filepath_and_dirty_flag() {
    let temp_dir = std::env::temp_dir().join(format!("siro_autosave_test_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let user_path = temp_dir.join("user_work.siro");
    let recovery_target = temp_dir.join("recovery_backup.siro");

    let mut project = Project::new("Autosave Test Project");
    project.filepath = Some(user_path.clone());
    project.mark_edited();

    // Call save_project_to_path (the isolated autosave persistence function)
    SiroFileManager::save_project_to_path(&project, &recovery_target).expect("Autosave target write must succeed");

    // CRITICAL: Verify project metadata was NOT hijacked or wiped!
    assert_eq!(project.filepath, Some(user_path), "Autosave must NOT overwrite user project filepath");
    assert!(project.is_unsaved(), "Autosave must NOT reset project dirty flag");

    // Verify recovery file is a valid Siro project
    assert!(recovery_target.exists());
    let loaded = SiroFileManager::load_project(&recovery_target).expect("Recovery file must deserialize cleanly");
    assert_eq!(loaded.manifest.name, "Autosave Test Project");

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_table_row_col_mutations_and_resizing() {
    use siro::canvas::tools::table_tool::TableTool;
    let mut table = TableElement::new(Uuid::new_v4(), Pos2::new(50.0, 50.0), 2, 3);
    assert_eq!(table.rows, 2);
    assert_eq!(table.cols, 3);
    assert_eq!(table.col_widths.len(), 3);
    assert_eq!(table.cells.len(), 2);

    // Add row
    TableTool::add_row(&mut table);
    assert_eq!(table.rows, 3);
    assert_eq!(table.cells.len(), 3);
    assert_eq!(table.cells[2].len(), 3);

    // Add column
    TableTool::add_column(&mut table);
    assert_eq!(table.cols, 4);
    assert_eq!(table.col_widths.len(), 4);
    for row in &table.cells {
        assert_eq!(row.len(), 4);
    }

    // Resize column
    let orig_w = table.col_widths[0];
    TableTool::resize_columns(&mut table, 20.0);
    assert_eq!(table.col_widths[0], orig_w + 20.0);

    // Remove row
    TableTool::remove_row(&mut table, 1);
    assert_eq!(table.rows, 2);
    assert_eq!(table.cells.len(), 2);

    // Remove column
    TableTool::remove_column(&mut table, 3);
    assert_eq!(table.cols, 3);
    assert_eq!(table.col_widths.len(), 3);
}

#[test]
fn test_element_duplication_and_id_regeneration() {
    let orig_id = Uuid::new_v4();
    let sticky = TextElement::new_sticky(
        orig_id,
        Pos2::new(100.0, 150.0),
        "Miro Sticky Note".to_string(),
        Color32::from_rgb(255, 249, 177),
    );
    let elem = BoardElement::Text(sticky);

    let delta = Vec2::new(24.0, 24.0);
    let dup = elem.duplicate_with_offset(delta);

    assert_ne!(dup.id(), orig_id);
    if let BoardElement::Text(dup_text) = dup {
        assert_eq!(dup_text.pos, Pos2::new(124.0, 174.0));
        assert_eq!(dup_text.text, "Miro Sticky Note");
        assert_eq!(dup_text.sticky_color, Color32::from_rgb(255, 249, 177));
        assert!(dup_text.is_sticky);
    } else {
        panic!("Duplicated element is not a text element");
    }
}

#[test]
fn test_project_clone_preserves_board_state() {
    let mut proj = Project::new("Embedded Drone Firmware");
    let sticky = TextElement::new_sticky(
        Uuid::new_v4(),
        Pos2::new(50.0, 50.0),
        "Pinout Mapping".to_string(),
        Color32::from_rgb(182, 207, 251),
    );
    proj.active_board_mut().add_element(BoardElement::Text(sticky));
    proj.mark_edited();

    // Clone project (simulating transition to Home Screen)
    let cloned_proj = proj.clone();

    assert_eq!(cloned_proj.manifest.name, "Embedded Drone Firmware");
    assert!(cloned_proj.is_unsaved());
    assert_eq!(cloned_proj.active_board().elements.len(), 1);
}

#[test]
fn test_b1_scenario_s1_save_untitled_synchronizes_name() {
    let mut proj = Project::new("Untitled 1");
    assert_eq!(proj.display_name(), "Untitled 1");
    assert_eq!(proj.manifest.name, "Untitled 1");

    let temp_dir = std::env::temp_dir().join(format!("siro_test_s1_{}", Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let save_path = temp_dir.join("Q3 Roadmap.siro");

    // Simulating save dialog workflow: manifest.name is set to file stem
    if let Some(stem) = save_path.file_stem().and_then(|s| s.to_str()) {
        proj.manifest.name = stem.to_string();
    }
    let res = SiroFileManager::save_project(&mut proj, &save_path);
    assert!(res.is_ok(), "Save should succeed: {:?}", res);

    assert_eq!(proj.display_name(), "Q3 Roadmap");
    assert_eq!(proj.manifest.name, "Q3 Roadmap");
    assert_eq!(proj.filepath, Some(save_path.clone()));
    assert!(!proj.is_unsaved());

    // Verify written manifest inside zip also contains synchronized name
    let loaded = SiroFileManager::load_project(&save_path).expect("Should load saved project");
    assert_eq!(loaded.display_name(), "Q3 Roadmap");
    assert_eq!(loaded.manifest.name, "Q3 Roadmap");

    let _ = std::fs::remove_file(&save_path);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_b1_scenario_s2_load_project_synchronizes_with_file_stem() {
    let mut proj = Project::new("InitialName");
    let temp_dir = std::env::temp_dir().join(format!("siro_test_s2_{}", Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let orig_path = temp_dir.join("InitialName.siro");
    let renamed_path = temp_dir.join("RenamedExternally.siro");

    SiroFileManager::save_project(&mut proj, &orig_path).expect("Save initial");
    std::fs::rename(&orig_path, &renamed_path).expect("Rename file on disk");

    let loaded = SiroFileManager::load_project(&renamed_path).expect("Load renamed");
    // display_name() is authoritative and matches the disk file stem
    assert_eq!(loaded.display_name(), "RenamedExternally");

    let _ = std::fs::remove_file(&renamed_path);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_b1_scenario_s3_save_as_synchronizes_new_stem() {
    let mut proj = Project::new("ProjectA");
    let temp_dir = std::env::temp_dir().join(format!("siro_test_s3_{}", Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let path_a = temp_dir.join("ProjectA.siro");
    let path_b = temp_dir.join("ProjectB.siro");

    SiroFileManager::save_project(&mut proj, &path_a).expect("Save A");
    assert_eq!(proj.display_name(), "ProjectA");

    // Save As ProjectB: manifest.name is updated to target stem
    if let Some(stem) = path_b.file_stem().and_then(|s| s.to_str()) {
        proj.manifest.name = stem.to_string();
    }
    SiroFileManager::save_project(&mut proj, &path_b).expect("Save As B");
    assert_eq!(proj.display_name(), "ProjectB");
    assert_eq!(proj.manifest.name, "ProjectB");
    assert_eq!(proj.filepath, Some(path_b.clone()));

    assert!(path_a.exists(), "Original ProjectA.siro must remain untouched");
    assert!(path_b.exists(), "New ProjectB.siro must exist");

    let loaded_b = SiroFileManager::load_project(&path_b).expect("Load Project B");
    assert_eq!(loaded_b.display_name(), "ProjectB");
    assert_eq!(loaded_b.manifest.name, "ProjectB");

    let _ = std::fs::remove_file(&path_a);
    let _ = std::fs::remove_file(&path_b);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_b1_scenario_s4_reopen_saved_file_shows_matching_name() {
    let mut proj = Project::new("OldDraft");
    let temp_dir = std::env::temp_dir().join(format!("siro_test_s4_{}", Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let path = temp_dir.join("Meeting Notes.siro");

    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
        proj.manifest.name = stem.to_string();
    }
    SiroFileManager::save_project(&mut proj, &path).expect("Save Meeting Notes");

    let loaded = SiroFileManager::load_project(&path).expect("Reopen Meeting Notes");
    assert_eq!(loaded.display_name(), "Meeting Notes");
    assert_eq!(loaded.manifest.name, "Meeting Notes");

    let _ = std::fs::remove_file(&path);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_b1_in_app_rename_synchronizes_disk_recents_starred_and_manifest() {
    use siro::app::SiroApp;

    let temp_dir = std::env::temp_dir().join(format!("siro_test_rename_{}", Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let orig_path = temp_dir.join("OriginalBoard.siro");

    let mut app = SiroApp::default();
    app.home.persist_on_change = false;

    let mut proj = Project::new("OriginalBoard");
    SiroFileManager::save_project(&mut proj, &orig_path).expect("Initial save");
    app.home.add_recent(&orig_path);
    app.home.toggle_star(&orig_path);
    assert!(app.home.is_starred(&orig_path));

    // Execute in-app rename to "RenamedBoard"
    app.rename_project(&mut proj, "RenamedBoard");

    let expected_new_path = temp_dir.join("RenamedBoard.siro");
    assert!(!orig_path.exists(), "Original file must be gone");
    assert!(expected_new_path.exists(), "Renamed file must exist on disk");
    assert_eq!(proj.display_name(), "RenamedBoard");
    assert_eq!(proj.manifest.name, "RenamedBoard");
    assert_eq!(proj.filepath, Some(expected_new_path.clone()));

    // Recents and starred must be synchronized to new path
    assert!(!app.home.is_starred(&orig_path));
    assert!(app.home.is_starred(&expected_new_path));
    assert!(app.home.recent_files.iter().any(|p| p == &expected_new_path));
    assert!(!app.home.recent_files.iter().any(|p| p == &orig_path));

    // Re-load from disk and verify manifest
    let loaded = SiroFileManager::load_project(&expected_new_path).expect("Reload renamed project");
    assert_eq!(loaded.display_name(), "RenamedBoard");
    assert_eq!(loaded.manifest.name, "RenamedBoard");

    let _ = std::fs::remove_file(&expected_new_path);
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_board_name_sanitization_and_extension_safety() {
    use siro::storage::siro_file::sanitize_board_name;

    assert_eq!(sanitize_board_name("Clean Title"), "Clean Title");
    assert_eq!(sanitize_board_name("My Board.siro"), "My Board");
    assert_eq!(sanitize_board_name("Nested.SIRO"), "Nested");
    assert_eq!(sanitize_board_name("Board: Phase 1/2 <v3>?*"), "Board_ Phase 1_2 _v3___");
    assert_eq!(sanitize_board_name("  Trailing Dot.  "), "Trailing Dot");
    assert_eq!(sanitize_board_name("   "), "Untitled Project");
}

#[test]
fn test_case_only_rename_on_windows() {
    use siro::app::SiroApp;

    let temp_dir = std::env::temp_dir().join(format!("siro_test_case_{}", Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let lower_path = temp_dir.join("lowercase.siro");

    let mut app = SiroApp::default();
    app.home.persist_on_change = false;

    let mut proj = Project::new("lowercase");
    SiroFileManager::save_project(&mut proj, &lower_path).expect("Initial lower save");

    // Perform case-only rename to "LowerCase"
    app.rename_project(&mut proj, "LowerCase");
    assert_eq!(proj.display_name(), "LowerCase");
    assert_eq!(proj.manifest.name, "LowerCase");

    let _ = std::fs::remove_file(temp_dir.join("LowerCase.siro"));
    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_generic_ic_pin_clamping_zero_and_odd() {
    let ic_zero = ComponentKind::GenericIC {
        pin_count: 0,
        labels: vec![],
    };
    let pins_zero = ic_zero.generate_terminals(80.0, 100.0);
    assert!(pins_zero.len() >= 2, "Must clamp to at least 2 pins");
    for pin in &pins_zero {
        assert!(pin.local_pos.x.is_finite());
        assert!(pin.local_pos.y.is_finite());
    }

    let ic_odd = ComponentKind::GenericIC {
        pin_count: 5,
        labels: vec![],
    };
    let pins_odd = ic_odd.generate_terminals(80.0, 100.0);
    assert_eq!(pins_odd.len(), 4, "5 pins clamped to 4 (even per side)");
    for pin in &pins_odd {
        assert!(pin.local_pos.x.is_finite());
        assert!(pin.local_pos.y.is_finite());
    }
}

#[test]
fn test_sheet_rename_logic_and_dirty_flag() {
    let mut proj = Project::new("Robotics Control System");
    let initial_sheet_id = proj.active_board_id;
    assert_eq!(proj.active_board().name, "Main Schematic");
    proj.mark_saved();

    // 1. Rename active sheet to "Power Subsystem"
    let changed = proj.rename_board(initial_sheet_id, "Power Subsystem");
    assert!(changed, "Rename to a new name must return true");
    assert_eq!(proj.active_board().name, "Power Subsystem");
    assert!(proj.is_unsaved(), "Renaming a sheet must set project.dirty to true");

    // 2. Renaming to identical name is a no-op
    proj.mark_saved();
    let changed_same = proj.rename_board(initial_sheet_id, "Power Subsystem");
    assert!(!changed_same, "Renaming to the identical name must return false");
    assert!(!proj.is_unsaved(), "No-op rename must not mark project dirty");

    // 3. Changing letter case only is allowed
    let changed_case = proj.rename_board(initial_sheet_id, "power subsystem");
    assert!(changed_case, "Case-only rename must return true");
    assert_eq!(proj.active_board().name, "power subsystem");
    assert!(proj.is_unsaved(), "Case-only rename must mark project dirty");

    // 4. Stable UUID: the sheet ID is identical before and after rename
    assert_eq!(proj.active_board_id, initial_sheet_id);

    // 5. Add second sheet and verify sheet_names()
    let sheet2_id = proj.add_board("Sensors");
    let names = proj.sheet_names();
    assert_eq!(names.len(), 2);
    assert_eq!(names[0], "power subsystem");
    assert_eq!(names[1], "Sensors");

    // Rename sheet 2
    proj.rename_board(sheet2_id, "Telemetry & IMU");
    assert_eq!(proj.boards.get(&sheet2_id).unwrap().name, "Telemetry & IMU");
}

#[test]
fn test_legacy_format_board_deserialization_and_roundtrip() {
    // Legacy board JSON lacking 'name', 'camera_pan', 'camera_zoom' fields
    let legacy_json = r#"{
        "id": "11111111-2222-3333-4444-555555555555",
        "elements": []
    }"#;

    let board: Board = serde_json::from_str(legacy_json).expect("Legacy board must deserialize with serde defaults");
    assert_eq!(board.id, Uuid::parse_str("11111111-2222-3333-4444-555555555555").unwrap());
    assert_eq!(board.name, "Sheet", "Missing name in legacy JSON must default to 'Sheet'");
    assert_eq!(board.camera_zoom, 1.0, "Missing camera_zoom must default to 1.0");
    assert_eq!(board.camera_pan, Pos2::ZERO, "Missing camera_pan must default to Pos2::ZERO");

    // Full roundtrip via SiroFileManager: build project with legacy board and verify save & load
    let temp_dir = std::env::temp_dir().join(format!("siro_test_legacy_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let file_path = temp_dir.join("Legacy Compat.siro");

    let mut proj = Project::new("Legacy Compat");
    let b_id = proj.active_board_id;
    proj.rename_board(b_id, "Custom Schematic");

    SiroFileManager::save_project(&mut proj, &file_path).expect("Save must succeed");
    let loaded = SiroFileManager::load_project(&file_path).expect("Load must succeed");

    assert_eq!(loaded.display_name(), "Legacy Compat");
    assert_eq!(loaded.boards.get(&b_id).unwrap().name, "Custom Schematic");

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_shortcut_registry_triggering_and_isolation() {
    use siro::shortcuts::AppShortcut;

    // Check all shortcuts are registered
    let all = AppShortcut::all();
    assert_eq!(all.len(), 4);
    assert_eq!(all[0], AppShortcut::NewProject);
    assert_eq!(all[1], AppShortcut::OpenProject);
    assert_eq!(all[2], AppShortcut::SaveProject);
    assert_eq!(all[3], AppShortcut::SaveProjectAs);

    // Verify platform-appropriate labels
    for s in all {
        let menu_label = s.menu_label();
        let display = s.label();
        assert!(menu_label.contains(&display), "Menu label '{}' must contain display label '{}'", menu_label, display);
        #[cfg(target_os = "macos")]
        assert!(display.starts_with("Cmd+"), "macOS shortcuts must start with Cmd+");
        #[cfg(not(target_os = "macos"))]
        assert!(display.starts_with("Ctrl+"), "Non-macOS shortcuts must start with Ctrl+");
    }

    // Context test: simulate keyboard events
    // 1. Without command modifier, N alone must NOT trigger
    {
        let ctx = egui::Context::default();
        let mut raw_n_only = egui::RawInput::default();
        raw_n_only.events.push(egui::Event::Key {
            key: egui::Key::N,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        });
        let _ = ctx.run(raw_n_only, |ctx| {
            assert!(!AppShortcut::NewProject.is_triggered(ctx), "N alone must not trigger NewProject");
        });
    }

    // 2. With command modifier, Ctrl+N MUST trigger on initial press
    {
        let ctx = egui::Context::default();
        let mut raw_ctrl_n = egui::RawInput::default();
        raw_ctrl_n.modifiers = egui::Modifiers::COMMAND;
        raw_ctrl_n.events.push(egui::Event::Key {
            key: egui::Key::N,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        });
        let _ = ctx.run(raw_ctrl_n, |ctx| {
            assert!(AppShortcut::NewProject.is_triggered(ctx), "Ctrl+N must trigger NewProject");
        });
    }

    // 3. Repeat events (holding down key across frames) must NOT trigger shortcut on subsequent frames
    {
        let ctx = egui::Context::default();
        // Frame 1: initial press triggers
        let mut raw_press = egui::RawInput::default();
        raw_press.modifiers = egui::Modifiers::COMMAND;
        raw_press.events.push(egui::Event::Key {
            key: egui::Key::N,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::COMMAND,
        });
        let _ = ctx.run(raw_press, |ctx| {
            assert!(AppShortcut::NewProject.is_triggered(ctx), "Initial press must trigger NewProject");
        });

        // Frame 2: key is held down (egui automatically treats as repeat)
        let mut raw_held = egui::RawInput::default();
        raw_held.modifiers = egui::Modifiers::COMMAND;
        raw_held.events.push(egui::Event::Key {
            key: egui::Key::N,
            physical_key: None,
            pressed: true,
            repeat: true,
            modifiers: egui::Modifiers::COMMAND,
        });
        let _ = ctx.run(raw_held, |ctx| {
            assert!(!AppShortcut::NewProject.is_triggered(ctx), "Holding key down (repeat event) must not trigger NewProject");
        });
    }

    // 4. Ctrl+Shift+S triggers SaveProjectAs, not SaveProject
    {
        let ctx = egui::Context::default();
        let mut raw_save_as = egui::RawInput::default();
        let cmd_shift = egui::Modifiers {
            command: true,
            shift: true,
            ..Default::default()
        };
        raw_save_as.modifiers = cmd_shift;
        raw_save_as.events.push(egui::Event::Key {
            key: egui::Key::S,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: cmd_shift,
        });
        let _ = ctx.run(raw_save_as, |ctx| {
            assert!(AppShortcut::SaveProjectAs.is_triggered(ctx), "Ctrl+Shift+S must trigger SaveProjectAs");
            assert!(!AppShortcut::SaveProject.is_triggered(ctx), "Ctrl+Shift+S must not trigger SaveProject");
        });
    }
}

#[test]
fn test_create_and_open_new_project_autosaves_dirty_project() {
    use siro::app::SiroApp;

    let temp_dir = std::env::temp_dir().join(format!("siro_test_new_proj_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let old_project_path = temp_dir.join("ActiveWork.siro");

    let mut app = SiroApp::default();
    app.home.persist_on_change = false;

    // Simulate being in the editor with a dirty project that has a file path
    let mut active_proj = Project::new("ActiveWork");
    SiroFileManager::save_project(&mut active_proj, &old_project_path).unwrap();
    active_proj.active_board_mut().add_element(BoardElement::Text(TextElement::new_sticky(
        Uuid::new_v4(),
        Pos2::new(10.0, 10.0),
        "Important Work".to_string(),
        Color32::YELLOW,
    )));
    active_proj.mark_edited();

    app.open_projects = vec![active_proj];
    app.active_project_idx = Some(0);
    app.state = siro::app::AppState::Editor;

    // Call create_and_open_new_project()
    app.create_and_open_new_project();

    // 1. A new project was created and is active
    assert_eq!(app.open_projects.len(), 2);
    assert_eq!(app.active_project_idx, Some(1));
    assert_eq!(app.open_projects[1].display_name(), "Untitled Project");
    assert_eq!(app.state, siro::app::AppState::Editor);

    // 2. The previous dirty project was saved to disk before creating the new project
    let reloaded = SiroFileManager::load_project(&old_project_path).expect("Old project must have been autosaved");
    assert_eq!(reloaded.active_board().elements.len(), 1, "Autosaved project must have saved changes");

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_open_file_autosaves_dirty_project() {
    use siro::app::SiroApp;

    let temp_dir = std::env::temp_dir().join(format!("siro_test_open_autosave_{}", Uuid::new_v4()));
    std::fs::create_dir_all(&temp_dir).unwrap();
    let current_project_path = temp_dir.join("CurrentDirty.siro");
    let other_project_path = temp_dir.join("OtherProject.siro");

    let mut current_proj = Project::new("CurrentDirty");
    SiroFileManager::save_project(&mut current_proj, &current_project_path).unwrap();
    current_proj.active_board_mut().add_element(BoardElement::Text(TextElement::new_sticky(
        Uuid::new_v4(),
        Pos2::new(20.0, 20.0),
        "Dirty Text".to_string(),
        Color32::GREEN,
    )));
    current_proj.mark_edited();

    let mut other_proj = Project::new("OtherProject");
    SiroFileManager::save_project(&mut other_proj, &other_project_path).unwrap();

    let mut app = SiroApp::default();
    app.home.persist_on_change = false;
    app.open_projects = vec![current_proj];
    app.active_project_idx = Some(0);
    app.state = siro::app::AppState::Editor;

    // Open the other project file
    app.open_or_focus_file(&other_project_path);

    // Current dirty project must have been autosaved to disk
    let reloaded_current = SiroFileManager::load_project(&current_project_path).expect("Reload current");
    assert_eq!(reloaded_current.active_board().elements.len(), 1, "Dirty changes must be saved");

    // Other project is now active in tab 1
    assert_eq!(app.open_projects.len(), 2);
    assert_eq!(app.active_project_idx, Some(1));

    let _ = std::fs::remove_dir_all(temp_dir);
}

#[test]
fn test_menu_width_set_on_context_for_shrinkwrap() {
    let ctx = egui::Context::default();
    assert_eq!(ctx.style().spacing.menu_width, 400.0, "egui default is 400.0");
    ctx.style_mut(|s| s.spacing.menu_width = 180.0);
    assert_eq!(ctx.style().spacing.menu_width, 180.0, "Style spacing menu_width must be 180.0 for shrink-wrapped menus");
}

#[test]
fn test_sheet_name_unicode_and_invisible_filtering() {
    use siro::model::sheet_ops::validate_sheet_name;

    let existing = ["Alimentation", "Übersicht", "Sensors"];
    // Case-insensitive Unicode match must reject duplicate
    assert!(validate_sheet_name("übersicht", &existing, None).is_err());
    assert!(validate_sheet_name("ÜBERSICHT", &existing, None).is_err());
    assert!(validate_sheet_name("alimentation", &existing, None).is_err());

    // Invisible control / formatting characters must be stripped or rejected
    assert!(validate_sheet_name("\u{200B}", &existing, None).is_err());
    assert!(validate_sheet_name("  \u{FEFF}  ", &existing, None).is_err());
    assert_eq!(
        validate_sheet_name("  Sensor\u{200B}  Board  ", &existing, None).unwrap(),
        "Sensor Board"
    );
}

