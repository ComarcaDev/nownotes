//! Bucle en segundo plano que dispara los recordatorios vencidos.

use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::{db, windows, AppState};

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        // Pequeña espera para que la ventana principal termine de cargar.
        std::thread::sleep(Duration::from_secs(2));
        loop {
            tick(&app);
            std::thread::sleep(Duration::from_secs(15));
        }
    });
}

pub fn tick(app: &AppHandle) {
    let state = app.state::<AppState>();
    let now = db::now_ms();
    if *state.paused_until.lock().unwrap() > now {
        return;
    }

    let (fired, titles, toast) = {
        let c = state.db.lock().unwrap();
        let fired = match db::fire_due(&c, now) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error al revisar recordatorios: {e}");
                return;
            }
        };
        let titles: Vec<String> = fired
            .iter()
            .filter_map(|id| db::get_item(&c, *id).ok().flatten())
            .map(|it| {
                if it.title.is_empty() {
                    "(sin título)".to_string()
                } else {
                    it.title
                }
            })
            .collect();
        (fired, titles, db::get_setting(&c, "native_toast") != "0")
    };
    if fired.is_empty() {
        return;
    }

    {
        let mut alerts = state.alerts.lock().unwrap();
        for id in &fired {
            if !alerts.contains(id) {
                alerts.push(*id);
            }
        }
    }

    if toast {
        let body = titles.join("\n");
        let _ = app
            .notification()
            .builder()
            .title("Apunta · recordatorio")
            .body(body)
            .show();
    }
    windows::show_reminder(app);
    let _ = app.emit("items-changed", ());
}
