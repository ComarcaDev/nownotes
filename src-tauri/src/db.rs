//! Almacenamiento SQLite: esquema, CRUD y lógica de recordatorios.

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{params, params_from_iter, types::Value, Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};

use crate::recurrence;

type R<T> = rusqlite::Result<T>;

/// Valores por defecto de los ajustes (lo que no está guardado en `settings`).
const DEFAULTS: &[(&str, &str)] = &[
    ("shortcut", "Ctrl+Alt+N"),
    ("sound", "chime"),
    ("sound_path", ""),
    ("volume", "0.7"),
    ("theme", "dark"),
    ("close_to_tray", "1"),
    ("native_toast", "1"),
    ("ui_mode", "full"),
    ("always_on_top", "0"),
    ("size_full", "1040,700"),
    ("size_mini", "380,560"),
    ("today_kind", "task"),
    ("shortcut_toggle", "Ctrl+Alt+A"),
    ("keys", ""),
    ("update_check", "1"),
];

/// Días que un elemento permanece en la papelera antes de borrarse definitivamente.
pub const TRASH_DAYS: i64 = 10;
pub const HOUR_MS: i64 = 3_600_000;
pub const DAY_MS: i64 = 24 * HOUR_MS;

const SCHEMA_V2: &str = "
ALTER TABLE items ADD COLUMN deleted_at INTEGER;
CREATE INDEX IF NOT EXISTS idx_items_deleted ON items(deleted_at);
";

/// v3: orden manual (arrastrar para reordenar). 0 = sin orden manual (aparece arriba).
const SCHEMA_V3: &str = "ALTER TABLE items ADD COLUMN sort INTEGER NOT NULL DEFAULT 0;";

