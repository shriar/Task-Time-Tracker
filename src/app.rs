use std::collections::BTreeSet;
use std::path::PathBuf;

use chrono::{Datelike, Days, NaiveDate};
use eframe::egui;

use crate::model::*;
use crate::store::*;
use crate::tray::{self, MENU_QUIT, MENU_SHOW, AppTray};
use crate::{ui_graph, ui_history, ui_mini, ui_projects, ui_today};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Track,
    Log,
    Stats,
    Projects,
}

/// Warm dark palette (matches reference: near-black bg, beige text).
pub const BEIGE: egui::Color32 = egui::Color32::from_rgb(231, 220, 200);
pub const DIM: egui::Color32 = egui::Color32::from_rgb(168, 152, 128);
pub const FAINT: egui::Color32 = egui::Color32::from_rgb(110, 100, 86);
pub const BG: egui::Color32 = egui::Color32::from_rgb(26, 23, 19);
pub const FIELD_BG: egui::Color32 = egui::Color32::from_rgb(42, 37, 31);
pub const BTN_BG: egui::Color32 = egui::Color32::from_rgb(52, 46, 38);
pub const BTN_HOVER: egui::Color32 = egui::Color32::from_rgb(74, 63, 48);
pub const SELECT: egui::Color32 = egui::Color32::from_rgb(74, 63, 48);
pub const TRACK_BG: egui::Color32 = egui::Color32::from_rgb(58, 53, 45);
pub const GOLD: egui::Color32 = egui::Color32::from_rgb(212, 175, 105);

/// Per-project accent bar colors, assigned by id.
pub const TASK_COLORS: [egui::Color32; 6] = [
    egui::Color32::from_rgb(66, 135, 245),  // blue
    egui::Color32::from_rgb(235, 98, 52),   // orange
    egui::Color32::from_rgb(32, 178, 120),  // green
    egui::Color32::from_rgb(230, 168, 40),  // yellow
    egui::Color32::from_rgb(165, 110, 230), // purple
    egui::Color32::from_rgb(60, 190, 200),  // teal
];

pub fn color_for_task(id: u64) -> egui::Color32 {
    TASK_COLORS[(id as usize) % TASK_COLORS.len()]
}

/// Thin vertical accent bar.
pub fn color_bar(ui: &mut egui::Ui, color: egui::Color32, height: f32) {
    let (r, _) = ui.allocate_exact_size(egui::vec2(4.0, height), egui::Sense::hover());
    ui.painter()
        .rect_filled(r, egui::CornerRadius::same(2), color);
}

/// Slim horizontal progress bar, fixed width.
pub fn progress_bar(ui: &mut egui::Ui, frac: f32, color: egui::Color32, width: f32) {
    let h = 6.0;
    let (r, _) = ui.allocate_exact_size(egui::vec2(width, h), egui::Sense::hover());
    let p = ui.painter();
    p.rect_filled(r, egui::CornerRadius::same(3), TRACK_BG);
    let w = r.width() * frac.clamp(0.0, 1.0);
    if w >= 1.0 {
        let fill = egui::Rect::from_min_size(r.min, egui::vec2(w.max(5.0), h));
        p.rect_filled(fill, egui::CornerRadius::same(3), color);
    }
}

pub struct TrackerApp {
    pub db: Db,
    pub data_path: PathBuf,
    pub tab: Tab,
    pub sub_for: Option<u64>,
    pub sub_name: String,
    pub history_date: NaiveDate,
    pub graph_days: i64,
    pub mini_visible: bool,
    pub new_project: String,
    pub rename_id: Option<u64>,
    pub rename_text: String,
    pub tray: Option<AppTray>,
    pub tray_showing: bool,
    pub allow_close: bool,
    pub tray_was_running: bool,
}

