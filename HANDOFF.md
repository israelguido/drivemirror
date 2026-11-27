# DriveMirror — Handoff

## Current state

- Repository: <https://github.com/israelguido/drivemirror> (public), branch `main`.
- Version 0.2.0 is implemented, installed at `~/.local/bin/drivemirror`, and pushed.
- Rust + GTK4 app that mirrors one local folder to a required rclone remote subfolder with `rclone sync` (local deletions propagate), with tray, interval, exclusions, TOML config, Arch and Debian packaging.
- 0.2.0 adds:
  - UI in Português (Brasil), English and Español; follows the system locale (`LC_ALL`, `LC_MESSAGES`, `LANG`) with a manual override persisted as `language = "system" | "pt-BR" | "en" | "es"`; legacy configs load as `system`.
  - "Rclone accounts" tab: list remotes, create a Google Drive remote (name, Client ID, masked Client Secret) via `rclone config create/update --non-interactive` continuation, show the OAuth link while waiting for the browser, cancel, reconnect (`config update <name> config_refresh_token=true`), remove with confirmation (never deletes Drive files).
  - Drive remotes without a token are flagged as incomplete (only presence of the key is checked; dump values never leave the runner thread) and sync refuses them.
  - One remote-name validator shared by sync and accounts; names starting with `-` are rejected so they cannot become rclone flags.
- Modules: `src/i18n.rs`, `src/accounts.rs` (pure protocol), `src/account_runner.rs` (process thread, redaction), `src/accounts_page.rs` (GTK).
- Tests: 32 (tests/core.rs, tests/accounts.rs, tests/account_runner.rs with a fake rclone).

## Rclone account status

- No real remote has been configured on this machine yet; `~/.config/rclone/rclone.conf` does not exist.
- Users need their own Google OAuth Client ID/Secret (rclone's shared ID is being retired in 2026). Never ask for the Client Secret in chat, tests, logs, or Git.

## Not verified

- The account flows (create, continue, OAuth, reconnect, remove) were exercised only with a fake rclone and a GUI smoke start; no click-through with real rclone and a real Google account yet.

## Accepted risks

- Client ID/Secret and password answers are passed as rclone argv, visible in `/proc` while rclone runs; rclone stores them in plain `rclone.conf` anyway.
- If the incomplete-remote check has not finished at startup, an immediate manual sync is not blocked by it.

## Verification and installation commands

```bash
cd /home/israel/Work/drivemirror
./bin/test
cargo build --release --locked
```

Install locally without an administrator password:

```bash
install -Dm755 target/release/drivemirror ~/.local/bin/drivemirror
install -Dm644 assets/drivemirror.svg ~/.local/share/icons/hicolor/scalable/apps/drivemirror.svg
install -Dm644 packaging/drivemirror.desktop ~/.local/share/applications/drivemirror.desktop
install -Dm644 packaging/drivemirror-autostart.desktop ~/.config/autostart/drivemirror.desktop
```

Build packages:

```bash
./packaging/debian/build-deb
(cd packaging && makepkg -df --noconfirm)
```

The Debian package should be built on the target distribution to match its GTK/glibc. System-wide Arch installation requires the user's administrator password; the current installation is user-local.

## Known accepted behavior and risks

- `rclone sync` is intentionally destructive on the remote: deletions in the local source are propagated.
- Remote root sync is blocked; a subfolder is required.
- Exclusions do not use `--delete-excluded`, so excluded remote items are preserved.
- A forced GUI kill during an active rclone child can leave that child running; normal tray Quit waits for completion.
- Do not run a real sync until the user has reviewed the selected source and destination and a valid remote exists.
