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
}
