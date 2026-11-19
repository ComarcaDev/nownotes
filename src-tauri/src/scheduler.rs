//! Bucle en segundo plano: dispara los recordatorios vencidos y hace el mantenimiento horario.

use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

use crate::{backup, commands, db, logger, windows, AppState, EV_ITEMS};

/// Cada cuánto se revisan los recordatorios.
const TICK: Duration = Duration::from_secs(15);
/// Vueltas entre mantenimientos: 240 × 15 s = 1 hora.
const TICKS_PER_HOUR: u32 = 240;

pub fn start(app: AppHandle) {
    std::thread::spawn(move || {
        // Pequeña espera para que la ventana principal termine de cargar.
        std::thread::sleep(Duration::from_secs(2));
        let mut ticks: u32 = 0;
        loop {
            if ticks.is_multiple_of(TICKS_PER_HOUR) {
                maintenance(&app);
            }
            tick(&app);
            ticks = ticks.wrapping_add(1);
            std::thread::sleep(TICK);
        }
    });
}

/// Mantenimiento horario: copia de seguridad del día y purga de la papelera caducada.
fn maintenance(app: &AppHandle) {
    let state = app.state::<AppState>();
    let purged = {
        let c = state.db();
        let day = chrono::Local::now().format("%Y-%m-%d").to_string();
        if let Err(e) = backup::daily(&c, &state.data_dir, &day) {
            logger::error(format!("No se pudo crear la copia de seguridad: {e}"));
        }
        db::purge_trash(&c, db::now_ms()).unwrap_or_else(|e| {
            logger::error(format!("No se pudo vaciar la papelera caducada: {e}"));
            0
        })
    };
    if purged > 0 {
        let _ = app.emit(EV_ITEMS, ());
    }
}

fn tick(app: &AppHandle) {
    // Mantiene al día el tooltip de la bandeja (incluido el cambio de día).
    commands::update_tray(app);
    let state = app.state::<AppState>();
    let now = db::now_ms();
    if *state.paused_until.lock().unwrap_or_else(|p| p.into_inner()) > now {
        return;
    }

    let (fired, titles, toast) = {
        let c = state.db();
        let fired = match db::fire_due(&c, now) {
            Ok(f) => f,
            Err(e) => {
                logger::error(format!("Error al revisar recordatorios: {e}"));
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
        let mut alerts = state.alerts();
        for id in &fired {
            if !alerts.contains(id) {
                alerts.push(*id);
            }
        }
    }

    if toast {
        let _ = app
            .notification()
            .builder()
            .title("NowNotes · recordatorio")
            .body(titles.join("\n"))
            .show();
    }
    windows::show_reminder(app);
    let _ = app.emit(EV_ITEMS, ());
}
