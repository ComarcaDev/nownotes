//! Comandos que llama la interfaz.

use std::collections::HashMap;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_opener::OpenerExt;

use crate::db::{self, Counts, Filter, Item, ItemInput, List, TagCount};
use crate::{backup, windows, AppState};

type Res<T> = Result<T, String>;

fn e<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

fn changed(app: &AppHandle) {
    let _ = app.emit("items-changed", ());
    update_tray(app);
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    data_dir: String,
    portable: bool,
    version: String,
}

#[tauri::command]
pub fn get_info(app: AppHandle, state: State<AppState>) -> Info {
    Info {
        data_dir: state.data_dir.display().to_string(),
        portable: state.portable,
        version: app.package_info().version.to_string(),
    }
}

// ---------- elementos ----------

#[tauri::command]
pub fn query_items(state: State<AppState>, filter: Filter) -> Res<Vec<Item>> {
    let c = state.db.lock().unwrap();
    db::query_items(&c, &filter, db::end_of_today_ms()).map_err(e)
}

#[tauri::command]
pub fn get_item(state: State<AppState>, id: i64) -> Res<Option<Item>> {
    let c = state.db.lock().unwrap();
    db::get_item(&c, id).map_err(e)
}

#[tauri::command]
pub fn save_item(app: AppHandle, state: State<AppState>, input: ItemInput) -> Res<Item> {
    let it = {
        let c = state.db.lock().unwrap();
        db::save_item(&c, &input, db::now_ms()).map_err(e)?
    };
    changed(&app);
    Ok(it)
}

