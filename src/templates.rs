//! Standard-Templates (Benutzer-Voreinstellungen) als JSON-Dateien im
//! Konfigurationsordner des Benutzers:
//! - macOS:   ~/Library/Application Support/radar-arpa-trainer/
//! - Windows: %APPDATA%\radar-arpa-trainer\
//! - Linux:   $XDG_CONFIG_HOME bzw. ~/.config/radar-arpa-trainer/

use std::path::PathBuf;

use serde::{de::DeserializeOwned, Serialize};

pub fn dir() -> PathBuf {
    if let Some(d) = std::env::var_os("RADAR_CONFIG_DIR") {
        return PathBuf::from(d);
    }
    let base: PathBuf = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME")
            .map(|h| PathBuf::from(h).join("Library").join("Application Support"))
            .unwrap_or_else(|| PathBuf::from("."))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."))
    };
    base.join("radar-arpa-trainer")
}

fn path(name: &str) -> PathBuf {
    dir().join(format!("{name}.json"))
}

pub fn load<T: DeserializeOwned>(name: &str) -> Option<T> {
    let text = std::fs::read_to_string(path(name)).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save<T: Serialize>(name: &str, value: &T) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir()).map_err(|e| e.to_string())?;
    let p = path(name);
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(&p, text).map_err(|e| e.to_string())?;
    Ok(p)
}

/// Template löschen (= zurück auf Werkseinstellung). Fehlt die Datei, ist das kein Fehler.
pub fn remove(name: &str) -> Result<(), String> {
    match std::fs::remove_file(path(name)) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.to_string()),
    }
}
