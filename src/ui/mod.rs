//! Rendering. `Saber`'s state lives in `app.rs`; this is how it looks.

pub mod card;
pub mod hero;
pub mod menu;
pub mod motion;
pub mod settings;
pub mod shelf;
pub mod sidebar;
pub mod widgets;

use crate::actions::SHELF;
use crate::app::{Page, Saber};
use crate::assets::Icon;
use crate::theme::theme;
use gpui_kit::component::TitleBar;
use gpui_kit::component::input::Input;
use gpui_kit::prelude::*;
use gpui_kit::*;
use saber_core::settings::Material;

pub const SIDEBAR_WIDTH: f32 = 232.;
pub const PAGE_PAD: f32 = 32.;
pub const GRID_GAP: f32 = 20.;
pub const TITLE_HEIGHT: f32 = 52.;

/// The Saber mark: a hilt and a blade of accent light.
pub fn blade_mark(size: Pixels, cx: &App) -> impl IntoElement {
    let c = theme(cx).colors;
    let s = f32::from(size);
    // Drawn with two rotated-looking bars faked by a skewed gradient isn't
    // possible in GPUI, so build it from a canvas path instead.
    canvas(
        |_, _, _| {},
        move |bounds, _, window, _| {
            let o = bounds.origin;
            let at = |x: f32, y: f32| point(o.x + px(x * s), o.y + px(y * s));
            let line = |from: Point<Pixels>, to: Point<Pixels>, width: f32| {
                let mut b = PathBuilder::stroke(px(width * s));
                b.move_to(from);
                b.line_to(to);
                b.build().ok()
            };
            if let Some(blade) = line(at(0.3, 0.7), at(0.76, 0.24), 0.075) {
                window.paint_path(blade, c.accent);
            }
            if let Some(hilt) = line(at(0.24, 0.76), at(0.36, 0.64), 0.11) {
                window.paint_path(hilt, c.text);
            }
        },
    )
    .size(size)
    .flex_none()
}

impl Render for Saber {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = *theme(cx);
        let c = t.colors;
        let translucent = self.settings.material != Material::Solid;

        let page = match self.page {
            Page::Library => shelf::shelf(self, cx),
            Page::Settings => settings::settings_page(self, cx).into_any_element(),
        };

        let root = div()
            .id("saber")
            .track_focus(&self.focus)
            .key_context(SHELF)
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(if translucent {
                c.bg.opacity(if c.dark { 0.72 } else { 0.8 })
            } else {
                c.bg
            })
            .text_color(c.text)
            .font_family(crate::theme::SANS)
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| this.close_menu(cx)),
            )
            .when_some(self.ambient(cx), |d, a| d.child(a))
            .child(self.title_bar(window, cx))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .flex()
                    .child(sidebar::sidebar(self, cx))
                    .child(div().flex_1().min_w_0().h_full().child(page)),
            )
            .children(menu::menu(self, cx));

        self.register_actions(root, cx)
    }
}

impl Saber {
    /// The selected game's art, huge and faint behind everything.
    fn ambient(&self, cx: &App) -> Option<AnyElement> {
        if !self.settings.ambient_art {
            return None;
        }
        let c = theme(cx).colors;
        let game = self.selected_game()?;
        let art = game.hero.clone().or_else(|| game.cover.clone())?;
        // Steam ships a pre-blurred hero; it's perfect for this.
        let blurred = art
            .file_name()
            .filter(|n| *n == "library_hero.jpg")
            .map(|_| art.with_file_name("library_hero_blur.jpg"))
            .filter(|p| p.is_file());
        let source = blurred.clone().unwrap_or(art);
        let strength = match (blurred.is_some(), c.dark) {
            (true, true) => 0.42,
            (true, false) => 0.28,
            (false, true) => 0.16,
            (false, false) => 0.1,
        };
        let layer = div()
            .absolute()
            .inset_0()
            .child(
                img(source)
                    .absolute()
                    .inset_0()
                    .size_full()
                    .object_fit(ObjectFit::Cover)
                    .opacity(strength),
            )
            .child(div().absolute().inset_0().bg(linear_gradient(
                180.,
                linear_color_stop(c.bg.opacity(0.15), 0.),
                linear_color_stop(c.bg, 0.72),
            )))
            .child(div().absolute().inset_0().bg(linear_gradient(
                90.,
                linear_color_stop(c.bg.opacity(0.85), 0.),
                linear_color_stop(c.bg.opacity(0.), 0.35),
            )));
        let key = saber_core::game::hash(&game.id) as usize;
        Some(motion::fade_in(layer, ("ambient", key), 0.6).into_any_element())
    }

    fn title_bar(&self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement + use<> {
        let c = theme(cx).colors;
        let searching = !self.query.is_empty();
        TitleBar::new()
            .h(px(TITLE_HEIGHT))
            .bg(gpui_kit::transparent_black())
            .border_b_0()
            .child(
                div()
                    .w_full()
                    .h_full()
                    .flex()
                    .items_center()
                    .child(
                        div()
                            .w(px(SIDEBAR_WIDTH - 12.))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .child(blade_mark(px(20.), cx))
                            .child(
                                div()
                                    .text_size(px(14.5))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(c.text)
                                    .child("Saber"),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .justify_center()
                            .px(px(PAGE_PAD))
                            .child(
                                div()
                                    .w_full()
                                    .max_w(px(520.))
                                    .h(px(34.))
                                    .rounded(px(10.))
                                    .bg(c.text.opacity(if searching { 0.08 } else { 0.05 }))
                                    .border_1()
                                    .border_color(if searching { c.accent_a(0.5) } else { c.line })
                                    .flex()
                                    .items_center()
                                    .child(
                                        Input::new(&self.search)
                                            .appearance(false)
                                            .cleanable(true)
                                            .prefix(
                                                Icon::Search
                                                    .el()
                                                    .size(px(14.))
                                                    .text_color(c.faint)
                                                    .ml(px(4.)),
                                            )
                                            .suffix(widgets::kbd("Ctrl K", cx))
                                            .text_size(px(13.)),
                                    ),
                            ),
                    )
                    // Room for the system window controls.
                    .child(div().w(px(150.)).flex_none()),
            )
    }
}
