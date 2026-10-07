//! Saber's own icons (a hand-picked slice of Lucide), embedded at compile
//! time. Anything else — the component library's icons — falls through to
//! the kit's default asset source.

use gpui_kit::*;
use std::borrow::Cow;

macro_rules! icons {
    ($($variant:ident => $file:literal,)*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Icon {
            $($variant,)*
        }

        impl Icon {
            pub fn path(self) -> &'static str {
                match self {
                    $(Icon::$variant => concat!("saber/icons/", $file),)*
                }
            }
        }

        fn embedded(path: &str) -> Option<&'static [u8]> {
            match path {
                $(concat!("saber/icons/", $file) => Some(include_bytes!(concat!("../assets/icons/", $file))),)*
                _ => None,
            }
        }
    };
}

icons! {
    ArrowUpDown => "arrow-up-down.svg",
    Check => "check.svg",
    Clock => "clock.svg",
    Ellipsis => "ellipsis.svg",
    Eye => "eye.svg",
    EyeOff => "eye-off.svg",
    FolderOpen => "folder-open.svg",
    Gamepad => "gamepad-2.svg",
    Heart => "heart.svg",
    Image => "image.svg",
    Grid => "layout-grid.svg",
    Library => "library.svg",
    Minus => "minus.svg",
    Play => "play.svg",
    Plus => "plus.svg",
    Refresh => "refresh-cw.svg",
    Reset => "rotate-ccw.svg",
    Search => "search.svg",
    Settings => "settings-2.svg",
    Sliders => "sliders-horizontal.svg",
    Sparkles => "sparkles.svg",
    External => "square-arrow-out-up-right.svg",
    Stop => "square.svg",
    Star => "star.svg",
    StarFill => "star-fill.svg",
    Timer => "timer.svg",
    Trash => "trash.svg",
    Close => "x.svg",
}

impl Icon {
    /// An svg element for this icon; size and color it like text.
    pub fn el(self) -> Svg {
        svg().path(self.path()).flex_none()
    }
}

pub struct SaberAssets;

impl AssetSource for SaberAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        match embedded(path) {
            Some(bytes) => Ok(Some(Cow::Borrowed(bytes))),
            None => gpui_kit::assets::Assets.load(path),
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        gpui_kit::assets::Assets.list(path)
    }
}
