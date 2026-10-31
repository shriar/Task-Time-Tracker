use chrono::{Local, NaiveDate, TimeZone};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub name: String,
    pub parent_id: Option<u64>,
    pub created_at: i64,
    #[serde(default)]
    pub archived: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: u64,
    pub task_id: u64,
    pub start: i64,
    pub end: Option<i64>,
}

impl TimeEntry {
    pub fn is_running(&self) -> bool {
        self.end.is_none()
    }

    pub fn duration_secs(&self, now: i64) -> i64 {
        let end = self.end.unwrap_or(now);
        (end - self.start).max(0)
    }
}

pub fn now_ts() -> i64 {
    Local::now().timestamp()
}

pub fn today() -> NaiveDate {
    Local::now().date_naive()
}

/// Local calendar day an entry belongs to (by its start time).
pub fn day_of(ts: i64) -> NaiveDate {
    Local
        .timestamp_opt(ts, 0)
        .single()
        .map(|d| d.date_naive())
        .unwrap_or_else(today)
}

/// 95 -> "1h 35m", 20 -> "20m", 0 -> "0m"
pub fn format_hm(secs: i64) -> String {
    let total_min = (secs.max(0)) / 60;
    if total_min < 60 {
        format!("{}m", total_min)
    } else {
        format!("{}h {:02}m", total_min / 60, total_min % 60)
    }
}

/// 3723 -> "01:02:03" (for the live running timer)
pub fn format_hms(secs: i64) -> String {
    let s = secs.max(0);
    format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
}
