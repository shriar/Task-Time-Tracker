use eframe::egui;

use crate::app::{BEIGE, DIM, TrackerApp, color_bar, color_for_task};
use crate::model::{format_hm, now_ts, today};

/// Log tab: per-day breakdown (history).
pub fn show(app: &mut TrackerApp, ui: &mut egui::Ui) {
    let now = now_ts();

    // Date navigation.
    ui.horizontal(|ui| {
        if ui.button("◀").clicked() {
            if let Some(d) = app.history_date.pred_opt() {
                app.history_date = d;
            }
        }
        ui.heading(app.history_date.format("%b %d, %Y").to_string());
        if ui.button("▶").clicked() {
            if let Some(d) = app.history_date.succ_opt() {
                if d <= today() {
                    app.history_date = d;
                }
            }
        }
        if ui.button("Today").clicked() {
            app.history_date = today();
        }
    });

    let day = app.history_date;
    ui.label(
        egui::RichText::new(format!("Total: {}", format_hm(app.day_total(day, now))))
            .strong()
            .color(BEIGE),
    );
    ui.separator();

    let tops = app.all_projects();
    let mut any = false;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for task in tops {
            let total = app.combined_secs_on_day(task.id, day, now);
            if total == 0 {
                continue;
            }
            any = true;
            let color = color_for_task(task.id);
            let sessions: usize = app.sessions_on_day(task.id, day)
                + app
                    .all_subtasks(task.id)
                    .iter()
                    .map(|s| app.sessions_on_day(s.id, day))
                    .sum::<usize>();
            ui.horizontal(|ui| {
                color_bar(ui, color, 22.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "{}{}  —  {}  ({} session{})",
                            task.name,
                            if task.archived { " [archived]" } else { "" },
                            format_hm(total),
                            sessions,
                            if sessions == 1 { "" } else { "s" }
                        ))
                        .strong(),
                    );
                    for sub in app.all_subtasks(task.id) {
                        let s = app.task_secs_on_day(sub.id, day, now);
                        if s == 0 {
                            continue;
                        }
                        let n = app.sessions_on_day(sub.id, day);
                        ui.label(
                            egui::RichText::new(format!(
                                "      └ {}  —  {}  ({}x)",
                                sub.name,
                                format_hm(s),
                                n
                            ))
                            .size(13.0)
                            .color(DIM),
                        );
                    }
                });
            });
        }
    });

    if !any {
        ui.label(
            egui::RichText::new("No time recorded on this day.")
                .size(13.0)
                .color(DIM),
        );
    }
}
