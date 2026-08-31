//! zeddy — a simple agent multiplexer.

use std::path::PathBuf;

use gpui::{App, AppContext as _, Bounds, Focusable as _, WindowBounds, WindowOptions, px, size};
use gpui_platform::application;

mod app;
mod assets;
mod chrome;
mod fonts;
mod keys;
mod mode;
mod palette;
mod session;
mod terminal;

fn main() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    application().with_assets(assets::Assets).run(move |cx: &mut App| {
        // `JustBase` loads no theme JSON, which means no asset source and no
        // bundled themes. zeddy has no theme picker, so the built-in dark theme
        // is the whole theming story until it does.
        theme::init(theme::LoadThemes::JustBase, cx);
        // Zed's components read their font through this, and zeddy has no
        // settings file for the `theme_settings` crate to read one from.
        theme::set_theme_settings_provider(Box::new(fonts::Fonts::default()), cx);

        // A build whose platform layer cannot rasterise glyphs paints every
        // quad and icon correctly and shows not one character. Saying so is
        // better than opening that window.
        if !fonts::text_renders(cx) {
            eprintln!(
                "zeddy cannot render text: this build's GPUI platform layer has no font \
                 backend. Check that `gpui_platform` is built with the `font-kit` feature."
            );
            cx.quit();
            return;
        }

        let bounds = Bounds::centered(None, size(px(1100.), px(720.)), cx);
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("zeddy".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| app::Zeddy::new(cwd.clone(), cx));
                window.focus(&view.read(cx).focus_handle(cx), cx);
                view
            },
        );

        if let Err(err) = window {
            eprintln!("zeddy could not open a window: {err}");
            cx.quit();
            return;
        }
        cx.activate(true);
    });
}
