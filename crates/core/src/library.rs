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

/// Which slice of the library the sidebar is showing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    All,
    Favorites,
    Recent,
    Source(&'static str),
    Hidden,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
    #[default]
    Recent,
    Title,
    Playtime,
    Added,
}

impl Sort {
    pub const ALL: [Sort; 4] = [Sort::Recent, Sort::Title, Sort::Playtime, Sort::Added];

    pub fn label(self) -> &'static str {
        match self {
            Sort::Recent => "Recently played",
            Sort::Title => "Title",
            Sort::Playtime => "Most played",
            Sort::Added => "Recently added",
        }
    }
}

impl View {
    fn admits(&self, game: &Game) -> bool {
        match self {
            View::Hidden => game.hidden,
            _ if game.hidden => false,
            View::All => true,
            View::Favorites => game.favorite,
            View::Recent => game.last_played.is_some(),
            View::Source(label) => game.source.label() == *label,
        }
    }
}

/// Case-insensitive search: every word of the query must appear in the
/// title, a tag or the source name.
pub fn matches(game: &Game, query: &str) -> bool {
    let query = query.trim();
    if query.is_empty() {
        return true;
    }
    let haystack = format!(
        "{} {} {}",
        game.title,
        game.tags.join(" "),
        game.source.label()
    )
    .to_lowercase();
    query
        .to_lowercase()
        .split_whitespace()
        .all(|term| haystack.contains(term))
}

impl Library {
    /// The games to show for a view, search query and sort order.
    pub fn query(&self, view: &View, search: &str, sort: Sort) -> Vec<&Game> {
        let mut games: Vec<&Game> = self
            .games
            .iter()
            .filter(|g| view.admits(g) && matches(g, search))
            .collect();
        let sort = if *view == View::Recent { Sort::Recent } else { sort };
        match sort {
            Sort::Recent => games.sort_by(|a, b| {
                b.last_played
                    .cmp(&a.last_played)
                    .then_with(|| a.title.to_lowercase().cmp(&b.title.to_lowercase()))
            }),
            Sort::Title => games.sort_by_key(|g| g.title.to_lowercase()),
            Sort::Playtime => games.sort_by(|a, b| b.playtime_secs.cmp(&a.playtime_secs)),
            Sort::Added => games.sort_by(|a, b| b.added_at.cmp(&a.added_at)),
        }
        games
    }

    /// The game to feature in the hero banner: the most recently played,
    /// otherwise the newest addition.
    pub fn featured(&self) -> Option<&Game> {
        let visible = self.games.iter().filter(|g| !g.hidden);
        visible
            .clone()
            .filter(|g| g.last_played.is_some())
            .max_by_key(|g| g.last_played)
            .or_else(|| visible.max_by_key(|g| g.added_at))
    }

    /// Count of visible games per source label, for the sidebar.
    pub fn count(&self, view: &View) -> usize {
        self.games.iter().filter(|g| view.admits(g)).count()
    }
}
