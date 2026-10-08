//! The built-in color moods. The website's swatches mirror these values.

/// Colors are `0xRRGGBB`; `line` is `0xRRGGBBAA`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Palette {
    pub id: &'static str,
    pub name: &'static str,
    pub dark: bool,
    pub bg: u32,
    pub surface: u32,
    pub surface_2: u32,
    pub line: u32,
    pub text: u32,
    pub muted: u32,
    pub faint: u32,
    pub accent: u32,
}

const DARK_TEXT: (u32, u32, u32, u32) = (0xf2efe9, 0x8b8a90, 0x55545a, 0xffffff12);

const fn dark(
    id: &'static str,
    name: &'static str,
    bg: u32,
    surface: u32,
    surface_2: u32,
    accent: u32,
) -> Palette {
    Palette {
        id,
        name,
        dark: true,
        bg,
        surface,
        surface_2,
        line: DARK_TEXT.3,
        text: DARK_TEXT.0,
        muted: DARK_TEXT.1,
        faint: DARK_TEXT.2,
        accent,
    }
}

pub const PALETTES: [Palette; 10] = [
    dark("ember", "Ember", 0x0b0b0c, 0x121214, 0x18181b, 0xff5a36),
    dark("glacier", "Glacier", 0x0a0d12, 0x10141b, 0x161c25, 0x7cc4ff),
    dark("moss", "Moss", 0x0b0d0a, 0x121510, 0x181c15, 0x9bd46a),
    dark("orchid", "Orchid", 0x0e0b10, 0x151118, 0x1c1620, 0xd78cff),
    dark("sand", "Sand", 0x0f0d0a, 0x171410, 0x1e1a15, 0xe8c27a),
    // True black for OLED panels.
    dark("midnight", "Midnight", 0x000000, 0x0a0a0b, 0x121214, 0x8ea2ff),
    dark("neon", "Neon", 0x0d0816, 0x150e22, 0x1c142d, 0xff4fd8),
    dark("fjord", "Fjord", 0x0c1116, 0x121a21, 0x18222b, 0x88c8d8),
    Palette {
        id: "paper",
        name: "Paper",
        dark: false,
        bg: 0xf4f1ea,
        surface: 0xfbf9f4,
        surface_2: 0xebe7de,
        line: 0x14121017,
        text: 0x1a1918,
        muted: 0x6b6862,
        faint: 0xa29e96,
        accent: 0xe04a2a,
    },
    Palette {
        id: "rose",
        name: "Rosé",
        dark: false,
        bg: 0xf7efee,
        surface: 0xfdf8f7,
        surface_2: 0xefe3e1,
        line: 0x2a14181a,
        text: 0x241618,
        muted: 0x7a6467,
        faint: 0xb19ea0,
        accent: 0xd6456b,
    },
];

/// Looks up a palette by id, falling back to Ember.
pub fn by_id(id: &str) -> &'static Palette {
    PALETTES.iter().find(|p| p.id == id).unwrap_or(&PALETTES[0])
}

/// Accent choices offered in settings, on top of each palette's own.
pub const ACCENTS: [u32; 12] = [
    0xff5a36, 0xff8a3d, 0xffc53d, 0xe8c27a, 0x9bd46a, 0x3ddc97, 0x4fd1c5, 0x7cc4ff, 0x8ea2ff,
    0xd78cff, 0xff4fd8, 0xff6b9a,
];
