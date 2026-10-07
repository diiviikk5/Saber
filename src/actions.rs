//! Keyboard commands.

use gpui_kit::*;

gpui_kit::actions!(
    saber,
    [
        FocusSearch,
        Escape,
        OpenSettings,
        AddGame,
        Rescan,
        PlaySelected,
        ToggleFavorite,
        SelectNext,
        SelectPrev,
        SelectUp,
        SelectDown,
        Quit,
    ]
);

/// Context name for bindings that only apply on the shelf, not while typing.
pub const SHELF: &str = "Shelf";

pub fn bind(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("ctrl-k", FocusSearch, None),
        KeyBinding::new("ctrl-f", FocusSearch, None),
        KeyBinding::new("/", FocusSearch, Some(SHELF)),
        KeyBinding::new("escape", Escape, None),
        KeyBinding::new("ctrl-,", OpenSettings, None),
        KeyBinding::new("ctrl-n", AddGame, None),
        KeyBinding::new("ctrl-r", Rescan, None),
        KeyBinding::new("ctrl-q", Quit, None),
        KeyBinding::new("enter", PlaySelected, Some(SHELF)),
        KeyBinding::new("f", ToggleFavorite, Some(SHELF)),
        KeyBinding::new("right", SelectNext, Some(SHELF)),
        KeyBinding::new("left", SelectPrev, Some(SHELF)),
        KeyBinding::new("down", SelectDown, Some(SHELF)),
        KeyBinding::new("up", SelectUp, Some(SHELF)),
    ]);
    cx.on_action(|_: &Quit, cx| cx.quit());
}
