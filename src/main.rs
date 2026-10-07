// Release builds are a GUI app; don't pop a console window on Windows.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod actions;
mod app;
mod assets;
mod theme;
mod ui;

use gpui_kit::component::TitleBar;
use gpui_kit::*;

fn main() {
    gpui_kit::application()
        .with_assets(assets::SaberAssets)
        .run(|cx| {
            gpui_kit::init(cx);
            actions::bind(cx);

            let bounds = Bounds::centered(None, size(px(1320.), px(840.)), cx);
            let options = WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(860.), px(560.))),
                app_id: Some("saber".into()),
                ..TitleBar::window_options()
            };
            gpui_kit::open_window(options, cx, |window, cx| {
                window.set_window_title("Saber");
                cx.new(|cx| app::Saber::new(window, cx))
            })
            .expect("failed to open the Saber window");
            cx.activate(true);
        });
}
