//! SQLite-backed persistence for UI state (selected site, product, tilt).
//! The database lives at `~/.rustywx/state.db` alongside the border cache.

use anyhow::{Result, anyhow};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

/// The keys the app saves and restores.
pub struct AppState {
    pub site_id: String,
    pub product: String, // "Reflectivity" or "Velocity"
    pub tilt_index: usize,
}

/// Path to the SQLite database, under a given home directory.
pub fn db_path_under(home: &Path) -> PathBuf {
    home.join(".rustywx").join("state.db")
}

/// Path to the SQLite database on this machine.
fn db_path() -> Result<PathBuf> {
    let home =
        std::env::var("HOME").map_err(|_| anyhow!("HOME environment variable is not set"))?;
    Ok(db_path_under(Path::new(&home)))
}

/// Open (or create) the database at `path`, ensuring the schema exists.
fn open_db_at(path: &Path) -> Result<Connection> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow!("creating {}: {e}", parent.display()))?;
    }
    let conn = Connection::open(path)
        .map_err(|e| anyhow!("opening {}: {e}", path.display()))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_state (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );",
    )
    .map_err(|e| anyhow!("creating schema: {e}"))?;
    Ok(conn)
}

/// Load saved state from the database. Returns `Ok(None)` when no state has
/// been saved yet (fresh install).
pub fn load_state() -> Result<Option<AppState>> {
    load_state_at(&db_path()?)
}

/// Load state from a specific database path (for testing).
pub fn load_state_at(path: &Path) -> Result<Option<AppState>> {
    let conn = open_db_at(path)?;
    let mut stmt = conn
        .prepare("SELECT key, value FROM app_state")
        .map_err(|e| anyhow!("preparing select: {e}"))?;
    let rows: Vec<(String, String)> = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| anyhow!("querying state: {e}"))?
        .filter_map(|r| r.ok())
        .collect();

    if rows.is_empty() {
        return Ok(None);
    }

    let mut site_id = String::new();
    let mut product = String::from("Reflectivity");
    let mut tilt_index = 0usize;

    for (key, value) in &rows {
        match key.as_str() {
            "site_id" => site_id = value.clone(),
            "product" => product = value.clone(),
            "tilt_index" => {
                tilt_index = value.parse().unwrap_or(0);
            }
            _ => {}
        }
    }

    if site_id.is_empty() {
        return Ok(None);
    }

    Ok(Some(AppState {
        site_id,
        product,
        tilt_index,
    }))
}

/// Persist the current UI state to the database (upsert each key).
pub fn save_state(state: &AppState) -> Result<()> {
    save_state_at(&db_path()?, state)
}

/// Save state to a specific database path (for testing).
pub fn save_state_at(path: &Path, state: &AppState) -> Result<()> {
    let conn = open_db_at(path)?;
    let mut stmt = conn
        .prepare("INSERT OR REPLACE INTO app_state (key, value) VALUES (?1, ?2)")
        .map_err(|e| anyhow!("preparing upsert: {e}"))?;
    stmt.execute(["site_id", &state.site_id])
        .map_err(|e| anyhow!("saving site_id: {e}"))?;
    stmt.execute(["product", &state.product])
        .map_err(|e| anyhow!("saving product: {e}"))?;
    stmt.execute(["tilt_index", &state.tilt_index.to_string()])
        .map_err(|e| anyhow!("saving tilt_index: {e}"))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_db_path(label: &str) -> (PathBuf, PathBuf) {
        let dir = std::env::temp_dir().join(format!("rustywx-test-{}-{}", std::process::id(), label));
        let _ = std::fs::create_dir_all(&dir);
        let db = dir.join("state.db");
        (db, dir)
    }

    #[test]
    fn save_and_load_round_trips() {
        let (db, dir) = temp_db_path("roundtrip");
        let state = AppState {
            site_id: "KFFC".to_string(),
            product: "Velocity".to_string(),
            tilt_index: 2,
        };
        save_state_at(&db, &state).unwrap();
        let loaded = load_state_at(&db).unwrap().expect("state should exist");
        assert_eq!(loaded.site_id, "KFFC");
        assert_eq!(loaded.product, "Velocity");
        assert_eq!(loaded.tilt_index, 2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn fresh_install_returns_none() {
        let (db, dir) = temp_db_path("fresh");
        // Don't create the db — simulate fresh install.
        let result = load_state_at(&db).unwrap();
        assert!(result.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn db_path_is_under_dot_rustywx() {
        let path = db_path_under(std::path::Path::new("/home/example"));
        assert_eq!(
            path,
            std::path::Path::new("/home/example/.rustywx/state.db")
        );
    }
}
