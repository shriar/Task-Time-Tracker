use eframe::egui;

use crate::app::{DIM, TrackerApp, color_bar, color_for_task};

/// Projects tab: add / rename / archive / unarchive / delete.
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

    let active: Vec<_> = app
        .all_projects()
        .into_iter()
        .filter(|t| !t.archived)
        .collect();
    let archived = app.archived_projects();

    if active.is_empty() {
        ui.label(
            egui::RichText::new("No active projects.")
                .size(13.0)
                .color(DIM),
        );
    }
    for task in active {
        let n_subs = app.all_subtasks(task.id).len();
        task_row(app, ui, task.id, &task.name, false, false, n_subs);
        for sub in app.all_subtasks(task.id) {
            task_row(app, ui, sub.id, &sub.name, false, true, 0);
        }
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
        task_row(app, ui, task.id, &task.name, true, false, 0);
        for sub in app.all_subtasks(task.id) {
            task_row(app, ui, sub.id, &sub.name, sub.archived, true, 0);
        }
    }
}

fn task_row(
    app: &mut TrackerApp,
    ui: &mut egui::Ui,
    id: u64,
    name: &str,
    archived: bool,
    is_sub: bool,
    n_subs: usize,
) {
    // Inline rename editor.
    if app.rename_id == Some(id) {
        ui.horizontal(|ui| {
            if is_sub {
                ui.label(egui::RichText::new("      └").size(13.0).color(DIM));
            }
            ui.text_edit_singleline(&mut app.rename_text);
            if ui.button("Save").clicked() {
                let new_name = app.rename_text.clone();
                app.rename_task(id, &new_name);
                app.rename_id = None;
            }
            if ui.button("Cancel").clicked() {
                app.rename_id = None;
            }
        });
        return;
    }

    ui.horizontal(|ui| {
        if is_sub {
            ui.label(egui::RichText::new("      └").size(13.0).color(DIM));
            color_bar(ui, DIM, 20.0);
        } else {
            color_bar(ui, color_for_task(id), 22.0);
        }
        let name_text = if archived && !is_sub {
            format!("{} [completed]", name)
        } else if archived {
            format!("{} [completed]", name)
        } else {
            name.to_owned()
        };
        if is_sub || archived {
            ui.label(egui::RichText::new(name_text).size(15.0).color(DIM));
        } else {
            ui.label(egui::RichText::new(name_text).strong());
            if n_subs > 0 {
                ui.label(
                    egui::RichText::new(format!(
                        "({} subtask{})",
                        n_subs,
                        if n_subs == 1 { "" } else { "s" }
                    ))
                    .size(13.0)
                    .color(DIM),
                );
            }
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.small_button("Delete").clicked() {
                app.delete_task(id);
            }
            if archived {
                if ui.small_button("Unarchive").clicked() {
                    app.set_archived(id, false);
                }
            } else if is_sub {
                if ui.small_button("Complete").clicked() {
                    app.complete_task(id);
                }
            } else if ui.small_button("Archive").clicked() {
                app.set_archived(id, true);
            }
            if ui.small_button("Rename").clicked() {
                app.rename_id = Some(id);
                app.rename_text = name.to_owned();
            }
        });
    });
}
