//! Turns the user's palette settings into colors, and keeps gpui-component's
//! own theme in step so inputs, scrollbars and toasts match the rest of Saber.

use gpui_kit::component::{Theme, ThemeMode};
use gpui_kit::*;
use saber_core::palette::{self, Palette};
use saber_core::settings::Settings;

#[derive(Clone, Copy, Debug)]
pub struct Colors {
    pub dark: bool,
    pub bg: Hsla,
    pub surface: Hsla,
    pub surface_2: Hsla,
    pub line: Hsla,
    pub line_strong: Hsla,
    pub text: Hsla,
    pub muted: Hsla,
    pub faint: Hsla,
    pub accent: Hsla,
    /// Text drawn on top of the accent.
    pub on_accent: Hsla,
    /// Text drawn on top of game art, regardless of light/dark palette.
    pub on_art: Hsla,
}

fn hex(c: u32) -> Hsla {
    rgb(c).into()
}

impl Colors {
    pub fn new(p: &Palette, accent: u32) -> Self {
        let accent = hex(accent);
        let text = hex(p.text);
        Self {
            dark: p.dark,
            bg: hex(p.bg),
            surface: hex(p.surface),
            surface_2: hex(p.surface_2),
            line: rgba(p.line).into(),
            line_strong: text.opacity(if p.dark { 0.16 } else { 0.18 }),
            text,
            muted: hex(p.muted),
            faint: hex(p.faint),
            accent,
            on_accent: if accent.l > 0.62 { hex(0x0b0b0c) } else { hex(0xffffff) },
            on_art: hex(0xf6f3ee),
        }
    }

    /// The accent at a given strength, for glows and tints.
    pub fn accent_a(&self, alpha: f32) -> Hsla {
        self.accent.opacity(alpha)
    }
}

/// Everything views need to draw themselves, as one cheap-to-copy global.
#[derive(Clone, Copy, Debug)]
pub struct SaberTheme {
    pub colors: Colors,
    pub radius: Pixels,
    pub card_width: Pixels,
    pub serif_titles: bool,
}

impl Global for SaberTheme {}

impl SaberTheme {
    pub fn from_settings(s: &Settings) -> Self {
        let palette = palette::by_id(&s.theme);
        Self {
            colors: Colors::new(palette, s.accent()),
            radius: px(s.radius),
            card_width: px(s.card_width),
            serif_titles: s.serif_titles,
        }
    }

    pub fn card_height(&self) -> Pixels {
        self.card_width * 1.5
    }

    /// Font for big display titles.
    pub fn display_font(&self) -> &'static str {
        if self.serif_titles { SERIF } else { SANS }
    }

    /// Installs the theme and restyles the component library to match.
    pub fn apply(self, cx: &mut App) {
        cx.set_global(self);
        let c = self.colors;
        let mode = if c.dark { ThemeMode::Dark } else { ThemeMode::Light };
        Theme::change(mode, None, cx);
        let theme = Theme::global_mut(cx);
        theme.font_family = SANS.into();
        theme.radius = px(8.);
        theme.radius_lg = self.radius;
        theme.shadow = true;
        let t = &mut theme.colors;
        t.background = c.bg;
        t.foreground = c.text;
        t.border = c.line;
        t.input = c.line_strong;
        t.ring = c.accent;
        t.caret = c.accent;
        t.selection = c.accent_a(0.3);
        t.primary = c.accent;
        t.primary_hover = c.accent.opacity(0.9);
        t.primary_active = c.accent.opacity(0.8);
        t.primary_foreground = c.on_accent;
        t.accent = c.surface_2;
        t.accent_foreground = c.text;
        t.muted = c.surface_2;
        t.muted_foreground = c.muted;
        t.popover = c.surface;
        t.popover_foreground = c.text;
        t.title_bar = c.bg;
        t.title_bar_border = c.bg;
        t.scrollbar = gpui_kit::transparent_black();
        t.scrollbar_thumb = c.text.opacity(0.14);
        t.scrollbar_thumb_hover = c.text.opacity(0.26);
        t.overlay = hsla(0., 0., 0., if c.dark { 0.55 } else { 0.25 });
        t.window_border = c.line;
        t.list_hover = c.surface_2;
        t.list_active = c.accent_a(0.14);
        t.list_active_border = c.accent;
        t.secondary = c.surface_2;
        t.secondary_hover = c.surface_2.opacity(0.8);
        t.secondary_foreground = c.text;
    }
}

pub fn theme(cx: &App) -> &SaberTheme {
    cx.global::<SaberTheme>()
}

#[cfg(windows)]
pub const SANS: &str = "Segoe UI Variable Text";
#[cfg(target_os = "macos")]
pub const SANS: &str = ".SystemUIFont";
#[cfg(all(unix, not(target_os = "macos")))]
pub const SANS: &str = "Inter";

pub const SERIF: &str = "Georgia";

#[cfg(windows)]
pub const MONO: &str = "Cascadia Mono";
#[cfg(not(windows))]
pub const MONO: &str = "Menlo";
