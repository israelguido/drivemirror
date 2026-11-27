🇧🇷 [[Português|Solucao-de-problemas]] · 🇺🇸 [[English|Troubleshooting]] · 🇪🇸 [[Español|Solucion-de-problemas]]

# Solución de problemas

### "El remoto "gdrive" no existe en rclone. Créalo en la pestaña Cuentas de rclone."

El nombre en **Remoto de rclone** no corresponde a ningún remoto configurado.

- Crea la cuenta en la pestaña **Cuentas de rclone**
  ([[Configuración de Google Drive|Configuracion-de-Google-Drive]]), o
- corrige el nombre: solo el nombre, sin `:` y sin la carpeta.

La sincronización automática muestra este aviso una vez y solo vuelve a
intentarlo cuando cambia la configuración o la lista de remotos.

### "Google Drive API has not been used in project … before or it is disabled"

La Google Drive API no está habilitada en el proyecto de tu Client ID.

1. Abre la [Google Cloud Console](https://console.cloud.google.com/) en el
   proyecto del Client ID.
2. **APIs y servicios → Biblioteca → Google Drive API → Habilitar**.
3. Espera unos minutos y haz clic en **Sincronizar ahora**.

### Remoto "incompleto" o "El remoto … no completó la autorización"

La creación de la cuenta se detuvo antes de que Google devolviera el token: se
cerró el navegador, se canceló la operación o falló la autorización. La app no
deja sincronizar con ese remoto.

- En la pestaña **Cuentas de rclone**, haz clic en **Reconectar** y autoriza en
  el navegador; o
- haz clic en **Eliminar** y crea la cuenta de nuevo.

### El navegador no se abrió para autorizar

Mientras la app muestra "Esperando la autorización en el navegador…", haz clic
en **Abrir enlace de autorización** (pasa el ratón por encima para ver la
dirección). La dirección empieza por `http://127.0.0.1` y solo funciona en este
equipo, mientras la autorización está pendiente.

### La autorización deja de funcionar después de unos días

Si la app en Google Cloud está en modo de **prueba**, Google puede hacer
caducar la autorización después de unos días. Usa **Reconectar** en la pestaña
Cuentas de rclone o publica la app en la pantalla de consentimiento de OAuth,
según la [guía de rclone](https://rclone.org/drive/#making-your-own-client-id).

### El panel no se abre debajo de la barra

El panel anclado usa el protocolo layer-shell, disponible en Hyprland, Sway y
KDE Plasma (Wayland). En GNOME, X11 y otros compositores se abre como ventana
pequeña, lo cual es lo esperado.

### No aparece el icono en la bandeja

La barra debe admitir StatusNotifierItem (Waybar, KDE Plasma, etc.). En GNOME
instala una extensión de AppIndicator. Sin bandeja, la app avisa en la ventana
completa y sigue funcionando desde ella; ábrela desde el menú de aplicaciones.

### "Elige una subcarpeta remota; la raíz de Drive no está permitida"

Indica una subcarpeta en **Carpeta en Drive**, por ejemplo `Backup/Obsidian`.

### "La configuración cambió externamente; revísala antes de guardar"

`config.toml` se modificó fuera de la app mientras había cambios sin guardar en
la ventana. Revisa los valores mostrados y guarda de nuevo.

### "No se encontró rclone; instálalo o revisa el PATH"

Instala rclone (`sudo pacman -S rclone` o `sudo apt install rclone`).

### ¿Sigues con problemas?

Abre una issue en <https://github.com/israelguido/drivemirror/issues>
describiendo la distribución, la versión y el mensaje mostrado. No incluyas el
Client Secret, tokens ni tu `rclone.conf`. Para fallos de seguridad, sigue
[SECURITY.md](https://github.com/israelguido/drivemirror/blob/main/SECURITY.md).
