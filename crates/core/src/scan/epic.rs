//! Epic Games Launcher: one JSON `.item` manifest per installed app.

use crate::game::{Game, Launch, Source};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Item {
    display_name: String,
    app_name: String,
    #[serde(default)]
    catalog_namespace: String,
    #[serde(default)]
    catalog_item_id: String,
    #[serde(default)]
    main_game_app_name: String,
    #[serde(default)]
    install_location: String,
    #[serde(default)]
    app_categories: Vec<String>,
    #[serde(default, rename = "bIsIncompleteInstall")]
    incomplete: bool,
}

pub fn manifests_dir() -> Option<PathBuf> {
    if let Some(p) = std::env::var_os("SABER_EPIC_MANIFESTS") {
        return Some(p.into());
    }
    let base = if cfg!(windows) {
        std::env::var_os("ProgramData").map(PathBuf::from)
    } else if cfg!(target_os = "macos") {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join("Library/Application Support"))
    } else {
        None
    }?;
    let dir = base.join("Epic/EpicGamesLauncher/Data/Manifests");
    dir.is_dir().then_some(dir)
}

fn parse_item(text: &str) -> Option<Game> {
    let item: Item = serde_json::from_str(text).ok()?;
    let is_game = item.app_categories.iter().any(|c| c == "games");
    // DLC manifests point at their parent via MainGameAppName.
    let is_addon = !item.main_game_app_name.is_empty() && item.main_game_app_name != item.app_name;
    if !is_game || is_addon || item.incomplete || item.display_name.trim().is_empty() {
        return None;
    }
    let uri = format!(
        "com.epicgames.launcher://apps/{}%3A{}%3A{}?action=launch&silent=true",
        item.catalog_namespace, item.catalog_item_id, item.app_name
    );
    let mut game = Game::new(
        item.display_name.trim(),
        Source::Epic { app_name: item.app_name },
        Launch::Uri { uri },
    );
    if !item.install_location.is_empty() {
        game.tags = Vec::new();
    }
    Some(game)
}

/// Every installed Epic game on this machine.
pub fn scan() -> Vec<Game> {
    let Some(dir) = manifests_dir() else { return Vec::new() };
    scan_dir(&dir)
}

fn scan_dir(dir: &Path) -> Vec<Game> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let mut games: Vec<Game> = entries
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "item"))
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .filter_map(|t| parse_item(&t))
        .collect();
    games.sort_by(|a, b| a.title.cmp(&b.title));
    games
}
