🇧🇷 [[Português|Solucao-de-problemas]] · 🇺🇸 [[English|Troubleshooting]] · 🇪🇸 [[Español|Solucion-de-problemas]]

# Troubleshooting

### "The remote "gdrive" does not exist in rclone. Create it in the Rclone accounts tab."

The name in **Rclone remote** does not match any configured remote.

- Create the account in the **Rclone accounts** tab
  ([[Set up Google Drive|Set-up-Google-Drive]]), or
- fix the name: only the name, without `:` and without the folder.

The automatic sync shows this warning once and only tries again when the
configuration or the remote list changes.

### "Google Drive API has not been used in project … before or it is disabled"

The Google Drive API is not enabled in your Client ID's project.

1. Open the [Google Cloud Console](https://console.cloud.google.com/) in the
   Client ID's project.
2. **APIs & Services → Library → Google Drive API → Enable**.
3. Wait a few minutes and click **Sync now**.

### Remote "incomplete" or "The remote … has not finished authorization"

Account creation stopped before Google returned the token: the browser was
closed, the operation was cancelled or the authorization failed. The app does
not let you sync with that remote.

- In the **Rclone accounts** tab, click **Reconnect** and authorize in the
  browser; or
- click **Remove** and create the account again.

### The browser did not open to authorize

While the app shows "Waiting for authorization in the browser…", click
**Open authorization link** (hover over it to see the address). The address
starts with `http://127.0.0.1` and only works on this computer, while the
authorization is pending.

### Authorization stops working after a few days

If the app in Google Cloud is in **testing** mode, Google may expire the
authorization after a few days. Use **Reconnect** in the Rclone accounts tab or
publish the app on the OAuth consent screen, as described in the
[rclone guide](https://rclone.org/drive/#making-your-own-client-id).

### The panel does not open below the bar

The anchored panel uses the layer-shell protocol, available on Hyprland, Sway
and KDE Plasma (Wayland). On GNOME, X11 and other compositors it opens as a
small window, which is expected.

### No tray icon appears

The bar needs to support StatusNotifierItem (Waybar, KDE Plasma etc.). On GNOME
install an AppIndicator extension. Without a tray, the app warns in the full
window and keeps working through it; open it from the applications menu.

### "Choose a remote subfolder; the Drive root is not allowed"

Enter a subfolder in **Drive folder**, for example `Backup/Obsidian`.

### "The config changed externally; review it before saving"

`config.toml` was changed outside the app while there were unsaved changes in
the window. Check the values shown and save again.

### "rclone was not found; install it or check PATH"

Install rclone (`sudo pacman -S rclone` or `sudo apt install rclone`).

### Still having problems?

Open an issue at <https://github.com/israelguido/drivemirror/issues>
describing the distribution, the version and the message shown. Do not include
the Client Secret, tokens or your `rclone.conf`. For security flaws, follow
[SECURITY.md](https://github.com/israelguido/drivemirror/blob/main/SECURITY.md).
