# Changelog

Todos los cambios relevantes de NowNotes (antes «Apunta») se documentan aquí. El formato sigue [Keep a Changelog](https://keepachangelog.com/es-ES/1.1.0/) y el proyecto usa [versionado semántico](https://semver.org/lang/es/).

## [2.0.2] - 2026-11-19

### Corregido
- **El contador de «Hoy» incluye las notas de hoy**: ahora coincide con lo que muestra la vista (antes solo contaba tareas). Al pasar el ratón se ve el desglose, por ejemplo «1 tarea · 1 nota».
- El tooltip de la bandeja también lo indica: «1 tarea y 1 nota para hoy».

## [2.0.1] - 2026-11-19

### Cambiado
- **Logo**: vuelve el diseño anterior (tarjeta con ✓), ahora con un discreto punto ámbar «ahora» en la esquina.

### Corregido
- Los errores al guardar ya no pasan desapercibidos: se muestra un aviso.
- Guardados rápidos seguidos en el editor ya no pueden pisarse entre sí.
- «Vaciar completadas» decía que se borraban; en realidad pasan a la papelera.
- Cancelar «Archivo propio…» en Ajustes deja el sonido que había.
- Los mensajes de Ajustes se limpian al hacer otra acción.
- `Esc` en el menú de recordatorio ya no cierra también el editor.
- Si la base de datos no se puede abrir al arrancar, se muestra un mensaje en lugar de cerrarse sin avisar.

### Mejorado
- Registro de errores en `nownotes.log` (carpeta de datos, con rotación).
- Accesibilidad: etiquetas en botones de icono y campos, menús con roles, diálogo de Ajustes modal con el foco dentro.
- Seguridad: permisos separados para las ventanas emergentes y CSP más estricta.
- Limpieza interna: código duplicado unificado, iconos y archivos sin uso eliminados, más tests.

## [2.0.0] - 2026-11-19

### Cambiado
- **Nuevo nombre: NowNotes** (antes «Apunta»), con logo «NN». El nombre funciona igual en español e inglés, de cara a la versión multilingüe.
- La app se instala como «NowNotes» (identificador nuevo). Los datos de «Apunta» se trasladan solos al abrirla, también el inicio con Windows.
- El orden de Tareas, Notas y Listas respeta el orden manual (los elementos nuevos aparecen arriba).

### Añadido
- **Fechas en lenguaje natural** (español e inglés) en la barra de creación y en la captura rápida: «mañana 18:00», «el lunes a las 9», «en 2 horas», «25/12», «tomorrow at 3pm»… Un chip muestra lo detectado y se puede descartar.
- **Atajos configurables** en Ajustes: los globales (captura rápida y el nuevo **mostrar/ocultar**, `Ctrl+Alt+A`) y los de la app. Avisa de conflictos internos y de combinaciones que ya usa Windows u otra aplicación.
- **Casillas clicables**: las listas `- [ ]` de una nota se marcan con un clic en la vista previa o en la tarjeta.
- **Arrastrar para reordenar** en Tareas, Notas y Listas, con el asa ⋮⋮.
- **Copias de seguridad automáticas** diarias (se guardan las últimas 7), con restauración y «Abrir carpeta» en Ajustes. Antes de restaurar se guarda una copia del estado actual.
- **Bandeja**: el tooltip muestra las tareas pendientes de hoy y el menú incluye «Nueva nota o tarea».
- **Aviso de versiones nuevas** (una vez al día, consultando GitHub; se puede desactivar).

### Mejorado
- Carga diferida de Ajustes, menú contextual y papelera: menos JavaScript al arrancar.
- Nuevos tests: migración de datos, orden manual, copias de seguridad y analizador de fechas (Vitest, también en CI).

## [1.2.0] - 2026-11-18

### Añadido
- **Tarea o nota en «Hoy»**: la barra de creación de Hoy tiene un selector Tarea/Nota (solo iconos en modo compacto, con `Tab` para alternar). Tarea viene por defecto y, si eliges Nota, se recuerda también tras reiniciar.
- **«Notas de hoy»**: la vista Hoy muestra, debajo de las tareas, las notas creadas hoy.
- **Buscador en modo compacto**: un icono de lupa (o `Ctrl+F`) cambia las pestañas por un campo de búsqueda; `Esc` lo cierra.

### Cambiado
- En modo compacto y ventana estrecha, solo la pestaña activa muestra su nombre; las demás muestran icono y contador.
- En modo compacto, crear una nota ya no abre el editor, para que la captura siga siendo rápida.

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

[2.0.2]: https://github.com/hujeanpaes/nownotes/releases/tag/v2.0.2
[2.0.1]: https://github.com/hujeanpaes/nownotes/releases/tag/v2.0.1
[2.0.0]: https://github.com/hujeanpaes/nownotes/releases/tag/v2.0.0
[1.2.0]: https://github.com/hujeanpaes/nownotes/releases/tag/v1.2.0
[1.1.0]: https://github.com/hujeanpaes/nownotes/releases/tag/v1.1.0
[1.0.0]: https://github.com/hujeanpaes/nownotes/releases/tag/v1.0.0
