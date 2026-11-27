🇧🇷 [[Português|Uso-diario]] · 🇺🇸 [[English|Daily-use]] · 🇪🇸 [[Español|Uso-cotidiano]]

# Uso cotidiano

## Configurar qué sincronizar

![Pestaña Sincronización de DriveMirror con carpeta local, remoto, carpeta en Drive, intervalo, idioma y exclusiones](https://raw.githubusercontent.com/israelguido/drivemirror/main/docs/images/janela-sincronizacao.png)


En la pestaña **Sincronización** de la ventana completa:

| Campo | Qué indicar |
|---|---|
| **Carpeta local** | La carpeta del equipo que se va a reflejar. Usa **Elegir…**. Debe existir. |
| **Remoto de rclone** | Solo el nombre del remoto, por ejemplo `gdrive`, sin `:`. |
| **Carpeta en Drive** | La subcarpeta de destino, por ejemplo `Backup/Obsidian`. No se acepta la raíz de Drive, ni `.` o `..`. |
| **Intervalo (min)** | De 1 a 1440 minutos (24 h). Predeterminado: 10. |
| **Idioma** | Sistema, Português (Brasil), English o Español. Cambia al instante. |

Haz clic en **Guardar configuración**, o directamente en **Sincronizar ahora**,
que guarda antes de ejecutar.

> **Atención:** es un espejo. Lo que borres en la carpeta local se borra en la
> subcarpeta de Drive en la siguiente sincronización, y los archivos que existan
> solo en el destino se eliminan. Usa una subcarpeta dedicada.

## Sincronizar

- **Manual:** **Sincronizar ahora** en la ventana, en el panel o en el menú del
  icono.
- **Automática:** en cada intervalo, mientras la app esté abierta (en la
  bandeja o con la ventana).
- Solo se ejecuta una sincronización a la vez. El resultado aparece en la línea
  de estado de la ventana y en el estado del panel.
- Si la sincronización automática no puede ejecutarse (configuración no
  válida, remoto inexistente o incompleto), el aviso aparece una vez. Solo
  vuelve a intentarlo cuando cambia la configuración o la lista de remotos. La
  sincronización manual siempre lo intenta.

## Exclusiones

Dos campos, un patrón por línea, en el formato de filtro de rclone:

- **Archivos ignorados**: valor inicial `.DS_Store`, `Thumbs.db`,
  `desktop.ini`, `*~`, `*.swp`, `*.tmp`, `*.bak`;
- **Carpetas ignoradas**: valor inicial `.Trash-*/**` y `.cache/**`. Para
  ignorar una carpeta, usa `nombre/**`.

Los elementos excluidos no se suben, y lo que ya esté en Drive con esos nombres
**no se borra** (la app nunca usa `--delete-excluded`).

## Panel de la bandeja

Un clic en el icono abre un panel con el estado (**Al día**,
**Sincronizando…**, **Error** o **No configurado**), la hora de la última
sincronización, el origen y el destino, y los botones **Sincronizar ahora**,
**Configuración…** y **Salir**.

- En Hyprland, Sway y KDE Plasma (Wayland) aparece en la esquina superior
  derecha, debajo de la barra, y se cierra con **Esc**, con otro clic en el
  icono o poco después de que el puntero salga de él.
- En otros entornos se abre como una ventana pequeña.
- El menú del icono (clic derecho) tiene **Configuración…**,
  **Sincronizar ahora** y **Salir**.
- Cerrar la ventana completa solo la oculta; la app sigue en la bandeja.
- **Salir** espera a que termine una sincronización en curso.

La hora de la última sincronización se guarda solo en memoria: después de
reiniciar la app, muestra "nunca" hasta la siguiente sincronización.

## Archivo de configuración

Todo está en `~/.config/drivemirror/config.toml` (botón
**Abrir archivo de configuración**). Si editas el archivo con la app abierta,
recarga los valores; si hay cambios sin guardar en la ventana, la app avisa en
lugar de sobrescribir.

```toml
local_folder = "/home/voce/Notas/Obsidian"
remote = "gdrive"
remote_folder = "Backup/Obsidian"
interval_minutes = 10
exclude = [".DS_Store", "*.tmp"]
exclude_folders = [".cache/**"]
language = "system"
```

El archivo no guarda credenciales.
