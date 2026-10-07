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

/// Every `steamapps` directory Steam knows about, the main one first.
pub fn library_folders(root: &Path) -> Vec<PathBuf> {
    let mut folders = vec![root.join("steamapps")];
    let file = root.join("steamapps").join("libraryfolders.vdf");
    if let Ok(text) = std::fs::read_to_string(&file) {
        let doc = vdf::parse(&text);
        if let Some(lib) = doc.get("libraryfolders") {
            for (_, entry) in lib.entries() {
                if let Some(path) = entry.str("path") {
                    let apps = PathBuf::from(path).join("steamapps");
                    if !folders.iter().any(|f| same_dir(f, &apps)) {
                        folders.push(apps);
                    }
                }
            }
        }
    }
    folders.retain(|f| f.is_dir());
    folders
}

fn same_dir(a: &Path, b: &Path) -> bool {
    match (a.canonicalize(), b.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => a.to_string_lossy().eq_ignore_ascii_case(&b.to_string_lossy()),
    }
}

/// An installed app read from `appmanifest_<id>.acf`.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub app_id: u32,
    pub name: String,
    pub install_dir: PathBuf,
}

pub fn read_manifest(path: &Path) -> Option<Manifest> {
    let text = std::fs::read_to_string(path).ok()?;
    parse_manifest(&text, path.parent()?)
}

fn parse_manifest(text: &str, steamapps: &Path) -> Option<Manifest> {
    let doc = vdf::parse(text);
    let app = doc.get("AppState")?;
    let app_id = app.str("appid")?.parse().ok()?;
    let name = app.str("name")?.trim().to_string();
    let install_dir = steamapps.join("common").join(app.str("installdir")?);
    if name.is_empty() || is_tool(app_id, &name) {
        return None;
    }
    Some(Manifest { app_id, name, install_dir })
}
