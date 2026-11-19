//! Comandos que llama la interfaz.

use std::collections::HashMap;

use rusqlite::Connection;
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_opener::OpenerExt;

use crate::db::{self, Counts, Filter, Item, ItemInput, List, TagCount};
use crate::{backup, logger, windows, AppState, EV_ITEMS, EV_SETTINGS};

type Res<T> = Result<T, String>;

fn e<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

/// Avisa a todas las ventanas de que los datos cambiaron y actualiza la bandeja.
fn changed(app: &AppHandle) {
    let _ = app.emit(EV_ITEMS, ());
    update_tray(app);
}

/// Lectura en la base de datos.
fn read<T>(state: &AppState, f: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Res<T> {
    f(&state.db()).map_err(e)
}

/// Escritura en la base de datos seguida del aviso `changed`.
fn mutate<T>(
    app: &AppHandle,
    state: &AppState,
    f: impl FnOnce(&Connection) -> rusqlite::Result<T>,
) -> Res<T> {
    let out = f(&state.db()).map_err(e)?;
    changed(app);
    Ok(out)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    data_dir: String,
    portable: bool,
    version: String,
    /// Días que se conserva un elemento en la papelera.
    trash_days: i64,
    /// Copias de seguridad diarias que se conservan.
    backup_keep: usize,
}

#[tauri::command]
pub fn get_info(app: AppHandle, state: State<AppState>) -> Info {
    Info {
        data_dir: state.data_dir.display().to_string(),
        portable: state.portable,
        version: app.package_info().version.to_string(),
        trash_days: db::TRASH_DAYS,
        backup_keep: backup::KEEP,
    }
}

// ---------- elementos ----------

#[tauri::command]
pub fn query_items(state: State<AppState>, filter: Filter) -> Res<Vec<Item>> {
    read(&state, |c| {
        db::query_items(c, &filter, db::end_of_today_ms())
    })
}

#[tauri::command]
pub fn get_item(state: State<AppState>, id: i64) -> Res<Option<Item>> {
    read(&state, |c| db::get_item(c, id))
}

#[tauri::command]
pub fn save_item(app: AppHandle, state: State<AppState>, input: ItemInput) -> Res<Item> {
    mutate(&app, &state, |c| db::save_item(c, &input, db::now_ms()))
}

#[tauri::command]
pub fn delete_item(app: AppHandle, state: State<AppState>, id: i64) -> Res<()> {
    state.drop_alert(id);
    mutate(&app, &state, |c| db::delete_item(c, id, db::now_ms()))
}

#[tauri::command]
pub fn restore_item(app: AppHandle, state: State<AppState>, id: i64) -> Res<()> {
    mutate(&app, &state, |c| db::restore_item(c, id))
}

#[tauri::command]
pub fn delete_forever(app: AppHandle, state: State<AppState>, id: i64) -> Res<()> {
    state.drop_alert(id);
    mutate(&app, &state, |c| db::delete_forever(c, id))
}

#[tauri::command]
pub fn empty_trash(app: AppHandle, state: State<AppState>) -> Res<usize> {
    mutate(&app, &state, db::empty_trash)
}

#[tauri::command]
pub fn toggle_done(app: AppHandle, state: State<AppState>, id: i64) -> Res<Item> {
    mutate(&app, &state, |c| db::toggle_done(c, id, db::now_ms()))
}

#[tauri::command]
pub fn clear_done(app: AppHandle, state: State<AppState>) -> Res<usize> {
    mutate(&app, &state, |c| db::clear_done(c, db::now_ms()))
}

#[tauri::command]
pub fn reorder_items(app: AppHandle, state: State<AppState>, ids: Vec<i64>) -> Res<()> {
    mutate(&app, &state, |c| db::reorder_items(c, &ids))
}

// ---------- listas y etiquetas ----------

#[tauri::command]
pub fn list_lists(state: State<AppState>) -> Res<Vec<List>> {
    read(&state, db::list_lists)
}

#[tauri::command]
pub fn save_list(
    app: AppHandle,
    state: State<AppState>,
    id: Option<i64>,
    name: String,
    color: String,
) -> Res<i64> {
    if name.trim().is_empty() {
        return Err("El nombre de la lista no puede estar vacío".into());
    }
    mutate(&app, &state, |c| db::save_list(c, id, &name, &color))
}

#[tauri::command]
pub fn delete_list(app: AppHandle, state: State<AppState>, id: i64) -> Res<()> {
    mutate(&app, &state, |c| db::delete_list(c, id))
}

#[tauri::command]
pub fn list_tags(state: State<AppState>) -> Res<Vec<TagCount>> {
    read(&state, db::list_tags)
}

#[tauri::command]
pub fn counts(state: State<AppState>) -> Res<Counts> {
    read(&state, |c| db::counts(c, db::end_of_today_ms()))
}

// ---------- avisos ----------

#[tauri::command]
pub fn pending_alerts(state: State<AppState>) -> Res<Vec<Item>> {
    let ids = state.alerts().clone();
    read(&state, |c| {
        let mut out = vec![];
        for id in ids {
            if let Some(it) = db::get_item(c, id)? {
                out.push(it);
            }
        }
        Ok(out)
    })
}

#[tauri::command]
pub fn snooze_alert(app: AppHandle, state: State<AppState>, id: i64, until: i64) -> Res<()> {
    state.drop_alert(id);
    mutate(&app, &state, |c| db::snooze(c, id, until))
}

#[tauri::command]
pub fn resolve_alert(app: AppHandle, state: State<AppState>, id: i64, complete: bool) -> Res<()> {
    state.drop_alert(id);
    mutate(&app, &state, |c| {
        if complete {
            db::complete_from_alert(c, id, db::now_ms())?;
        }
        Ok(())
    })
}

// ---------- ajustes ----------

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Res<HashMap<String, String>> {
    read(&state, db::all_settings)
}

#[tauri::command]
pub fn set_setting(app: AppHandle, state: State<AppState>, key: String, value: String) -> Res<()> {
    match key.as_str() {
        "shortcut" => apply_shortcut(&app, "quick", &value)?,
        "shortcut_toggle" => apply_shortcut(&app, "toggle", &value)?,
        "theme" => {
            let theme = windows::theme_from(&value);
            for w in app.webview_windows().values() {
                let _ = w.set_theme(theme);
            }
        }
        "always_on_top" => {
            if let Some(w) = app.get_webview_window("main") {
                w.set_always_on_top(value == "1").map_err(e)?;
            }
        }
        "ui_mode" => return Err("Usa set_ui_mode para cambiar de modo".into()),
        _ => {}
    }
    read(&state, |c| db::set_setting(c, &key, &value))?;
    let _ = app.emit(EV_SETTINGS, (key, value));
    Ok(())
}

// ---------- atajos globales ----------

/// Acciones con atajo global y el ajuste donde se guarda cada una.
pub const GLOBAL_ACTIONS: &[(&str, &str)] = &[("quick", "shortcut"), ("toggle", "shortcut_toggle")];

fn action_label(action: &str) -> &'static str {
    match action {
        "quick" => "la captura rápida",
        "toggle" => "mostrar/ocultar NowNotes",
        _ => "otra acción",
    }
}

