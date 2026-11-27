# DriveMirror releases

| Sistema | Arquivo |
|---|---|
| Omarchy / Arch Linux | `drivemirror-0.3.0-1-x86_64.pkg.tar.zst` |
| Debian 13 / Ubuntu 24.10+ | `drivemirror_0.3.0_amd64.deb` |

Omarchy / Arch:

```bash
sudo pacman -U drivemirror-0.3.0-1-x86_64.pkg.tar.zst
```

Debian / Ubuntu (requires `libgtk4-layer-shell0`, available from Debian 13 and Ubuntu 24.10):

```bash
sudo apt install ./drivemirror_0.3.0_amd64.deb
```

Verify downloads with `sha256sum -c SHA256SUMS`.
