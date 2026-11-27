# DriveMirror Design

## Intent

DriveMirror keeps one local folder as the source of truth for a private folder on an existing rclone Google Drive remote. It runs on Omarchy, Ubuntu, and Pop!_OS, offers manual sync from its window and tray menu, and automatically syncs at a configurable interval (10 minutes by default).

## Goals

- Run `rclone sync` from the selected local folder to a required subfolder on a configured remote, intentionally propagating local deletions.
- Never create public links, shares, or permissions, and never read or log rclone credentials.
- Provide a small Rust/GTK4 interface that follows the system light/dark theme and uses Omarchy's live accent when available.
- Stay resident in a StatusNotifierItem tray, start with the desktop session, serialize sync runs, and expose clear idle/running/success/error state.
- Store settings in a human-editable TOML file and open it in the system editor from the app.
- Let users edit file and folder exclusions. Ship conservative defaults for OS litter and temporary files; excluded remote files remain untouched because `--delete-excluded` is never used.
- Ship an Arch package and a Debian package, install locally on Omarchy, and publish source without secrets to the public GitHub repository `israelguido/drivemirror`.

## Non-goals

Bidirectional sync, multiple sync pairs, Google login/configuration inside the app, version history, and sharing controls are excluded.

## Constraints and safety

- GTK4 plus a D-Bus StatusNotifierItem is preferred over egui for native desktop theming and a smaller implementation.
- Local folder, remote name, non-root remote folder, and an interval of at least one minute are required.
- `rclone sync` is intentionally destructive on the remote. The app shows the exact source and destination and does not run until validation succeeds.
- Only one rclone child process runs at a time. Errors remain visible and are not treated as partial success.
- Configuration contains paths, patterns, remote names, and timing only; credentials remain in rclone's own configuration.

## Configuration

The TOML file lives under the platform configuration directory as `drivemirror/config.toml` and contains `local_folder`, `remote`, `remote_folder`, `interval_minutes`, `exclude`, and `exclude_folders`. Defaults include `.DS_Store`, `Thumbs.db`, `desktop.ini`, `*~`, `*.swp`, `*.tmp`, `*.bak`, `.Trash-*/**`, and `.cache/**`; `*.sql` is not excluded by default.

## Approaches

The selected approach is a single GTK4 Rust process containing the window, interval timer, rclone child process, and StatusNotifierItem. A systemd timer was rejected because it duplicates state and packaging. egui was rejected because it integrates less naturally with GTK desktops and file portals.