fn parse_shortcut(accel: &str) -> Option<Shortcut> {
    accel.parse::<Shortcut>().ok()
}

/// Registra (o cambia) el atajo global de una acción, comprobando conflictos.
pub fn apply_shortcut(app: &AppHandle, action: &str, accel: &str) -> Res<()> {
    let gs = app.global_shortcut();
    let state = app.state::<AppState>();
    let mut globals = state.globals.lock().unwrap_or_else(|p| p.into_inner());
    let wanted = accel.trim();
    let wanted_sc = if wanted.is_empty() {
        None
    } else {
        Some(parse_shortcut(wanted).ok_or_else(|| format!("Combinación no válida: «{wanted}»"))?)
    };
    if let Some(sc) = &wanted_sc {
        for (other, acc) in globals.iter() {
            if other != action && parse_shortcut(acc).as_ref() == Some(sc) {
                return Err(format!(
                    "«{wanted}» ya se usa para {}.",
                    action_label(other)
                ));
            }
        }
    }
    let old = globals.get(action).cloned();
    if let Some(o) = &old {
        let _ = gs.unregister(o.as_str());
    }
    let Some(sc) = wanted_sc else {
        globals.remove(action);
        return Ok(());
    };
    match gs.register(sc) {
        Ok(()) => {
            globals.insert(action.to_string(), wanted.to_string());
            Ok(())
        }
        Err(_) => {
            if let Some(o) = &old {
                let _ = gs.register(o.as_str());
            }
            Err(format!(
                "«{wanted}» está en uso por Windows u otra aplicación. Elige otra combinación."
            ))
        }
    }
}

