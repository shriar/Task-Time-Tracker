#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod model;
mod store;
mod tray;
mod ui_graph;
mod ui_history;
mod ui_mini;
mod ui_projects;
mod ui_today;

use app::TrackerApp;

fn main() {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default().with_inner_size([760.0, 640.0]),
        ..Default::default()
    };
    let _ = eframe::run_native(
        "Task Tracker",
        options,
        Box::new(|cc| Ok(Box::new(TrackerApp::new(cc)))),
    );
}
