//! Comandos que llama la interfaz.

use std::collections::HashMap;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

use crate::db::{self, Counts, Filter, Item, ItemInput, List, TagCount};
use crate::{windows, AppState};

type Res<T> = Result<T, String>;

fn e<E: std::fmt::Display>(err: E) -> String {
    err.to_string()
}

fn changed(app: &AppHandle) {
    let _ = app.emit("items-changed", ());
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
        db::delete_item(&c, id).map_err(e)?;
    }
    state.alerts.lock().unwrap().retain(|a| *a != id);
    changed(&app);
    Ok(())
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
        db::clear_done(&c).map_err(e)?
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

pub fn apply_shortcut(app: &AppHandle, shortcut: &str) -> Res<()> {
    let gs = app.global_shortcut();
    let state = app.state::<AppState>();
    let mut current = state.shortcut.lock().unwrap();
    if !current.is_empty() {
        let _ = gs.unregister(current.as_str());
    }
    let wanted = shortcut.trim();
    if wanted.is_empty() {
        current.clear();
        return Ok(());
    }
    match gs.register(wanted) {
        Ok(()) => {
            *current = wanted.to_string();
            Ok(())
        }
        Err(err) => {
            if !current.is_empty() {
                let _ = gs.register(current.as_str());
            }
            Err(format!("No se pudo registrar el atajo «{wanted}»: {err}"))
        }
    }
}

#[tauri::command]
pub fn set_setting(app: AppHandle, state: State<AppState>, key: String, value: String) -> Res<()> {
    match key.as_str() {
        "shortcut" => apply_shortcut(&app, &value)?,
        "theme" => {
            let theme = windows::theme_from(&value);
            for w in app.webview_windows().values() {
                let _ = w.set_theme(theme);
            }
        }
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