impl TrackerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Monospace type, slightly larger, warm dark theme.
        let mut style = (*cc.egui_ctx.style_of(egui::Theme::Dark)).clone();
        use egui::{FontFamily, FontId, TextStyle};
        style.text_styles.insert(
            TextStyle::Body,
            FontId::new(16.0, FontFamily::Monospace),
        );
        style.text_styles.insert(
            TextStyle::Button,
            FontId::new(16.0, FontFamily::Monospace),
        );
        style.text_styles.insert(
            TextStyle::Heading,
            FontId::new(22.0, FontFamily::Monospace),
        );
        style.text_styles.insert(
            TextStyle::Small,
            FontId::new(13.0, FontFamily::Monospace),
        );
        style.spacing.item_spacing = egui::vec2(8.0, 8.0);
        style.spacing.button_padding = egui::vec2(10.0, 6.0);
        cc.egui_ctx.set_style_of(egui::Theme::Dark, style);

        let mut visuals = egui::Visuals::dark();
        visuals.window_fill = BG;
        visuals.panel_fill = BG;
        visuals.override_text_color = Some(BEIGE);
        visuals.extreme_bg_color = FIELD_BG;
        visuals.selection.bg_fill = SELECT;
        visuals.hyperlink_color = GOLD;
        visuals.widgets.inactive.bg_fill = BTN_BG;
        visuals.widgets.hovered.bg_fill = BTN_HOVER;
        visuals.widgets.active.bg_fill = SELECT;
        cc.egui_ctx.set_visuals(visuals);

        let data_path = data_path();
        let mut db = Db::load(&data_path);
        // If the app was killed while a timer ran, stop it now so it
        // can't silently inflate. Normal quit already stops timers (on_exit).
        let now = now_ts();
        let mut dirty = false;
        for e in db.entries.iter_mut().filter(|e| e.is_running()) {
            e.end = Some(now);
            dirty = true;
        }
        if dirty {
            db.save(&data_path);
        }
        // Screenshot/testing hook: TASK_TRACKER_TAB=track|log|stats|projects
        let tab = std::env::var("TASK_TRACKER_TAB")
            .ok()
            .and_then(|s| match s.to_lowercase().as_str() {
                "log" => Some(Tab::Log),
                "stats" => Some(Tab::Stats),
                "projects" => Some(Tab::Projects),
                _ => Some(Tab::Track),
            })
            .unwrap_or(Tab::Track);
        Self {
            db,
            data_path,
            tab,
            sub_for: None,
            sub_name: String::new(),
            history_date: today(),
            graph_days: 7,
            mini_visible: false,
            new_project: String::new(),
            rename_id: None,
            rename_text: String::new(),
            tray: AppTray::new(),
            tray_showing: true,
            allow_close: false,
            tray_was_running: false,
        }
    }

    fn persist(&self) {
        self.db.save(&self.data_path);
    }

    fn set_window_visible(&mut self, ctx: &egui::Context, visible: bool) {
        crate::tray::log_line(&format!("tray: set_visible({})", visible));
        self.tray_showing = visible;
        ctx.send_viewport_cmd(egui::ViewportCommand::Visible(visible));
        if let Some(t) = &self.tray {
            t.set_shown(visible);
        }
    }

    fn poll_tray(&mut self, ctx: &egui::Context) {
        // X button minimizes to tray instead of quitting.
        if ctx.input(|i| i.viewport().close_requested()) && !self.allow_close {
            crate::tray::log_line("tray: X pressed -> hide to tray");
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.set_window_visible(ctx, false);
        }
        // Tray menu clicks (menu only offers Show + Quit).
        for id in tray::poll_menu() {
            crate::tray::log_line(&format!("tray: menu event '{}'", id));
            if id == MENU_SHOW {
                self.set_window_visible(ctx, true);
            } else if id == MENU_QUIT {
                crate::tray::log_line("tray: quit requested");
                self.allow_close = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }
        // Left-click toggles the window.
        if tray::poll_left_click() {
            crate::tray::log_line("tray: left click -> toggle");
            let visible = !self.tray_showing;
            self.set_window_visible(ctx, visible);
        }
        // Icon color follows the timer state.
        let running_id = self.running_task_id();
        let running = running_id.is_some();
        if running != self.tray_was_running {
            self.tray_was_running = running;
            if let Some(t) = &self.tray {
                let now = now_ts();
                let tip = match running_id {
                    Some(id) => format!(
                        "Task Tracker — {} {}",
                        self.task_name(id),
                        format_hms(self.running_secs(now).unwrap_or(0))
                    ),
                    None => "Task Tracker".to_owned(),
                };
                t.set_running(running, &tip);
            }
        }
    }

    pub fn running_task_id(&self) -> Option<u64> {
        self.db
            .entries
            .iter()
            .find(|e| e.is_running())
            .map(|e| e.task_id)
    }

    pub fn running_secs(&self, now: i64) -> Option<i64> {
        self.db
            .entries
            .iter()
            .find(|e| e.is_running())
            .map(|e| e.duration_secs(now))
    }

    /// Start a timer; stops any other running timer first (single active timer).
    pub fn start(&mut self, task_id: u64) {
        let now = now_ts();
        for e in self.db.entries.iter_mut().filter(|e| e.is_running()) {
            e.end = Some(now);
        }
        let id = self.db.alloc_id();
        self.db.entries.push(TimeEntry {
            id,
            task_id,
            start: now,
            end: None,
        });
        self.persist();
    }

    pub fn stop_running(&mut self) {
        let now = now_ts();
        for e in self.db.entries.iter_mut().filter(|e| e.is_running()) {
            e.end = Some(now);
        }
        self.persist();
    }

    pub fn add_task(&mut self, name: &str, parent_id: Option<u64>) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        let id = self.db.alloc_id();
        self.db.tasks.push(Task {
            id,
            name: name.to_owned(),
            parent_id,
            created_at: now_ts(),
            archived: false,
        });
        self.persist();
    }

    pub fn set_archived(&mut self, id: u64, archived: bool) {
        if let Some(t) = self.db.tasks.iter_mut().find(|t| t.id == id) {
            t.archived = archived;
        }
        if archived {
            // Stopping is handled by hide; ensure no timer keeps running on it.
            let running_on = self
                .db
                .entries
                .iter()
                .any(|e| e.is_running() && e.task_id == id);
            if running_on {
                self.stop_running();
            }
        }
        self.persist();
    }

    /// Delete a task, its subtasks, and all their entries.
    pub fn delete_task(&mut self, id: u64) {
        let mut ids = vec![id];
        ids.extend(
            self.db
                .tasks
                .iter()
                .filter(|t| t.parent_id == Some(id))
                .map(|t| t.id),
        );
        self.db.tasks.retain(|t| !ids.contains(&t.id));
        self.db.entries.retain(|e| !ids.contains(&e.task_id));
        if self.sub_for == Some(id) {
            self.sub_for = None;
            self.sub_name.clear();
        }
        self.persist();
    }

    /// Rename a task or subtask (empty names are ignored).
    pub fn rename_task(&mut self, id: u64, name: &str) {
        let name = name.trim();
        if name.is_empty() {
            return;
        }
        if let Some(t) = self.db.tasks.iter_mut().find(|t| t.id == id) {
            t.name = name.to_owned();
        }
        self.persist();
    }

    /// Top-level, non-archived projects (Track tab).
    pub fn active_projects(&self) -> Vec<Task> {
        let mut v: Vec<Task> = self
            .db
            .tasks
            .iter()
            .filter(|t| t.parent_id.is_none() && !t.archived)
            .cloned()
            .collect();
        v.sort_by_key(|t| t.id);
        v
    }

    /// Top-level archived projects (Projects tab).
    pub fn archived_projects(&self) -> Vec<Task> {
        let mut v: Vec<Task> = self
            .db
            .tasks
            .iter()
            .filter(|t| t.parent_id.is_none() && t.archived)
            .cloned()
            .collect();
        v.sort_by_key(|t| t.id);
        v
    }

    /// All top-level projects (Projects tab).
    pub fn all_projects(&self) -> Vec<Task> {
        let mut v: Vec<Task> = self
            .db
            .tasks
            .iter()
            .filter(|t| t.parent_id.is_none())
            .cloned()
            .collect();
        v.sort_by_key(|t| t.id);
        v
    }

    pub fn subtasks(&self, parent: u64) -> Vec<Task> {
        let mut v: Vec<Task> = self
            .db
            .tasks
            .iter()
            .filter(|t| t.parent_id == Some(parent))
            .cloned()
            .collect();
        v.sort_by_key(|t| t.id);
        v
    }

    pub fn task_name(&self, id: u64) -> String {
        self.db
            .tasks
            .iter()
            .find(|t| t.id == id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "(deleted)".to_owned())
    }

    /// Own entries only, for one day.
    pub fn task_secs_on_day(&self, task_id: u64, day: NaiveDate, now: i64) -> i64 {
        self.db
            .entries
            .iter()
            .filter(|e| e.task_id == task_id && day_of(e.start) == day)
            .map(|e| e.duration_secs(now))
            .sum()
    }

    /// Own + subtasks, for one day.
    pub fn combined_secs_on_day(&self, task_id: u64, day: NaiveDate, now: i64) -> i64 {
        let mut total = self.task_secs_on_day(task_id, day, now);
        for sub in self
            .db
            .tasks
            .iter()
            .filter(|t| t.parent_id == Some(task_id))
        {
            total += self.task_secs_on_day(sub.id, day, now);
        }
        total
    }

    pub fn day_total(&self, day: NaiveDate, now: i64) -> i64 {
        self.db
            .entries
            .iter()
            .filter(|e| day_of(e.start) == day)
            .map(|e| e.duration_secs(now))
            .sum()
    }

    pub fn sessions_on_day(&self, task_id: u64, day: NaiveDate) -> usize {
        self.db
            .entries
            .iter()
            .filter(|e| e.task_id == task_id && day_of(e.start) == day)
            .count()
    }

    /// Monday-start week containing today.
    pub fn week_total(&self, now: i64) -> i64 {
        let dow = today().weekday().num_days_from_monday() as u64;
        let start = today().checked_sub_days(Days::new(dow)).unwrap_or_else(today);
        (0..7)
            .filter_map(|i| start.checked_add_days(Days::new(i)))
            .map(|d| self.day_total(d, now))
            .sum()
    }

    /// Consecutive days with tracked time (today, or yesterday if today is empty).
    pub fn streak(&self, now: i64) -> u32 {
        let mut day = today();
        if self.day_total(day, now) == 0 {
            match day.pred_opt() {
                Some(d) => day = d,
                None => return 0,
            }
        }
        let mut n = 0;
        loop {
            if self.day_total(day, now) == 0 {
                break;
            }
            n += 1;
            match day.pred_opt() {
                Some(d) => day = d,
                None => break,
            }
        }
        n
    }

    /// Average per active day for a project (own + subtasks).
    pub fn combined_avg_per_day(&self, task_id: u64, now: i64) -> i64 {
        let mut ids = vec![task_id];
        ids.extend(
            self.db
                .tasks
                .iter()
                .filter(|t| t.parent_id == Some(task_id))
                .map(|t| t.id),
        );
        let mut days = BTreeSet::new();
        let mut total = 0;
        for e in self.db.entries.iter().filter(|e| ids.contains(&e.task_id)) {
            days.insert(day_of(e.start));
            total += e.duration_secs(now);
        }
        if days.is_empty() {
            0
        } else {
            total / days.len() as i64
        }
    }
}

