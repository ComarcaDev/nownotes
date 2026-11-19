//! Decide dónde se guardan los datos: junto al .exe (modo portable) o en %APPDATA%.
//! También migra los datos de la versión anterior, cuando la app se llamaba «Apunta».

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub struct Location {
    pub data_dir: PathBuf,
    pub portable: bool,
    /// Carpeta de datos de la versión anterior («Apunta»), solo en modo instalado.
    pub legacy_dir: Option<PathBuf>,
}

/// Nombre del archivo marcador que activa el modo portable.
pub const PORTABLE_MARKER: &str = "portable";
/// Base de datos actual y la de la versión anterior.
pub const DB_FILE: &str = "nownotes.db";
const LEGACY_DB_FILE: &str = "apunta.db";

fn app_folder() -> &'static str {
    if cfg!(debug_assertions) {
        "NowNotes-dev"
    } else {
        "NowNotes"
    }
}

fn legacy_folder() -> &'static str {
    if cfg!(debug_assertions) {
        "Apunta-dev"
    } else {
        "Apunta"
    }
}

pub fn resolve(exe_dir: Option<&Path>, appdata: Option<PathBuf>) -> Location {
    if let Some(dir) = exe_dir {
        if dir.join(PORTABLE_MARKER).is_file() {
            return Location {
                data_dir: dir.join("data"),
                portable: true,
                legacy_dir: None,
            };
        }
    }
    let base = appdata.unwrap_or_else(std::env::temp_dir);
    Location {
        data_dir: base.join(app_folder()),
        portable: false,
        legacy_dir: Some(base.join(legacy_folder())),
    }
}

pub fn resolve_current() -> Location {
    let exe = std::env::current_exe().ok();
    let dir = exe.as_deref().and_then(|p| p.parent());
    resolve(dir, std::env::var_os("APPDATA").map(PathBuf::from))
}

fn copy_dir(from: &Path, to: &Path) -> io::Result<()> {
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

/// Prepara la carpeta de datos, migrando la de «Apunta» si hace falta.
/// Devuelve `true` si se migraron datos de la versión anterior.
pub fn prepare(loc: &Location) -> io::Result<bool> {
    let mut migrated = false;

    // 1. Carpeta %APPDATA%\Apunta → %APPDATA%\NowNotes (se mueve; si falla, se copia).
    if let Some(legacy) = &loc.legacy_dir {
        if !loc.data_dir.exists() && legacy.is_dir() {
            if fs::rename(legacy, &loc.data_dir).is_err() {
                copy_dir(legacy, &loc.data_dir)?;
            }
            migrated = true;
        }
    }
    fs::create_dir_all(&loc.data_dir)?;

    // 2. apunta.db (+ -wal / -shm) → nownotes.db. También en modo portable.
    let new_db = loc.data_dir.join(DB_FILE);
    let old_db = loc.data_dir.join(LEGACY_DB_FILE);
    if !new_db.exists() && old_db.is_file() {
        for suffix in ["", "-wal", "-shm"] {
            let from = loc.data_dir.join(format!("{LEGACY_DB_FILE}{suffix}"));
            if from.exists() {
                fs::rename(&from, loc.data_dir.join(format!("{DB_FILE}{suffix}")))?;
            }
        }
        migrated = true;
    }
    Ok(migrated)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("nownotes-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn portable_when_marker_exists() {
        let exe = tmp("portable");
        fs::write(exe.join(PORTABLE_MARKER), "").unwrap();
        let loc = resolve(Some(&exe), Some(PathBuf::from("C:/AppData")));
        assert!(loc.portable);
        assert!(loc.legacy_dir.is_none());
        assert_eq!(loc.data_dir, exe.join("data"));
    }

    #[test]
    fn appdata_without_marker() {
        let exe = tmp("installed");
        let loc = resolve(Some(&exe), Some(PathBuf::from("C:/AppData")));
        assert!(!loc.portable);
        assert_eq!(loc.data_dir, PathBuf::from("C:/AppData").join(app_folder()));
    }

    #[test]
    fn migrates_legacy_folder_and_db() {
        let appdata = tmp("migrate");
        let legacy = appdata.join(legacy_folder());
        fs::create_dir_all(legacy.join("backups")).unwrap();
        fs::write(legacy.join(LEGACY_DB_FILE), "db").unwrap();
        fs::write(legacy.join(format!("{LEGACY_DB_FILE}-wal")), "wal").unwrap();
        let loc = resolve(None, Some(appdata.clone()));

        assert!(prepare(&loc).unwrap());
        assert!(!legacy.exists());
        assert_eq!(
            fs::read_to_string(loc.data_dir.join(DB_FILE)).unwrap(),
            "db"
        );
        assert!(loc.data_dir.join(format!("{DB_FILE}-wal")).exists());
        assert!(loc.data_dir.join("backups").is_dir());
        // Una segunda vez no hace nada.
        assert!(!prepare(&loc).unwrap());
    }

    #[test]
    fn portable_db_is_renamed() {
        let exe = tmp("portable-db");
        fs::write(exe.join(PORTABLE_MARKER), "").unwrap();
        fs::create_dir_all(exe.join("data")).unwrap();
        fs::write(exe.join("data").join(LEGACY_DB_FILE), "db").unwrap();
        let loc = resolve(Some(&exe), None);
        assert!(prepare(&loc).unwrap());
        assert!(loc.data_dir.join(DB_FILE).exists());
        assert!(!loc.data_dir.join(LEGACY_DB_FILE).exists());
    }

    #[test]
    fn fresh_install_creates_folder() {
        let appdata = tmp("fresh");
        let loc = resolve(None, Some(appdata));
        assert!(!prepare(&loc).unwrap());
        assert!(loc.data_dir.is_dir());
    }
}
