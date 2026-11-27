🇧🇷 [[Português|Instalacao]] · 🇺🇸 [[English|Installation]] · 🇪🇸 [[Español|Instalacion]]

# Installation

DriveMirror 0.3.0 has packages for two systems, published in the
[`releases/`](https://github.com/israelguido/drivemirror/tree/main/releases)
folder of the repository.

| System | File |
|---|---|
| Omarchy / Arch Linux (x86_64) | `drivemirror-0.3.0-1-x86_64.pkg.tar.zst` |
| Debian 13 / Ubuntu 24.10 or newer (amd64) | `drivemirror_0.3.0_amd64.deb` |

## 1. Download

```bash
# Omarchy / Arch
curl -LO https://github.com/israelguido/drivemirror/raw/main/releases/drivemirror-0.3.0-1-x86_64.pkg.tar.zst
# Debian / Ubuntu
curl -LO https://github.com/israelguido/drivemirror/raw/main/releases/drivemirror_0.3.0_amd64.deb
```

## 2. Verify the file

Also download `SHA256SUMS` and check:

```bash
curl -LO https://github.com/israelguido/drivemirror/raw/main/releases/SHA256SUMS
sha256sum -c SHA256SUMS --ignore-missing
```

Install only if the file's line ends in `OK`.

## 3. Install

```bash
# Omarchy / Arch
sudo pacman -U drivemirror-0.3.0-1-x86_64.pkg.tar.zst
# Debian 13 / Ubuntu 24.10+
sudo apt install ./drivemirror_0.3.0_amd64.deb
```

The package manager installs the dependencies along with it: GTK 4,
`gtk4-layer-shell` (`libgtk4-layer-shell0` on Debian/Ubuntu), `rclone` and
`xdg-desktop-portal`. The `.deb` does not install on versions older than
Debian 13 or Ubuntu 24.10, because they do not have `libgtk4-layer-shell0`.

## What gets installed

- the `drivemirror` program in `/usr/bin`;
- the shortcut in the applications menu;
- an autostart entry in `/etc/xdg/autostart`, which opens DriveMirror straight
  into the tray when you log in (`drivemirror --tray`).

## First run

Open **DriveMirror** from the applications menu. The full window appears for
the initial setup. Continue at
[[Set up Google Drive|Set-up-Google-Drive]].

## Uninstall

```bash
# Omarchy / Arch
sudo pacman -R drivemirror
# Debian / Ubuntu
sudo apt remove drivemirror
```

The DriveMirror configuration (`~/.config/drivemirror/`) and the rclone one
(`~/.config/rclone/rclone.conf`) stay in your user account; delete them by hand
if you want. No Drive files are deleted.

## Build from source

See the
[Build from source](https://github.com/israelguido/drivemirror#compilar-do-c%C3%B3digo-fonte)
section of the README.