const SCHEMA_V1: &str = "
CREATE TABLE IF NOT EXISTS lists(
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  color TEXT NOT NULL DEFAULT '#0067c0',
  sort INTEGER NOT NULL DEFAULT 0
);
CREATE TABLE IF NOT EXISTS items(
  id INTEGER PRIMARY KEY,
  kind TEXT NOT NULL DEFAULT 'task',
  title TEXT NOT NULL DEFAULT '',
  body TEXT NOT NULL DEFAULT '',
  list_id INTEGER NOT NULL DEFAULT 1,
  done INTEGER NOT NULL DEFAULT 0,
  done_at INTEGER,
  due_at INTEGER,
  remind_at INTEGER,
  recur TEXT NOT NULL DEFAULT 'none',
  snoozed_until INTEGER,
  notified INTEGER NOT NULL DEFAULT 0,
  pinned INTEGER NOT NULL DEFAULT 0,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_items_alert ON items(done, notified, remind_at);
CREATE INDEX IF NOT EXISTS idx_items_list ON items(list_id, done);
CREATE TABLE IF NOT EXISTS tags(
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE COLLATE NOCASE
);
CREATE TABLE IF NOT EXISTS item_tags(
  item_id INTEGER NOT NULL REFERENCES items(id) ON DELETE CASCADE,
  tag_id INTEGER NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
  PRIMARY KEY(item_id, tag_id)
);
CREATE TABLE IF NOT EXISTS settings(key TEXT PRIMARY KEY, value TEXT NOT NULL);
INSERT OR IGNORE INTO lists(id, name, color, sort) VALUES (1, 'Bandeja de entrada', '#0067c0', 0);
";

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: i64,
    pub kind: String,
    pub title: String,
    pub body: String,
    pub list_id: i64,
    pub done: bool,
    pub done_at: Option<i64>,
    pub due_at: Option<i64>,
    pub remind_at: Option<i64>,
    pub recur: String,
    pub snoozed_until: Option<i64>,
    pub pinned: bool,
    pub created_at: i64,
    pub updated_at: i64,
    /// Momento en que se movió a la papelera (None = no está en la papelera).
    #[serde(default)]
    pub deleted_at: Option<i64>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ItemInput {
    pub id: Option<i64>,
    pub kind: String,
    pub title: String,
    #[serde(default)]
    pub body: String,
    pub list_id: Option<i64>,
    pub due_at: Option<i64>,
    pub remind_at: Option<i64>,
    #[serde(default)]
    pub recur: String,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct List {
    pub id: i64,
    pub name: String,
    pub color: String,
    pub sort: i64,
    #[serde(default)]
    pub count: i64,
}

#[derive(Serialize)]
pub struct TagCount {
    pub name: String,
    pub count: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Counts {
    /// Todo lo que muestra «Hoy»: tareas que vencen hoy o antes + notas creadas hoy.
    pub today: i64,
    /// Solo las tareas de «Hoy».
    pub today_tasks: i64,
    pub upcoming: i64,
    pub tasks: i64,
    pub notes: i64,
    pub done: i64,
    pub trash: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Filter {
    pub view: String,
    pub list_id: Option<i64>,
    pub tag: Option<String>,
    pub search: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct ExportData {
    pub version: u32,
    pub lists: Vec<List>,
    pub items: Vec<Item>,
    pub settings: HashMap<String, String>,
}

pub fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

pub fn end_of_today_ms() -> i64 {
    use chrono::{Local, TimeZone};
    let d = Local::now()
        .date_naive()
        .and_hms_milli_opt(23, 59, 59, 999)
        .expect("23:59:59.999 siempre es una hora válida");
    Local
        .from_local_datetime(&d)
        .earliest()
        .map(|t| t.timestamp_millis())
        .unwrap_or_else(now_ms)
}

/// Inicio (00:00 local) del día que contiene `ts_ms`.
pub fn start_of_day(ts_ms: i64) -> i64 {
    use chrono::{Local, TimeZone};
    Local
        .timestamp_millis_opt(ts_ms)
        .single()
        .and_then(|t| t.date_naive().and_hms_opt(0, 0, 0))
        .and_then(|d| Local.from_local_datetime(&d).earliest())
        .map(|t| t.timestamp_millis())
        .unwrap_or(ts_ms - DAY_MS + 1)
}

// ---------- apertura y migraciones ----------

pub fn open(path: &Path) -> R<Connection> {
    let c = Connection::open(path)?;
    c.query_row("PRAGMA journal_mode=WAL", [], |_| Ok(()))?;
    init(&c)?;
    Ok(c)
}

#[cfg(test)]
pub fn open_in_memory() -> R<Connection> {
    let c = Connection::open_in_memory()?;
    init(&c)?;
    Ok(c)
}

/// Aplica las migraciones pendientes (también tras restaurar una copia antigua).
pub fn init(c: &Connection) -> R<()> {
    c.pragma_update(None, "foreign_keys", "ON")?;
    let v: i64 = c.query_row("PRAGMA user_version", [], |r| r.get(0))?;
    if v < 1 {
        c.execute_batch(SCHEMA_V1)?;
        c.pragma_update(None, "user_version", 1)?;
    }
    if v < 2 {
        c.execute_batch(SCHEMA_V2)?;
        c.pragma_update(None, "user_version", 2)?;
    }
    if v < 3 {
        c.execute_batch(SCHEMA_V3)?;
        c.pragma_update(None, "user_version", 3)?;
    }
    Ok(())
}

// ---------- ajustes ----------

pub fn default_setting(key: &str) -> String {
    DEFAULTS
        .iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| v.to_string())
        .unwrap_or_default()
}

pub fn get_setting(c: &Connection, key: &str) -> String {
    c.query_row("SELECT value FROM settings WHERE key=?1", [key], |r| {
        r.get::<_, String>(0)
    })
    .optional()
    .ok()
    .flatten()
    .unwrap_or_else(|| default_setting(key))
}

pub fn all_settings(c: &Connection) -> R<HashMap<String, String>> {
    let mut m: HashMap<String, String> = DEFAULTS
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let mut st = c.prepare("SELECT key, value FROM settings")?;
    let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    for row in rows {
        let (k, v) = row?;
        m.insert(k, v);
    }
    Ok(m)
}

pub fn set_setting(c: &Connection, key: &str, value: &str) -> R<()> {
    c.execute(
        "INSERT INTO settings(key, value) VALUES(?1, ?2) ON CONFLICT(key) DO UPDATE SET value=excluded.value",
        params![key, value],
    )?;
    Ok(())
}

// ---------- elementos ----------

const ITEM_COLS: &str = "id, kind, title, body, list_id, done, done_at, due_at, remind_at, recur, snoozed_until, pinned, created_at, updated_at, deleted_at";

/// Condición SQL para excluir lo que está en la papelera.
const ALIVE: &str = "deleted_at IS NULL";

fn map_item(r: &Row) -> R<Item> {
    Ok(Item {
        id: r.get(0)?,
        kind: r.get(1)?,
        title: r.get(2)?,
        body: r.get(3)?,
        list_id: r.get(4)?,
        done: r.get(5)?,
        done_at: r.get(6)?,
        due_at: r.get(7)?,
        remind_at: r.get(8)?,
        recur: r.get(9)?,
        snoozed_until: r.get(10)?,
        pinned: r.get(11)?,
        created_at: r.get(12)?,
        updated_at: r.get(13)?,
        deleted_at: r.get(14)?,
        tags: vec![],
    })
}

fn tags_for(c: &Connection, id: i64) -> R<Vec<String>> {
    let mut st = c.prepare(
        "SELECT t.name FROM item_tags it JOIN tags t ON t.id = it.tag_id WHERE it.item_id = ?1 ORDER BY t.name",
    )?;
    let v = st
        .query_map([id], |r| r.get(0))?
        .collect::<R<Vec<String>>>()?;
    Ok(v)
}

fn all_tags_map(c: &Connection) -> R<HashMap<i64, Vec<String>>> {
    let mut st = c.prepare(
        "SELECT it.item_id, t.name FROM item_tags it JOIN tags t ON t.id = it.tag_id ORDER BY t.name",
    )?;
    let mut m: HashMap<i64, Vec<String>> = HashMap::new();
    for row in st.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))? {
        let (id, name) = row?;
        m.entry(id).or_default().push(name);
    }
    Ok(m)
}

fn set_tags(c: &Connection, id: i64, tags: &[String]) -> R<()> {
    c.execute("DELETE FROM item_tags WHERE item_id = ?1", [id])?;
    let mut seen: Vec<String> = vec![];
    for raw in tags {
        let name = raw.trim().trim_start_matches('#').trim().to_string();
        if name.is_empty() || seen.iter().any(|s| s.eq_ignore_ascii_case(&name)) {
            continue;
        }
        c.execute("INSERT OR IGNORE INTO tags(name) VALUES(?1)", [&name])?;
        let tag_id: i64 =
            c.query_row("SELECT id FROM tags WHERE name = ?1", [&name], |r| r.get(0))?;
        c.execute(
            "INSERT OR IGNORE INTO item_tags(item_id, tag_id) VALUES(?1, ?2)",
            params![id, tag_id],
        )?;
        seen.push(name);
    }
    drop_orphan_tags(c)
}

/// Añade a cada elemento sus etiquetas (una sola consulta para todos).
fn attach_tags(c: &Connection, items: &mut [Item]) -> R<()> {
    let tags = all_tags_map(c)?;
    for it in items.iter_mut() {
        if let Some(t) = tags.get(&it.id) {
            it.tags = t.clone();
        }
    }
    Ok(())
}

pub fn get_item(c: &Connection, id: i64) -> R<Option<Item>> {
    let sql = format!("SELECT {ITEM_COLS} FROM items WHERE id = ?1");
    let item = c.query_row(&sql, [id], map_item).optional()?;
    match item {
        Some(mut it) => {
            it.tags = tags_for(c, id)?;
            Ok(Some(it))
        }
        None => Ok(None),
    }
}

/// Fijados → orden manual (los nuevos, sin orden, arriba) → fecha → más recientes.
const TASK_ORDER: &str = "pinned DESC, (sort <> 0), sort, (COALESCE(due_at, remind_at) IS NULL), COALESCE(due_at, remind_at), created_at DESC";
const NOTE_ORDER: &str = "pinned DESC, (sort <> 0), sort, updated_at DESC";

pub fn query_items(c: &Connection, f: &Filter, end_of_today: i64) -> R<Vec<Item>> {
    let mut wh: Vec<String> = vec![];
    let mut ps: Vec<Value> = vec![];
    let order: &str;
    let search = f.search.as_deref().map(str::trim).filter(|s| !s.is_empty());

    if f.view == "trash" && search.is_none() {
        wh.push("deleted_at IS NOT NULL".into());
        order = "deleted_at DESC";
    } else if let Some(q) = search {
        wh.push("(title LIKE ?1 OR body LIKE ?1)".into());
        wh.push(ALIVE.into());
        ps.push(Value::Text(format!("%{q}%")));
        order = "done, updated_at DESC";
    } else {
        match f.view.as_str() {
            // Tareas que vencen hoy o antes + notas creadas hoy («Notas de hoy»).
            "today" => {
                wh.push(
                    "done=0 AND ((kind='task' AND COALESCE(due_at, remind_at) <= ?1)
                       OR (kind='note' AND created_at >= ?2))"
                        .into(),
                );
                ps.push(Value::Integer(end_of_today));
                ps.push(Value::Integer(start_of_day(end_of_today)));
                order = "kind='note', COALESCE(due_at, remind_at), created_at";
            }
            "upcoming" => {
                wh.push("kind='task' AND done=0 AND COALESCE(due_at, remind_at) > ?1".into());
                ps.push(Value::Integer(end_of_today));
                order = "COALESCE(due_at, remind_at), created_at";
            }
            "notes" => {
                wh.push("kind='note' AND done=0".into());
                order = NOTE_ORDER;
            }
            "done" => {
                wh.push("done=1".into());
                order = "done_at DESC";
            }
            "list" => {
                wh.push("list_id=?1 AND done=0".into());
                ps.push(Value::Integer(f.list_id.unwrap_or(1)));
                order = TASK_ORDER;
            }
            "tag" => {
                wh.push("done=0 AND id IN (SELECT it.item_id FROM item_tags it JOIN tags t ON t.id = it.tag_id WHERE t.name = ?1)".into());
                ps.push(Value::Text(f.tag.clone().unwrap_or_default()));
                order = TASK_ORDER;
            }
            // "tasks" y cualquier vista desconocida
            _ => {
                wh.push("kind='task' AND done=0".into());
                order = TASK_ORDER;
            }
        }
        wh.push(ALIVE.into());
    }

    let sql = format!(
        "SELECT {ITEM_COLS} FROM items WHERE {} ORDER BY {order} LIMIT 1000",
        wh.join(" AND ")
    );
    let mut st = c.prepare(&sql)?;
    let mut items = st
        .query_map(params_from_iter(ps.iter()), map_item)?
        .collect::<R<Vec<Item>>>()?;
    attach_tags(c, &mut items)?;
    Ok(items)
}

pub fn save_item(c: &Connection, inp: &ItemInput, now: i64) -> R<Item> {
    let list_id = inp.list_id.unwrap_or(1);
    let kind = if inp.kind == "note" { "note" } else { "task" };
    let recur = recurrence::normalize(&inp.recur);
    let title = inp.title.trim();
    let id = match inp.id {
        Some(id) => {
            let old: Option<Option<i64>> = c
                .query_row("SELECT remind_at FROM items WHERE id = ?1", [id], |r| {
                    r.get(0)
                })
                .optional()?;
            let Some(old_remind) = old else {
                return Err(rusqlite::Error::QueryReturnedNoRows);
            };
            c.execute(
                "UPDATE items SET kind=?2, title=?3, body=?4, list_id=?5, due_at=?6, remind_at=?7, recur=?8, pinned=?9, updated_at=?10 WHERE id=?1",
                params![id, kind, title, inp.body, list_id, inp.due_at, inp.remind_at, recur, inp.pinned, now],
            )?;
            if old_remind != inp.remind_at {
                c.execute(
                    "UPDATE items SET notified=0, snoozed_until=NULL WHERE id=?1",
                    [id],
                )?;
            }
            id
        }
        None => {
            c.execute(
                "INSERT INTO items(kind, title, body, list_id, due_at, remind_at, recur, pinned, created_at, updated_at) VALUES(?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?9)",
                params![kind, title, inp.body, list_id, inp.due_at, inp.remind_at, recur, inp.pinned, now],
            )?;
            c.last_insert_rowid()
        }
    };
    set_tags(c, id, &inp.tags)?;
    get_item(c, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

/// Guarda el orden manual: `ids` en el orden en que deben mostrarse.
pub fn reorder_items(c: &Connection, ids: &[i64]) -> R<()> {
    let tx = c.unchecked_transaction()?;
    for (i, id) in ids.iter().enumerate() {
        tx.execute(
            "UPDATE items SET sort=?2 WHERE id=?1",
            params![id, i as i64 + 1],
        )?;
    }
    tx.commit()
}

// ---------- papelera ----------

fn drop_orphan_tags(c: &Connection) -> R<()> {
    c.execute(
        "DELETE FROM tags WHERE id NOT IN (SELECT tag_id FROM item_tags)",
        [],
    )?;
    Ok(())
}

/// Mueve todos los completados a la papelera. Devuelve cuántos se movieron.
pub fn clear_done(c: &Connection, now: i64) -> R<usize> {
    c.execute(
        "UPDATE items SET deleted_at=?1, snoozed_until=NULL WHERE done=1 AND deleted_at IS NULL",
        [now],
    )
}

/// Mueve un elemento a la papelera (se puede restaurar durante `TRASH_DAYS` días).
pub fn delete_item(c: &Connection, id: i64, now: i64) -> R<()> {
    c.execute(
        "UPDATE items SET deleted_at=?2, snoozed_until=NULL WHERE id=?1",
        params![id, now],
    )?;
    Ok(())
}

pub fn restore_item(c: &Connection, id: i64) -> R<()> {
    c.execute("UPDATE items SET deleted_at=NULL WHERE id=?1", [id])?;
    // Si su lista se borró mientras estaba en la papelera, vuelve a la bandeja de entrada.
    c.execute(
        "UPDATE items SET list_id=1 WHERE id=?1 AND list_id NOT IN (SELECT id FROM lists)",
        [id],
    )?;
    Ok(())
}

pub fn delete_forever(c: &Connection, id: i64) -> R<()> {
    c.execute("DELETE FROM items WHERE id=?1", [id])?;
    drop_orphan_tags(c)
}

pub fn empty_trash(c: &Connection) -> R<usize> {
    let n = c.execute("DELETE FROM items WHERE deleted_at IS NOT NULL", [])?;
    drop_orphan_tags(c)?;
    Ok(n)
}

/// Borra definitivamente lo que lleva más de `TRASH_DAYS` días en la papelera.
pub fn purge_trash(c: &Connection, now: i64) -> R<usize> {
    let n = c.execute(
        "DELETE FROM items WHERE deleted_at IS NOT NULL AND deleted_at <= ?1",
        [now - TRASH_DAYS * DAY_MS],
    )?;
    if n > 0 {
        drop_orphan_tags(c)?;
    }
    Ok(n)
}

/// Marca/desmarca. En tareas que se repiten, "hecha" mueve las fechas a la siguiente repetición.
pub fn toggle_done(c: &Connection, id: i64, now: i64) -> R<Item> {
    let it = get_item(c, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)?;
    if !it.done && it.recur != "none" && (it.due_at.is_some() || it.remind_at.is_some()) {
        let adv = |d: i64| recurrence::next_after_ms(d, &it.recur, now.max(d)).unwrap_or(d);
        c.execute(
            "UPDATE items SET due_at=?2, remind_at=?3, notified=0, snoozed_until=NULL, updated_at=?4 WHERE id=?1",
            params![id, it.due_at.map(adv), it.remind_at.map(adv), now],
        )?;
    } else if it.done {
        c.execute(
            "UPDATE items SET done=0, done_at=NULL, updated_at=?2 WHERE id=?1",
            params![id, now],
        )?;
    } else {
        c.execute(
            "UPDATE items SET done=1, done_at=?2, snoozed_until=NULL, updated_at=?2 WHERE id=?1",
            params![id, now],
        )?;
    }
    get_item(c, id)?.ok_or(rusqlite::Error::QueryReturnedNoRows)
}

// ---------- recordatorios ----------

/// Elemento con un aviso vencido.
struct DueRow {
    id: i64,
    recur: String,
    remind_at: Option<i64>,
    due_at: Option<i64>,
    snoozed: bool,
}

/// Devuelve los ids cuyo aviso ha vencido y actualiza su estado para no repetirlo.
pub fn fire_due(c: &Connection, now: i64) -> R<Vec<i64>> {
    let rows: Vec<DueRow> = {
        let mut st = c.prepare(
            "SELECT id, recur, remind_at, due_at, snoozed_until IS NOT NULL FROM items
             WHERE done=0 AND deleted_at IS NULL AND (
               (snoozed_until IS NOT NULL AND snoozed_until <= ?1) OR
               (snoozed_until IS NULL AND notified=0 AND remind_at IS NOT NULL AND remind_at <= ?1))",
        )?;
        let v = st
            .query_map([now], |r| {
                Ok(DueRow {
                    id: r.get(0)?,
                    recur: r.get(1)?,
                    remind_at: r.get(2)?,
                    due_at: r.get(3)?,
                    snoozed: r.get(4)?,
                })
            })?
            .collect::<R<Vec<_>>>()?;
        v
    };
    for DueRow {
        id,
        recur,
        remind_at: remind,
        due_at: due,
        snoozed,
    } in &rows
    {
        if *snoozed {
            // Aviso pospuesto: si se repite, sus fechas ya avanzaron al dispararse la primera vez.
            let notified = if recur == "none" { 1 } else { 0 };
            c.execute(
                "UPDATE items SET snoozed_until=NULL, notified=?2 WHERE id=?1",
                params![id, notified],
            )?;
        } else if recur != "none" {
            let next = remind.and_then(|r| recurrence::next_after_ms(r, recur, now));
            let next_due = due.map(|d| {
                if d <= now {
                    recurrence::next_after_ms(d, recur, now).unwrap_or(d)
                } else {
                    d
                }
            });
            let notified = if next.is_some() { 0 } else { 1 };
            c.execute(
                "UPDATE items SET remind_at=?2, due_at=?3, notified=?4 WHERE id=?1",
                params![id, next, next_due, notified],
            )?;
        } else {
            c.execute("UPDATE items SET notified=1 WHERE id=?1", [id])?;
        }
    }
    Ok(rows.into_iter().map(|r| r.id).collect())
}

pub fn snooze(c: &Connection, id: i64, until: i64) -> R<()> {
    c.execute(
        "UPDATE items SET snoozed_until=?2 WHERE id=?1",
        params![id, until],
    )?;
    Ok(())
}

/// "Hecho" desde el aviso: las tareas que se repiten ya están reprogramadas, solo se completan las únicas.
pub fn complete_from_alert(c: &Connection, id: i64, now: i64) -> R<()> {
    c.execute(
        "UPDATE items SET done=1, done_at=?2, updated_at=?2 WHERE id=?1 AND recur='none' AND kind='task'",
        params![id, now],
    )?;
    Ok(())
}

// ---------- listas, etiquetas, contadores ----------

pub fn list_lists(c: &Connection) -> R<Vec<List>> {
    let mut st = c.prepare(
        "SELECT l.id, l.name, l.color, l.sort,
                (SELECT COUNT(*) FROM items i WHERE i.list_id = l.id AND i.done = 0 AND i.deleted_at IS NULL)
         FROM lists l ORDER BY l.sort, l.id",
    )?;
    let v = st
        .query_map([], |r| {
            Ok(List {
                id: r.get(0)?,
                name: r.get(1)?,
                color: r.get(2)?,
                sort: r.get(3)?,
                count: r.get(4)?,
            })
        })?
        .collect::<R<Vec<_>>>()?;
    Ok(v)
}

pub fn save_list(c: &Connection, id: Option<i64>, name: &str, color: &str) -> R<i64> {
    let name = name.trim();
    match id {
        Some(id) => {
            c.execute(
                "UPDATE lists SET name=?2, color=?3 WHERE id=?1",
                params![id, name, color],
            )?;
            Ok(id)
        }
        None => {
            let sort: i64 =
                c.query_row("SELECT COALESCE(MAX(sort), 0) + 1 FROM lists", [], |r| {
                    r.get(0)
                })?;
            c.execute(
                "INSERT INTO lists(name, color, sort) VALUES(?1, ?2, ?3)",
                params![name, color, sort],
            )?;
            Ok(c.last_insert_rowid())
        }
    }
}

pub fn delete_list(c: &Connection, id: i64) -> R<()> {
    if id == 1 {
        return Ok(()); // la bandeja de entrada no se borra
    }
    c.execute("UPDATE items SET list_id=1 WHERE list_id=?1", [id])?;
    c.execute("DELETE FROM lists WHERE id=?1", [id])?;
    Ok(())
}

pub fn list_tags(c: &Connection) -> R<Vec<TagCount>> {
    let mut st = c.prepare(
        "SELECT t.name, SUM(CASE WHEN i.done = 0 THEN 1 ELSE 0 END)
         FROM tags t JOIN item_tags it ON it.tag_id = t.id
           JOIN items i ON i.id = it.item_id AND i.deleted_at IS NULL
         GROUP BY t.id ORDER BY t.name COLLATE NOCASE",
    )?;
    let v = st
        .query_map([], |r| {
            Ok(TagCount {
                name: r.get(0)?,
                count: r.get(1)?,
            })
        })?
        .collect::<R<Vec<_>>>()?;
    Ok(v)
}

pub fn counts(c: &Connection, end_of_today: i64) -> R<Counts> {
    c.query_row(
        // Mismas condiciones que la vista «today» de `query_items`.
        "SELECT
           SUM(d IS NULL AND kind='task' AND done=0 AND COALESCE(due_at, remind_at) <= ?1),
           SUM(d IS NULL AND kind='note' AND done=0 AND created_at >= ?2),
           SUM(d IS NULL AND kind='task' AND done=0 AND COALESCE(due_at, remind_at) > ?1),
           SUM(d IS NULL AND kind='task' AND done=0),
           SUM(d IS NULL AND kind='note' AND done=0),
           SUM(d IS NULL AND done=1),
           SUM(d IS NOT NULL)
         FROM (SELECT *, deleted_at AS d FROM items)",
        [end_of_today, start_of_day(end_of_today)],
        |r| {
            let n = |i: usize| r.get::<_, Option<i64>>(i).map(|v| v.unwrap_or(0));
            let today_tasks = n(0)?;
            Ok(Counts {
                today: today_tasks + n(1)?,
                today_tasks,
                upcoming: n(2)?,
                tasks: n(3)?,
                notes: n(4)?,
                done: n(5)?,
                trash: n(6)?,
            })
        },
    )
}

// ---------- exportar / importar ----------

pub fn export_all(c: &Connection) -> R<ExportData> {
    let sql = format!("SELECT {ITEM_COLS} FROM items WHERE {ALIVE} ORDER BY id");
    let mut st = c.prepare(&sql)?;
    let mut items = st.query_map([], map_item)?.collect::<R<Vec<Item>>>()?;
    attach_tags(c, &mut items)?;
    Ok(ExportData {
        version: 1,
        lists: list_lists(c)?,
        items,
        settings: all_settings(c)?,
    })
}

/// Añade (no reemplaza) listas y elementos de una exportación. Devuelve cuántos elementos se importaron.
pub fn import_all(c: &Connection, data: &ExportData, now: i64) -> R<usize> {
    let tx = c.unchecked_transaction()?;
    let existing = list_lists(&tx)?;
    let mut map: HashMap<i64, i64> = HashMap::new();
    for l in &data.lists {
        let id = match existing
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(&l.name))
        {
            Some(e) => e.id,
            None => save_list(&tx, None, &l.name, &l.color)?,
        };
        map.insert(l.id, id);
    }
    for it in &data.items {
        let inp = ItemInput {
            id: None,
            kind: it.kind.clone(),
            title: it.title.clone(),
            body: it.body.clone(),
            list_id: Some(*map.get(&it.list_id).unwrap_or(&1)),
            due_at: it.due_at,
            remind_at: it.remind_at,
            recur: it.recur.clone(),
            pinned: it.pinned,
            tags: it.tags.clone(),
        };
        let saved = save_item(&tx, &inp, it.created_at)?;
        tx.execute(
            "UPDATE items SET done=?2, done_at=?3, updated_at=?4,
               notified = CASE WHEN recur='none' AND remind_at IS NOT NULL AND remind_at <= ?5 THEN 1 ELSE 0 END
             WHERE id=?1",
            params![saved.id, it.done, it.done_at, it.updated_at, now],
        )?;
    }
    tx.commit()?;
    Ok(data.items.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn task(title: &str, remind: Option<i64>, recur: &str) -> ItemInput {
        ItemInput {
            id: None,
            kind: "task".into(),
            title: title.into(),
            body: String::new(),
            list_id: None,
            due_at: None,
            remind_at: remind,
            recur: recur.into(),
            pinned: false,
            tags: vec!["Trabajo".into(), "#trabajo".into()],
        }
    }

    #[test]
    fn crud_and_tags() {
        let c = open_in_memory().unwrap();
        let it = save_item(&c, &task("Hola", None, "none"), 1000).unwrap();
        assert_eq!(it.tags, vec!["Trabajo".to_string()]); // sin duplicados
        let mut note = task("Idea", None, "none");
        note.kind = "note".into();
        save_item(&c, &note, 1000).unwrap();
        let f = |view: &str| Filter {
            view: view.into(),
            list_id: None,
            tag: None,
            search: None,
        };
        assert_eq!(query_items(&c, &f("tasks"), 0).unwrap().len(), 1);
        assert_eq!(query_items(&c, &f("notes"), 0).unwrap().len(), 1);
        assert_eq!(query_items(&c, &f("list"), 0).unwrap().len(), 2);
        let ct = counts(&c, 0).unwrap();
        assert_eq!((ct.tasks, ct.notes), (1, 1));
        let found = query_items(
            &c,
            &Filter {
                view: "all".into(),
                list_id: None,
                tag: None,
                search: Some("hol".into()),
            },
            0,
        )
        .unwrap();
        assert_eq!(found.len(), 1);
        let done = toggle_done(&c, it.id, 2000).unwrap();
        assert!(done.done);
        assert_eq!(query_items(&c, &f("done"), 0).unwrap().len(), 1);
        // «Vaciar completadas» mueve a la papelera
        assert_eq!(clear_done(&c, 3000).unwrap(), 1);
        assert!(query_items(&c, &f("done"), 0).unwrap().is_empty());
        assert_eq!(query_items(&c, &f("trash"), 0).unwrap().len(), 1);
    }

    fn f(view: &str) -> Filter {
        Filter {
            view: view.into(),
            list_id: None,
            tag: None,
            search: None,
        }
    }

    #[test]
    fn trash_hides_restores_and_purges() {
        let c = open_in_memory().unwrap();
        let day = DAY_MS;
        let a = save_item(&c, &task("A", Some(5_000), "none"), 1000).unwrap();
        let b = save_item(&c, &task("B", None, "none"), 1000).unwrap();

        delete_item(&c, a.id, 2_000).unwrap();
        // oculto en vistas, búsqueda, contadores, etiquetas y avisos
        assert_eq!(query_items(&c, &f("tasks"), 0).unwrap().len(), 1);
        let mut s = f("tasks");
        s.search = Some("A".into());
        assert!(query_items(&c, &s, 0).unwrap().iter().all(|i| i.id != a.id));
        let ct = counts(&c, i64::MAX).unwrap();
        assert_eq!((ct.tasks, ct.trash), (1, 1));
        assert!(fire_due(&c, 10_000).unwrap().is_empty());
        let trash = query_items(&c, &f("trash"), 0).unwrap();
        assert_eq!(trash.len(), 1);
        assert_eq!(trash[0].deleted_at, Some(2_000));

        // restaurar
        restore_item(&c, a.id).unwrap();
        assert_eq!(query_items(&c, &f("tasks"), 0).unwrap().len(), 2);
        assert_eq!(counts(&c, 0).unwrap().trash, 0);

        // purga: solo lo que lleva más de 10 días
        delete_item(&c, a.id, 1_000_000).unwrap();
        delete_item(&c, b.id, 1_000_000 + 5 * day).unwrap();
        assert_eq!(
            purge_trash(&c, 1_000_000 + TRASH_DAYS * day - 1).unwrap(),
            0
        );
        assert_eq!(purge_trash(&c, 1_000_000 + TRASH_DAYS * day).unwrap(), 1);
        assert!(get_item(&c, a.id).unwrap().is_none());
        assert!(get_item(&c, b.id).unwrap().is_some());

        // vaciar y eliminar para siempre
        assert_eq!(empty_trash(&c).unwrap(), 1);
        assert!(get_item(&c, b.id).unwrap().is_none());
        assert!(list_tags(&c).unwrap().is_empty());
        let x = save_item(&c, &task("X", None, "none"), 1).unwrap();
        delete_forever(&c, x.id).unwrap();
        assert!(get_item(&c, x.id).unwrap().is_none());
    }

    #[test]
    fn today_includes_notes_created_today() {
        let c = open_in_memory().unwrap();
        let eot = end_of_today_ms();
        let sod = start_of_day(eot);
        assert!(sod < eot && eot - sod < DAY_MS + HOUR_MS);
        let mut note = task("Idea de hoy", None, "none");
        note.kind = "note".into();
        let today = save_item(&c, &note, sod + 1_000).unwrap();
        let mut old = note.clone();
        old.title = "Idea de ayer".into();
        save_item(&c, &old, sod - 1_000).unwrap();
        let mut due = task("Tarea de hoy", None, "none");
        due.due_at = Some(sod);
        save_item(&c, &due, 1).unwrap();
        let items = query_items(&c, &f("today"), eot).unwrap();
        let titles: Vec<&str> = items.iter().map(|i| i.title.as_str()).collect();
        assert_eq!(titles, vec!["Tarea de hoy", "Idea de hoy"]); // tareas primero
        assert_eq!(items[1].id, today.id);
        // El contador coincide con lo que muestra la vista.
        let n = counts(&c, eot).unwrap();
        assert_eq!(n.today, items.len() as i64);
        assert_eq!(n.today_tasks, 1);
    }

    #[test]
    fn manual_order_and_new_items_on_top() {
        let c = open_in_memory().unwrap();
        let a = save_item(&c, &task("A", None, "none"), 1).unwrap();
        let b = save_item(&c, &task("B", None, "none"), 2).unwrap();
        let cc = save_item(&c, &task("C", None, "none"), 3).unwrap();
        let ids = |c: &Connection| -> Vec<i64> {
            query_items(c, &f("tasks"), 0)
                .unwrap()
                .iter()
                .map(|i| i.id)
                .collect()
        };
        assert_eq!(ids(&c), vec![cc.id, b.id, a.id]); // por defecto, más recientes arriba
        reorder_items(&c, &[a.id, cc.id, b.id]).unwrap();
        assert_eq!(ids(&c), vec![a.id, cc.id, b.id]);
        let d = save_item(&c, &task("D", None, "none"), 4).unwrap();
        assert_eq!(ids(&c), vec![d.id, a.id, cc.id, b.id]); // lo nuevo, arriba
    }

    #[test]
    fn migration_v1_to_v2_keeps_data() {
        let c = Connection::open_in_memory().unwrap();
        c.execute_batch(SCHEMA_V1).unwrap();
        c.pragma_update(None, "user_version", 1).unwrap();
        c.execute(
            "INSERT INTO items(kind, title, created_at, updated_at) VALUES('task', 'Vieja', 1, 1)",
            [],
        )
        .unwrap();
        init(&c).unwrap();
        let v: i64 = c
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v, 3);
        let items = query_items(&c, &f("tasks"), 0).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].title, "Vieja");
        assert_eq!(items[0].deleted_at, None);
    }

    #[test]
    fn one_shot_reminder_fires_once_and_snoozes() {
        let c = open_in_memory().unwrap();
        let it = save_item(&c, &task("Llamar", Some(5_000), "none"), 1000).unwrap();
        assert!(fire_due(&c, 4_000).unwrap().is_empty());
        assert_eq!(fire_due(&c, 6_000).unwrap(), vec![it.id]);
        assert!(fire_due(&c, 7_000).unwrap().is_empty());
        snooze(&c, it.id, 10_000).unwrap();
        assert!(fire_due(&c, 9_000).unwrap().is_empty());
        assert_eq!(fire_due(&c, 10_000).unwrap(), vec![it.id]);
        assert!(fire_due(&c, 20_000).unwrap().is_empty());
    }

    #[test]
    fn repeating_reminder_advances() {
        let c = open_in_memory().unwrap();
        let day = DAY_MS;
        let start = 1_800_000_000_000; // ~2027
        let it = save_item(&c, &task("Pastilla", Some(start), "daily"), 1000).unwrap();
        assert_eq!(fire_due(&c, start + 1).unwrap(), vec![it.id]);
        let after = get_item(&c, it.id).unwrap().unwrap();
        let next = after.remind_at.unwrap();
        assert!(next > start && next <= start + day + HOUR_MS);
        assert!(fire_due(&c, start + 2).unwrap().is_empty());
    }

    #[test]
    fn export_import_roundtrip() {
        let c = open_in_memory().unwrap();
        save_list(&c, None, "Casa", "#ff0000").unwrap();
        save_item(&c, &task("A", None, "none"), 1).unwrap();
        let data = export_all(&c).unwrap();
        let c2 = open_in_memory().unwrap();
        assert_eq!(import_all(&c2, &data, 10).unwrap(), 1);
        assert_eq!(list_lists(&c2).unwrap().len(), 2);
    }
}
