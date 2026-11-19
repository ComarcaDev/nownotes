//! Copias de seguridad diarias de la base de datos (se conservan las últimas `KEEP`).

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;
use serde::Serialize;

/// Copias diarias que se conservan.
pub const KEEP: usize = 7;
/// Margen extra para las copias «antes de restaurar».
const KEEP_WITH_SNAPSHOTS: usize = KEEP + 3;
const PREFIX: &str = "nownotes-";
const EXT: &str = ".db";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupInfo {
    pub name: String,
    pub modified: i64,
    pub size: u64,
}

pub fn dir(data_dir: &Path) -> PathBuf {
    data_dir.join("backups")
}

fn is_backup_name(name: &str) -> bool {
    name.starts_with(PREFIX)
        && name.ends_with(EXT)
        && !name.contains(['/', '\\'])
        && !name.contains("..")
}

/// Copia `c` en `target` (que no debe existir) de forma consistente.
fn copy_to(c: &Connection, target: &Path) -> Result<(), String> {
    c.execute("VACUUM INTO ?1", [target.to_string_lossy()])
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Crea la copia del día si aún no existe. Devuelve su ruta si se creó.
pub fn daily(c: &Connection, data_dir: &Path, day: &str) -> Result<Option<PathBuf>, String> {
    let d = dir(data_dir);
    fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let target = d.join(format!("{PREFIX}{day}{EXT}"));
    if target.exists() {
        return Ok(None);
    }
    copy_to(c, &target)?;
    prune(&d, KEEP);
    Ok(Some(target))
}

/// Copia manual antes de restaurar otra (no cuenta para el límite diario).
pub fn snapshot(c: &Connection, data_dir: &Path, stamp: &str) -> Result<PathBuf, String> {
    let d = dir(data_dir);
    fs::create_dir_all(&d).map_err(|e| e.to_string())?;
    let target = d.join(format!("{PREFIX}{stamp}-antes-de-restaurar{EXT}"));
    if target.exists() {
        fs::remove_file(&target).map_err(|e| e.to_string())?;
    }
    copy_to(c, &target)?;
    prune(&d, KEEP_WITH_SNAPSHOTS);
    Ok(target)
}

/// Lista las copias, la más reciente primero.
pub fn list(data_dir: &Path) -> Vec<BackupInfo> {
    let Ok(rd) = fs::read_dir(dir(data_dir)) else {
        return vec![];
    };
    let mut out: Vec<BackupInfo> = rd
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            if !is_backup_name(&name) {
                return None;
            }
            let meta = e.metadata().ok()?;
            let modified = meta
                .modified()
                .ok()?
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_millis() as i64;
            Some(BackupInfo {
                name,
                modified,
                size: meta.len(),
            })
        })
        .collect();
    out.sort_by(|a, b| b.modified.cmp(&a.modified).then(b.name.cmp(&a.name)));
    out
}

/// Borra las copias más antiguas dejando `keep`.
fn prune(d: &Path, keep: usize) {
    let Ok(rd) = fs::read_dir(d) else { return };
    let mut names: Vec<String> = rd
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| is_backup_name(n))
        .collect();
    names.sort();
    names.reverse(); // el nombre empieza por la fecha: orden descendente = más reciente primero
    for old in names.iter().skip(keep) {
        if let Err(e) = fs::remove_file(d.join(old)) {
            crate::logger::error(format!("No se pudo borrar la copia antigua {old}: {e}"));
        }
    }
}

/// Ruta segura de una copia a partir de su nombre (rechaza rutas manipuladas).
pub fn path_of(data_dir: &Path, name: &str) -> Result<PathBuf, String> {
    if !is_backup_name(name) {
        return Err("Nombre de copia no válido".into());
    }
    let p = dir(data_dir).join(name);
    if !p.is_file() {
        return Err("La copia ya no existe".into());
    }
    Ok(p)
}

/// Sustituye el contenido de `c` por el de la copia indicada.
pub fn restore(c: &mut Connection, source: &Path) -> Result<(), String> {
    c.restore(
        rusqlite::DatabaseName::Main,
        source,
        None::<fn(rusqlite::backup::Progress)>,
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("nownotes-bk-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn daily_once_and_prune() {
        let data = tmp("daily");
        let c = crate::db::open(&data.join("nownotes.db")).unwrap();
        assert!(daily(&c, &data, "2026-11-19").unwrap().is_some());
        assert!(daily(&c, &data, "2026-11-19").unwrap().is_none());
        for d in 1..=9 {
            daily(&c, &data, &format!("2026-12-{d:02}")).unwrap();
        }
        let names: Vec<String> = list(&data).into_iter().map(|b| b.name).collect();
        assert_eq!(names.len(), KEEP);
        assert!(names.contains(&"nownotes-2026-12-09.db".to_string()));
        assert!(!names.contains(&"nownotes-2026-11-19.db".to_string()));
    }

    #[test]
    fn restore_brings_back_old_state() {
        let data = tmp("restore");
        let mut c = crate::db::open(&data.join("nownotes.db")).unwrap();
        crate::db::set_setting(&c, "volume", "0.1").unwrap();
        let bk = daily(&c, &data, "2026-11-19").unwrap().unwrap();
        crate::db::set_setting(&c, "volume", "0.9").unwrap();
        restore(&mut c, &bk).unwrap();
        assert_eq!(crate::db::get_setting(&c, "volume"), "0.1");
    }

    #[test]
    fn rejects_bad_names() {
        let data = tmp("names");
        assert!(path_of(&data, "../nownotes.db").is_err());
        assert!(path_of(&data, "otra.db").is_err());
        assert!(path_of(&data, "nownotes-2026-01-01.db").is_err()); // no existe
    }
}
