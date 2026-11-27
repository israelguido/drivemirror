🇧🇷 [[Português|Home]] · 🇺🇸 [[English|Home-EN]] · 🇪🇸 [[Español|Inicio]]

# DriveMirror

DriveMirror is a small Linux app that keeps a **mirrored copy of a folder on
your computer** in a private subfolder of **Google Drive**. It uses
[rclone](https://rclone.org/) under the hood and sits in the system tray,
syncing when you ask and at every configured interval.

![DriveMirror Sync tab with local folder, remote, Drive folder, interval, language and exclusions](https://raw.githubusercontent.com/israelguido/drivemirror/main/docs/images/janela-sincronizacao.png)

*Screenshots taken with demo data.*

## Who it is for

- People on Linux (Omarchy, Arch, Debian 13, Ubuntu 24.10+) who want a folder
  always copied to Drive, without an official Google client.
- People who prefer a simple interface to assembling rclone commands and
  timers by hand.
- People who understand it is a **one-way mirror**: Drive comes to reflect the
  local folder, including the exclusions.

It is not a backup with version history nor a two-way sync. Files deleted on
the computer are deleted on Drive at the next sync. See
[[FAQ|FAQ]].

## Where to start

1. [[Installation|Installation]]: download and install the package.
2. [[Set up Google Drive|Set-up-Google-Drive]]: Client ID, Google Drive API
   and creating the account in the app.
3. [[Daily use|Daily-use]]: tray panel, syncing, interval and exclusions.
4. [[Troubleshooting|Troubleshooting]]: common error messages and what to do.

## Links

- Repository and README: <https://github.com/israelguido/drivemirror>
- Packages: <https://github.com/israelguido/drivemirror/tree/main/releases>
- Security and how to report vulnerabilities:
  <https://github.com/israelguido/drivemirror/blob/main/SECURITY.md>
- License: MIT
