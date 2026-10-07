//! The root view: owns the library and settings, and wires user intent
//! (clicks, keys) to changes in them.

use crate::actions::*;
use crate::theme::SaberTheme;
use gpui_kit::component::WindowExt as _;
use gpui_kit::component::input::{InputEvent, InputState};
use gpui_kit::component::notification::Notification;
use gpui_kit::*;
use saber_core::format;
use saber_core::game::Game;
use saber_core::launch::{self, Started};
use saber_core::library::{Library, Sort, View};
use saber_core::settings::{Material, Settings};
use saber_core::{manual, scan, storage};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    Library,
    Settings,
}

/// A game Saber started and is still timing.
pub struct Session {
    pub game_id: String,
    pub title: String,
    pub started: Instant,
    pub started_at: u64,
    pub tracked: bool,
}

pub struct Saber {
    pub library: Library,
    pub settings: Settings,
    pub page: Page,
    pub view: View,
    pub selected: Option<String>,
    pub search: Entity<InputState>,
    pub query: String,
    pub session: Option<Session>,
    pub scanning: bool,
    pub focus: FocusHandle,
    /// Bumped whenever the shelf contents change, to replay its entrance.
    pub shelf_epoch: usize,
    pub menu: Option<crate::ui::menu::MenuState>,
    _subscriptions: Vec<Subscription>,
}

impl Saber {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let library: Library = storage::load_or_recover(&storage::library_path());
        let settings: Settings = storage::load_or_recover::<Settings>(&storage::settings_path()).clamped();
        SaberTheme::from_settings(&settings).apply(cx);
        apply_material(settings.material, window);

