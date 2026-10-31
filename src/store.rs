use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::model::{Task, TimeEntry};

/// Whole database. Saved as one JSON file (no C compiler / SQLite needed).
#[derive(Default, Serialize, Deserialize)]
pub struct Db {
    pub tasks: Vec<Task>,
    pub entries: Vec<TimeEntry>,
    pub next_id: u64,
}

impl Db {
    pub fn alloc_id(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Atomic save: write temp file, then rename.
    pub fn save(&self, path: &Path) {
        if let Ok(text) = serde_json::to_string_pretty(self) {
            let tmp = path.with_extension("tmp");
            if fs::write(&tmp, text).is_ok() {
                let _ = fs::rename(&tmp, path);
            }
        }
    }
}

/// Data file lives next to the .exe (falls back to current dir).
pub fn data_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("tasks_data.json");
        }
    }
    PathBuf::from("tasks_data.json")
}