impl eframe::App for TrackerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Steady heartbeat (4x/sec): cheap for this UI, and it guarantees
        // tray menu / left-click events are processed even when the window
        // is idle or hidden in the tray. Also drives the live timer.
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_millis(250));

        // Tray events: X minimizes to tray, icon color follows timer.
        self.poll_tray(&ui.ctx().clone());

        // Top nav: Track | Log | Stats | Projects
        egui::Panel::top("tabs").show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Track, "Track");
                ui.selectable_value(&mut self.tab, Tab::Log, "Log");
                ui.selectable_value(&mut self.tab, Tab::Stats, "Stats");
                ui.selectable_value(&mut self.tab, Tab::Projects, "Projects");
            });
        });

        // Bottom status bar: Today | This week | streak
        egui::Panel::bottom("status").show(ui, |ui| {
            let now = now_ts();
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(format!("Today {}", format_hm(self.day_total(today(), now))))
                        .size(13.0)
                        .color(DIM),
                );
                ui.label(
                    egui::RichText::new(format!("This week {}", format_hm(self.week_total(now))))
                        .size(13.0)
                        .color(DIM),
                );
                ui.label(
                    egui::RichText::new(format!("{} days in a row", self.streak(now)))
                        .size(13.0)
                        .color(DIM),
                );
            });
        });

        // Floating always-on-top mini timer (separate OS window).
        let ctx = ui.ctx().clone();
        ui_mini::show(self, &ctx);

        let tab = self.tab;
        egui::CentralPanel::default().show(ui, |ui| match tab {
            Tab::Track => ui_today::show(self, ui),
            Tab::Log => ui_history::show(self, ui),
            Tab::Stats => ui_graph::show(self, ui),
            Tab::Projects => ui_projects::show(self, ui),
        });
    }

    /// Closing the app stops a running timer so it can't inflate overnight.
    fn on_exit(&mut self) {
        let now = now_ts();
        for e in self.db.entries.iter_mut().filter(|e| e.is_running()) {
            e.end = Some(now);
        }
        self.persist();
    }
}