        let search = cx.new(|cx| InputState::new(window, cx).placeholder("Search your library"));
        let sub = cx.subscribe(&search, |this: &mut Self, input, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.query = input.read(cx).value().to_string();
                this.page = Page::Library;
                this.shelf_epoch += 1;
                cx.notify();
            }
        });

        let selected = library.featured().map(|g| g.id.clone());
        let scan_on_start = settings.scan_on_start;
        let mut this = Self {
            library,
            settings,
            page: Page::Library,
            view: View::All,
            selected,
            search,
            query: String::new(),
            session: None,
            scanning: false,
            focus: cx.focus_handle(),
            shelf_epoch: 0,
            menu: None,
            _subscriptions: vec![sub],
        };
        if scan_on_start || this.library.is_empty() {
            this.rescan(false, window, cx);
        }
        this.focus.focus(window, cx);
        this
    }

    // ---- queries ------------------------------------------------------

    pub fn visible(&self) -> Vec<&Game> {
        self.library.query(&self.view, &self.query, self.settings.sort)
    }

    pub fn selected_game(&self) -> Option<&Game> {
        self.selected
            .as_deref()
            .and_then(|id| self.library.get(id))
            .or_else(|| self.library.featured())
    }

    pub fn is_running(&self, id: &str) -> bool {
        self.session.as_ref().is_some_and(|s| s.game_id == id)
    }

    // ---- persistence --------------------------------------------------

    pub fn save_library(&self) {
        if let Err(e) = storage::save(&storage::library_path(), &self.library) {
            eprintln!("saber: failed to save library: {e}");
        }
    }

    pub fn save_settings(&self) {
        if let Err(e) = storage::save(&storage::settings_path(), &self.settings) {
            eprintln!("saber: failed to save settings: {e}");
        }
    }

    /// Applies a settings change everywhere: theme, window, disk.
    pub fn update_settings(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
        edit: impl FnOnce(&mut Settings),
    ) {
        let before = self.settings.material;
        edit(&mut self.settings);
        self.settings = self.settings.clone().clamped();
        SaberTheme::from_settings(&self.settings).apply(cx);
        if before != self.settings.material {
            apply_material(self.settings.material, window);
        }
        self.save_settings();
        window.refresh();
        cx.notify();
    }

    // ---- navigation ---------------------------------------------------

    pub fn set_view(&mut self, view: View, cx: &mut Context<Self>) {
        self.page = Page::Library;
        if self.view != view {
            self.view = view;
            self.shelf_epoch += 1;
        }
        cx.notify();
    }

    pub fn select(&mut self, id: &str, cx: &mut Context<Self>) {
        if self.selected.as_deref() != Some(id) {
            self.selected = Some(id.to_string());
            cx.notify();
        }
    }

    pub fn set_sort(&mut self, sort: Sort, window: &mut Window, cx: &mut Context<Self>) {
        self.shelf_epoch += 1;
        self.update_settings(window, cx, |s| s.sort = sort);
    }

    // ---- game actions -------------------------------------------------

    pub fn toggle_favorite(&mut self, id: &str, cx: &mut Context<Self>) {
        if let Some(game) = self.library.get_mut(id) {
            game.favorite = !game.favorite;
            self.save_library();
            cx.notify();
        }
    }

    pub fn toggle_hidden(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(game) = self.library.get_mut(id) else { return };
        game.hidden = !game.hidden;
        let msg = if game.hidden {
            format!("{} hidden — find it under Hidden", game.title)
        } else {
            format!("{} is back on the shelf", game.title)
        };
        self.save_library();
        window.push_notification(Notification::new().message(msg), cx);
        cx.notify();
    }

    pub fn remove(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(game) = self.library.remove(id) {
            if self.selected.as_deref() == Some(id) {
                self.selected = None;
            }
            self.save_library();
            window.push_notification(
                Notification::new().message(format!("Removed {}", game.title)),
                cx,
            );
            cx.notify();
        }
    }

    pub fn open_folder(&self, id: &str, cx: &mut App) {
        if let Some(dir) = self.library.get(id).and_then(|g| g.install_dir.clone()) {
            cx.reveal_path(&dir);
        }
    }

    pub fn play(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(game) = self.library.get(id).cloned() else { return };
        if self.session.is_some() {
            window.push_notification(
                Notification::new().message("A game is already running"),
                cx,
            );
            return;
        }
        match launch::launch(&game) {
            Ok(started) => {
                let now = format::now();
                self.library.record_launch(&game.id, now);
                self.save_library();
                let tracked = matches!(started, Started::Tracked(_));
                self.session = Some(Session {
                    game_id: game.id.clone(),
                    title: game.title.clone(),
                    started: Instant::now(),
                    started_at: now,
                    tracked,
                });
                self.selected = Some(game.id.clone());
                self.watch_session(started, window, cx);
                if self.settings.minimize_on_launch {
                    window.minimize_window();
                }
                cx.notify();
            }
            Err(err) => window.push_notification(
                Notification::error(format!("Couldn't start {}: {err}", game.title)),
                cx,
            ),
        }
    }

    /// Waits for a tracked game to exit and books the time; for hand-offs,
    /// keeps the "now playing" pill up briefly so the click feels answered.
    fn watch_session(&mut self, started: Started, window: &mut Window, cx: &mut Context<Self>) {
        let ticker = cx.spawn_in(window, async move |this, cx| {
            match started {
                Started::Tracked(mut child) => {
                    let wait = cx.background_spawn(async move { child.wait() });
                    // Repaint once a second so the session clock ticks.
                    let mut wait = std::pin::pin!(wait);
                    loop {
                        let tick = cx.background_executor().timer(Duration::from_secs(1));
                        match futures_lite_select(&mut wait, tick).await {
                            Some(_) => break,
                            None => {
                                if this.update(cx, |_, cx| cx.notify()).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                }
                Started::Handed => {
                    cx.background_executor().timer(Duration::from_secs(8)).await;
                }
            }
            let _ = this.update_in(cx, |this, window, cx| this.end_session(window, cx));
        });
        ticker.detach();
    }

    pub fn end_session(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(session) = self.session.take() else { return };
        if session.tracked {
            let secs = session.started.elapsed().as_secs();
            self.library.record_session(&session.game_id, session.started_at, secs);
            self.save_library();
            window.push_notification(
                Notification::new().message(format!(
                    "{} · {} this session",
                    session.title,
                    format::playtime(secs)
                )),
                cx,
            );
        }
        cx.notify();
    }

    // ---- library maintenance ------------------------------------------

    pub fn rescan(&mut self, announce: bool, window: &mut Window, cx: &mut Context<Self>) {
        if self.scanning {
            return;
        }
        self.scanning = true;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let found = cx.background_spawn(async move { scan::all() }).await;
            let _ = this.update_in(cx, |this, window, cx| {
                this.scanning = false;
                let added = found
                    .into_iter()
                    .filter(|g| this.library.upsert(g.clone()))
                    .count();
                if this.selected.is_none() {
                    this.selected = this.library.featured().map(|g| g.id.clone());
                }
                this.save_library();
                this.shelf_epoch += 1;
                if announce || added > 0 {
                    let msg = match added {
                        0 => "Library is up to date".to_string(),
                        1 => "Found 1 new game".to_string(),
                        n => format!("Found {n} new games"),
                    };
                    window.push_notification(Notification::new().message(msg), cx);
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub fn add_game(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: true,
            prompt: Some("Add to Saber".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else { return };
            let _ = this.update_in(cx, |this, window, cx| {
                let mut last = None;
                for path in paths {
                    let game = manual::game_from_exe(path);
                    last = Some(game.id.clone());
                    this.library.upsert(game);
                }
                if let Some(id) = last {
                    this.selected = Some(id);
                }
                this.save_library();
                this.page = Page::Library;
                this.shelf_epoch += 1;
                window.push_notification(Notification::new().message("Added to your shelf"), cx);
                cx.notify();
            });
        })
        .detach();
    }

    /// Lets the user pick a cover image; it's copied into Saber's art folder.
    pub fn change_cover(&mut self, id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let id = id.to_string();
        let paths = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Use as cover".into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = paths.await else { return };
            let Some(src) = paths.into_iter().next() else { return };
            let _ = this.update_in(cx, |this, window, cx| {
                let ext = src.extension().and_then(|e| e.to_str()).unwrap_or("png");
                let dest = storage::art_dir().join(format!("{id}-cover.{ext}"));
                let copied = std::fs::create_dir_all(storage::art_dir())
                    .and_then(|_| std::fs::copy(&src, &dest));
                match copied {
                    Ok(_) => {
                        if let Some(game) = this.library.get_mut(&id) {
                            game.cover = Some(dest);
                        }
                        this.save_library();
                        cx.notify();
                    }
                    Err(e) => window.push_notification(
                        Notification::error(format!("Couldn't use that image: {e}")),
                        cx,
                    ),
                }
            });
        })
        .detach();
    }

    // ---- keyboard -----------------------------------------------------

    pub fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        let ids: Vec<String> = self.visible().iter().map(|g| g.id.clone()).collect();
        if ids.is_empty() {
            return;
        }
        let current = self
            .selected
            .as_ref()
            .and_then(|id| ids.iter().position(|i| i == id));
        let next = match current {
            Some(i) => (i as isize + delta).clamp(0, ids.len() as isize - 1) as usize,
            None => 0,
        };
        self.selected = Some(ids[next].clone());
        cx.notify();
    }

    pub fn columns(&self, window: &Window) -> usize {
        let available = window.viewport_size().width - px(crate::ui::SIDEBAR_WIDTH + 2. * crate::ui::PAGE_PAD);
        let card = self.settings.card_width + crate::ui::GRID_GAP;
        ((f32::from(available) + crate::ui::GRID_GAP) / card).floor().max(1.) as usize
    }

    pub fn register_actions(&self, el: Stateful<Div>, cx: &mut Context<Self>) -> Stateful<Div> {
        el.on_action(cx.listener(|this, _: &FocusSearch, window, cx| {
            this.page = Page::Library;
            this.search.update(cx, |s, cx| s.focus(window, cx));
            cx.notify();
        }))
        .on_action(cx.listener(|this, _: &Escape, window, cx| {
            if !this.query.is_empty() {
                this.search.update(cx, |s, cx| s.set_value("", window, cx));
                this.query.clear();
                this.shelf_epoch += 1;
            } else if this.page == Page::Settings {
                this.page = Page::Library;
            }
            this.focus.focus(window, cx);
            cx.notify();
        }))
        .on_action(cx.listener(|this, _: &OpenSettings, _, cx| {
            this.page = if this.page == Page::Settings { Page::Library } else { Page::Settings };
            cx.notify();
        }))
        .on_action(cx.listener(|this, _: &AddGame, window, cx| this.add_game(window, cx)))
        .on_action(cx.listener(|this, _: &Rescan, window, cx| this.rescan(true, window, cx)))
        .on_action(cx.listener(|this, _: &PlaySelected, window, cx| {
            if let Some(id) = this.selected_game().map(|g| g.id.clone()) {
                this.play(&id, window, cx);
            }
        }))
        .on_action(cx.listener(|this, _: &ToggleFavorite, _, cx| {
            if let Some(id) = this.selected_game().map(|g| g.id.clone()) {
                this.toggle_favorite(&id, cx);
            }
        }))
        .on_action(cx.listener(|this, _: &SelectNext, _, cx| this.move_selection(1, cx)))
        .on_action(cx.listener(|this, _: &SelectPrev, _, cx| this.move_selection(-1, cx)))
        .on_action(cx.listener(|this, _: &SelectDown, window, cx| {
            let cols = this.columns(window) as isize;
            this.move_selection(cols, cx)
        }))
        .on_action(cx.listener(|this, _: &SelectUp, window, cx| {
            let cols = this.columns(window) as isize;
            this.move_selection(-cols, cx)
        }))
    }
}

fn apply_material(material: Material, window: &mut Window) {
    window.set_background_appearance(match material {
        Material::Solid => WindowBackgroundAppearance::Opaque,
        Material::Mica => WindowBackgroundAppearance::MicaBackdrop,
        Material::Acrylic => WindowBackgroundAppearance::Blurred,
    });
}

/// Resolves with `Some` when `a` finishes first, `None` when `b` does.
async fn futures_lite_select<A, B>(a: &mut std::pin::Pin<&mut A>, b: B) -> Option<A::Output>
where
    A: std::future::Future,
    B: std::future::Future<Output = ()>,
{
    use std::task::Poll;
    let mut b = std::pin::pin!(b);
    std::future::poll_fn(|cx| {
        if let Poll::Ready(v) = a.as_mut().poll(cx) {
            return Poll::Ready(Some(v));
        }
        if b.as_mut().poll(cx).is_ready() {
            return Poll::Ready(None);
        }
        Poll::Pending
    })
    .await
}
