//! Creación y gestión de las tres ventanas: principal, captura rápida y aviso.

use tauri::window::{Effect, EffectsBuilder};
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, Theme, WebviewUrl, WebviewWindowBuilder, Wry,
};

use crate::{db, AppState};

// Todas las ventanas deben compartir los mismos argumentos de WebView2.
// Los primeros son los que Tauri usa por defecto; el último permite reproducir el sonido del aviso sin interacción.
const BROWSER_ARGS: &str =
    "--disable-features=msWebOOUI,msPdfOOUI,msSmartScreenProtection --autoplay-policy=no-user-gesture-required";

pub fn theme_from(value: &str) -> Option<Theme> {
    match value {
        "dark" => Some(Theme::Dark),
        "light" => Some(Theme::Light),
        _ => None,
    }
}

fn current_theme(app: &AppHandle) -> Option<Theme> {
    let st = app.state::<AppState>();
    let c = st.db.lock().unwrap();
    theme_from(&db::get_setting(&c, "theme"))
}

fn base<'a>(
    app: &'a AppHandle,
    label: &str,
    url: &str,
) -> WebviewWindowBuilder<'a, Wry, AppHandle> {
    let mut b = WebviewWindowBuilder::new(app, label, WebviewUrl::App(url.into()))
        .additional_browser_args(BROWSER_ARGS)
        .theme(current_theme(app));
    let st = app.state::<AppState>();
    if st.portable {
        // En modo portable, también los datos del WebView van junto al .exe.
        b = b.data_directory(st.data_dir.join("webview"));
    }
    b
}

// ---------- modos de ventana (completo / mínimo) ----------

const MIN_FULL: (f64, f64) = (760.0, 480.0);
const MIN_MINI: (f64, f64) = (320.0, 400.0);

fn min_for(mode: &str) -> (f64, f64) {
    if mode == "mini" {
        MIN_MINI
    } else {
        MIN_FULL
    }
}

/// "1040,700" → (1040.0, 700.0), respetando el mínimo del modo.
fn parse_size(value: &str, mode: &str) -> (f64, f64) {
    let min = min_for(mode);
    let mut it = value.split(',').map(|v| v.trim().parse::<f64>().ok());
    match (it.next().flatten(), it.next().flatten()) {
        (Some(w), Some(h)) => (w.max(min.0), h.max(min.1)),
        _ if mode == "mini" => (380.0, 560.0),
        _ => (1040.0, 700.0),
    }
}

/// Guarda el tamaño actual de la ventana principal como el del modo activo.
pub fn save_size(app: &AppHandle) {
    let Some(w) = app.get_webview_window("main") else {
        return;
    };
    if w.is_maximized().unwrap_or(false) || w.is_minimized().unwrap_or(false) {
        return;
    }
    let (Ok(size), Ok(scale)) = (w.inner_size(), w.scale_factor()) else {
        return;
    };
    let logical = size.to_logical::<f64>(scale);
    let st = app.state::<AppState>();
    let c = st.db.lock().unwrap();
    let mode = db::get_setting(&c, "ui_mode");
    let _ = db::set_setting(
        &c,
        &format!("size_{mode}"),
        &format!("{},{}", logical.width.round(), logical.height.round()),
    );
}

/// Cambia de modo: guarda el tamaño del modo saliente y aplica el del nuevo.
pub fn apply_mode(app: &AppHandle, mode: &str) -> tauri::Result<()> {
    let Some(w) = app.get_webview_window("main") else {
        return Ok(());
    };
    save_size(app);
    let size = {
        let st = app.state::<AppState>();
        let c = st.db.lock().unwrap();
        let _ = db::set_setting(&c, "ui_mode", mode);
        parse_size(&db::get_setting(&c, &format!("size_{mode}")), mode)
    };
    if w.is_maximized()? {
        w.unmaximize()?;
    }
    let min = min_for(mode);
    w.set_min_size(Some(LogicalSize::new(min.0, min.1)))?;
    w.set_size(LogicalSize::new(size.0, size.1))?;
    Ok(())
}

/// Crea la ventana principal oculta; la interfaz la muestra al terminar de pintar.
/// Si en 4 s no lo ha hecho (p. ej. error de carga), se muestra igualmente.
pub fn create_main(app: &AppHandle, show: bool) -> tauri::Result<()> {
    let (mode, size, on_top) = {
        let st = app.state::<AppState>();
        let c = st.db.lock().unwrap();
        let mode = db::get_setting(&c, "ui_mode");
        let size = parse_size(&db::get_setting(&c, &format!("size_{mode}")), &mode);
        (mode, size, db::get_setting(&c, "always_on_top") == "1")
    };
    let min = min_for(&mode);
    base(app, "main", "index.html")
        .title("Apunta")
        .inner_size(size.0, size.1)
        .min_inner_size(min.0, min.1)
        .always_on_top(on_top)
        .center()
        .visible(false)
        .transparent(true)
        .effects(EffectsBuilder::new().effect(Effect::Mica).build())
        .build()?;
    if show {
        let handle = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(4));
            if let Some(w) = handle.get_webview_window("main") {
                if !w.is_visible().unwrap_or(true) {
                    let _ = w.show();
                }
            }
        });
    }
    Ok(())
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

pub fn toggle_quick(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("quick") {
        if w.is_visible().unwrap_or(false) && w.is_focused().unwrap_or(false) {
            let _ = w.hide();
        } else {
            let _ = w.center();
            let _ = w.show();
            let _ = w.set_focus();
            let _ = w.emit_to("quick", "quick-open", ());
        }
        return;
    }
    let r = base(app, "quick", "quick.html")
        .title("Apunta · captura rápida")
        .inner_size(580.0, 244.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .center()
        .focused(true)
        .build();
    if let Err(e) = r {
        eprintln!("No se pudo abrir la captura rápida: {e}");
    }
}

pub fn show_reminder(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("reminder") {
        let _ = w.show();
        let _ = w.emit_to("reminder", "alerts-changed", ());
        return;
    }
    let (w, h) = (390.0, 270.0);
    let mut b = base(app, "reminder", "reminder.html")
        .title("Apunta · recordatorio")
        .inner_size(w, h)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false);
    if let Ok(Some(m)) = app.primary_monitor() {
        let s = m.scale_factor();
        let (pos, size) = (m.position(), m.size());
        let x = pos.x as f64 / s + size.width as f64 / s - w - 16.0;
        let y = pos.y as f64 / s + size.height as f64 / s - h - 64.0;
        b = b.position(x, y);
    }
    if let Err(e) = b.build() {
        eprintln!("No se pudo abrir el aviso: {e}");
    }
}
