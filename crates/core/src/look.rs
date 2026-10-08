//! What a game *looks* like, distilled from its art: a vibrant tint to use as
//! an accent, a deep shade to wash backgrounds with, and a soft pre-blurred
//! backdrop. GPUI can't blur at draw time, so the blur is baked once here.

use crate::game::{Game, hash};
use image::imageops::{self, FilterType};
use image::{DynamicImage, RgbImage};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Look {
    /// A saturated, mid-light color from the art (`0xRRGGBB`).
    pub tint: u32,
    /// A very dark color in the art's dominant hue (`0xRRGGBB`).
    pub shade: u32,
    /// A small, heavily blurred copy of the wide art.
    pub backdrop: PathBuf,
}

/// The art a look is built from: wide art if there is any, else the cover.
pub fn source(game: &Game) -> Option<&Path> {
    game.hero
        .as_deref()
        .or(game.cover.as_deref())
        .filter(|p| p.is_file())
}

/// Where the backdrop for this game's current art should live. Changing the
/// art changes the name, so stale looks are easy to spot.
pub fn backdrop_path(game: &Game, dir: &Path) -> Option<PathBuf> {
    let src = source(game)?;
    let key = hash(&src.to_string_lossy());
    Some(dir.join(format!("{}-{key:x}.jpg", game.id)))
}

/// True when the game has art but no (or an outdated) look.
pub fn is_stale(game: &Game, dir: &Path) -> bool {
    match (backdrop_path(game, dir), &game.look) {
        (None, _) => false,
        (Some(_), None) => true,
        (Some(want), Some(look)) => look.backdrop != want || !want.is_file(),
    }
}

/// Builds the look for a game. Slow-ish (decodes an image); run it off the UI
/// thread.
pub fn build(game: &Game, dir: &Path) -> Option<Look> {
    let src = source(game)?;
    let dest = backdrop_path(game, dir)?;
    let img = image::open(src).ok()?;

    // Colors come from the cover when there is one: it's the art people
    // recognise the game by.
    let palette_img = game
        .cover
        .as_deref()
        .and_then(|p| image::open(p).ok())
        .unwrap_or_else(|| img.clone());
    let (tint, shade) = colors(&palette_img);

    std::fs::create_dir_all(dir).ok()?;
    let small = img.resize(320, 320, FilterType::Triangle).to_rgb8();
    let blurred = imageops::blur(&small, 9.0);
    save_jpeg(&blurred, &dest)?;

    Some(Look {
        tint,
        shade,
        backdrop: dest,
    })
}

fn save_jpeg(img: &RgbImage, dest: &Path) -> Option<()> {
    let tmp = dest.with_extension("part");
    let file = std::fs::File::create(&tmp).ok()?;
    let mut writer = std::io::BufWriter::new(file);
    image::codecs::jpeg::JpegEncoder::new_with_quality(&mut writer, 88)
        .encode_image(img)
        .ok()?;
    drop(writer);
    std::fs::rename(&tmp, dest).ok()
}

