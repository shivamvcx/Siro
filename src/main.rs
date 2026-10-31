#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod app;
pub mod canvas;
pub mod model;
pub mod shortcuts;
pub mod storage;
pub mod ui;

use app::SiroApp;
use eframe::egui::Vec2;
use siro::APP_NAME;

fn main() -> eframe::Result<()> {
    let icon_data = eframe::icon_data::from_png_bytes(include_bytes!("../assets/icon_256.png")).ok();

    let mut viewport_builder = eframe::egui::ViewportBuilder::default()
        .with_inner_size(Vec2::new(1380.0, 860.0))
        .with_min_inner_size(Vec2::new(800.0, 500.0))
        .with_title(format!("{} - Native STEM Whiteboard & Planner", APP_NAME))
        .with_active(true)
        .with_drag_and_drop(true);

    if let Some(icon) = icon_data {
        viewport_builder = viewport_builder.with_icon(icon);
    }

    let native_options = eframe::NativeOptions {
        viewport: viewport_builder,
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(SiroApp::new(cc)))),
    )
}
