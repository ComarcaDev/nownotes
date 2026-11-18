#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod db;
mod paths;
mod recurrence;
mod scheduler;
mod windows;

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
    /// Elementos cuyo aviso está a la vista en la ventana de recordatorio.
    pub alerts: Mutex<Vec<i64>>,
    /// Avisos en pausa hasta este instante (ms).
    pub paused_until: Mutex<i64>,
    pub shortcut: Mutex<String>,
    pub data_dir: PathBuf,
    pub portable: bool,
    pub start_minimized: bool,
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir Apunta", true, None::<&str>)?;
    let quick = MenuItem::with_id(app, "quick", "Captura rápida", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, "pause", "Pausar avisos 1 hora", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quick, &pause, &sep, &quit])?;

    let mut tray = TrayIconBuilder::with_id("main")
        .tooltip("Apunta")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => windows::show_main(app),
            "quick" => windows::toggle_quick(app),
            "pause" => {
                let st = app.state::<AppState>();
                *st.paused_until.lock().unwrap() = db::now_ms() + 3_600_000;
            }
            "quit" => app.exit(0),
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

fn main() {
    let loc = paths::resolve_current();
    std::fs::create_dir_all(&loc.data_dir).expect("No se pudo crear la carpeta de datos");
    let conn =
        db::open(&loc.data_dir.join("apunta.db")).expect("No se pudo abrir la base de datos");
    let start_minimized = std::env::args().any(|a| a == "--minimized");

    let app = tauri::Builder::default()
        // single-instance tiene que ir el primero
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            windows::show_main(app)
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        windows::toggle_quick(app);
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
            shortcut: Mutex::new(String::new()),
            data_dir: loc.data_dir.clone(),
            portable: loc.portable,
            start_minimized,
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            windows::create_main(&handle, !start_minimized)?;
            build_tray(&handle)?;
            let shortcut = {
                let st = handle.state::<AppState>();
                let c = st.db.lock().unwrap();
                db::get_setting(&c, "shortcut")
            };
            if let Err(err) = commands::apply_shortcut(&handle, &shortcut) {
                eprintln!("{err}");
            }
            scheduler::start(handle);
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                match window.label() {
                    "main" => {
                        let to_tray = {
                            let st = window.state::<AppState>();
                            let c = st.db.lock().unwrap();
                            db::get_setting(&c, "close_to_tray") != "0"
                        };
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
        ])
        .build(tauri::generate_context!())
        .expect("Error al iniciar Apunta");

    app.run(|_app, event| {
        // Si se cierra la última ventana visible (p. ej. el aviso), la app sigue viva en la bandeja.
        if let RunEvent::ExitRequested { api, code, .. } = event {
            if code.is_none() {
                api.prevent_exit();
            }
        }
    });
}
