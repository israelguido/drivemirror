# DriveMirror — Handoff

## Current state

- Repository: <https://github.com/israelguido/drivemirror> (public)
- Local checkout: `/home/israel/Work/drivemirror`
- Branch: `main`
- Last implemented/pushed commit: `9a55fd6` (`Sync selected settings before rclone`)
- The release binary from that commit is installed at `~/.local/bin/drivemirror`.
- Launcher, icon, tray entry, and autostart are installed under the user's `~/.local` / `~/.config/autostart` directories.
- The app is not currently running.

The implemented MVP is Rust + GTK4. It mirrors one local folder to a required rclone remote subfolder with `rclone sync`, propagating local deletions. It has a tray menu, configurable interval, file/folder exclusions, TOML configuration, system-theme integration, Arch packaging, and a Debian package builder.

## Latest bug fix

The user selected a local folder and clicked **Sync now** without clicking **Save settings**. The old UI ignored the current fields, loaded a missing config file, and incorrectly reported that the folder did not exist.

Commit `9a55fd6` fixes this: a manual sync validates and persists the current UI values before constructing the rclone command. Regression test:

- `manual_sync_persists_the_selected_config_before_running`

At the last verification, all 9 tests passed and clippy was clean. The fix was installed locally and pushed.

## Rclone account status

- `rclone` is installed.
- No remote is configured.
- `~/.config/rclone/rclone.conf` does not exist.
- An interactive setup was started with remote name `gdrive` and backend Google Drive, then cancelled at the Client ID prompt. It did not create a partial remote.
- Current rclone warns that its shared Google Client ID is being retired during 2026. Users need their own Google OAuth Client ID and Client Secret.
- Never request that the user paste the Client Secret into chat, tests, logs, or Git history.

## Approved but not implemented

The user approved both items below:

1. Internationalization:
   - Languages: Português (Brasil), English, Español.
   - Default follows the system locale.
   - Manual language override in the app.
   - Persist preference as `system`, `pt-BR`, `en`, or `es`.

2. Native rclone accounts panel:
   - List configured remotes.
   - Create a Google Drive remote from remote name, Client ID, and masked Client Secret.
   - Drive `rclone config create --non-interactive` and its continuation JSON protocol.
   - Let rclone open the browser for OAuth.
   - Reconnect and remove remotes; removal needs explicit confirmation.
   - The app must never persist or log Client Secret/token values. Rclone alone owns `rclone.conf`.
   - The app links to official Google Client ID instructions; it does not create Google Cloud projects or credentials.

## Approved design and plan

Read these before changing code:

- `docs/superpowers/specs/2026-11-27-i18n-design.md`
- `docs/superpowers/specs/2026-11-27-rclone-accounts-design.md`
- `docs/superpowers/plans/2026-11-27-i18n-rclone-accounts.md`

These three files and this handoff are documentation only. No translation or accounts-panel production code has been written.

## Recommended continuation

Follow the plan in native/inline mode to minimize cost:

1. Commit the handoff, specs, and plan before implementation.
2. Task 1: typed compile-time message catalog, legacy TOML migration, locale detection, and live manual selector.
3. Task 2: pure/testable rclone non-interactive protocol and redacted secret-bearing types.
4. Task 3: GTK accounts page, background process flow, packaging, local installation, security scan, and push.

Use TDD. Important inputs to pin down:

- Legacy TOML without `language` loads as `system`.
- `pt_BR.UTF-8` resolves to `pt-BR`; `es-MX` resolves to `es`; unknown locales resolve to `en`.
- Client Secret never appears in `Debug`, status text, process errors, test snapshots, or Git diffs.
- Rclone state/result values are separate argv elements, never a shell command string.
- Malformed JSON, browser cancellation, missing rclone, and incomplete remote creation stay visible and retryable.

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
