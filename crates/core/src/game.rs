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
