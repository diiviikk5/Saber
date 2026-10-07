//! The right-click menu for a game.

use crate::app::Saber;
use crate::assets::Icon;
use crate::theme::theme;
use crate::ui::motion;
use crate::ui::widgets::lift_shadow;
use gpui_kit::prelude::*;
use gpui_kit::*;

pub struct MenuState {
    pub game_id: String,
    pub position: Point<Pixels>,
}

impl Saber {
    pub fn open_menu(&mut self, game_id: String, position: Point<Pixels>, cx: &mut Context<Self>) {
        self.menu = Some(MenuState { game_id, position });
        cx.notify();
    }

    pub fn close_menu(&mut self, cx: &mut Context<Self>) {
        if self.menu.take().is_some() {
            cx.notify();
        }
    }
}

#[derive(Clone, Copy)]
enum Entry {
    Play,
    Favorite,
    Cover,
    Folder,
    Hide,
    Remove,
}

pub fn menu(app: &Saber, cx: &mut Context<Saber>) -> Option<impl IntoElement + use<>> {
    let state = app.menu.as_ref()?;
    let game = app.library.get(&state.game_id)?;
    let c = theme(cx).colors;

    let entries = [
        (Entry::Play, Icon::Play, "Play".to_string(), false),
        (
            Entry::Favorite,
            if game.favorite {
                Icon::StarFill
            } else {
                Icon::Star
            },
            if game.favorite {
                "Remove from favorites"
            } else {
                "Add to favorites"
            }
            .to_string(),
            false,
        ),
        (
            Entry::Cover,
            Icon::Image,
            "Change cover…".to_string(),
            false,
        ),
        (
            Entry::Folder,
            Icon::FolderOpen,
            "Open install folder".to_string(),
            game.install_dir.is_none(),
        ),
        (
            Entry::Hide,
            if game.hidden { Icon::Eye } else { Icon::EyeOff },
            if game.hidden { "Show on shelf" } else { "Hide" }.to_string(),
            false,
        ),
        (
            Entry::Remove,
            Icon::Trash,
            "Remove from Saber".to_string(),
            false,
        ),
    ];

    let id = state.game_id.clone();
    let panel =
        div()
            .id("game-menu")
            .w(px(232.))
            .p(px(5.))
            .flex()
            .flex_col()
            .rounded(px(12.))
            .bg(c.surface)
            .border_1()
            .border_color(c.line_strong)
            .shadow(vec![lift_shadow(c.dark)])
            .on_mouse_down_out(cx.listener(|this, _, _, cx| this.close_menu(cx)))
            .child(
                div()
                    .px(px(10.))
                    .pt(px(6.))
                    .pb(px(8.))
                    .text_size(px(12.))
                    .text_color(c.faint)
                    .truncate()
                    .child(game.title.clone()),
            )
            .children(entries.into_iter().enumerate().map(
                |(i, (entry, icon, label, disabled))| {
                    let id = id.clone();
                    let danger = matches!(entry, Entry::Remove);
                    div()
                        .id(("menu-item", i))
                        .h(px(32.))
                        .px(px(10.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .rounded(px(7.))
                        .text_size(px(13.))
                        .when(danger, |d| d.mt(px(4.)))
                        .when(disabled, |d| d.opacity(0.4))
                        .when(!disabled, |d| {
                            d.cursor_pointer()
                                .hover(|d| {
                                    d.bg(if danger {
                                        hsla(0., 0.8, 0.55, 0.14)
                                    } else {
                                        c.text.opacity(0.06)
                                    })
                                })
                                .on_click(cx.listener(move |this, _, window, cx| {
                                    this.close_menu(cx);
                                    match entry {
                                        Entry::Play => this.play(&id, window, cx),
                                        Entry::Favorite => this.toggle_favorite(&id, cx),
                                        Entry::Cover => this.change_cover(&id, window, cx),
                                        Entry::Folder => this.open_folder(&id, cx),
                                        Entry::Hide => this.toggle_hidden(&id, window, cx),
                                        Entry::Remove => this.remove(&id, window, cx),
                                    }
                                }))
                        })
                        .text_color(if danger {
                            hsla(0.01, 0.85, 0.66, 1.)
                        } else {
                            c.text
                        })
                        .child(icon.el().size(px(14.)).text_color(if danger {
                            hsla(0.01, 0.85, 0.66, 1.)
                        } else {
                            c.muted
                        }))
                        .child(label)
                },
            ));

    Some(
        deferred(
            anchored()
                .position(state.position)
                .snap_to_window_with_margin(px(8.))
                .child(motion::fade_in(panel, "menu-in", 0.12)),
        )
        .with_priority(10),
    )
}
