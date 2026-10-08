//! A single entry on the shelf.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Where a game came from. Imported games remember their store id so a
/// rescan can update them instead of adding duplicates.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Source {
    Manual,
    Steam { app_id: u32 },
    Epic { app_name: String },
}

impl Source {
    pub fn label(&self) -> &'static str {
        match self {
            Source::Manual => "Manual",
            Source::Steam { .. } => "Steam",
            Source::Epic { .. } => "Epic",
        }
    }
}

/// How to start the game.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Launch {
    /// Run an executable directly. Saber can time these sessions.
    Exe {
        path: PathBuf,
        #[serde(default)]
        args: Vec<String>,
        #[serde(default)]
        working_dir: Option<PathBuf>,
    },
    /// Hand a URI to the OS, e.g. `steam://rungameid/570`.
    Uri { uri: String },
}

/// A game in the library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub source: Source,
    pub launch: Launch,
    /// Portrait cover (2:3). Falls back to a generated poster when missing.
    #[serde(default)]
    pub cover: Option<PathBuf>,
    /// Wide hero art shown in the featured banner.
    #[serde(default)]
    pub hero: Option<PathBuf>,
    /// Transparent title logo, drawn over hero art instead of plain text.
    #[serde(default)]
    pub logo: Option<PathBuf>,
    /// Where the game's files live, for "open folder".
    #[serde(default)]
    pub install_dir: Option<PathBuf>,
    /// Set once Saber has asked the store for missing art, hit or miss, so
    /// it doesn't ask again on every start.
    #[serde(default)]
    pub art_checked: bool,
    #[serde(default)]
    pub favorite: bool,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub playtime_secs: u64,
    #[serde(default)]
    pub launch_count: u32,
    #[serde(default)]
    pub last_played: Option<u64>,
    pub added_at: u64,
}

impl Game {
    pub fn new(title: impl Into<String>, source: Source, launch: Launch) -> Self {
        let title = title.into();
        let added_at = crate::format::now();
        Self {
            id: make_id(&title, &source, added_at),
            title,
            source,
            launch,
            cover: None,
            hero: None,
            logo: None,
            install_dir: None,
            art_checked: false,
            favorite: false,
            hidden: false,
            tags: Vec::new(),
            playtime_secs: 0,
            launch_count: 0,
            last_played: None,
            added_at,
        }
    }
}

/// Stable ids for imported games, unique-enough ids for manual ones.
fn make_id(title: &str, source: &Source, added_at: u64) -> String {
    match source {
        Source::Steam { app_id } => format!("steam-{app_id}"),
        Source::Epic { app_name } => format!("epic-{app_name}"),
        Source::Manual => format!("manual-{:x}-{:x}", added_at, hash(title)),
    }
}

/// FNV-1a. Tiny, deterministic, good enough for ids and poster colors.
pub fn hash(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

impl Game {
    /// Two hues (0..360) picked from the title, used to paint a poster for
    /// games without cover art. The same title always gets the same poster.
    pub fn poster_hues(&self) -> (f32, f32) {
        let h = hash(&self.title);
        let a = (h % 360) as f32;
        let b = (a + 25. + ((h >> 16) % 50) as f32) % 360.;
        (a, b)
    }

    /// Up to two letters for the generated poster: `Hollow Tides` → `HT`.
    pub fn initials(&self) -> String {
        self.title
            .split_whitespace()
            .filter_map(|w| w.chars().find(|c| c.is_alphanumeric()))
            .take(2)
            .flat_map(char::to_uppercase)
            .collect()
    }
}
