//! Discovering installed games from other launchers.

pub mod epic;
pub mod steam;
pub mod vdf;

use crate::game::Game;

/// Runs every scanner. Blocking; call it off the UI thread.
pub fn all() -> Vec<Game> {
    let mut games = steam::scan();
    games.extend(epic::scan());
    games
}
