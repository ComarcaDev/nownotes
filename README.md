<div align="center">

<img src="src-tauri/icons/128x128.png" width="88" alt="Apunta" />

# Apunta

**Notas y tareas rápidas para Windows, con recordatorios.**
Ligera (unos 2 MB), moderna y sin cuentas: todo se guarda en tu equipo.

[![CI](https://github.com/hujeanpaes/apunta/actions/workflows/ci.yml/badge.svg)](https://github.com/hujeanpaes/apunta/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/hujeanpaes/apunta)](https://github.com/hujeanpaes/apunta/releases/latest)
[![Licencia: MIT](https://img.shields.io/badge/licencia-MIT-blue.svg)](LICENSE)

<img src="docs/screenshot.png" width="640" alt="Apunta en modo completo" />&nbsp;&nbsp;<img src="docs/screenshot-mini.png" width="235" alt="Apunta en modo mínimo" />

</div>

## Funciones

- **Tareas y notas con identidad propia.** Las tareas se completan, tienen fecha y aparecen en *Hoy* y *Próximas*. Las notas son tarjetas de texto con formato (negrita, listas, casillas, enlaces).
- **Captura rápida desde cualquier sitio** con `Ctrl+Alt+N` (configurable).
- **Recordatorios** con notificación de Windows, ventana de aviso y sonido. Pueden repetirse (cada día, días laborables, semana, mes o año) y posponerse 5 min, 1 h o hasta mañana.
- **Organización** con listas de colores, etiquetas, búsqueda y elementos fijados.
- **Papelera** que guarda lo eliminado 10 días por si necesitas recuperarlo.
- **Modo mínimo**: una ventana compacta tipo widget con Hoy, Tareas y Notas, buscador y selector de tarea o nota.
- **Anclar la ventana** encima de todas las demás, en cualquiera de los dos modos.
- **Diseño Windows 11** con efecto Mica. Tema oscuro por defecto y claro con un clic en el sol o la luna.
- **Bandeja del sistema e inicio con Windows**, para que los avisos lleguen aunque cierres la ventana.
- **Instalable o portable.** La versión portable guarda los datos junto al ejecutable.
- **Exportar e importar** los datos en JSON.

## Descarga

Descarga la última versión desde **[Releases](https://github.com/hujeanpaes/apunta/releases/latest)**:

| Archivo | Para quién |
| --- | --- |
| `Apunta_x.y.z_x64-setup.exe` | Instalación normal para tu usuario, sin permisos de administrador. Datos en `%APPDATA%\Apunta`. |
| `Apunta_x.y.z_x64_portable.zip` | Sin instalación: descomprime y ejecuta `Apunta.exe`. Mientras exista el archivo `portable` junto al ejecutable, los datos se guardan en la carpeta `data/` de al lado. |

Requisitos: Windows 11 de 64 bits. No está probada en Windows 10.

## Atajos de teclado

| Atajo | Acción |
| --- | --- |
| `Ctrl+Alt+N` | Captura rápida (desde cualquier programa) |
| `Ctrl+N` | Escribir una tarea o nota nueva |
| `Ctrl+F` | Buscar (también en modo mínimo) |
| `Tab` | Alternar entre tarea y nota al escribir en Hoy, listas o etiquetas |
| `Espacio` | Completar la tarea seleccionada |
| `Enter` | Editar el título del elemento seleccionado |
| `Supr` | Mover a la papelera (con opción de deshacer) |
| `Ctrl+M` | Alternar modo mínimo y modo completo |
| `Ctrl+P` | Anclar o desanclar la ventana encima de las demás |
| `Ctrl+Mayús+L` | Cambiar entre tema oscuro y claro |
| `↑` / `↓` | Moverse entre elementos |
| `Esc` | Cerrar el editor o la búsqueda |

En la captura rápida: `Enter` guarda, `Mayús+Enter` añade una línea, `Tab` cambia entre tarea y nota y `Esc` cierra.

## Compilar desde el código

Necesitas [Rust](https://rustup.rs/) (toolchain MSVC), [Node.js LTS](https://nodejs.org/) y Visual Studio Build Tools con la carga de trabajo *Desarrollo para el escritorio con C++*.

```bash
npm install
npm run tauri dev     # modo desarrollo (datos en %APPDATA%\Apunta-dev)
npm run check         # comprobación de tipos de la interfaz
npm test              # tests de Rust
npm run release       # instalador + zip portable en release/
```

## Estructura

```
src/                   Interfaz (Svelte 5 + TypeScript)
  main/                Ventana principal
  quick/               Ventana de captura rápida
  reminder/            Ventana de aviso
  lib/                 Componentes, API, fechas, sonidos, Markdown
src-tauri/src/         Backend (Rust + Tauri 2)
  db.rs                SQLite: esquema, CRUD y estado de los avisos
  scheduler.rs         Revisión de recordatorios cada 15 s
  recurrence.rs        Cálculo de repeticiones
  windows.rs           Creación de ventanas (Mica)
  paths.rs             Modo portable o instalado
scripts/               Empaquetado del zip portable
```

## Licencia

[MIT](LICENSE) © 2026 hujeanpaes
