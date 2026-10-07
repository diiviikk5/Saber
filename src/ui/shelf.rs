//! The library page: hero, heading, and the grid of posters.

use crate::app::Saber;
use crate::assets::Icon;
use crate::theme::{MONO, theme};
use crate::ui::card::{CardState, card};
use crate::ui::hero::hero;
use crate::ui::widgets::{glow, kbd, segmented};
use crate::ui::{GRID_GAP, PAGE_PAD, motion};
use gpui_kit::prelude::*;
use gpui_kit::*;
use saber_core::library::{Sort, View};

fn view_title(view: &View) -> &'static str {
    match view {
        View::All => "Library",
        View::Favorites => "Favorites",
        View::Recent => "Recently played",
        View::Source(s) => s,
        View::Hidden => "Hidden",
    }
}

pub fn shelf(app: &Saber, cx: &mut Context<Saber>) -> AnyElement {
    let t = *theme(cx);
    let c = t.colors;

    if app.library.is_empty() {
        return onboarding(app.scanning, cx).into_any_element();
    }

    let games = app.visible();
    let count = games.len();
    let searching = !app.query.trim().is_empty();
    let selected = app.selected_game().map(|g| g.id.clone());

    let featured = (app.settings.show_hero && !searching && app.view != View::Hidden)
        .then(|| app.selected_game())
        .flatten()
        .map(|g| hero(g, app.is_running(&g.id), cx));

    let cards: Vec<_> = games
        .iter()
        .enumerate()
        .map(|(i, g)| {
            let state = CardState {
                selected: selected.as_deref() == Some(g.id.as_str()),
                running: app.is_running(&g.id),
                show_playtime: app.settings.show_playtime,
                epoch: app.shelf_epoch,
            };
            card(g, i, state, cx)
        })
        .collect();

    let title = if searching {
        format!("Results for “{}”", app.query.trim())
    } else {
        view_title(&app.view).to_string()
    };

    let sort_picker = segmented(
        "sort",
        &[
            (Sort::Recent, "Recent"),
            (Sort::Title, "A–Z"),
            (Sort::Playtime, "Most played"),
            (Sort::Added, "New"),
        ],
        app.settings.sort,
        cx,
        {
            let entity = cx.entity().downgrade();
            move |sort, window, cx| {
                let _ = entity.update(cx, |this, cx| this.set_sort(sort, window, cx));
            }
        },
    );

    div()
        .id("shelf")
        .size_full()
        .overflow_y_scroll()
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(30.))
                .px(px(PAGE_PAD))
                .pt(px(6.))
                .pb(px(48.))
                .when_some(featured, |d, h| d.child(h))
                .child(
                    div()
                        .flex()
                        .items_end()
                        .justify_between()
                        .gap(px(16.))
                        .child(
                            div()
                                .flex()
                                .items_baseline()
                                .gap(px(12.))
                                .child(
                                    div()
                                        .font_family(t.display_font())
                                        .text_size(px(30.))
                                        .line_height(relative(1.))
                                        .text_color(c.text)
                                        .child(title),
                                )
                                .child(
                                    div()
                                        .font_family(MONO)
                                        .text_size(px(12.))
                                        .text_color(c.faint)
                                        .child(format!(
                                            "{count} {}",
                                            if count == 1 { "game" } else { "games" }
                                        )),
                                ),
                        )
                        .when(app.view != View::Recent, |d| d.child(sort_picker)),
                )
                .child(if cards.is_empty() {
                    no_results(app, cx).into_any_element()
                } else {
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(GRID_GAP))
                        .children(cards)
                        .into_any_element()
                }),
        )
        .into_any_element()
}

fn no_results(app: &Saber, cx: &App) -> impl IntoElement {
    let c = theme(cx).colors;
    let (title, hint) = if !app.query.trim().is_empty() {
        (
            "Nothing matches that",
            "Try a shorter search, or press Esc to clear it.",
        )
    } else {
        match app.view {
            View::Favorites => (
                "No favorites yet",
                "Press F on any game, or use the star in its banner.",
            ),
            View::Recent => (
                "Nothing played yet",
                "Games you launch from Saber show up here.",
            ),
            _ => ("Empty shelf", "Nothing to show here."),
        }
    };
    div()
        .py(px(64.))
        .flex()
        .flex_col()
        .items_center()
        .gap(px(8.))
        .child(Icon::Search.el().size(px(22.)).text_color(c.faint))
        .child(div().text_size(px(15.)).text_color(c.text).child(title))
        .child(div().text_size(px(13.)).text_color(c.muted).child(hint))
}

/// First run: no games at all.
fn onboarding(scanning: bool, cx: &mut Context<Saber>) -> impl IntoElement + use<> {
    let t = *theme(cx);
    let c = t.colors;
    let body = div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(18.))
        .max_w(px(460.))
        .child(crate::ui::blade_mark(px(56.), cx))
        .child(
            div()
                .font_family(t.display_font())
                .text_size(px(44.))
                .line_height(relative(1.))
                .text_color(c.text)
                .child(if scanning { "Looking for games…" } else { "An empty shelf." }),
        )
        .child(
            div()
                .text_size(px(14.5))
                .text_color(c.muted)
                .text_center()
                .child("Saber picks up Steam and Epic installs on its own. Anything else — an emulator, an itch.io download — just add it."),
        )
        .child(
            div()
                .mt(px(8.))
                .flex()
                .gap(px(10.))
                .child(
                    div()
                        .id("onboard-add")
                        .h(px(40.))
                        .px(px(18.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .rounded_full()
                        .cursor_pointer()
                        .bg(c.accent)
                        .text_color(c.on_accent)
                        .text_size(px(13.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .shadow(vec![glow(c.accent, 22.)])
                        .on_click(cx.listener(|this, _, window, cx| this.add_game(window, cx)))
                        .child(Icon::Plus.el().size(px(14.)).text_color(c.on_accent))
                        .child("Add a game"),
                )
                .child(
                    div()
                        .id("onboard-scan")
                        .h(px(40.))
                        .px(px(18.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .rounded_full()
                        .cursor_pointer()
                        .border_1()
                        .border_color(c.line_strong)
                        .text_color(c.text)
                        .text_size(px(13.5))
                        .hover(|d| d.bg(c.text.opacity(0.05)))
                        .on_click(cx.listener(|this, _, window, cx| this.rescan(true, window, cx)))
                        .child(Icon::Refresh.el().size(px(14.)).text_color(c.muted))
                        .child("Scan again"),
                ),
        )
        .child(
            div()
                .mt(px(14.))
                .flex()
                .items_center()
                .gap(px(8.))
                .text_size(px(12.))
                .text_color(c.faint)
                .child(kbd("Ctrl N", cx))
                .child("add")
                .child(kbd("Ctrl K", cx))
                .child("search")
                .child(kbd("Ctrl ,", cx))
                .child("settings"),
        );
    div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .child(motion::rise_in(body, "onboarding", 0))
}
