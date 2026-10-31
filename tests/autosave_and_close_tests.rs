use siro::model::element::{BoardElement, TextElement};
use siro::model::planner::{BomItem, GoalItem};
use siro::model::project::{has_content, is_unsaved, Project};
use siro::storage::autosave::is_autosave_eligible;
use siro::storage::settings::{
    load_settings_from, save_settings_to, AppSettings, DEFAULT_AUTOSAVE_INTERVAL_SECS,
    MAX_AUTOSAVE_INTERVAL_SECS, MIN_AUTOSAVE_INTERVAL_SECS,
};
use siro::ui::close_dialog::{app_close_plan, close_plan, ClosePlan};
use std::path::PathBuf;
use uuid::Uuid;

#[test]
fn test_close_plan_pure_logic() {
    // 0 open projects -> Allow
    let empty: Vec<Project> = Vec::new();
    assert_eq!(app_close_plan(&empty), ClosePlan::Allow);

    // 1 open, no content, unsaved -> Allow (pristine/empty project can be closed without nagging)
    let mut p_empty = Project::new("Untitled");
    p_empty.mark_edited();
    assert!(!p_empty.has_content());
    assert!(p_empty.is_unsaved());
    assert_eq!(close_plan(p_empty.has_content() && p_empty.is_unsaved()), ClosePlan::Allow);
    assert_eq!(app_close_plan(std::slice::from_ref(&p_empty)), ClosePlan::Allow);

    // 1 open, has content, not unsaved -> Allow (already saved)
    let mut p_saved = Project::new("Saved Work");
    p_saved.active_board_mut().add_element(BoardElement::Text(TextElement::new_sticky(
        Uuid::new_v4(),
        egui::pos2(10.0, 10.0),
        "Important Note".to_string(),
        egui::Color32::YELLOW,
    )));
    p_saved.mark_saved();
    assert!(p_saved.has_content());
    assert!(!p_saved.is_unsaved());
    assert_eq!(close_plan(p_saved.has_content() && p_saved.is_unsaved()), ClosePlan::Allow);
    assert_eq!(app_close_plan(std::slice::from_ref(&p_saved)), ClosePlan::Allow);

    // 1 open, has content, unsaved -> Ask
    let mut p_dirty = Project::new("Dirty Work");
    p_dirty.active_board_mut().add_element(BoardElement::Text(TextElement::new_sticky(
        Uuid::new_v4(),
        egui::pos2(20.0, 20.0),
        "Unsaved note".to_string(),
        egui::Color32::GREEN,
    )));
    p_dirty.mark_edited();
    assert!(p_dirty.has_content());
    assert!(p_dirty.is_unsaved());
    assert_eq!(close_plan(p_dirty.has_content() && p_dirty.is_unsaved()), ClosePlan::Ask);
    assert_eq!(app_close_plan(std::slice::from_ref(&p_dirty)), ClosePlan::Ask);

    // 2 open: one pristine, one has-content+unsaved -> Ask
    let projects = vec![p_empty, p_dirty];
    assert_eq!(app_close_plan(&projects), ClosePlan::Ask);
}

#[test]
fn test_revision_counter_is_unsaved() {
    let mut proj = Project::new("Test Counters");

    // Project::new -> false
    assert!(!proj.is_unsaved());
    assert!(!is_unsaved(&proj));

    // mark_edited -> true
    proj.mark_edited();
    assert!(proj.is_unsaved());
    assert!(is_unsaved(&proj));

    // mark_saved -> false
    proj.mark_saved();
    assert!(!proj.is_unsaved());
    assert!(!is_unsaved(&proj));

    // 2 edits + 1 save -> false
    proj.mark_edited();
    proj.mark_edited();
    assert!(proj.is_unsaved());
    proj.mark_saved();
    assert!(!proj.is_unsaved());

    // 2 edits + 1 save + 1 edit -> true
    proj.mark_edited();
    assert!(proj.is_unsaved());
}

#[test]
fn test_has_content_detection() {
    // Project::new -> false
    let mut proj = Project::new("Default");
    assert!(!proj.has_content());
    assert!(!has_content(&proj));

    // add sticky -> true
    let elem_id = Uuid::new_v4();
    proj.active_board_mut().add_element(BoardElement::Text(TextElement::new_sticky(
        elem_id,
        egui::pos2(50.0, 50.0),
        "Chip Pinout".to_string(),
        egui::Color32::WHITE,
    )));
    assert!(proj.has_content());

    // remove sticky -> false
    proj.active_board_mut().elements.clear();
    assert!(!proj.has_content());

    // custom sheet name -> true
    let sheet_id = proj.active_board_id;
    proj.rename_board(sheet_id, "Custom Schematic");
    assert!(proj.has_content());

    // reset sheet name to default -> false
    proj.rename_board(sheet_id, "Main Schematic");
    assert!(!proj.has_content());

    // planner milestone -> true
    proj.planner.goals.push(GoalItem {
        id: Uuid::new_v4(),
        title: "Ship Firmware".to_string(),
        completed: false,
        is_hackclub_milestone: false,
    });
    assert!(proj.has_content());
    proj.planner = siro::model::planner::PlannerData::default();
    assert!(!proj.has_content());

    // planner bom item -> true
    proj.planner.bom.push(BomItem::new("Resistor 10k", "0805", 10, 0.05));
    assert!(proj.has_content());
    proj.planner = siro::model::planner::PlannerData::default();
    assert!(!proj.has_content());
}

