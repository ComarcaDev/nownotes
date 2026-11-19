#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod backup;
mod commands;
mod db;
mod logger;
mod paths;
mod recurrence;
mod scheduler;
mod windows;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

/// Evento: cambiaron tareas, notas o listas.
pub const EV_ITEMS: &str = "items-changed";
/// Evento: cambió un ajuste (`(clave, valor)`).
pub const EV_SETTINGS: &str = "settings-changed";

pub struct AppState {
    db: Mutex<rusqlite::Connection>,
    /// Elementos cuyo aviso está a la vista en la ventana de recordatorio.
    alerts: Mutex<Vec<i64>>,
    /// Avisos en pausa hasta este instante (ms).
    pub paused_until: Mutex<i64>,
    /// Atajos globales registrados: acción → combinación.
    pub globals: Mutex<HashMap<String, String>>,
    pub data_dir: PathBuf,
    pub portable: bool,
    pub start_minimized: bool,
}

impl AppState {
    /// Conexión a la base de datos. Si otro hilo falló con el bloqueo tomado,
    /// se recupera igualmente: SQLite mantiene la base consistente.
    pub fn db(&self) -> MutexGuard<'_, rusqlite::Connection> {
        self.db.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Avisos visibles en la ventana de recordatorio.
    pub fn alerts(&self) -> MutexGuard<'_, Vec<i64>> {
        self.alerts.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Quita un elemento de los avisos visibles.
    pub fn drop_alert(&self, id: i64) {
        self.alerts().retain(|a| *a != id);
    }

    pub fn setting(&self, key: &str) -> String {
        db::get_setting(&self.db(), key)
    }
}

/// Error al arrancar: se registra y se muestra en pantalla (en release no hay consola).
fn fatal(msg: &str, err: impl std::fmt::Display) -> ! {
    logger::error(format!("{msg}: {err}"));
    rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Error)
        .set_title("NowNotes")
        .set_description(format!("{msg}.\n\n{err}"))
        .set_buttons(rfd::MessageButtons::Ok)
        .show();
    std::process::exit(1);
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir NowNotes", true, None::<&str>)?;
    let quick = MenuItem::with_id(app, "quick", "Nueva nota o tarea", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pausar avisos 1 hora", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quick, &pause, &sep, &quit])?;

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("NowNotes")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => windows::show_main(app),
            "quick" => windows::toggle_quick(app),
            "pause" => {
                let state = app.state::<AppState>();
                *state.paused_until.lock().unwrap_or_else(|p| p.into_inner()) =
                    db::now_ms() + db::HOUR_MS;
            }
            "quit" => {
                windows::save_size(app);
                app.exit(0)
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                windows::show_main(tray.app_handle());
            }
        });
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.build(app)?;
    Ok(())
}

/// Si «Apunta» (nombre anterior) estaba configurada para iniciar con Windows,
/// se quita esa entrada y se registra NowNotes en su lugar.
#[cfg(windows)]
fn migrate_autostart(app: &AppHandle) {
    use std::os::windows::process::CommandExt;
    use std::process::Command;
    use tauri_plugin_autostart::ManagerExt;
    const NO_WINDOW: u32 = 0x0800_0000;
    const RUN: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";
    let had_legacy = Command::new("reg")
        .args(["query", RUN, "/v", "Apunta"])
        .creation_flags(NO_WINDOW)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if had_legacy {
        let _ = Command::new("reg")
            .args(["delete", RUN, "/v", "Apunta", "/f"])
            .creation_flags(NO_WINDOW)
            .output();
        let _ = app.autolaunch().enable();
    }
}

fn main() {
    let loc = paths::resolve_current();
    let migrated = paths::prepare(&loc)
        .unwrap_or_else(|e| fatal("No se pudo preparar la carpeta de datos", e));
    logger::init(&loc.data_dir);
    let conn = db::open(&loc.data_dir.join(paths::DB_FILE))
        .unwrap_or_else(|e| fatal("No se pudo abrir la base de datos", e));
    let start_minimized = std::env::args().any(|a| a == "--minimized");

    let app = tauri::Builder::default()
        // single-instance tiene que ir el primero
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            windows::show_main(app)
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, shortcut, event| {
                    if event.state() != ShortcutState::Pressed {
                        return;
                    }
                    match commands::action_for(app, shortcut).as_deref() {
                        Some("toggle") => windows::toggle_main(app),
                        Some("quick") => windows::toggle_quick(app),
                        _ => {}
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            db: Mutex::new(conn),
            alerts: Mutex::new(vec![]),
            paused_until: Mutex::new(0),
            globals: Mutex::new(HashMap::new()),
            data_dir: loc.data_dir.clone(),
            portable: loc.portable,
            start_minimized,
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            windows::create_main(&handle, !start_minimized)?;
            build_tray(&handle)?;
            for (action, key) in commands::GLOBAL_ACTIONS {
                let accel = handle.state::<AppState>().setting(key);
                if let Err(err) = commands::apply_shortcut(&handle, action, &accel) {
                    logger::error(format!("Atajo global «{action}»: {err}"));
                }
            }
            #[cfg(windows)]
            if migrated {
                migrate_autostart(&handle);
            }
            commands::update_tray(&handle);
            scheduler::start(handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                match window.label() {
                    "main" => {
                        windows::save_size(window.app_handle());
                        let to_tray = window.state::<AppState>().setting("close_to_tray") != "0";
                        if to_tray {
                            api.prevent_close();
                            let _ = window.hide();
                        } else {
                            window.app_handle().exit(0);
                        }
                    }
                    "quick" => {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                    _ => {}
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_info,
            commands::query_items,
            commands::get_item,
            commands::save_item,
            commands::delete_item,
            commands::toggle_done,
            commands::list_lists,
            commands::save_list,
            commands::delete_list,
            commands::list_tags,
            commands::counts,
            commands::pending_alerts,
            commands::snooze_alert,
            commands::resolve_alert,
            commands::get_settings,
            commands::set_setting,
            commands::read_file_bytes,
            commands::export_data,
            commands::import_data,
            commands::show_main,
            commands::open_quick,
            commands::startup_minimized,
            commands::clear_done,
            commands::restore_item,
            commands::delete_forever,
            commands::empty_trash,
            commands::set_ui_mode,
            commands::reorder_items,
            commands::list_backups,
            commands::restore_backup,
            commands::open_backups_dir,
        ])
        .build(tauri::generate_context!())
        .unwrap_or_else(|e| fatal("No se pudo iniciar NowNotes", e));

    app.run(|_app, event| {
        // Si se cierra la última ventana visible (p. ej. el aviso), la app sigue viva en la bandeja.
        if let RunEvent::ExitRequested { api, code, .. } = event {
            if code.is_none() {
                api.prevent_exit();
            }
        }
    });
}
