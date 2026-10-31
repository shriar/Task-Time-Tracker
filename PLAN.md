# Task Time Tracker — Plan (Rust, Minimal)

## 1. Goal
Minimal desktop time tracker in the style of the reference screenshot.
- Projects (+ subtasks), archive / unarchive
- One Start/Stop timer at a time, live elapsed
- Track tab with per-project color bars, progress, today + avg
- Log tab (history), Stats tab (graph + averages + streak)
- Bottom status bar: Today · This week · N days in a row
- Floating always-on-top mini timer (show/hide)
- Warm dark theme, monospace type

## 2. Tech Stack (proposed)
- **Language:** Rust (stable)
- **GUI:** `eframe` / `egui` — pure Rust, single .exe, minimal, easy light theme
- **Storage:** JSON file (`tasks_data.json` next to the .exe, atomic writes) —
  pure Rust, no C compiler needed on the GNU toolchain
- **Time:** `chrono`
- **Graph:** hand-drawn bars with plain egui shapes (no extra crate)
- **Serialization:** `serde`, `serde_json` (for export, optional)

Why this stack:
- No Node.js / webview needed (unlike Tauri)
- Fast build, small binary
- Minimal UI matches requirement
- SQLite is reliable for history + totals

Alternatives (if you want change):
- Tauri + HTML/CSS (prettier but heavier)
- Iced / Slint (also OK, but egui is simplest)
- JSON file instead of SQLite (simpler but breaks with large history)

## 3. Features

### Tab 1: Track
- Rows: `▍ Project — [▶]` + progress bar + `2h 35m today · avg 5h 05m`
- Per-project accent color bar (blue/orange/green/yellow/purple/teal)
- Progress = today's time vs the day's max project
- Only 1 timer runs at a time; live `HH:MM:SS` while running
- Subtasks indented (`└`), same play buttons
- `[Mini timer]` toggle button
- Archived projects are hidden here

### Tab 2: Log
- Date selector: `< 2026-10-30 >` + `[Today]` button
- Per-project rows with color bars, sessions, incl. `[archived]` tag
- Daily total at top
- (No "sheets" concept — one continuous log per day)

### Tab 3: Stats
- Toggle: `[7 days] [30 days]`
- Gold bar chart with hour labels; list `date — total` below
- Header: `This week X · N days in a row`
- `Averages` per project (avg/day, archived included)

### Tab 4: Projects
- `[+ New project]` textbox + button
- Active list: color bar, name, subtask count, `[Archive] [Delete]`
- Archived list: `[Unarchive] [Delete]` + note that Log/Stats history is kept
- Archiving a running project stops its timer

### 3.5 Floating Mini Timer (always on top)
- Small window (~260×110) that stays on top of all windows
- Shows: running task name + live `HH:MM:SS` + `[Stop]` button
- When idle: shows "No timer running" + today's total
- Show/hide: `[Show mini timer]` / `[Hide mini timer]` button in Today tab;
  closing the mini window (✕) hides it, reopen anytime from main window
- Implemented as an egui immediate child viewport
  (`ViewportBuilder::with_always_on_top()`), same process, no extra exe

## 4. Data Model + Storage

- **Location:** `tasks_data.json` in the SAME folder as the `.exe`
  (falls back to current dir if exe path is unavailable)
- Copy/backup = just copy that one file next to the exe

```sql
-- data model (stored as JSON in tasks_data.json) --
tasks:   { id, name, parent_id, created_at }
entries: { id, task_id, start, end (null = running), duration = end - start }
```

- Task with `parent_id = NULL` = top-level, else subtask (1 level deep for simplicity)
- Running timer = `time_entries` row with `end = NULL`
- On Stop: set `end = now`, `duration_secs = end - start`
- Daily total = SUM(duration_secs) + live running timer
- History = GROUP BY date(start)

## 5. File Structure
```
Task Tracker/
  Cargo.toml
  PLAN.md (this file)
  src/
    main.rs       — entry, light theme setup
    app.rs        — App state, tabs, timer tick
    model.rs      — Task, TimeEntry structs, helpers (format time)
    store.rs      — JSON load/save (tasks_data.json next to exe)
    ui_today.rs   — Today tab UI (+ mini-timer toggle)
    ui_mini.rs    — floating always-on-top timer window
    ui_history.rs — History tab UI
    ui_graph.rs   — Graph tab UI
  tasks_data.json (auto-created next to the exe at runtime)
```

## 6. UI Style
- Light theme with blue accent (#2563EB), green Start / red Stop buttons
- Body 16px, headings 24px, roomy padding
- Sans-serif, minimal spacing, clear hierarchy

## 7. Build & Run
```powershell
cargo run
# exe in target/debug/task-tracker.exe
cargo build --release
```

## 8. Steps After Approval
1. `cargo init` + add deps
2. Implement model + store
3. Implement Today + timer
4. Implement History + Graph
5. Light theme polish + test start/stop/persist/restart
6. README with usage

## 9. Approval Needed
- [x] Stack: egui + JSON file (SQLite dropped — no C compiler on GNU toolchain)
- [x] 3 tabs: Today / History / Graph
- [x] One active timer at a time + 1-level subtasks
- [x] NEW: floating always-on-top mini timer with show/hide
- [x] NEW: data file lives next to the exe