#[test]
fn test_is_autosave_eligible() {
    let mut proj = Project::new("Eligible Test");
    let fake_path = PathBuf::from(r"C:\test\eligible.siro");

    // Off (None) -> false for all
    let off_settings = AppSettings { autosave_interval_secs: None, smart_snapping_enabled: true };
    proj.filepath = Some(fake_path.clone());
    proj.mark_edited();
    proj.active_board_mut().add_element(BoardElement::Text(TextElement::new_sticky(
        Uuid::new_v4(),
        egui::pos2(0.0, 0.0),
        "Text".to_string(),
        egui::Color32::BLUE,
    )));
    assert!(!is_autosave_eligible(&proj, off_settings.autosave_interval_secs));

    // On, no content -> false
    let on_settings = AppSettings { autosave_interval_secs: Some(60), smart_snapping_enabled: true };
    let mut empty_proj = Project::new("Empty");
    empty_proj.filepath = Some(fake_path.clone());
    empty_proj.mark_edited();
    assert!(!is_autosave_eligible(&empty_proj, on_settings.autosave_interval_secs));

    // On, has content, no filepath -> false (never auto-name or pop dialog during background sweep)
    let mut unsaved_unnamed = Project::new("Unnamed");
    unsaved_unnamed.active_board_mut().add_element(BoardElement::Text(TextElement::new_sticky(
        Uuid::new_v4(),
        egui::pos2(0.0, 0.0),
        "Text".to_string(),
        egui::Color32::BLUE,
    )));
    unsaved_unnamed.mark_edited();
    assert_eq!(unsaved_unnamed.filepath, None);
    assert!(!is_autosave_eligible(&unsaved_unnamed, on_settings.autosave_interval_secs));

    // On, has content, not unsaved -> false (no wasteful disk writes)
    let mut saved_proj = Project::new("Saved");
    saved_proj.filepath = Some(fake_path.clone());
    saved_proj.active_board_mut().add_element(BoardElement::Text(TextElement::new_sticky(
        Uuid::new_v4(),
        egui::pos2(0.0, 0.0),
        "Text".to_string(),
        egui::Color32::BLUE,
    )));
    saved_proj.mark_saved();
    assert!(!is_autosave_eligible(&saved_proj, on_settings.autosave_interval_secs));

    // On, has content, unsaved, has filepath -> true!
    let mut ready_proj = Project::new("Ready");
    ready_proj.filepath = Some(fake_path);
    ready_proj.active_board_mut().add_element(BoardElement::Text(TextElement::new_sticky(
        Uuid::new_v4(),
        egui::pos2(0.0, 0.0),
        "Text".to_string(),
        egui::Color32::BLUE,
    )));
    ready_proj.mark_edited();
    assert!(is_autosave_eligible(&ready_proj, on_settings.autosave_interval_secs));
}

#[test]
fn test_app_settings_persistence() {
    let temp_dir = std::env::temp_dir().join(format!("siro_settings_test_{}", Uuid::new_v4()));
    let _ = std::fs::create_dir_all(&temp_dir);
    let settings_file = temp_dir.join("settings.json");

    // 1. Missing file -> loads default (60s)
    let loaded_default = load_settings_from(&settings_file);
    assert_eq!(loaded_default.autosave_interval_secs, Some(DEFAULT_AUTOSAVE_INTERVAL_SECS));

    // 2. Corrupted file -> loads default (60s)
    std::fs::write(&settings_file, b"corrupted garbage json").unwrap();
    let loaded_corrupt = load_settings_from(&settings_file);
    assert_eq!(loaded_corrupt.autosave_interval_secs, Some(DEFAULT_AUTOSAVE_INTERVAL_SECS));

    // 3. Null interval -> loads Off (None)
    std::fs::write(&settings_file, br#"{"autosave_interval_secs": null}"#).unwrap();
    let loaded_off = load_settings_from(&settings_file);
    assert_eq!(loaded_off.autosave_interval_secs, None);
    assert!(!loaded_off.is_autosave_enabled());

    // 4. Out-of-range value (2s) -> clamped to MIN (5s)
    std::fs::write(&settings_file, br#"{"autosave_interval_secs": 2}"#).unwrap();
    let loaded_under = load_settings_from(&settings_file);
    assert_eq!(loaded_under.autosave_interval_secs, Some(MIN_AUTOSAVE_INTERVAL_SECS));

    // Out-of-range value (1200s) -> clamped to MAX (600s)
    std::fs::write(&settings_file, br#"{"autosave_interval_secs": 1200}"#).unwrap();
    let loaded_over = load_settings_from(&settings_file);
    assert_eq!(loaded_over.autosave_interval_secs, Some(MAX_AUTOSAVE_INTERVAL_SECS));

    // 5. Save then load round-trips correctly
    let mut custom = AppSettings { autosave_interval_secs: Some(120), smart_snapping_enabled: true };
    save_settings_to(&settings_file, &custom).expect("Save settings should succeed");
    let loaded_custom = load_settings_from(&settings_file);
    assert_eq!(loaded_custom.autosave_interval_secs, Some(120));

    // Round-trip Off (None)
    custom.autosave_interval_secs = None;
    save_settings_to(&settings_file, &custom).expect("Save settings off should succeed");
    let loaded_custom_off = load_settings_from(&settings_file);
    assert_eq!(loaded_custom_off.autosave_interval_secs, None);

    let _ = std::fs::remove_dir_all(temp_dir);
}