/// Acción global asociada a la combinación pulsada.
pub fn action_for(app: &AppHandle, pressed: &Shortcut) -> Option<String> {
    let state = app.state::<AppState>();
    let globals = state.globals.lock().unwrap_or_else(|p| p.into_inner());
    globals
        .iter()
        .find(|(_, acc)| parse_shortcut(acc).as_ref() == Some(pressed))
        .map(|(a, _)| a.clone())
}

// ---------- bandeja ----------

/// Tooltip de la bandeja con las tareas pendientes de hoy.
pub fn update_tray(app: &AppHandle) {
    let n = {
        let state = app.state::<AppState>();
        let c = state.db();
        db::counts(&c, db::end_of_today_ms())
            .map(|c| c.today)
            .unwrap_or(0)
    };
    let tip = match n {
        0 => "NowNotes · Nada pendiente hoy".to_string(),
        1 => "NowNotes · 1 tarea para hoy".to_string(),
        n => format!("NowNotes · {n} tareas para hoy"),
    };
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(tip));
    }
}

// ---------- copias de seguridad ----------

#[tauri::command]
pub fn list_backups(state: State<AppState>) -> Vec<backup::BackupInfo> {
    backup::list(&state.data_dir)
}

#[tauri::command]
pub fn restore_backup(app: AppHandle, state: State<AppState>, name: String) -> Res<()> {
    let source = backup::path_of(&state.data_dir, &name)?;
    {
        let mut c = state.db();
        let stamp = chrono::Local::now().format("%Y-%m-%d-%H%M%S").to_string();
        backup::snapshot(&c, &state.data_dir, &stamp)?;
        backup::restore(&mut c, &source)?;
        db::init(&c).map_err(e)?;
    }
    state.alerts().clear();
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn open_backups_dir(app: AppHandle, state: State<AppState>) -> Res<()> {
    let dir = backup::dir(&state.data_dir);
    std::fs::create_dir_all(&dir).map_err(e)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(e)
}

// ---------- archivos ----------
// Las rutas llegan de diálogos nativos de la propia app (sin contenido remoto y con CSP estricta).

#[tauri::command]
pub fn read_file_bytes(path: String) -> Res<tauri::ipc::Response> {
    let bytes = std::fs::read(&path).map_err(e)?;
    Ok(tauri::ipc::Response::new(bytes))
}

#[tauri::command]
pub fn export_data(state: State<AppState>, path: String) -> Res<()> {
    let data = read(&state, db::export_all)?;
    let json = serde_json::to_string_pretty(&data).map_err(e)?;
    std::fs::write(path, json).map_err(e)
}

#[tauri::command]
pub fn import_data(app: AppHandle, state: State<AppState>, path: String) -> Res<usize> {
    let text = std::fs::read_to_string(path).map_err(e)?;
    let data: db::ExportData =
        serde_json::from_str(&text).map_err(|err| format!("Archivo no válido: {err}"))?;
    mutate(&app, &state, |c| db::import_all(c, &data, db::now_ms()))
}

// ---------- ventanas ----------

#[tauri::command]
pub async fn show_main(app: AppHandle) {
    windows::show_main(&app);
}

#[tauri::command]
pub async fn open_quick(app: AppHandle) {
    windows::toggle_quick(&app);
}

/// La ventana principal se muestra desde la interfaz cuando ya está pintada (evita el parpadeo).
#[tauri::command]
pub fn startup_minimized(state: State<AppState>) -> bool {
    state.start_minimized
}

/// Cambia entre el modo completo y el mínimo, recordando el tamaño de cada uno.
#[tauri::command]
pub async fn set_ui_mode(app: AppHandle, mode: String) -> Res<()> {
    let mode = if mode == "mini" { "mini" } else { "full" };
    windows::apply_mode(&app, mode).map_err(|err| {
        logger::error(format!("No se pudo cambiar al modo {mode}: {err}"));
        e(err)
    })?;
    let _ = app.emit(EV_SETTINGS, ("ui_mode", mode));
    Ok(())
}
