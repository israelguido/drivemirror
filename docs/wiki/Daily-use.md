🇧🇷 [[Português|Uso-diario]] · 🇺🇸 [[English|Daily-use]] · 🇪🇸 [[Español|Uso-cotidiano]]

# Daily use

## Choose what to sync

![DriveMirror Sync tab with local folder, remote, Drive folder, interval, language and exclusions](https://raw.githubusercontent.com/israelguido/drivemirror/main/docs/images/janela-sincronizacao.png)


In the **Sync** tab of the full window:

| Field | What to enter |
|---|---|
| **Local folder** | The folder on the computer to mirror. Use **Choose…**. It must exist. |
| **Rclone remote** | Only the remote name, for example `gdrive`, without `:`. |
| **Drive folder** | The destination subfolder, for example `Backup/Obsidian`. The Drive root is not accepted, nor `.` or `..`. |
| **Interval (min)** | From 1 to 1440 minutes (24 h). Default: 10. |
| **Language** | System, Português (Brasil), English or Español. Changes immediately. |

Click **Save settings**, or go straight to **Sync now**, which saves before
running.

> **Warning:** this is a mirror. Whatever you delete in the local folder is
> deleted in the Drive subfolder at the next sync, and files that exist only at
> the destination are removed. Use a dedicated subfolder.

## Syncing

- **Manual:** **Sync now** in the window, in the panel or in the icon menu.
- **Automatic:** at every interval, while the app is open (in the tray or with
  the window).
- Only one sync runs at a time. The result appears in the window's status line
  and in the panel state.
- If the automatic sync cannot run (invalid configuration, missing or
  incomplete remote), the warning appears once. It only tries again when the
  configuration or the remote list changes. A manual sync always tries.

## Exclusions

Two fields, one pattern per line, in rclone filter format:

- **Ignored files**: initial default `.DS_Store`, `Thumbs.db`, `desktop.ini`,
  `*~`, `*.swp`, `*.tmp`, `*.bak`;
- **Ignored folders**: initial default `.Trash-*/**` and `.cache/**`. To ignore
  a folder, use `name/**`.

Excluded items are not uploaded, and anything already on Drive with those names
**is not deleted** (the app never uses `--delete-excluded`).

## Tray panel

A click on the icon opens a panel with the state (**Up to date**,
**Syncing…**, **Failed** or **Not configured**), the time of the last sync, the
source and destination, and the buttons **Sync now**, **Settings…** and
**Quit**.

- On Hyprland, Sway and KDE Plasma (Wayland) it appears in the top-right
  corner, below the bar, and closes with **Esc**, with another click on the
  icon or shortly after the pointer leaves it.
- In other environments it opens as a small window.
- The icon menu (right click) has **Settings…**, **Sync now** and **Quit**.
- Closing the full window only hides it; the app stays in the tray.
- **Quit** waits for a sync in progress to finish.

The time of the last sync is kept only in memory: after restarting the app, it
shows "never" until the next sync.

## Configuration file

Everything lives in `~/.config/drivemirror/config.toml` (button
**Open config file**). If you edit the file while the app is open, it reloads
the values; if there are unsaved changes in the window, the app warns you
instead of overwriting.

```toml
local_folder = "/home/voce/Notas/Obsidian"
remote = "gdrive"
remote_folder = "Backup/Obsidian"
interval_minutes = 10
exclude = [".DS_Store", "*.tmp"]
exclude_folders = [".cache/**"]
language = "system"
```

The file does not store credentials.
