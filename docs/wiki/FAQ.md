🇧🇷 [[Português|Perguntas-frequentes]] · 🇺🇸 [[English|FAQ]] · 🇪🇸 [[Español|Preguntas-frecuentes]]

# FAQ

### Is it a backup?

Not in the sense of keeping old versions. DriveMirror makes Drive match the
local folder: if a file is deleted or damaged on the computer, the next sync
carries that change to Drive. For history, combine it with another backup tool
or with Google Drive's own version history.

### Do changes made on Drive come back to the computer?

No. The copy is one-way (computer → Drive). Changes made directly in the Drive
subfolder are undone at the next sync.

### Can I sync the Drive root?

No. The app requires a subfolder so that a mirror cannot delete the rest of
your Drive.

### Can I sync more than one folder?

Not in this version: one local folder to one Drive subfolder.

### Why do I need to create my own Client ID?

rclone's shared Client ID is being retired by Google, and a Client ID of your
own avoids shared usage limits. The app does not create projects or credentials
in Google Cloud for you.

### What does DriveMirror store of my credentials?

Nothing. rclone is what stores the Client Secret and the token, in
`~/.config/rclone/rclone.conf`. Details in
[SECURITY.md](https://github.com/israelguido/drivemirror/blob/main/SECURITY.md).

### What Drive access does the app request?

The remote is created with rclone's `drive` scope, which gives full access to
the authorized account's Drive. This is needed to sync into any subfolder.

### Does removing a remote delete my files?

No. **Remove** deletes only the remote's configuration in rclone. The files on
Drive stay there.

### Are ignored files deleted from Drive?

No. Items that match the exclusions are not uploaded and are not deleted from
the destination either.

### Does the app sync with the computer off or with the app closed?

No. The automatic sync happens while DriveMirror is open (it starts in the tray
when you log in).

### Does it work on GNOME?

It does, with two differences: the panel opens as a small window (GNOME does
not offer the layer-shell protocol), and the tray icon needs an AppIndicator
extension. Without an icon, use the app from the applications menu.

### How do I change the language?

In the **Language** selector of the Sync tab. "System" follows your session's
language.
