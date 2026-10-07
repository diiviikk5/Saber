//! The featured banner for the selected game.

use crate::app::Saber;
use crate::assets::Icon;
use crate::theme::theme;
use crate::ui::motion;
use crate::ui::widgets::{glow, lift_shadow};
use gpui_kit::prelude::*;
use gpui_kit::*;
use saber_core::format;
use saber_core::game::Game;

pub const HERO_HEIGHT: f32 = 320.;

pub fn hero(game: &Game, running: bool, cx: &mut Context<Saber>) -> impl IntoElement + use<> {
    let t = *theme(cx);
    let c = t.colors;
    let id = game.id.clone();
    let now = format::now();

    let eyebrow = if running {
        "Now playing"
    } else if game.last_played.is_some() {
        "Continue playing"
    } else {
        "Ready to play"
    };

    let mut stats: Vec<String> = vec![game.source.label().to_string()];
    if game.playtime_secs > 0 {
        stats.push(format!("{} played", format::playtime(game.playtime_secs)));
    }
    if let Some(at) = game.last_played {
        stats.push(format!("last played {}", format::relative(at, now)));
    } else {
        stats.push(format!("added {}", format::relative(game.added_at, now)));
    }

    let radius = t.radius * 1.5;

    // Wide art if we have it; otherwise the cover, stretched and dimmed.
    let art = match (&game.hero, &game.cover) {
        (Some(hero), _) => img(hero.clone())
            .size_full()
            .rounded(radius)
            .object_fit(ObjectFit::Cover)
            .into_any_element(),
        _ => div()
            .size_full()
            .opacity(0.55)
            .child(crate::ui::card::poster_backdrop(game).rounded(radius))
            .into_any_element(),
    };

    let play_id = id.clone();
    let fav_id = id.clone();
    let menu_id = id.clone();

    let banner = div()
        .relative()
        .w_full()
        .h(px(HERO_HEIGHT))
        .flex_none()
        .rounded(radius)
        .overflow_hidden()
        .bg(c.surface)
        .border_1()
        .border_color(c.line)
        .shadow(vec![lift_shadow(c.dark)])
        .child(motion::fade_in(
            div().absolute().inset_0().child(art),
            ("hero-art", hash_id(&id)),
            0.5,
        ))
        // Scrims: from the left for legibility, from below to seat the content.
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded(radius)
                .bg(linear_gradient(
                    90.,
                    linear_color_stop(hsla(0., 0., 0.02, 0.88), 0.),
                    linear_color_stop(hsla(0., 0., 0.02, 0.), 0.75),
                )),
        )
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded(radius)
                .bg(linear_gradient(
                    180.,
                    linear_color_stop(hsla(0., 0., 0., 0.), 0.45),
                    linear_color_stop(hsla(0., 0., 0., 0.55), 1.),
                )),
        )
        .child(
            div()
                .absolute()
                .left(px(36.))
                .bottom(px(32.))
                .right(px(36.))
                .child(motion::rise_in(
                    div()
                        .relative()
                        .flex()
                        .flex_col()
                        .gap(px(10.))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .when(running, |d| {
                                    d.child(motion::pulse_dot(c.accent, "hero-pulse"))
                                })
                                .child(
                                    div()
                                        .font_family(crate::theme::MONO)
                                        .text_size(px(11.))
                                        .text_color(c.accent)
                                        .child(eyebrow.to_uppercase()),
                                ),
                        )
                        .child(
                            div()
                                .max_w(px(640.))
                                .font_family(t.display_font())
                                .text_size(px(46.))
                                .line_height(relative(1.02))
                                .text_color(c.on_art)
                                .line_clamp(2)
                                .child(game.title.clone()),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .text_size(px(13.))
                                .text_color(c.on_art.opacity(0.7))
                                .children(stats.into_iter().enumerate().map(|(i, s)| {
                                    div()
                                        .flex()
                                        .gap(px(8.))
                                        .when(i > 0, |d| {
                                            d.child(
                                                div().text_color(c.on_art.opacity(0.35)).child("·"),
                                            )
                                        })
                                        .child(s)
                                })),
                        )
                        .child(
                            div()
                                .mt(px(10.))
                                .flex()
                                .items_center()
                                .gap(px(10.))
                                .child(
                                    div()
                                        .id("hero-play")
                                        .h(px(42.))
                                        .px(px(22.))
                                        .flex()
                                        .items_center()
                                        .gap(px(9.))
                                        .rounded_full()
                                        .cursor_pointer()
                                        .bg(c.accent)
                                        .text_color(c.on_accent)
                                        .text_size(px(14.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .shadow(vec![glow(c.accent, 24.)])
                                        .hover(|d| {
                                            d.bg(c.accent.opacity(0.9))
                                                .shadow(vec![glow(c.accent, 34.)])
                                        })
                                        .active(|d| d.opacity(0.85))
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.play(&play_id, window, cx)
                                        }))
                                        .child(
                                            Icon::Play.el().size(px(14.)).text_color(c.on_accent),
                                        )
                                        .child(if running { "Running" } else { "Play" }),
                                )
                                .child(
                                    round_button(
                                        "hero-fav",
                                        if game.favorite {
                                            Icon::StarFill
                                        } else {
                                            Icon::Star
                                        },
                                        game.favorite,
                                        cx,
                                    )
                                    .on_click(cx.listener(
                                        move |this, _, _, cx| this.toggle_favorite(&fav_id, cx),
                                    )),
                                )
                                .child(
                                    round_button("hero-more", Icon::Ellipsis, false, cx).on_click(
                                        cx.listener(move |this, ev: &ClickEvent, _, cx| {
                                            let pos = ev.position();
                                            this.open_menu(menu_id.clone(), pos, cx);
                                        }),
                                    ),
                                ),
                        ),
                    ("hero-content", hash_id(&id)),
                    0,
                )),
        );
    banner
}

fn round_button(id: &'static str, icon: Icon, active: bool, cx: &App) -> Stateful<Div> {
    let c = theme(cx).colors;
    div()
        .id(id)
        .size(px(42.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .cursor_pointer()
        .bg(hsla(0., 0., 1., 0.1))
        .border_1()
        .border_color(hsla(0., 0., 1., 0.14))
        .hover(|d| d.bg(hsla(0., 0., 1., 0.18)))
        .child(
            icon.el()
                .size(px(16.))
                .text_color(if active { c.accent } else { c.on_art }),
        )
}

fn hash_id(id: &str) -> usize {
    saber_core::game::hash(id) as usize
}
