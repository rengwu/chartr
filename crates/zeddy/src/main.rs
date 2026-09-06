//! Chartr — a multi-space agent multiplexer.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use gpui::{
    App, AppContext as _, Bounds, Focusable as _, WindowBounds, WindowOptions, point, px, size,
};
use gpui_platform::application;

mod actions;
#[path = "../../../plugins/agent/src/lib.rs"]
mod agent_plugin;
mod app;
mod assets;
mod browser_plugin;
mod chrome;
mod components;
mod fonts;
#[path = "../../../plugins/hello/src/lib.rs"]
mod hello_plugin;
mod item;
mod keymap;
mod mode;
mod persistence;
mod plugin_installer;
mod plugin_settings;
mod process;
mod session;
mod settings;
mod settings_window;
#[path = "../../../plugins/skills/src/lib.rs"]
mod skills_plugin;
mod space;
mod spaces;
mod terminal_host;
mod text_input;
mod title_bar;
#[path = "../../../plugins/wayfinder/src/lib.rs"]
mod wayfinder_plugin;
mod web_plugin;
mod workspace;

fn main() {
    for error in plugin_installer::activate_pending(&app::plugin_paths()) {
        eprintln!("Could not activate plugin update: {error:#}");
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let opened_path = opened_path_from_args(std::env::args_os(), &cwd);

    application().with_assets(assets::Assets).run(move |cx: &mut App| {
        // Zed's terminal model/view keeps its native emulator settings graph
        // (cursor, scrollback, mouse behavior, and escape-sequence policy).
        // Chartr owns product settings and adapts its terminal typography into
        // Zed's shared theme provider below; it does not duplicate shell or PTY
        // settings that belong to Herdr's persistent session.
        ::settings::init(cx);
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
        cx.set_reduce_motion(settings.resolved().reduce_motion);
        if let Err(error) = zed_assets::Assets.load_fonts(cx) {
            eprintln!("Chartr could not load its bundled fonts: {error}");
        }
        if let Err(error) = fonts::load_bundled(cx) {
            eprintln!("Chartr could not load IBM Plex Mono: {error}");
        }
        // Zed's components read their font through this, and zeddy has no
        // settings file for the `theme_settings` crate to read one from.
        fonts::install(settings.resolved(), cx);
        actions::init(&keymap, cx);
        text_input::init(cx);
        browser_plugin::init(cx);
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
                titlebar: Some(title_bar::options("Chartr")),
                app_owns_titlebar_drag: title_bar::app_owns_drag(),
                ..Default::default()
            },
            |window, cx| {
                let view =
                    cx.new(|cx| app::Zeddy::new(cwd.clone(), opened_path.clone(), window, cx));
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
            let _ = window.update(cx, |zeddy, _, cx| {
                zeddy.apply_exit_policy(cx);
                zeddy.flush_state(cx);
            });
            async {}
        })
        .detach();
        cx.activate(true);
    });
}

/// Return only a folder explicitly passed to Chartr.
///
/// A desktop launcher controls the process working directory; on macOS that is
/// commonly `/`. It is therefore never evidence that the operator opened a
/// project. Relative command-line paths still resolve against the shell's
/// working directory, as users expect from `Chartr .`.
fn opened_path_from_args(args: impl IntoIterator<Item = OsString>, cwd: &Path) -> Option<PathBuf> {
    let mut args = args.into_iter();
    args.next();
    let mut argument = args.next()?;
    if argument == "--" {
        argument = args.next()?;
    }
    // Older macOS launch services may inject this process-serial-number
    // argument. It is launcher metadata, not a path selected by the user.
    if argument.to_string_lossy().starts_with("-psn_") {
        return None;
    }
    let path = PathBuf::from(argument);
    Some(if path.is_absolute() { path } else { cwd.join(path) })
}

#[cfg(test)]
mod launch_tests {
    use super::*;

    #[test]
    fn a_plain_desktop_launch_does_not_open_its_inherited_working_directory() {
        assert_eq!(opened_path_from_args([OsString::from("Chartr")], Path::new("/")), None);
        assert_eq!(
            opened_path_from_args(
                [OsString::from("Chartr"), OsString::from("-psn_0_12345")],
                Path::new("/"),
            ),
            None
        );
    }

    #[test]
    fn an_explicit_relative_path_is_resolved_from_the_shell_directory() {
        assert_eq!(
            opened_path_from_args(
                [OsString::from("Chartr"), OsString::from("project")],
                Path::new("/work"),
            ),
            Some(PathBuf::from("/work/project"))
        );
        assert_eq!(
            opened_path_from_args(
                [OsString::from("Chartr"), OsString::from("--"), OsString::from(".")],
                Path::new("/work"),
            ),
            Some(PathBuf::from("/work/."))
        );
    }
}
