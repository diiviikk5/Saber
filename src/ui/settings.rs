//! Settings: every change applies live.

use crate::app::Saber;
use crate::assets::Icon;
use crate::theme::{MONO, theme};
use crate::ui::widgets::{glow, kbd, segmented, switch};
use crate::ui::{PAGE_PAD, motion};
use gpui_kit::prelude::*;
use gpui_kit::*;
use saber_core::palette::{ACCENTS, PALETTES, Palette};
use saber_core::settings::{Material, Settings};
use saber_core::storage;

type Edit = fn(&mut Settings);

pub fn settings_page(app: &Saber, cx: &mut Context<Saber>) -> impl IntoElement + use<> {
    let t = *theme(cx);
    let c = t.colors;
    let s = app.settings.clone();
    let weak = cx.entity().downgrade();

    // Runs a settings edit from any callback.
    let edit = move |f: Box<dyn FnOnce(&mut Settings)>, window: &mut Window, cx: &mut App| {
        let _ = weak.update(cx, |this, cx| this.update_settings(window, cx, f));
    };

    let palettes = div().flex().flex_wrap().gap(px(12.)).children(PALETTES.iter().map(|p| {
        let edit = edit.clone();
        let id = p.id;
        palette_card(p, s.theme == p.id, cx).on_click(move |_, window, cx| {
            edit(Box::new(move |s| {
                s.theme = id.into();
                s.accent = None;
            }), window, cx)
        })
    }));

    let accents = {
        let palette_accent = saber_core::palette::by_id(&s.theme).accent;
        let mut row = div().flex().items_center().gap(px(10.));
        let options = std::iter::once((None, palette_accent)).chain(ACCENTS.iter().map(|a| (Some(*a), *a)));
        for (i, (value, color)) in options.enumerate() {
            let selected = s.accent == value;
            let edit = edit.clone();
            let color: Hsla = rgb(color).into();
            row = row.child(
                div()
                    .id(("accent", i))
                    .size(px(24.))
                    .rounded_full()
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(color)
                    .when(selected, |d| d.border_2().border_color(c.text).shadow(vec![glow(color, 14.)]))
                    .when(!selected, |d| d.hover(|d| d.shadow(vec![glow(color, 10.)])))
                    .when(value.is_none(), |d| {
                        d.child(div().size(px(6.)).rounded_full().bg(hsla(0., 0., 0., 0.45)))
                    })
                    .on_click(move |_, window, cx| edit(Box::new(move |s| s.accent = value), window, cx)),
            );
        }
        row
    };

    macro_rules! pick {
        ($edit:expr, $field:ident) => {{
            let edit = $edit.clone();
            move |v, window: &mut Window, cx: &mut App| edit(Box::new(move |s: &mut Settings| s.$field = v), window, cx)
        }};
    }

    let toggle = |id: &'static str, on: bool, f: Edit, cx: &App| {
        let edit = edit.clone();
        switch(id, on, cx).on_click(move |_, window, cx| edit(Box::new(f), window, cx))
    };

    let appearance = section("Appearance", cx)
        .child(block("Theme", "Six moods. The website wears the same ones.", palettes, cx))
        .child(row("Accent", "The color of focus, play buttons and glow. The dotted swatch follows the theme.", accents, cx))
        .child(row(
            "Window material",
            "Mica and Acrylic let your desktop show through (Windows 11).",
            segmented(
                "material",
                &Material::ALL.map(|m| (m, m.label())),
                s.material,
                cx,
                pick!(edit, material),
            ),
            cx,
        ))
        .child(row(
            "Titles",
            "Typeface for game titles and headings.",
            segmented("serif", &[(true, "Serif"), (false, "Sans")], s.serif_titles, cx, pick!(edit, serif_titles)),
            cx,
        ))
        .child(row(
            "Poster size",
            "How big covers are on the shelf.",
            segmented(
                "size",
                &[(136., "S"), (168., "M"), (200., "L"), (236., "XL")],
                s.card_width,
                cx,
                pick!(edit, card_width),
            ),
            cx,
        ))
        .child(row(
            "Corners",
            "From razor to pebble.",
            segmented(
                "radius",
                &[(2., "Sharp"), (8., "Soft"), (14., "Round"), (22., "Pebble")],
                s.radius,
                cx,
                pick!(edit, radius),
            ),
            cx,
        ))
        .child(row("Featured banner", "Show the selected game big above the shelf.", toggle("hero", s.show_hero, |s| s.show_hero = !s.show_hero, cx), cx))
        .child(row("Ambient art", "Let the selected game's art tint the whole window.", toggle("ambient", s.ambient_art, |s| s.ambient_art = !s.ambient_art, cx), cx))
        .child(row("Playtime on posters", "Show hours played under each title.", toggle("playtime", s.show_playtime, |s| s.show_playtime = !s.show_playtime, cx), cx));

    let behavior = section("Behavior", cx)
        .child(row("Minimize on launch", "Get out of the way while you play.", toggle("minimize", s.minimize_on_launch, |s| s.minimize_on_launch = !s.minimize_on_launch, cx), cx))
        .child(row("Scan at startup", "Pick up new Steam and Epic installs automatically.", toggle("scan", s.scan_on_start, |s| s.scan_on_start = !s.scan_on_start, cx), cx));

    let data_dir = storage::data_dir();
    let library = section("Library", cx)
        .child(row(
            "Data folder",
            data_dir.display().to_string(),
            pill_button("open-data", Icon::FolderOpen, "Open", cx)
                .on_click(move |_, _, cx| cx.reveal_path(&data_dir.join("library.json"))),
            cx,
        ))
        .child(row(
            "Rescan",
            "Look for Steam and Epic games now.",
            pill_button("rescan-now", Icon::Refresh, "Scan", cx)
                .on_click(cx.listener(|this, _, window, cx| this.rescan(true, window, cx))),
            cx,
        ))
        .child(row(
            "Reset appearance",
            "Back to Ember and the defaults.",
            pill_button("reset", Icon::Reset, "Reset", cx).on_click({
                let edit = edit.clone();
                move |_, window, cx| {
                    edit(Box::new(|s| {
                        let keep = (s.minimize_on_launch, s.scan_on_start, s.sort);
                        *s = Settings::default();
                        (s.minimize_on_launch, s.scan_on_start, s.sort) = keep;
                    }), window, cx)
                }
            }),
            cx,
        ));

    let shortcuts = [
        ("Ctrl K", "Search"),
        ("Enter", "Play selected"),
        ("Arrows", "Move around the shelf"),
        ("F", "Favorite"),
        ("Ctrl N", "Add a game"),
        ("Ctrl R", "Rescan"),
        ("Ctrl ,", "Settings"),
        ("Esc", "Clear / back"),
    ];
    let keys = section("Keyboard", cx).child(
        div().flex().flex_wrap().gap_y(px(10.)).py(px(14.)).children(shortcuts.into_iter().map(|(k, label)| {
            div()
                .w(relative(0.5))
                .flex()
                .items_center()
                .gap(px(10.))
                .child(div().w(px(64.)).child(kbd(k, cx)))
                .child(div().text_size(px(13.)).text_color(c.muted).child(label))
        })),
    );

    let page = div()
        .w_full()
        .max_w(px(780.))
        .flex()
        .flex_col()
        .gap(px(36.))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .font_family(t.display_font())
                        .text_size(px(40.))
                        .line_height(relative(1.))
                        .text_color(c.text)
                        .child("Make it yours."),
                )
                .child(div().text_size(px(14.)).text_color(c.muted).child("Everything here applies instantly and is saved as you go.")),
        )
        .child(appearance)
        .child(behavior)
        .child(library)
        .child(keys)
        .child(
            div()
                .font_family(MONO)
                .text_size(px(11.))
                .text_color(c.faint)
                .child(format!("Saber {} · Rust + GPUI", env!("CARGO_PKG_VERSION"))),
        );

    div()
        .id("settings")
        .size_full()
        .overflow_y_scroll()
        .child(
            div()
                .flex()
                .justify_center()
                .px(px(PAGE_PAD))
                .pt(px(18.))
                .pb(px(64.))
                .child(motion::rise_in(page, "settings-in", 0)),
        )
}

