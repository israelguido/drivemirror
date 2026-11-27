# DriveMirror Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build, package, install, and publish a Rust desktop app that mirrors one local folder to a private rclone remote folder manually and on a configurable timer.

**Architecture:** A small GTK4 process owns a settings window and a D-Bus StatusNotifierItem. Pure configuration and command-building modules feed a single serialized rclone process; GLib timers trigger it without a service daemon.

**Tech Stack:** Rust, GTK4, zbus, serde/TOML, ashpd, rclone, Cargo, Arch PKGBUILD, Debian packaging

**Spec:** `docs/superpowers/specs/2026-11-27-drivemirror-design.md`

## Global Constraints

- Support Omarchy, Ubuntu, and Pop!_OS.
- Require a remote subfolder and `interval_minutes >= 1`.
- Never pass sharing flags, `--delete-excluded`, or credentials to rclone.
- Keep one process and one sync at a time; default interval is 10 minutes.
- Do not commit generated packages, user configuration, tokens, or rclone data.

## Review Focus

- Empty/root remote destinations must fail validation before spawning rclone.
- Shell metacharacters in paths and patterns must remain literal process arguments.
- Invalid external TOML edits must leave a visible error rather than overwrite the file.
- Timer and manual requests during a run must not create concurrent children.
- A missing rclone binary or unavailable tray must produce an actionable visible error.

---

### Task 1: Configuration and safe command model

**Files:** Create `Cargo.toml`, `src/config.rs`, `src/sync.rs`, `src/lib.rs`, `.gitignore`; test inline modules.

**Interfaces:** Produce `Config::{load,save,validate}`, `default_config_path()`, and `SyncCommand::from_config(&Config)` returning literal arguments.

- [ ] Write tests for defaults, TOML round-trip, interval and destination validation, exclusion arguments, and metacharacter preservation.
- [ ] Run `cargo test` and confirm the tests fail before implementation.
- [ ] Implement the minimum pure configuration and command-building code.
- [ ] Run `cargo test` and confirm all tests pass.
- [ ] Commit the independently working core.

### Task 2: GTK window, timer, tray, and process lifecycle

**Files:** Create `src/main.rs`, `src/app.rs`, `src/tray.rs`, `src/theme.rs`, `assets/drivemirror.svg`.

**Interfaces:** Consume `Config` and `SyncCommand`; produce one GTK application with settings editing, portal folder selection, system-editor launch, manual/timed sync, serialized process state, and tray actions.

- [ ] Add a lifecycle test for rejecting a second sync and state transitions using a fake executable.
- [ ] Implement the GTK settings window and status presentation.
- [ ] Implement the StatusNotifierItem menu and 10-minute/reschedulable GLib timer.
- [ ] Implement live Omarchy accent CSS with GTK fallback and actionable missing-rclone/tray errors.
- [ ] Run `cargo test`, `cargo clippy --all-targets -- -D warnings`, and an offscreen startup smoke test.
- [ ] Commit the complete application.

### Task 3: Desktop integration and packaging

**Files:** Create `packaging/drivemirror.desktop`, `packaging/drivemirror-autostart.desktop`, `packaging/PKGBUILD`, `packaging/debian/*`, `bin/build`, `bin/test`, `bin/install`, `LICENSE`, `README.md`.

**Interfaces:** Produce launcher/autostart entries, an Arch package for Omarchy, and a Debian package definition for Ubuntu/Pop!_OS.

- [ ] Add packaging metadata with runtime dependencies on GTK4, rclone, and desktop portals.
- [ ] Build the release binary and packages; inspect their file lists.
- [ ] Install the Arch package and verify the launcher, autostart entry, icon, and binary.
- [ ] Launch and visually inspect the real window without running a destructive sync.
- [ ] Commit packaging and documentation.

### Task 4: Security review and publication

**Files:** Modify only files found by review; no credentials or user config may be added.

**Interfaces:** Produce a clean public `main` branch at `israelguido/drivemirror`.

- [ ] Inspect `git diff`, tracked files, history, common secret patterns, and package contents.
- [ ] Run the complete test/build/lint suite from a clean tree.
- [ ] Push `main`, inspect the GitHub repository visibility and file list, and report the installed app.

