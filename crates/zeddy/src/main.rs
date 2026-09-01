//! Chartr — a multi-space agent multiplexer.

use std::path::PathBuf;

use gpui::{
    App, AppContext as _, Bounds, Focusable as _, WindowBounds, WindowOptions, point, px, size,
};
use gpui_platform::application;

mod actions;
mod app;
mod chrome;
mod fonts;
mod item;
mod keymap;
mod keys;
mod mode;
mod palette;
mod persistence;
mod session;
mod settings;
mod settings_window;
mod space;
mod spaces;
mod terminal;
mod text_input;
mod web_plugin;
mod workspace;

fn main() {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    application().with_assets(zed_assets::Assets).run(move |cx: &mut App| {
        let settings = settings::settings_file()
            .map(settings::SettingsStore::load)
            .unwrap_or_else(|_| settings::SettingsStore::bare());
        let keymap = keymap::keymap_file()
            .map(keymap::KeymapStore::load)
            .unwrap_or_else(|_| keymap::KeymapStore::bare());
        // Keep Zed's assets on the registry for the component and icon layer;
        // `settings::init_themes` registers Chartr's theme catalog as ordinary
        // Zed themes before applying the user-global selection.
        theme::init(theme::LoadThemes::All(Box::new(zed_assets::Assets)), cx);
        settings::init_themes(settings.resolved(), cx);
        if let Err(error) = zed_assets::Assets.load_fonts(cx) {
            eprintln!("Chartr could not load its bundled fonts: {error}");
        }
        if let Err(error) = fonts::load_bundled(cx) {
            eprintln!("Chartr could not load IBM Plex Mono: {error}");
        }
        // Zed's components read their font through this, and zeddy has no
        // settings file for the `theme_settings` crate to read one from.
        theme::set_theme_settings_provider(
            Box::new(fonts::Fonts::from_settings(settings.resolved())),
            cx,
        );
        actions::init(&keymap, cx);
        text_input::init(cx);
        settings_window::init(&keymap, cx);
        cx.set_global(settings.clone());
        cx.set_global(keymap);

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

        let bounds = persistence::state_file()
            .ok()
            .and_then(|path| persistence::StateStore::open(path).ok())
            .and_then(|store| store.load().ok())
            .and_then(|snapshot| snapshot.window.bounds)
            .filter(|bounds| {
                bounds.x.is_finite()
                    && bounds.y.is_finite()
                    && bounds.width.is_finite()
                    && bounds.height.is_finite()
                    && bounds.width >= 640.
                    && bounds.height >= 420.
            })
            .map(|bounds| {
                Bounds::new(
                    point(px(bounds.x), px(bounds.y)),
                    size(px(bounds.width), px(bounds.height)),
                )
            })
            .unwrap_or_else(|| Bounds::centered(None, size(px(1100.), px(720.)), cx));
        let window = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("Chartr".into()),
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

        let window = match window {
            Ok(window) => window,
            Err(err) => {
                eprintln!("zeddy could not open a window: {err}");
                cx.quit();
                return;
            }
        };
        cx.on_app_quit(move |cx| {
            let _ = window.update(cx, |zeddy, _, cx| zeddy.apply_exit_policy(cx));
            async {}
        })
        .detach();
        cx.activate(true);
    });
}
