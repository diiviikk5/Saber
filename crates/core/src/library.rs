//! The collection of games and the queries the UI runs against it.

use crate::game::{Game, Source};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Library {
    #[serde(default)]
    pub games: Vec<Game>,
}

impl Library {
    pub fn get(&self, id: &str) -> Option<&Game> {
        self.games.iter().find(|g| g.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Game> {
        self.games.iter_mut().find(|g| g.id == id)
    }

    pub fn len(&self) -> usize {
        self.games.len()
    }

    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }

    pub fn remove(&mut self, id: &str) -> Option<Game> {
        let index = self.games.iter().position(|g| g.id == id)?;
        Some(self.games.remove(index))
    }

    /// Adds a game, or refreshes an existing import in place.
    ///
    /// Scanners call this repeatedly. Store-provided fields (title, launch
    /// target, art) are updated; everything the user owns — favorites, tags,
    /// hidden state and play stats — is left alone. Returns `true` when the
    /// game was new.
    pub fn upsert(&mut self, incoming: Game) -> bool {
        match self.get_mut(&incoming.id) {
            Some(existing) => {
                existing.title = incoming.title;
                existing.launch = incoming.launch;
                if incoming.cover.is_some() {
                    existing.cover = incoming.cover;
                }
                if incoming.hero.is_some() {
                    existing.hero = incoming.hero;
                }
                false
            }
            None => {
                self.games.push(incoming);
                true
            }
        }
    }

    /// Records a finished play session.
    pub fn record_session(&mut self, id: &str, started_at: u64, secs: u64) {
        if let Some(game) = self.get_mut(id) {
            game.playtime_secs += secs;
            game.last_played = Some(started_at);
        }
    }

    /// Records a launch we can't time (e.g. handed off to Steam).
    pub fn record_launch(&mut self, id: &str, at: u64) {
        if let Some(game) = self.get_mut(id) {
            game.launch_count += 1;
            game.last_played = Some(at);
        }
    }
}
