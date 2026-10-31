use std::fs::OpenOptions;
use std::io::Write;

use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

/// Append one line to debug.log next to the exe (tray diagnostics).
pub fn log_line(msg: &str) {
    let path = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|d| d.join("debug.log")));
    if let Some(p) = path {
        if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(p) {
            let _ = writeln!(f, "{}", msg);
        }
    }
}

pub const MENU_SHOW: &str = "tray_show_hide";
pub const MENU_QUIT: &str = "tray_quit";

/// 32x32 circle icon: solid disc in `main` with a center `dot`.
fn circle_icon(main: [u8; 4], dot: [u8; 4]) -> Icon {
    const S: u32 = 32;
    let mut rgba = vec![0u8; (S * S * 4) as usize];
    let c = S as f32 / 2.0;
    for y in 0..S {
        for x in 0..S {
            let d = (x as f32 - c + 0.5).hypot(y as f32 - c + 0.5);
            let px = if d <= 5.0 {
                Some(dot)
            } else if d <= 14.0 {
                Some(main)
            } else {
                None
            };
            if let Some(p) = px {
                let i = ((y * S + x) * 4) as usize;
                rgba[i..i + 4].copy_from_slice(&p);
            }
        }
    }
    Icon::from_rgba(rgba, S, S).expect("tray icon")
}

/// Idle: beige disc. Running: red disc with white dot.
pub fn idle_icon() -> Icon {
    circle_icon([231, 220, 200, 255], [26, 23, 19, 255])
}

pub fn running_icon() -> Icon {
    circle_icon([220, 38, 38, 255], [255, 255, 255, 255])
}

pub struct AppTray {
    tray: TrayIcon,
    show_item: MenuItem,
}

impl AppTray {
    pub fn new() -> Option<Self> {
        let show_item = MenuItem::with_id(MENU_SHOW, "Show", true, None);
        let quit_item = MenuItem::with_id(MENU_QUIT, "Quit", true, None);
        let menu = Menu::new();
        menu.append(&show_item).ok()?;
        menu.append(&PredefinedMenuItem::separator()).ok()?;
        menu.append(&quit_item).ok()?;
        let tray = match TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Task Tracker")
            .with_icon(idle_icon())
            .build()
        {
            Ok(t) => {
                log_line("tray: icon created ok");
                t
            }
            Err(e) => {
                log_line(&format!("tray: BUILD FAILED: {:?}", e));
                return None;
            }
        };
        Some(Self { tray, show_item })
    }

    pub fn set_running(&self, running: bool, tooltip: &str) {
        let _ = self
            .tray
            .set_icon(Some(if running { running_icon() } else { idle_icon() }));
        let _ = self.tray.set_tooltip(Some(tooltip));
    }

    pub fn set_shown(&self, _visible: bool) {
        // Menu only offers "Show" (hiding is done with the X button).
        self.show_item.set_text("Show");
    }
}

/// Drain pending tray-menu clicks, returning their ids.
pub fn poll_menu() -> Vec<String> {
    let mut out = Vec::new();
    while let Ok(ev) = MenuEvent::receiver().try_recv() {
        out.push(ev.id.0.clone());
    }
    out
}

/// True if the tray icon was left-clicked since the last call.
pub fn poll_left_click() -> bool {
    let mut clicked = false;
    while let Ok(ev) = tray_icon::TrayIconEvent::receiver().try_recv() {
        if let tray_icon::TrayIconEvent::Click {
            button,
            button_state,
            ..
        } = ev
        {
            if button == tray_icon::MouseButton::Left
                && button_state == tray_icon::MouseButtonState::Up
            {
                clicked = true;
            }
        }
    }
    clicked
}
