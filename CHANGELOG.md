# Changelog

Todos los cambios relevantes de Apunta se documentan aquí. El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [versionado semántico](https://semver.org/lang/es/).

## [1.0.0] - 2026-11-18

Primera versión estable.

### Notas y tareas
- **Tareas**: se marcan como hechas, tienen fecha y recordatorio, y aparecen en *Hoy* y *Próximas*.
- **Notas**: texto con formato Markdown ligero (negrita, cursiva, listas, casillas, enlaces), mostradas como tarjetas.
- Vistas Hoy (con sección de vencidas), Próximas (agrupadas por día), Tareas, Notas y Completadas (con «Vaciar completadas»).
- Listas con color y etiquetas. Búsqueda en títulos y contenido.
- Fijar arriba, mover entre listas y convertir entre nota y tarea.

### Captura rápida
- Atajo global configurable (por defecto `Ctrl+Alt+N`) que abre una ventanita flotante desde cualquier programa.
- Barra de creación con botón «Añadir ↵» y chips de fecha y recordatorio.

### Recordatorios
- Aviso con fecha y hora, con notificación de Windows y ventana de aviso con sonido.
- Repetición diaria, días laborables, semanal, mensual y anual.
- Posponer 5 min, 1 h o hasta mañana. Los avisos perdidos con el PC apagado se muestran al arrancar.
- Sonidos incluidos o un archivo propio, con volumen ajustable.

### Experiencia
- Diseño estilo Windows 11 con efecto Mica y tema claro u oscuro según el sistema o elegido a mano.
- «Deshacer» al completar o eliminar, menú contextual, acciones al pasar el ratón y atajos de teclado.
- Barra lateral adaptable al ancho de la ventana.
- Se queda en la bandeja del sistema al cerrar y puede iniciarse con Windows.
- Exportar e importar los datos en JSON.

### Distribución
- Instalador NSIS para el usuario actual (sin permisos de administrador) y versión portable en .zip, que guarda los datos junto al ejecutable.

[1.0.0]: https://github.com/hujeanpaes/apunta/releases/tag/v1.0.0
