//! Adding games by hand: from an executable the user picked.

use crate::game::{Game, Launch, Source};
use std::path::{Path, PathBuf};

/// Guesses a readable title from an executable name:
/// `HollowTides_x64.exe` → `Hollow Tides`.
pub fn title_from_path(path: &Path) -> String {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();

    const NOISE: [&str; 8] = ["win64", "win32", "x64", "x86", "shipping", "launcher", "dx11", "dx12"];
    let mut words: Vec<String> = Vec::new();
    for chunk in stem.split(['_', '-', '.', ' ']) {
        if chunk.is_empty() || NOISE.contains(&chunk.to_ascii_lowercase().as_str()) {
            continue;
        }
        words.extend(split_camel(chunk));
    }
    if words.is_empty() {
        return stem;
    }
    words
        .into_iter()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                Some(f) => f.to_uppercase().chain(c).collect(),
                None => w,
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// `HollowTides` → [`Hollow`, `Tides`]; acronyms like `GTA` stay together.
fn split_camel(s: &str) -> Vec<String> {
    let chars: Vec<char> = s.chars().collect();
    let mut words = Vec::new();
    let mut start = 0;
    for i in 1..chars.len() {
        let prev = chars[i - 1];
        let cur = chars[i];
        let next_lower = chars.get(i + 1).is_some_and(|c| c.is_lowercase());
        let boundary = (prev.is_lowercase() && cur.is_uppercase())
            || (prev.is_uppercase() && cur.is_uppercase() && next_lower)
            || (prev.is_alphabetic() != cur.is_alphabetic() && !(prev.is_numeric() && cur.is_numeric()));
        if boundary {
            words.push(chars[start..i].iter().collect());
            start = i;
        }
    }
    words.push(chars[start..].iter().collect());
    words
}

/// Builds a manual library entry for an executable.
pub fn game_from_exe(path: PathBuf) -> Game {
    let mut game = Game::new(
        title_from_path(&path),
        Source::Manual,
        Launch::Exe {
            path: path.clone(),
            args: Vec::new(),
            working_dir: None,
        },
    );
    game.install_dir = path.parent().map(Path::to_path_buf);
    game
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(p: &str) -> String {
        title_from_path(Path::new(p))
    }

    #[test]
    fn tidies_executable_names() {
        assert_eq!(t("C:/Games/HollowTides_x64.exe"), "Hollow Tides");
        assert_eq!(t("neon-drift.exe"), "Neon Drift");
        assert_eq!(t("GTA5.exe"), "GTA 5");
        assert_eq!(t("Starfall-Win64-Shipping.exe"), "Starfall");
        assert_eq!(t("XMLParserGame.exe"), "XML Parser Game");
    }

    #[test]
    fn falls_back_to_the_stem() {
        assert_eq!(t("x64.exe"), "x64");
    }
}
