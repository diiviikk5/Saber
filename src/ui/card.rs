//! A poster on the shelf.

use crate::app::Saber;
use crate::assets::Icon;
use crate::theme::{SaberTheme, theme};
use crate::ui::motion;
use crate::ui::widgets::{glow, lift_shadow};
use gpui_kit::prelude::*;
use gpui_kit::*;
use saber_core::format;
use saber_core::game::Game;

/// The art for a game: its cover, or a generated poster if it has none
/// (or the file has gone missing).
pub fn poster_art(game: &Game, t: &SaberTheme, title_size: Pixels) -> AnyElement {
    let generated = generated_poster(game, t, title_size);
    match &game.cover {
        Some(path) => img(path.clone())
            .size_full()
            .object_fit(ObjectFit::Cover)
            .with_fallback(move || generated.clone()())
            .into_any_element(),
        None => generated(),
    }
}

/// A deterministic two-tone poster with the game's initials and title.
fn generated_poster(game: &Game, t: &SaberTheme, title_size: Pixels) -> std::rc::Rc<dyn Fn() -> AnyElement> {
    let (a, b) = game.poster_hues();
    let initials = game.initials();
    let title = game.title.clone();
    let font = t.display_font();
    let on_art = t.colors.on_art;
    std::rc::Rc::new(move || {
        div()
            .size_full()
            .relative()
            .bg(linear_gradient(
                160.,
                linear_color_stop(hsla(a / 360., 0.42, 0.34, 1.), 0.),
                linear_color_stop(hsla(b / 360., 0.5, 0.09, 1.), 1.),
            ))
            .child(
                div()
                    .absolute()
                    .top(px(10.))
                    .left(px(12.))
                    .font_family(font)
                    .text_size(title_size * 2.6)
                    .line_height(relative(1.))
                    .font_weight(FontWeight::BOLD)
                    .text_color(on_art.opacity(0.14))
                    .child(initials.clone()),
            )
            .child(
                div()
                    .absolute()
                    .bottom(px(12.))
                    .left(px(12.))
                    .right(px(12.))
                    .font_family(font)
                    .text_size(title_size)
                    .line_height(relative(1.1))
                    .text_color(on_art)
                    .child(title.clone()),
            )
            .into_any_element()
    })
}

pub fn card(game: &Game, index: usize, selected: bool, running: bool, cx: &mut Context<Saber>) -> impl IntoElement + use<> {
    let t = *theme(cx);
    let c = t.colors;
    let id = game.id.clone();
    let group: SharedString = format!("card-{id}").into();
    let w = t.card_width;
    let h = t.card_height();
    let has_cover = game.cover.is_some();
    let show_playtime = cx.entity().read(cx).settings.show_playtime;
    let epoch = cx.entity().read(cx).shelf_epoch;

    let caption = if show_playtime && game.playtime_secs > 0 {
        format!("{} played", format::playtime(game.playtime_secs))
    } else {
        game.source.label().to_string()
    };

    let select_id = id.clone();
    let play_id = id.clone();
    let menu_id = id.clone();
    let play_btn_id = id.clone();

    let poster = div()
        .id(SharedString::from(format!("poster-{id}")))
        .group(group.clone())
        .relative()
        .w(w)
        .h(h)
        .flex_none()
        .rounded(t.radius)
        .overflow_hidden()
        .bg(c.surface_2)
        .border_1()
        .border_color(if selected { c.accent } else { c.line })
        .cursor_pointer()
        .shadow(if selected {
            vec![glow(c.accent, 28.), lift_shadow(c.dark)]
        } else {
            vec![]
        })
        .hover(|d| d.border_color(if selected { c.accent } else { c.line_strong }).shadow(vec![lift_shadow(c.dark)]))
        .on_click(cx.listener(move |this, ev: &ClickEvent, window, cx| {
            if ev.click_count() >= 2 {
                this.play(&play_id, window, cx);
            } else {
                this.select(&select_id, cx);
            }
        }))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                this.select(&menu_id, cx);
                this.open_menu(menu_id.clone(), ev.position, cx);
            }),
        )
        .child(poster_art(game, &t, px(f32::from(w) / 9.)))
        // Bottom scrim with title, revealed on hover for real covers.
        .child(
            div()
                .absolute()
                .bottom_0()
                .left_0()
                .right_0()
                .h(h * 0.45)
                .flex()
                .flex_col()
                .justify_end()
                .px(px(12.))
                .pb(px(11.))
                .bg(linear_gradient(
                    180.,
                    linear_color_stop(hsla(0., 0., 0., 0.), 0.),
                    linear_color_stop(hsla(0., 0., 0., 0.85), 1.),
                ))
                .when(has_cover, |d| d.opacity(0.).group_hover(group.clone(), |s| s.opacity(1.)))
                .when(!has_cover, |d| d.opacity(0.))
                .child(
                    div()
                        .text_size(px(13.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(c.on_art)
                        .line_clamp(2)
                        .child(game.title.clone()),
                )
                .child(
                    div()
                        .text_size(px(11.))
                        .text_color(c.on_art.opacity(0.65))
                        .child(caption),
                ),
        )
        // Play button, top-right on hover.
        .child(
            div()
                .id(SharedString::from(format!("play-{id}")))
                .absolute()
                .top(px(10.))
                .right(px(10.))
                .size(px(34.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(c.accent)
                .shadow(vec![glow(c.accent, 18.)])
                .opacity(0.)
                .group_hover(group.clone(), |s| s.opacity(1.))
                .hover(|d| d.bg(c.accent.opacity(0.88)))
                .on_click(cx.listener(move |this, _, window, cx| {
                    cx.stop_propagation();
                    this.play(&play_btn_id, window, cx);
                }))
                .child(Icon::Play.el().size(px(14.)).text_color(c.on_accent)),
        )
        .when(game.favorite, |d| {
            d.child(
                div()
                    .absolute()
                    .top(px(10.))
                    .left(px(10.))
                    .size(px(22.))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(hsla(0., 0., 0., 0.45))
                    .child(Icon::StarFill.el().size(px(11.)).text_color(c.accent)),
            )
        })
        .when(running, |d| d.child(now_playing_badge(&t)));

    motion::rise_in(poster, ("card", epoch * 10_000 + index), index)
}

fn now_playing_badge(t: &SaberTheme) -> impl IntoElement {
    let c = t.colors;
    div()
        .absolute()
        .bottom(px(10.))
        .left(px(10.))
        .flex()
        .items_center()
        .gap(px(6.))
        .px(px(8.))
        .py(px(3.))
        .rounded_full()
        .bg(hsla(0., 0., 0., 0.6))
        .text_size(px(10.5))
        .text_color(c.on_art)
        .child(motion::pulse_dot(c.accent, "card-pulse"))
        .child("Playing")
}
