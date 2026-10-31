use eframe::egui;

use crate::app::TrackerApp;
use crate::model::{format_hm, format_hms, now_ts, today};

/// Floating always-on-top mini timer (separate OS window).
/// Shown only while `app.mini_visible`; closing the window hides it.
pub fn show(app: &mut TrackerApp, ctx: &egui::Context) {
    if !app.mini_visible {
        return;
    }
    let now = now_ts();

    ctx.show_viewport_immediate(
        egui::ViewportId::from_hash_of("task_tracker_mini_timer"),
        egui::ViewportBuilder::default()
            .with_title("Task Timer")
            .with_inner_size([260.0, 120.0])
            .with_resizable(false)
            .with_always_on_top(),
        |ui, _class| {
            // User pressed X on the mini window -> hide it (reopen from main).
            if ui.input(|i| i.viewport().close_requested()) {
                app.mini_visible = false;
            }

            match app.running_task_id() {
                Some(id) => {
                    let secs = app.running_secs(now).unwrap_or(0);
                    ui.heading(format_hms(secs));
                    ui.label(app.task_name(id));
                    if ui.button("Stop").clicked() {
                        app.stop_running();
                    }
                }
                None => {
                    ui.label("No timer running");
                    ui.label(format!("Today: {}", format_hm(app.day_total(today(), now))));
                }
            }

            // Keep the live counter ticking.
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_secs(1));
        },
    );
}
