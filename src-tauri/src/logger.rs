//! Registro de errores en `nownotes.log`, dentro de la carpeta de datos.
//! En release no hay consola, así que `eprintln!` no llegaría a ningún sitio.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// Tamaño máximo antes de rotar a `nownotes.log.old`.
const MAX_BYTES: u64 = 256 * 1024;

static LOG_PATH: OnceLock<PathBuf> = OnceLock::new();
static WRITE: Mutex<()> = Mutex::new(());

pub fn init(data_dir: &std::path::Path) {
    let _ = LOG_PATH.set(data_dir.join("nownotes.log"));
}

/// Anota un error (con fecha). Nunca falla: si no se puede escribir, se ignora.
pub fn error(msg: impl std::fmt::Display) {
    let line = format!(
        "[{}] {msg}\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    );
    if cfg!(debug_assertions) {
        eprint!("{line}");
    }
    let Some(path) = LOG_PATH.get() else { return };
    let _guard = WRITE.lock().unwrap_or_else(|p| p.into_inner());
    if fs::metadata(path)
        .map(|m| m.len() > MAX_BYTES)
        .unwrap_or(false)
    {
        let _ = fs::rename(path, path.with_extension("log.old"));
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = f.write_all(line.as_bytes());
    }
}
