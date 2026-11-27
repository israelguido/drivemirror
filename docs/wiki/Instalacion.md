🇧🇷 [[Português|Instalacao]] · 🇺🇸 [[English|Installation]] · 🇪🇸 [[Español|Instalacion]]

# Instalación

DriveMirror 0.3.0 tiene paquetes para dos sistemas, publicados en la carpeta
[`releases/`](https://github.com/israelguido/drivemirror/tree/main/releases)
del repositorio.

| Sistema | Archivo |
|---|---|
| Omarchy / Arch Linux (x86_64) | `drivemirror-0.3.0-1-x86_64.pkg.tar.zst` |
| Debian 13 / Ubuntu 24.10 o más reciente (amd64) | `drivemirror_0.3.0_amd64.deb` |

## 1. Descargar

```bash
# Omarchy / Arch
curl -LO https://github.com/israelguido/drivemirror/raw/main/releases/drivemirror-0.3.0-1-x86_64.pkg.tar.zst
# Debian / Ubuntu
curl -LO https://github.com/israelguido/drivemirror/raw/main/releases/drivemirror_0.3.0_amd64.deb
```

## 2. Verificar el archivo

Descarga también `SHA256SUMS` y verifica:

```bash
curl -LO https://github.com/israelguido/drivemirror/raw/main/releases/SHA256SUMS
sha256sum -c SHA256SUMS --ignore-missing
```

Instala solo si la línea del archivo termina en `OK`.

## 3. Instalar

```bash
# Omarchy / Arch
sudo pacman -U drivemirror-0.3.0-1-x86_64.pkg.tar.zst
# Debian 13 / Ubuntu 24.10+
sudo apt install ./drivemirror_0.3.0_amd64.deb
```

El gestor de paquetes instala también las dependencias: GTK 4,
`gtk4-layer-shell` (`libgtk4-layer-shell0` en Debian/Ubuntu), `rclone` y
`xdg-desktop-portal`. El `.deb` no se instala en versiones anteriores a
Debian 13 o Ubuntu 24.10, porque no tienen `libgtk4-layer-shell0`.

## Qué se instala

- el programa `drivemirror` en `/usr/bin`;
- el acceso directo en el menú de aplicaciones;
- una entrada de inicio automático en `/etc/xdg/autostart`, que abre
  DriveMirror directamente en la bandeja al iniciar sesión
  (`drivemirror --tray`).

## Primera ejecución

Abre **DriveMirror** desde el menú de aplicaciones. La ventana completa aparece
para la configuración inicial. Continúa en
[[Configuración de Google Drive|Configuracion-de-Google-Drive]].

## Desinstalar

```bash
# Omarchy / Arch
sudo pacman -R drivemirror
# Debian / Ubuntu
sudo apt remove drivemirror
```

La configuración de DriveMirror (`~/.config/drivemirror/`) y la de rclone
(`~/.config/rclone/rclone.conf`) quedan en tu usuario; bórralas a mano si
quieres. No se borra ningún archivo de Drive.

## Compilar desde el código fuente

Consulta la sección
[Compilar desde el código fuente](https://github.com/israelguido/drivemirror#compilar-do-c%C3%B3digo-fonte)
del README.
