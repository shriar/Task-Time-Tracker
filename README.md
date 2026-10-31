# Task Time Tracker

Minimal desktop time tracker in Rust (`eframe`/`egui`, light theme).

## Features
- Tasks + 1-level subtasks
- One active timer at a time (starting another auto-stops the current one)
- Live elapsed counter, daily total header
- Quick manual add (`+15m`, `+1h`)
- **Today / History / Graph** tabs (7-day / 30-day bar chart)
- **Floating always-on-top mini timer** — Show/Hide from the Today tab,
  closing the mini window just hides it
- Closing the app auto-stops a running timer (it can't inflate overnight);
  a timer left running by a crash is stopped on next start

## Data
- Single file `tasks_data.json` in the **same folder as the `.exe`**
- Atomic writes (temp file + rename), plain JSON — copy it to back up

## Run / Build
```powershell
$env:Path = "D:\Rust\cargo\bin;$env:Path"
cargo run            # debug exe: target\debug\task-tracker.exe
cargo build --release  # release exe: target\release\task-tracker.exe
```
Copy the `.exe` anywhere — `tasks_data.json` is created next to it on first save.

## Toolchain (this machine)
- Rust stable `x86_64-pc-windows-gnu` in `D:\Rust` (`CARGO_HOME`/`RUSTUP_HOME`)
- Linker from WinLibs MinGW in `D:\mingw64\bin` (on PATH)
