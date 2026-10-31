use eframe::egui;

use crate::app::{DIM, TrackerApp, color_bar, color_for_task};

/// Projects tab: add / archive / unarchive / delete.
/// Archived projects are hidden from Track but keep their Log & Stats history.
pub fn show(app: &mut TrackerApp, ui: &mut egui::Ui) {
    ui.heading("Projects");
    ui.separator();

    ui.horizontal(|ui| {
        ui.text_edit_singleline(&mut app.new_project);
        if ui.button("Add project").clicked() {
            let name = app.new_project.clone();
            app.add_task(&name, None);
            app.new_project.clear();
        }
    });
    ui.separator();

    let active = app.all_projects().into_iter().filter(|t| !t.archived).collect::<Vec<_>>();
    let archived = app.archived_projects();

    if active.is_empty() {
        ui.label(
            egui::RichText::new("No active projects.")
                .size(13.0)
                .color(DIM),
        );
    }
    for task in active {
        let n_subs = app.subtasks(task.id).len();
        ui.horizontal(|ui| {
            color_bar(ui, color_for_task(task.id), 22.0);
            ui.label(egui::RichText::new(&task.name).strong());
            if n_subs > 0 {
                ui.label(
                    egui::RichText::new(format!("({} subtask{})", n_subs, if n_subs == 1 { "" } else { "s" }))
                        .size(13.0)
                        .color(DIM),
                );
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Delete").clicked() {
                    app.delete_task(task.id);
                }
                if ui.small_button("Archive").clicked() {
                    app.set_archived(task.id, true);
                }
            });
        });
    }

    ui.separator();
    ui.label(egui::RichText::new("Archived").strong());
    ui.label(
        egui::RichText::new("Hidden from Track. History in Log & Stats is kept.")
            .size(13.0)
            .color(DIM),
    );
    if archived.is_empty() {
        ui.label(
            egui::RichText::new("Nothing archived.")
                .size(13.0)
                .color(DIM),
        );
    }
    for task in archived {
        ui.horizontal(|ui| {
            color_bar(ui, DIM, 22.0);
            ui.label(&task.name);
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Delete").clicked() {
                    app.delete_task(task.id);
                }
                if ui.small_button("Unarchive").clicked() {
                    app.set_archived(task.id, false);
                }
            });
        });
    }
}
