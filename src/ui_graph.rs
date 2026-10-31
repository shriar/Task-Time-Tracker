use chrono::{Days, NaiveDate};
use eframe::egui;

use crate::app::{BEIGE, DIM, FAINT, GOLD, TrackerApp, color_bar, color_for_task};
use crate::model::{format_hm, now_ts, today};

/// Stats tab: bar chart + per-project averages.
pub fn show(app: &mut TrackerApp, ui: &mut egui::Ui) {
    let now = now_ts();

    ui.horizontal(|ui| {
        ui.heading("Stats");
        ui.selectable_value(&mut app.graph_days, 7, "7 days");
        ui.selectable_value(&mut app.graph_days, 30, "30 days");
    });
    ui.label(
        egui::RichText::new(format!(
            "This week {} · {} days in a row",
            format_hm(app.week_total(now)),
            app.streak(now)
        ))
        .size(13.0)
        .color(DIM),
    );
    ui.separator();

    let n = app.graph_days.max(1) as u64;
    let days: Vec<(NaiveDate, i64)> = (0..n)
        .map(|i| {
            let d = today().checked_sub_days(Days::new(i)).unwrap_or_else(today);
            (d, app.day_total(d, now))
        })
        .rev()
        .collect();

    let max_secs = days.iter().map(|(_, s)| *s).max().unwrap_or(0).max(1);

    // Bar chart area.
    let height = 200.0;
    let (resp, painter) =
        ui.allocate_painter(egui::vec2(ui.available_width(), height), egui::Sense::hover());
    let rect = resp.rect;
    let label_space = 26.0; // room for date labels at the bottom
    let plot_h = rect.height() - label_space;

    // Gridlines at 0 / half / max with hour labels.
    for frac in [0.0f32, 0.5, 1.0] {
        let y = rect.top() + plot_h * (1.0 - frac);
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            egui::Stroke::new(1.0, FAINT),
        );
        painter.text(
            egui::pos2(rect.left() + 2.0, y - 2.0),
            egui::Align2::LEFT_BOTTOM,
            format_hm((max_secs as f32 * frac) as i64),
            egui::FontId::new(10.0, egui::FontFamily::Monospace),
            DIM,
        );
    }

    let gap = 5.0;
    let bar_w = (rect.width() - gap * (n as f32 + 1.0)) / n as f32;
    for (i, (d, secs)) in days.iter().enumerate() {
        let h = (*secs as f32 / max_secs as f32) * (plot_h - 16.0);
        let x0 = rect.left() + gap + i as f32 * (bar_w + gap);
        let bar = egui::Rect::from_min_max(
            egui::pos2(x0, rect.top() + plot_h - h),
            egui::pos2(x0 + bar_w, rect.top() + plot_h),
        );
        painter.rect_filled(bar, egui::CornerRadius::same(3), GOLD);
        // Value label on top of each non-empty bar.
        if *secs > 0 {
            painter.text(
                egui::pos2(x0 + bar_w / 2.0, bar.top() - 2.0),
                egui::Align2::CENTER_BOTTOM,
                format_hm(*secs),
                egui::FontId::new(10.0, egui::FontFamily::Monospace),
                BEIGE,
            );
        }
        // Date label (sparse for 30 days).
        if n <= 10 || i % 5 == 0 || i as u64 == n - 1 {
            painter.text(
                egui::pos2(x0 + bar_w / 2.0, rect.bottom() - 2.0),
                egui::Align2::CENTER_BOTTOM,
                d.format("%m/%d").to_string(),
                egui::FontId::new(10.0, egui::FontFamily::Monospace),
                DIM,
            );
        }
    }

    ui.separator();

    // Per-project averages.
    ui.label(egui::RichText::new("Averages").strong());
    egui::ScrollArea::vertical().show(ui, |ui| {
        for task in app.all_projects() {
            let avg = app.combined_avg_per_day(task.id, now);
            if avg == 0 {
                continue;
            }
            ui.horizontal(|ui| {
                color_bar(ui, color_for_task(task.id), 20.0);
                ui.label(format!(
                    "{}{}  —  avg {}",
                    task.name,
                    if task.archived { " [archived]" } else { "" },
                    format_hm(avg)
                ));
            });
        }
    });
}
