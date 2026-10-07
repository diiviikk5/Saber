//! Steam: library folders → app manifests → games, with Steam's own cached art.

use super::vdf;
use crate::game::{Game, Launch, Source};
use std::path::{Path, PathBuf};

/// Tools and runtimes Steam installs alongside games.
const NOT_GAMES: [u32; 6] = [
    228980,  // Steamworks Common Redistributables
    1070560, // Steam Linux Runtime
    1391110, // Steam Linux Runtime - Soldier
    1628350, // Steam Linux Runtime - Sniper
    1493710, // Proton Experimental
    1826330, // Proton EasyAntiCheat Runtime
];

fn is_tool(app_id: u32, name: &str) -> bool {
    NOT_GAMES.contains(&app_id)
        || name.starts_with("Proton ")
        || name.starts_with("Steam Linux Runtime")
        || name.starts_with("Steamworks")
        || name.contains("Dedicated Server")
}

/// Where Steam itself is installed.
pub fn steam_root() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("SABER_STEAM_ROOT") {
        return Some(p.into());
    }
    candidates().into_iter().find(|p| p.join("steamapps").is_dir())
}

#[cfg(windows)]
fn candidates() -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Some(path) = registry_steam_path() {
        out.push(path);
    }
    for var in ["ProgramFiles(x86)", "ProgramFiles"] {
        if let Some(dir) = std::env::var_os(var) {
            out.push(PathBuf::from(dir).join("Steam"));
        }
    }
    out
}

/// Asks `reg.exe` rather than pulling in a registry crate.
#[cfg(windows)]
fn registry_steam_path() -> Option<PathBuf> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let out = std::process::Command::new("reg")
        .args(["query", r"HKCU\Software\Valve\Steam", "/v", "SteamPath"])
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    let line = text.lines().find(|l| l.contains("SteamPath"))?;
    let value = line.split("REG_SZ").nth(1)?.trim();
    Some(PathBuf::from(value.replace('/', "\\")))
}

#[cfg(target_os = "macos")]
fn candidates() -> Vec<PathBuf> {
    std::env::var_os("HOME")
        .map(|h| vec![PathBuf::from(h).join("Library/Application Support/Steam")])
        .unwrap_or_default()
}

#[cfg(all(unix, not(target_os = "macos")))]
fn candidates() -> Vec<PathBuf> {
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    vec![
        home.join(".steam/steam"),
        home.join(".local/share/Steam"),
        home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
    ]
}