/// Picks the most prominent vivid hue in the image, then derives an accent
/// and a background shade from it.
fn colors(img: &DynamicImage) -> (u32, u32) {
    let thumb = img.thumbnail(48, 48).to_rgb8();
    const BINS: usize = 24;
    let mut weight = [0f32; BINS];
    let mut sums = [(0f32, 0f32, 0f32); BINS];
    let mut all = (0f32, 0f32, 0f32, 0f32);

    for p in thumb.pixels() {
        let [r, g, b] = p.0.map(|c| c as f32 / 255.);
        let (h, s, v) = hsv(r, g, b);
        all = (all.0 + h.cos(), all.1 + h.sin(), all.2 + s, all.3 + 1.);
        // Ignore greys, near-blacks and blown-out highlights.
        if s < 0.25 || v < 0.2 || v > 0.98 {
            continue;
        }
        let bin = ((h / std::f32::consts::TAU) * BINS as f32) as usize % BINS;
        let w = s * s * v;
        weight[bin] += w;
        // Hues are angles, so average them as vectors.
        let (x, y, sat) = &mut sums[bin];
        *x += h.cos() * w;
        *y += h.sin() * w;
        *sat += s * w;
    }

    // Smooth neighbouring bins so a hue split across a boundary still wins.
    let score =
        |i: usize| weight[(i + BINS - 1) % BINS] * 0.5 + weight[i] + weight[(i + 1) % BINS] * 0.5;
    let best = (0..BINS)
        .max_by(|a, b| score(*a).total_cmp(&score(*b)))
        .unwrap_or(0);

    let (hue, sat) = if weight[best] > 0.5 {
        let (x, y, s) = sums[best];
        (
            y.atan2(x).rem_euclid(std::f32::consts::TAU),
            (s / weight[best]).clamp(0., 1.),
        )
    } else {
        // A monochrome cover: keep its average hue but stay muted.
        let n = all.3.max(1.);
        (
            (all.1 / n)
                .atan2(all.0 / n)
                .rem_euclid(std::f32::consts::TAU),
            0.18,
        )
    };

    let tint = hsl_to_rgb(hue, sat.max(0.6).min(0.92), 0.62);
    let shade = hsl_to_rgb(hue, (sat * 0.7).min(0.55), 0.075);
    (tint, shade)
}

fn hsv(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let d = max - min;
    let h = if d == 0. {
        0.
    } else if max == r {
        ((g - b) / d).rem_euclid(6.)
    } else if max == g {
        (b - r) / d + 2.
    } else {
        (r - g) / d + 4.
    };
    let s = if max == 0. { 0. } else { d / max };
    (h / 6. * std::f32::consts::TAU, s, max)
}

/// `h` in radians, `s` and `l` in 0..1 → `0xRRGGBB`.
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> u32 {
    let h = h / std::f32::consts::TAU * 6.;
    let c = (1. - (2. * l - 1.).abs()) * s;
    let x = c * (1. - (h.rem_euclid(2.) - 1.).abs());
    let (r, g, b) = match h as u32 {
        0 => (c, x, 0.),
        1 => (x, c, 0.),
        2 => (0., c, x),
        3 => (0., x, c),
        4 => (x, 0., c),
        _ => (c, 0., x),
    };
    let m = l - c / 2.;
    let to = |v: f32| ((v + m).clamp(0., 1.) * 255.).round() as u32;
    (to(r) << 16) | (to(g) << 8) | to(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgb;

    fn solid(r: u8, g: u8, b: u8) -> DynamicImage {
        DynamicImage::ImageRgb8(RgbImage::from_pixel(32, 32, Rgb([r, g, b])))
    }

    fn channels(c: u32) -> (u32, u32, u32) {
        (c >> 16, (c >> 8) & 0xff, c & 0xff)
    }

    #[test]
    fn a_red_cover_gives_a_red_tint() {
        let (tint, shade) = colors(&solid(200, 30, 30));
        let (r, g, b) = channels(tint);
        assert!(r > 180 && g < 120 && b < 120, "{tint:06x}");
        let (r, g, b) = channels(shade);
        assert!(r < 40 && g < 30 && b < 30, "{shade:06x}");
    }

    #[test]
    fn the_dominant_vivid_hue_wins_over_greys() {
        let mut img = RgbImage::from_pixel(40, 40, Rgb([120, 120, 120]));
        for x in 0..14 {
            for y in 0..40 {
                img.put_pixel(x, y, Rgb([20, 90, 220]));
            }
        }
        let (tint, _) = colors(&DynamicImage::ImageRgb8(img));
        let (r, _, b) = channels(tint);
        assert!(b > r + 60, "{tint:06x}");
    }

    #[test]
    fn hsl_round_trips_primaries() {
        assert_eq!(hsl_to_rgb(0., 1., 0.5), 0xff0000);
        assert_eq!(hsl_to_rgb(std::f32::consts::TAU / 3., 1., 0.5), 0x00ff00);
    }
}
