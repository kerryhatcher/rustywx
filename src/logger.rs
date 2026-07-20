//! Bare-bones file logger: appends timestamped messages to
//! `logs/rustywx.log`. Used for debugging data-fetch pipelines.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static LOGGER: Mutex<Option<File>> = Mutex::new(None);

fn log_path() -> PathBuf {
    PathBuf::from("logs").join("rustywx.log")
}

fn ensure_open() {
    let mut guard = LOGGER.lock().unwrap();
    if guard.is_none() {
        let path = log_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        *guard = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .ok();
    }
}

/// Append a line to the log file with a UTC timestamp.
pub fn log(msg: &str) {
    ensure_open();
    let mut guard = LOGGER.lock().unwrap();
    if let Some(ref mut file) = *guard {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let _ = writeln!(file, "[{ts}] {msg}");
        let _ = file.flush();
    }
}
