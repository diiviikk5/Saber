//! Where Saber keeps its files, and how they are written safely.

use serde::{Serialize, de::DeserializeOwned};
use std::io;
use std::path::{Path, PathBuf};

/// `%APPDATA%\Saber`, `~/Library/Application Support/Saber` or
/// `$XDG_CONFIG_HOME/saber`. Override with `SABER_HOME` for testing.
pub fn data_dir() -> PathBuf {
    if let Some(home) = std::env::var_os("SABER_HOME") {
        return PathBuf::from(home);
    }
    let base = if cfg!(windows) {
        std::env::var_os("APPDATA").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
    };
    let name = if cfg!(any(windows, target_os = "macos")) { "Saber" } else { "saber" };
    base.unwrap_or_else(|| PathBuf::from(".")).join(name)
}

pub fn library_path() -> PathBuf {
    data_dir().join("library.json")
}

pub fn settings_path() -> PathBuf {
    data_dir().join("settings.json")
}

/// Folder for art the user picks, copied in so it survives the original moving.
pub fn art_dir() -> PathBuf {
    data_dir().join("art")
}

/// Reads JSON, treating a missing file as the default value.
pub fn load<T: DeserializeOwned + Default>(path: &Path) -> io::Result<T> {
    match std::fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(T::default()),
        Err(e) => Err(e),
    }
}

/// Writes JSON atomically: to a sibling temp file first, then renamed over
/// the target, so a crash mid-write never leaves a truncated library.
pub fn save<T: Serialize>(path: &Path, value: &T) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let json = serde_json::to_vec_pretty(value)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path)
}
