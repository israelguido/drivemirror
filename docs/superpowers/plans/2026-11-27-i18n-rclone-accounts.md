# DriveMirror I18n and Rclone Accounts Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add system-aware Portuguese, English, and Spanish UI plus native Google Drive remote setup through rclone.

**Architecture:** A dependency-free typed message catalog translates the GTK and tray UI from a persisted language preference. A separate rclone account module executes literal argv, models the non-interactive JSON continuation protocol, and exposes sanitized state to GTK without retaining secrets.

**Tech Stack:** Rust, GTK4, serde/TOML, rclone CLI, StatusNotifierItem

**Spec:** `docs/superpowers/specs/2026-11-27-i18n-design.md` and `docs/superpowers/specs/2026-11-27-rclone-accounts-design.md`

## Global Constraints

- Languages are `system`, `pt-BR`, `en`, and `es`; existing configs default to `system`.
- Portuguese system locales select `pt-BR`, Spanish locales select `es`, and all others select `en`.
- Client Secret is never persisted or logged by DriveMirror; only rclone owns its config.
- Every process uses literal argv without a shell.
- Google Drive is the only remote type created in this MVP.
- Creating Google Cloud projects or OAuth credentials remains outside the app.

## Review Focus

- Configs written before the language field existed must continue to load.
- Locale variants such as `pt_BR.UTF-8` and `es-MX` must resolve correctly.
- Process errors and JSON parsing must not include Client Secret in user messages.
- Continuation state/result must round-trip as individual argv values, never shell text.
- Cancelling the browser flow must leave no half-configured remote presented as usable.

---

### Task 1: Typed translations

**Files:** Create `src/i18n.rs`; modify `src/config.rs`, `src/lib.rs`, `src/app.rs`, `src/tray.rs`, desktop entries, and `tests/core.rs`.

**Interfaces:** Produce `Language::{resolve,code}` and `tr(Language, Message) -> &'static str`; add `Config.language` with serde default.

- [ ] Write failing tests for legacy TOML, system locale resolution, and representative messages in all three languages.
- [ ] Implement the typed catalog and config migration.
- [ ] Add the language selector, immediate window refresh, translated tray menu, status/errors, and desktop metadata.
- [ ] Run `./bin/test`, then commit.

### Task 2: Safe rclone account protocol

**Files:** Create `src/accounts.rs`; modify `src/lib.rs` and `tests/core.rs`.

**Interfaces:** Produce `list_remotes`, `ConfigQuestion::parse`, `CreateDriveRequest::command`, `ContinueRequest::command`, `reconnect_command`, and `delete_command` as literal programs/args; secret-bearing types have redacted `Debug`.

- [ ] Write failing tests for remote listing, question JSON, literal continuation arguments, redacted secrets, malformed output, and command construction.
- [ ] Implement the minimum synchronous command/protocol model; process execution remains off the GTK thread.
- [ ] Run `./bin/test`, then commit.

### Task 3: Accounts UI, packaging, and release

**Files:** Modify `src/app.rs`, `README.md`, packaging files, and version metadata.

**Interfaces:** Consume Task 1 messages and Task 2 protocol to provide an accounts page with list/create/reconnect/remove, browser progress, confirmation, and official-help link.

- [ ] Add a process-lifecycle test using a fake rclone executable, including cancellation/failure and no secret leakage.
- [ ] Implement the accounts page and background process state without blocking GTK.
- [ ] Build packages, update the local installation, smoke-test all locales and exact autostart command.
- [ ] Audit history/diff for secrets, run `./bin/test` and release build, commit, push, and verify GitHub.
