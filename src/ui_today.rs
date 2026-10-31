use eframe::egui;

use crate::app::{DIM, TrackerApp, color_bar, color_for_task, progress_bar};
use crate::model::{format_hm, format_hms, now_ts, today};

/// Track tab: project rows with accent bar, progress, play buttons.
pub fn show(app: &mut TrackerApp, ui: &mut egui::Ui) {
    let now = now_ts();
    let day = today();
    let running = app.running_task_id();

    ui.horizontal(|ui| {
        ui.heading("Track");
        if ui.small_button("Mini timer").clicked() {
            app.mini_visible = !app.mini_visible;
        }
    });
    ui.separator();

    let tops = app.active_projects();
    if tops.is_empty() {
        ui.label("No projects yet. Add one in the Projects tab.");
    }

    let max_today = tops
        .iter()
        .map(|t| app.combined_secs_on_day(t.id, day, now))
        .max()
        .unwrap_or(0)
        .max(1);

    egui::ScrollArea::vertical().show(ui, |ui| {
        for task in tops {
            project_row(app, ui, task.id, &task.name, day, now, running, max_today);

            // Subtasks, compact.
            for sub in app.subtasks(task.id) {
                subtask_row(app, ui, sub.id, &sub.name, day, now, running, max_today);
            }

            // Subtask creation row.
            if app.sub_for == Some(task.id) {
                ui.horizontal(|ui| {
                    ui.label("      └");
                    ui.text_edit_singleline(&mut app.sub_name);
                    if ui.button("Add").clicked() {
                        let name = app.sub_name.clone();
                        app.add_task(&name, Some(task.id));
                        app.sub_name.clear();
                        app.sub_for = None;
                    }
                    if ui.button("Cancel").clicked() {
                        app.sub_for = None;
                        app.sub_name.clear();
                    }
                });
            } else if ui
                .small_button("+ subtask")
                .on_hover_text("Add a subtask")
                .clicked()
            {
                app.sub_for = Some(task.id);
                app.sub_name.clear();
            }

            ui.separator();
        }
    });
}

fn project_row(
    app: &mut TrackerApp,
    ui: &mut egui::Ui,
    id: u64,
    name: &str,
    day: chrono::NaiveDate,
    now: i64,
    running: Option<u64>,
    max_today: i64,
) {
    let secs = app.combined_secs_on_day(id, day, now);
    let avg = app.combined_avg_per_day(id, now);
    let color = color_for_task(id);

    ui.horizontal(|ui| {
        color_bar(ui, color, 34.0);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(name).strong());
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.small_button("✓").on_hover_text("Complete (moves to archive)").clicked() {
                            app.complete_task(id);
                        }
                        play_button(app, ui, id, running, now);
                    },
                );
            });
            ui.horizontal(|ui| {
                progress_bar(ui, secs as f32 / max_today as f32, color, 140.0);
                let sub = if running == Some(id) {
                    format!(
                        "{} today · avg {}",
                        format_hms(app.running_secs(now).unwrap_or(0)),
                        format_hm(avg)
                    )
                } else {
                    format!("{} today · avg {}", format_hm(secs), format_hm(avg))
                };
                ui.label(egui::RichText::new(sub).size(13.0).color(DIM));
            });
        });
    });
}

fn subtask_row(
    app: &mut TrackerApp,
    ui: &mut egui::Ui,
    id: u64,
    name: &str,
    day: chrono::NaiveDate,
    now: i64,
    running: Option<u64>,
    max_today: i64,
) {
    let secs = app.task_secs_on_day(id, day, now);
    let color = color_for_task(id);

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("      └").size(13.0).color(DIM));
        color_bar(ui, color, 24.0);
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("{}  {}", name, format_hm(secs)));
                ui.with_layout(
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        if ui.small_button("✓").on_hover_text("Complete (moves to archive)").clicked() {
                            app.complete_task(id);
                        }
                        play_button(app, ui, id, running, now);
                    },
                );
            });
            progress_bar(ui, secs as f32 / max_today as f32, color, 100.0);
        });
    });
}

fn play_button(
    app: &mut TrackerApp,
    ui: &mut egui::Ui,
    id: u64,
    running: Option<u64>,
    _now: i64,
) {
    if running == Some(id) {
        if ui.button("■").on_hover_text("Stop").clicked() {
            app.stop_running();
        }
    } else if ui.button("▶").on_hover_text("Start").clicked() {
        app.start(id);
    }
}
