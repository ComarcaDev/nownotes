//! Decide dónde se guardan los datos: junto al .exe (modo portable) o en %APPDATA%.

use std::path::{Path, PathBuf};

pub struct Location {
    pub data_dir: PathBuf,
    pub portable: bool,
}

/// Nombre del archivo marcador que activa el modo portable.
pub const PORTABLE_MARKER: &str = "portable";

fn app_folder() -> &'static str {
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
            };
        }
    }
    let base = appdata.unwrap_or_else(std::env::temp_dir);
    Location {
        data_dir: base.join(app_folder()),
        portable: false,
    }
}

pub fn resolve_current() -> Location {
    let exe = std::env::current_exe().ok();
    let dir = exe.as_deref().and_then(|p| p.parent());
    resolve(dir, std::env::var_os("APPDATA").map(PathBuf::from))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("apunta-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn portable_when_marker_exists() {
        let exe = tmp("portable");
        std::fs::write(exe.join(PORTABLE_MARKER), "").unwrap();
        let loc = resolve(Some(&exe), Some(PathBuf::from("C:/AppData")));
        assert!(loc.portable);
        assert_eq!(loc.data_dir, exe.join("data"));
    }

    #[test]
    fn appdata_without_marker() {
        let exe = tmp("installed");
        let loc = resolve(Some(&exe), Some(PathBuf::from("C:/AppData")));
        assert!(!loc.portable);
        assert_eq!(loc.data_dir, PathBuf::from("C:/AppData").join(app_folder()));
    }
}