fn section(title: &str, cx: &App) -> Div {
    let c = theme(cx).colors;
    div().flex().flex_col().child(
        div()
            .pb(px(6.))
            .border_b_1()
            .border_color(c.line)
            .font_family(MONO)
            .text_size(px(11.))
            .text_color(c.accent)
            .child(title.to_uppercase()),
    )
}

fn label(title: &str, hint: impl Into<SharedString>, cx: &App) -> Div {
    let c = theme(cx).colors;
    div()
        .flex()
        .flex_col()
        .gap(px(2.))
        .min_w_0()
        .child(div().text_size(px(14.)).text_color(c.text).child(title.to_string()))
        .child(div().text_size(px(12.5)).text_color(c.muted).truncate().child(hint.into()))
}

fn row(title: &str, hint: impl Into<SharedString>, control: impl IntoElement, cx: &App) -> Div {
    let c = theme(cx).colors;
    div()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(24.))
        .py(px(14.))
        .border_b_1()
        .border_color(c.line)
        .child(label(title, hint, cx).flex_1())
        .child(div().flex_none().child(control))
}

fn block(title: &str, hint: &str, content: impl IntoElement, cx: &App) -> Div {
    let c = theme(cx).colors;
    div()
        .flex()
        .flex_col()
        .gap(px(14.))
        .py(px(16.))
        .border_b_1()
        .border_color(c.line)
        .child(label(title, hint.to_string(), cx))
        .child(content)
}