#[tauri::command]
pub fn delete_item(app: AppHandle, state: State<AppState>, id: i64) -> Res<()> {
    {
        let c = state.db.lock().unwrap();
        db::delete_item(&c, id, db::now_ms()).map_err(e)?;
    }
    state.alerts.lock().unwrap().retain(|a| *a != id);
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn restore_item(app: AppHandle, state: State<AppState>, id: i64) -> Res<()> {
    {
        let c = state.db.lock().unwrap();
        db::restore_item(&c, id).map_err(e)?;
    }
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn delete_forever(app: AppHandle, state: State<AppState>, id: i64) -> Res<()> {
    {
        let c = state.db.lock().unwrap();
        db::delete_forever(&c, id).map_err(e)?;
    }
    state.alerts.lock().unwrap().retain(|a| *a != id);
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn empty_trash(app: AppHandle, state: State<AppState>) -> Res<usize> {
    let n = {
        let c = state.db.lock().unwrap();
        db::empty_trash(&c).map_err(e)?
    };
    changed(&app);
    Ok(n)
}

#[tauri::command]
pub fn toggle_done(app: AppHandle, state: State<AppState>, id: i64) -> Res<Item> {
    let it = {
        let c = state.db.lock().unwrap();
        db::toggle_done(&c, id, db::now_ms()).map_err(e)?
    };
    changed(&app);
    Ok(it)
}

#[tauri::command]
pub fn clear_done(app: AppHandle, state: State<AppState>) -> Res<usize> {
    let n = {
        let c = state.db.lock().unwrap();
        db::clear_done(&c, db::now_ms()).map_err(e)?
    };
    changed(&app);
    Ok(n)
}

// ---------- listas y etiquetas ----------

#[tauri::command]
pub fn list_lists(state: State<AppState>) -> Res<Vec<List>> {
    let c = state.db.lock().unwrap();
    db::list_lists(&c).map_err(e)
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
    let id = {
        let c = state.db.lock().unwrap();
        db::save_list(&c, id, &name, &color).map_err(e)?
    };
    changed(&app);
    Ok(id)
}

#[tauri::command]
pub fn delete_list(app: AppHandle, state: State<AppState>, id: i64) -> Res<()> {
    {
        let c = state.db.lock().unwrap();
        db::delete_list(&c, id).map_err(e)?;
    }
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn list_tags(state: State<AppState>) -> Res<Vec<TagCount>> {
    let c = state.db.lock().unwrap();
    db::list_tags(&c).map_err(e)
}

#[tauri::command]
pub fn counts(state: State<AppState>) -> Res<Counts> {
    let c = state.db.lock().unwrap();
    db::counts(&c, db::end_of_today_ms()).map_err(e)
}

// ---------- avisos ----------

#[tauri::command]
pub fn pending_alerts(state: State<AppState>) -> Res<Vec<Item>> {
    let ids = state.alerts.lock().unwrap().clone();
    let c = state.db.lock().unwrap();
    let mut out = vec![];
    for id in ids {
        if let Some(it) = db::get_item(&c, id).map_err(e)? {
            out.push(it);
        }
    }
    Ok(out)
}

#[tauri::command]
pub fn snooze_alert(app: AppHandle, state: State<AppState>, id: i64, until: i64) -> Res<()> {
    {
        let c = state.db.lock().unwrap();
        db::snooze(&c, id, until).map_err(e)?;
    }
    state.alerts.lock().unwrap().retain(|a| *a != id);
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn resolve_alert(app: AppHandle, state: State<AppState>, id: i64, complete: bool) -> Res<()> {
    if complete {
        let c = state.db.lock().unwrap();
        db::complete_from_alert(&c, id, db::now_ms()).map_err(e)?;
    }
    state.alerts.lock().unwrap().retain(|a| *a != id);
    changed(&app);
    Ok(())
}

// ---------- ajustes ----------

#[tauri::command]
pub fn get_settings(state: State<AppState>) -> Res<HashMap<String, String>> {
    let c = state.db.lock().unwrap();
    db::all_settings(&c).map_err(e)
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
    let mut globals = state.globals.lock().unwrap();
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
    let globals = state.globals.lock().unwrap();
    globals
        .iter()
        .find(|(_, acc)| parse_shortcut(acc).as_ref() == Some(pressed))
        .map(|(a, _)| a.clone())
}

// ---------- bandeja ----------

/// Tooltip de la bandeja con las tareas pendientes de hoy.
pub fn update_tray(app: &AppHandle) {
    let n = {
        let st = app.state::<AppState>();
        let c = st.db.lock().unwrap();
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

// ---------- orden manual y copias de seguridad ----------

#[tauri::command]
pub fn reorder_items(app: AppHandle, state: State<AppState>, ids: Vec<i64>) -> Res<()> {
    {
        let c = state.db.lock().unwrap();
        db::reorder_items(&c, &ids).map_err(e)?;
    }
    changed(&app);
    Ok(())
}

#[tauri::command]
pub fn list_backups(state: State<AppState>) -> Vec<backup::BackupInfo> {
    backup::list(&state.data_dir)
}

#[tauri::command]
pub fn restore_backup(app: AppHandle, state: State<AppState>, name: String) -> Res<()> {
    let source = backup::path_of(&state.data_dir, &name)?;
    {
        let mut c = state.db.lock().unwrap();
        let stamp = chrono::Local::now().format("%Y-%m-%d-%H%M%S").to_string();
        backup::snapshot(&c, &state.data_dir, &stamp)?;
        backup::restore(&mut c, &source)?;
        db::init(&c).map_err(e)?;
    }
    state.alerts.lock().unwrap().clear();
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
    {
        let c = state.db.lock().unwrap();
        db::set_setting(&c, &key, &value).map_err(e)?;
    }
    let _ = app.emit("settings-changed", (key, value));
    Ok(())
}

// ---------- archivos ----------

#[tauri::command]
pub fn read_file_bytes(path: String) -> Res<tauri::ipc::Response> {
    let bytes = std::fs::read(&path).map_err(e)?;
    Ok(tauri::ipc::Response::new(bytes))
}

#[tauri::command]
pub fn export_data(state: State<AppState>, path: String) -> Res<()> {
    let data = {
        let c = state.db.lock().unwrap();
        db::export_all(&c).map_err(e)?
    };
    let json = serde_json::to_string_pretty(&data).map_err(e)?;
    std::fs::write(path, json).map_err(e)
}

#[tauri::command]
pub fn import_data(app: AppHandle, state: State<AppState>, path: String) -> Res<usize> {
    let text = std::fs::read_to_string(path).map_err(e)?;
    let data: db::ExportData =
        serde_json::from_str(&text).map_err(|err| format!("Archivo no válido: {err}"))?;
    let n = {
        let c = state.db.lock().unwrap();
        db::import_all(&c, &data, db::now_ms()).map_err(e)?
    };
    changed(&app);
    Ok(n)
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
    windows::apply_mode(&app, mode).map_err(e)?;
    let _ = app.emit("settings-changed", ("ui_mode", mode));
    Ok(())
}
