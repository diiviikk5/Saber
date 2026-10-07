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
                if incoming.install_dir.is_some() {
                    existing.install_dir = incoming.install_dir;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{Launch, Source};

    fn game(title: &str, source: Source) -> Game {
        let mut g = Game::new(
            title,
            source,
            Launch::Uri {
                uri: "steam://rungameid/1".into(),
            },
        );
        g.added_at = 100;
        g
    }

    fn sample() -> Library {
        let mut lib = Library::default();
        let mut a = game("Hollow Tides", Source::Steam { app_id: 1 });
        a.last_played = Some(500);
        a.playtime_secs = 10;
        let mut b = game("ashen crown", Source::Epic { app_name: "ash".into() });
        b.favorite = true;
        b.playtime_secs = 99;
        b.tags = vec!["souls".into()];
        let mut c = game("Neon Drift", Source::Manual);
        c.last_played = Some(900);
        let mut d = game("Old Thing", Source::Manual);
        d.hidden = true;
        lib.games = vec![a, b, c, d];
        lib
    }

    fn titles(games: Vec<&Game>) -> Vec<&str> {
        games.into_iter().map(|g| g.title.as_str()).collect()
    }

    #[test]
    fn views_filter_and_hide() {
        let lib = sample();
        assert_eq!(lib.count(&View::All), 3);
        assert_eq!(lib.count(&View::Favorites), 1);
        assert_eq!(lib.count(&View::Source("Manual")), 1);
        assert_eq!(lib.count(&View::Hidden), 1);
        assert_eq!(
            titles(lib.query(&View::Recent, "", Sort::Title)),
            ["Neon Drift", "Hollow Tides"]
        );
    }

    #[test]
    fn sorting() {
        let lib = sample();
        assert_eq!(
            titles(lib.query(&View::All, "", Sort::Title)),
            ["ashen crown", "Hollow Tides", "Neon Drift"]
        );
        assert_eq!(
            titles(lib.query(&View::All, "", Sort::Playtime))[0],
            "ashen crown"
        );
    }

    #[test]
    fn search_terms_match_title_tags_and_source() {
        let lib = sample();
        assert_eq!(titles(lib.query(&View::All, "SOULS", Sort::Title)), ["ashen crown"]);
        assert_eq!(titles(lib.query(&View::All, "neon manual", Sort::Title)), ["Neon Drift"]);
        assert!(lib.query(&View::All, "nothing here", Sort::Title).is_empty());
    }

    #[test]
    fn featured_prefers_last_played() {
        assert_eq!(sample().featured().unwrap().title, "Neon Drift");
    }

    #[test]
    fn upsert_keeps_user_data() {
        let mut lib = sample();
        let mut fresh = game("Hollow Tides: Remastered", Source::Steam { app_id: 1 });
        fresh.cover = Some("cover.jpg".into());
        assert!(!lib.upsert(fresh));
        let g = lib.get("steam-1").unwrap();
        assert_eq!(g.title, "Hollow Tides: Remastered");
        assert_eq!(g.playtime_secs, 10);
        assert!(g.cover.is_some());
        assert!(lib.upsert(game("New", Source::Steam { app_id: 2 })));
    }

    #[test]
    fn sessions_accumulate() {
        let mut lib = sample();
        lib.record_launch("steam-1", 1000);
        lib.record_session("steam-1", 1000, 50);
        let g = lib.get("steam-1").unwrap();
        assert_eq!((g.playtime_secs, g.launch_count, g.last_played), (60, 1, Some(1000)));
    }
}
