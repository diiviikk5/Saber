//! User preferences, persisted next to the library.

use crate::library::Sort;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Palette id, see [`crate::palette::PALETTES`].
    pub theme: String,
    /// Overrides the palette's accent when set.
    pub accent: Option<u32>,
    /// Poster width in pixels; height follows at 3:2.
    pub card_width: f32,
    /// Corner radius for posters and panels.
    pub radius: f32,
    pub show_hero: bool,
    pub show_playtime: bool,
    pub sort: Sort,
    /// Minimize Saber while a game is running.
    pub minimize_on_launch: bool,
    /// Look for new Steam/Epic installs at startup.
    pub scan_on_start: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            theme: "ember".into(),
            accent: None,
            card_width: 168.,
            radius: 12.,
            show_hero: true,
            show_playtime: true,
            sort: Sort::Recent,
            minimize_on_launch: true,
            scan_on_start: true,
        }
    }
}

impl Settings {
    pub const CARD_WIDTH: (f32, f32) = (128., 240.);
    pub const RADIUS: (f32, f32) = (0., 24.);

    pub fn accent(&self) -> u32 {
        self.accent
            .unwrap_or_else(|| crate::palette::by_id(&self.theme).accent)
    }

    /// Keeps hand-edited values inside the ranges the UI can draw.
    pub fn clamped(mut self) -> Self {
        self.card_width = self.card_width.clamp(Self::CARD_WIDTH.0, Self::CARD_WIDTH.1);
        self.radius = self.radius.clamp(Self::RADIUS.0, Self::RADIUS.1);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_files_fill_in_defaults() {
        let s: Settings = serde_json::from_str(r#"{ "theme": "moss" }"#).unwrap();
        assert_eq!(s.theme, "moss");
        assert_eq!(s.card_width, 168.);
        assert_eq!(s.accent(), 0x9bd46a);
    }

    #[test]
    fn clamps_out_of_range_values() {
        let s = Settings { card_width: 9000., radius: -3., ..Default::default() }.clamped();
        assert_eq!((s.card_width, s.radius), (240., 0.));
    }
}