/// A miniature of the palette: background, a panel, a poster row, the accent.
fn palette_card(p: &Palette, selected: bool, cx: &App) -> Stateful<Div> {
    let c = theme(cx).colors;
    let bg: Hsla = rgb(p.bg).into();
    let surface: Hsla = rgb(p.surface_2).into();
    let accent: Hsla = rgb(p.accent).into();
    let text: Hsla = rgb(p.text).into();
    div()
        .id(SharedString::from(format!("palette-{}", p.id)))
        .w(px(116.))
        .flex()
        .flex_col()
        .gap(px(8.))
        .cursor_pointer()
        .child(
            div()
                .h(px(76.))
                .rounded(px(10.))
                .overflow_hidden()
                .bg(bg)
                .border_1()
                .border_color(if selected { accent } else { c.line_strong })
                .when(selected, |d| d.shadow(vec![glow(accent, 16.)]))
                .flex()
                .child(div().w(px(26.)).h_full().bg(surface.opacity(0.6)).child(
                    div().mt(px(12.)).ml(px(6.)).w(px(3.)).h(px(9.)).rounded(px(2.)).bg(accent),
                ))
                .child(
                    div()
                        .flex_1()
                        .p(px(8.))
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(div().h(px(14.)).rounded(px(3.)).bg(surface))
                        .child(
                            div()
                                .flex()
                                .gap(px(4.))
                                .children((0..4).map(|i| {
                                    div()
                                        .w(px(14.))
                                        .h(px(21.))
                                        .rounded(px(2.))
                                        .bg(if i == 0 { accent.opacity(0.85) } else { text.opacity(0.12) })
                                })),
                        ),
                ),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .text_size(px(12.5))
                .text_color(if selected { c.text } else { c.muted })
                .when(selected, |d| d.child(Icon::Check.el().size(px(12.)).text_color(c.accent)))
                .child(p.name),
        )
}

fn pill_button(id: &'static str, icon: Icon, label: &'static str, cx: &App) -> Stateful<Div> {
    let c = theme(cx).colors;
    div()
        .id(id)
        .h(px(30.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(7.))
        .rounded_full()
        .border_1()
        .border_color(c.line_strong)
        .cursor_pointer()
        .text_size(px(12.5))
        .text_color(c.text)
        .hover(|d| d.bg(c.text.opacity(0.05)))
        .child(icon.el().size(px(13.)).text_color(c.muted))
        .child(label)
}
