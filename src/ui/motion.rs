//! Motion: quiet, quick, and never in the way.

use gpui_kit::prelude::*;
use gpui_kit::*;
use std::time::Duration;

const RISE: f32 = 0.32;
const STAGGER: f32 = 0.028;
/// Past this many items, everything else enters together.
const MAX_STAGGER: usize = 16;

fn ease_out_cubic(t: f32) -> f32 {
    1. - (1. - t).powi(3)
}

/// Fades and lifts an element into place, staggered by its index.
pub fn rise_in<E>(el: E, id: impl Into<ElementId>, index: usize) -> AnimationElement<E>
where
    E: Styled + IntoElement + 'static,
{
    let delay = STAGGER * index.min(MAX_STAGGER) as f32;
    let total = RISE + delay;
    el.with_animation(
        id,
        Animation::new(Duration::from_secs_f32(total)),
        move |el, t| {
            let local = ((t * total - delay) / RISE).clamp(0., 1.);
            let e = ease_out_cubic(local);
            el.opacity(e).top(px(10. * (1. - e)))
        },
    )
}

/// A plain fade, for whole panels.
pub fn fade_in<E>(el: E, id: impl Into<ElementId>, secs: f32) -> AnimationElement<E>
where
    E: Styled + IntoElement + 'static,
{
    el.with_animation(
        id,
        Animation::new(Duration::from_secs_f32(secs)),
        |el, t| el.opacity(ease_out_cubic(t)),
    )
}

/// A small dot that breathes, for "now playing".
pub fn pulse_dot(color: Hsla, id: &'static str) -> impl IntoElement {
    div().size(px(7.)).rounded_full().bg(color).with_animation(
        id,
        Animation::new(Duration::from_millis(1600)).repeat(),
        move |el, t| {
            let breath = 0.5 + 0.5 * (t * std::f32::consts::TAU).cos();
            el.opacity(0.35 + 0.65 * breath)
        },
    )
}
