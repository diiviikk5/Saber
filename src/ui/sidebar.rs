//! The left rail: views, sources, the running game, quick actions.

use crate::app::{Page, Saber};
use crate::assets::Icon;
use crate::theme::{MONO, theme};
use crate::ui::card::poster_art;
use crate::ui::motion;
use crate::ui::widgets::{IconButton, eyebrow, glow};
use crate::ui::SIDEBAR_WIDTH;
use gpui_kit::prelude::*;
use gpui_kit::*;
use saber_core::library::View;
use std::time::Duration;

pub fn sidebar(app: &Saber, cx: &mut Context<Saber>) -> impl IntoElement + use<> {
    let c = theme(cx).colors;
    let lib = &app.library;
    let on_library = app.page == Page::Library;

    let mut nav = div().flex().flex_col().gap(px(2.));
    let main_views = [
        (View::All, Icon::Grid, "Library"),
        (View::Favorites, Icon::Star, "Favorites"),
        (View::Recent, Icon::Clock, "Recently played"),
    ];
    for (view, icon, label) in main_views {
        let count = lib.count(&view);
        let active = on_library && app.view == view;
        nav = nav.child(item(view, icon, label, count, active, cx));
    }

    let sources: Vec<_> = ["Steam", "Epic", "Manual"]
        .into_iter()
        .map(|s| (View::Source(s), lib.count(&View::Source(s)), s))
        .filter(|(_, n, _)| *n > 0)
        .collect();
    let hidden = lib.count(&View::Hidden);

    div()
        .w(px(SIDEBAR_WIDTH))
        .h_full()
        .flex_none()
        .flex()
        .flex_col()
        .px(px(12.))
        .pt(px(8.))
        .pb(px(12.))
        .gap(px(22.))
        .child(nav)
        .when(!sources.is_empty(), |d| {
            d.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .child(eyebrow("Sources", cx).px(px(12.)).pb(px(6.)))
                    .children(sources.into_iter().map(|(view, n, label)| {
                        let active = on_library && app.view == view;
                        item(view, Icon::Gamepad, label, n, active, cx)
                    })),
            )
        })
        .when(hidden > 0, |d| {
            let active = on_library && app.view == View::Hidden;
            d.child(item(View::Hidden, Icon::EyeOff, "Hidden", hidden, active, cx))
        })
        .child(div().flex_1())
        .when_some(now_playing(app, cx), |d, el| d.child(el))
        .child(footer(app, cx).text_color(c.muted))
}

fn item(view: View, icon: Icon, label: &'static str, count: usize, active: bool, cx: &mut Context<Saber>) -> impl IntoElement + use<> {
    let c = theme(cx).colors;
    let target = view.clone();
    div()
        .id(SharedString::from(format!("nav-{label}")))
        .relative()
        .h(px(34.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(10.))
        .rounded(px(9.))
        .cursor_pointer()
        .text_size(px(13.5))
        .when(active, |d| d.bg(c.text.opacity(0.07)).text_color(c.text))
        .when(!active, |d| {
            d.text_color(c.muted)
                .hover(|d| d.bg(c.text.opacity(0.04)).text_color(c.text))
        })
        .on_click(cx.listener(move |this, _, _, cx| this.set_view(target.clone(), cx)))
        .when(active, |d| {
            d.child(
                div()
                    .absolute()
                    .left(px(-12.))
                    .top(px(9.))
                    .w(px(3.))
                    .h(px(16.))
                    .rounded_r(px(3.))
                    .bg(c.accent)
                    .shadow(vec![glow(c.accent, 10.)]),
            )
        })
        .child(icon.el().size(px(15.)).text_color(if active { c.accent } else { c.faint }))
        .child(div().flex_1().child(label))
        .child(
            div()
                .font_family(MONO)
                .text_size(px(11.))
                .text_color(c.faint)
                .child(count.to_string()),
        )
}

/// The running game with a live session clock.
fn now_playing(app: &Saber, cx: &mut Context<Saber>) -> Option<AnyElement> {
    let session = app.session.as_ref()?;
    let game = app.library.get(&session.game_id)?;
    let t = *theme(cx);
    let c = t.colors;
    let secs = session.started.elapsed().as_secs();
    let clock = if session.tracked {
        format!("{:02}:{:02}:{:02}", secs / 3600, secs / 60 % 60, secs % 60)
    } else {
        "Launched".to_string()
    };
    let el = div()
        .flex()
        .gap(px(12.))
        .p(px(10.))
        .rounded(px(14.))
        .bg(c.accent_a(0.08))
        .border_1()
        .border_color(c.accent_a(0.25))
        .child(
            div()
                .w(px(38.))
                .h(px(57.))
                .flex_none()
                .rounded(px(6.))
                .overflow_hidden()
                .child(poster_art(game, &t, px(7.))),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .justify_center()
                .gap(px(3.))
                .min_w_0()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(motion::pulse_dot(c.accent, "np-pulse"))
                        .child(eyebrow("Now playing", cx).text_color(c.accent)),
                )
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(c.text)
                        .truncate()
                        .child(session.title.clone()),
                )
                .child(
                    div()
                        .font_family(MONO)
                        .text_size(px(11.))
                        .text_color(c.muted)
                        .child(clock),
                ),
        );
    Some(motion::fade_in(el, "now-playing", 0.3).into_any_element())
}

fn footer(app: &Saber, cx: &mut Context<Saber>) -> Div {
    let scanning = app.scanning;
    let settings_open = app.page == Page::Settings;
    let rescan = IconButton::new("rescan", Icon::Refresh)
        .tooltip(if scanning { "Scanning…" } else { "Rescan Steam & Epic  (Ctrl R)" })
        .on_click(cx.listener(|this, _, window, cx| this.rescan(true, window, cx)));
    div()
        .flex()
        .items_center()
        .gap(px(4.))
        .child(
            IconButton::new("add", Icon::Plus)
                .tooltip("Add a game  (Ctrl N)")
                .on_click(cx.listener(|this, _, window, cx| this.add_game(window, cx))),
        )
        .child(if scanning {
            div()
                .child(rescan.active(true))
                .with_animation(
                    "scan-spin",
                    Animation::new(Duration::from_millis(900)).repeat(),
                    |el, t| el.opacity(0.45 + 0.55 * (t * std::f32::consts::TAU).cos().abs()),
                )
                .into_any_element()
        } else {
            rescan.into_any_element()
        })
        .child(div().flex_1())
        .child(
            IconButton::new("settings", Icon::Settings)
                .active(settings_open)
                .tooltip("Settings  (Ctrl ,)")
                .on_click(cx.listener(|this, _, _, cx| {
                    this.page = if this.page == Page::Settings { Page::Library } else { Page::Settings };
                    cx.notify();
                })),
        )
}
