# Changelog

Todos los cambios relevantes de Apunta se documentan aquí. El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [versionado semántico](https://semver.org/lang/es/).

## [1.1.0] - 2026-11-18

### Añadido
- **Papelera**: lo que eliminas se guarda 10 días, con opciones para restaurar o eliminar para siempre y «Vaciar papelera». Pasado ese plazo se borra solo. «Deshacer» recupera el mismo elemento.
- **Anclar la ventana** encima de todas las demás (`Ctrl+P`), en modo completo y en modo mínimo.
- **Modo mínimo** (`Ctrl+M`): ventana compacta tipo widget con pestañas Hoy, Tareas y Notas, creación rápida y editor a pantalla completa. Cada modo recuerda su tamaño.
- **Tema claro y oscuro** con un solo icono de sol o luna (`Ctrl+Mayús+L`).
- La app recuerda al reiniciar el modo, el tamaño, el anclado y el tema.

### Cambiado
- El tema oscuro pasa a ser el predeterminado y el selector de tema sale de Ajustes.
- «Vaciar completadas» mueve los elementos a la papelera en lugar de borrarlos.
- La base de datos se migra automáticamente (esquema v2) sin perder datos.

### Corregido
- Con poca altura de ventana, el título del editor podía quedar oculto.

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

[1.1.0]: https://github.com/hujeanpaes/apunta/releases/tag/v1.1.0
[1.0.0]: https://github.com/hujeanpaes/apunta/releases/tag/v1.0.0
