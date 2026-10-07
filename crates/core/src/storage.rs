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

/// Like [`load`], but a corrupt file is moved aside to `<name>.broken`
/// instead of being silently overwritten by the next save.
pub fn load_or_recover<T: DeserializeOwned + Default>(path: &Path) -> T {
    match load(path) {
        Ok(value) => value,
        Err(err) => {
            eprintln!("saber: could not read {}: {err}", path.display());
            let _ = std::fs::rename(path, path.with_extension("json.broken"));
            T::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Library;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("saber-test-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        dir.join("library.json")
    }

    #[test]
    fn missing_file_is_default() {
        let lib: Library = load(&temp("missing")).unwrap();
        assert!(lib.is_empty());
    }

    #[test]
    fn round_trips() {
        let path = temp("roundtrip");
        let mut lib = Library::default();
        lib.upsert(crate::game::Game::new(
            "Neon Drift",
            crate::game::Source::Manual,
            crate::game::Launch::Uri { uri: "x".into() },
        ));
        save(&path, &lib).unwrap();
        let back: Library = load(&path).unwrap();
        assert_eq!(back.games, lib.games);
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    fn corrupt_file_is_set_aside() {
        let path = temp("corrupt");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, b"{ not json").unwrap();
        let lib: Library = load_or_recover(&path);
        assert!(lib.is_empty());
        assert!(path.with_extension("json.broken").exists());
    }
}
