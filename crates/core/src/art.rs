//! Finding cover art for games that didn't come with any (Epic, manual
//! adds) by matching the title against the Steam store.
//!
//! Uses the system `curl` instead of an HTTP stack: it ships with Windows 10+,
//! macOS and nearly every Linux, and keeps Saber's binary small.

use crate::game::Game;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

const SEARCH: &str = "https://store.steampowered.com/api/storesearch/?l=english&cc=US&term=";
const CDN: &str = "https://shared.akamai.steamstatic.com/store_item_assets/steam/apps";

#[derive(Deserialize)]
struct SearchResults {
    #[serde(default)]
    items: Vec<SearchItem>,
}

#[derive(Deserialize)]
struct SearchItem {
    id: u32,
    name: String,
}

/// Lowercase letters and digits only, so `DEATHLOOP™` matches `Deathloop`.
fn normalize(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Picks the result that is the same game, not a sequel, soundtrack or DLC.
fn best_match(title: &str, items: &[SearchItem]) -> Option<u32> {
    let want = normalize(title);
    if want.is_empty() {
        return None;
    }
    items
        .iter()
        .find(|i| normalize(&i.name) == want)
        .or_else(|| {
            // "Grand Theft Auto V Enhanced" → "Grand Theft Auto V": accept a
            // result whose name is a prefix of ours, longest first.
            items
                .iter()
                .filter(|i| {
                    let n = normalize(&i.name);
                    n.len() >= 4 && want.starts_with(&n)
                })
                .max_by_key(|i| i.name.len())
        })
        .map(|i| i.id)
}

fn curl() -> Command {
    let mut cmd = Command::new("curl");
    cmd.args(["-fsSL", "--max-time", "15"]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000); // CREATE_NO_WINDOW
    }
    cmd
}

fn encode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            b' ' => "+".into(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// Looks a title up on the Steam store.
pub fn find_steam_app(title: &str) -> Option<u32> {
    let out = curl()
        .arg(format!("{SEARCH}{}", encode(title)))
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let results: SearchResults = serde_json::from_slice(&out.stdout).ok()?;
    best_match(title, &results.items)
}

fn download(url: &str, dest: &Path) -> bool {
    let tmp = dest.with_extension("part");
    let ok = curl()
        .arg("-o")
        .arg(&tmp)
        .arg(url)
        .status()
        .is_ok_and(|s| s.success());
    ok && std::fs::rename(&tmp, dest).is_ok()
}

/// Downloads cover and hero art for a game into `dir`. Returns what it got.
pub fn fetch(game: &Game, dir: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
    let app_id = match &game.source {
        crate::game::Source::Steam { app_id } => Some(*app_id),
        _ => find_steam_app(&game.title),
    };
    let Some(app_id) = app_id else {
        return (None, None);
    };
    if std::fs::create_dir_all(dir).is_err() {
        return (None, None);
    }
    let get = |file: &str, suffix: &str| {
        let dest = dir.join(format!("{}-{suffix}.jpg", game.id));
        download(&format!("{CDN}/{app_id}/{file}"), &dest).then_some(dest)
    };
    let cover = if game.cover.is_none() {
        get("library_600x900.jpg", "cover")
    } else {
        None
    };
    let hero = if game.hero.is_none() {
        get("library_hero.jpg", "hero").or_else(|| get("header.jpg", "hero"))
    } else {
        None
    };
    (cover, hero)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items(names: &[(&str, u32)]) -> Vec<SearchItem> {
        names
            .iter()
            .map(|(n, id)| SearchItem {
                name: n.to_string(),
                id: *id,
            })
            .collect()
    }

    #[test]
    fn exact_match_ignores_case_and_symbols() {
        let r = items(&[("DEATHLOOP 4 YOU", 2), ("DEATHLOOP™", 1)]);
        assert_eq!(best_match("Deathloop", &r), Some(1));
    }

    #[test]
    fn falls_back_to_longest_prefix() {
        let r = items(&[("Grand Theft Auto", 1), ("Grand Theft Auto V", 2)]);
        assert_eq!(best_match("Grand Theft Auto V Enhanced", &r), Some(2));
    }

    #[test]
    fn rejects_unrelated_results() {
        let r = items(&[("Fall Guys Soundtrack", 1)]);
        assert_eq!(best_match("Hollow Tides", &r), None);
    }

    #[test]
    fn encodes_query() {
        assert_eq!(encode("Dead Island 2: Gold"), "Dead+Island+2%3A+Gold");
    }
}
