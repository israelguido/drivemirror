🇧🇷 [[Português|Perguntas-frequentes]] · 🇺🇸 [[English|FAQ]] · 🇪🇸 [[Español|Preguntas-frecuentes]]

# Preguntas frecuentes

### ¿Es una copia de seguridad?

No en el sentido de guardar versiones antiguas. DriveMirror hace que Drive
quede igual a la carpeta local: si un archivo se borra o se daña en el equipo,
la siguiente sincronización lleva ese cambio a Drive. Para tener historial,
combínalo con otra herramienta de copia de seguridad o con el historial de
versiones del propio Google Drive.

### ¿Los cambios hechos en Drive vuelven al equipo?

No. La copia es de un solo sentido (equipo → Drive). Los cambios hechos
directamente en la subcarpeta de Drive se deshacen en la siguiente
sincronización.

### ¿Puedo sincronizar la raíz de Drive?

No. La app exige una subcarpeta para evitar que un espejo borre el resto de tu
Drive.

### ¿Puedo sincronizar más de una carpeta?

No en esta versión: una carpeta local a una subcarpeta de Drive.

### ¿Por qué necesito crear mi propio Client ID?

Google está retirando el Client ID compartido de rclone, y un Client ID propio
evita límites de uso compartidos. La app no crea proyectos ni credenciales en
Google Cloud por ti.

### ¿Qué guarda DriveMirror de mis credenciales?

Nada. Quien guarda el Client Secret y el token es rclone, en
`~/.config/rclone/rclone.conf`. Detalles en
[SECURITY.md](https://github.com/israelguido/drivemirror/blob/main/SECURITY.md).

### ¿Qué acceso a Drive pide la app?

El remoto se crea con el alcance `drive` de rclone, que da acceso completo al
Drive de la cuenta autorizada. Es necesario para sincronizar en cualquier
subcarpeta.

### ¿Eliminar un remoto borra mis archivos?

No. **Eliminar** borra solo la configuración del remoto en rclone. Los archivos
en Drive siguen ahí.

### ¿Los archivos ignorados se borran de Drive?

No. Los elementos que coinciden con las exclusiones no se suben y tampoco se
borran del destino.

### ¿La app sincroniza con el equipo apagado o con la app cerrada?

No. La sincronización automática ocurre mientras DriveMirror está abierto (se
inicia en la bandeja al iniciar sesión).

### ¿Funciona en GNOME?

Sí, con dos diferencias: el panel se abre como ventana pequeña (GNOME no ofrece
el protocolo layer-shell), y el icono en la bandeja requiere una extensión de
AppIndicator. Sin icono, usa la app desde el menú de aplicaciones.

### ¿Cómo cambio el idioma?

En el selector **Idioma** de la pestaña Sincronización. "Sistema" sigue el
idioma de tu sesión.
