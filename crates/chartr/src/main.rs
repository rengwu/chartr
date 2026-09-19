//! chartr — a multi-space agent multiplexer.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use gpui::{
    App, AppContext as _, Bounds, Focusable as _, WindowBounds, WindowOptions, point, px, size,
};
use gpui_platform::application;

mod actions;
mod agent_icons;
#[path = "../../../plugins/agent/src/lib.rs"]
mod agent_plugin;
mod app;
mod assets;
mod chrome;
mod components;
mod conversations;
mod fonts;
mod item;
mod keymap;
#[path = "../../../plugins/markdown-prompt/src/lib.rs"]
mod markdown_prompt_plugin;
mod mode;
mod native_plugin;
#[cfg(any(target_os = "macos", target_os = "linux"))]
mod native_webview;
mod persistence;
mod plugin_installer;
mod plugin_settings;
mod process;
#[path = "../../../plugins/prompts/src/lib.rs"]
mod prompts_plugin;
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
    #[cfg(target_os = "linux")]
    if Path::new("/sys/module/nvidia").is_dir()
        && std::env::var_os("WEBKIT_DMABUF_RENDERER_FORCE_SHM").is_none()
        && std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none()
    {
        // NVIDIA's X11 GBM buffers can leave WebKit views black. Keep its
        // compositor enabled, but use shared-memory buffers by default.
        // SAFETY: this is the first action in main, before GTK, GPUI, or any
        // application threads start and can read the process environment.
        unsafe { std::env::set_var("WEBKIT_DMABUF_RENDERER_FORCE_SHM", "1") };
    }

    // GPUI uses X11 on Linux. Wry's child views must use the same backend,
    // including when the desktop prefers GTK's Wayland backend.
    #[cfg(target_os = "linux")]
    gtk::gdk::set_allowed_backends("x11");

    for error in plugin_installer::activate_pending(&app::plugin_paths()) {
        eprintln!("Could not activate plugin update: {error:#}");
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let opened_path = opened_path_from_args(std::env::args_os(), &cwd);

    application().with_assets(assets::Assets).run(move |cx: &mut App| {
        // Zed's terminal model/view keeps its native emulator settings graph
        // (cursor, scrollback, mouse behavior, and escape-sequence policy).
        // chartr owns product settings and adapts its terminal typography into
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
        // `settings::init_themes` registers chartr's theme catalog as ordinary
        // Zed themes before applying the user-global selection.
        theme::init(theme::LoadThemes::All(Box::new(zed_assets::Assets)), cx);
        settings::init_themes(settings.resolved(), cx);
        cx.set_reduce_motion(settings.resolved().reduce_motion);
        if let Err(error) = zed_assets::Assets.load_fonts(cx) {
            eprintln!("chartr could not load its bundled fonts: {error}");
        }
        if let Err(error) = fonts::load_bundled(cx) {
            eprintln!("chartr could not load its interface and terminal fonts: {error}");
        }
        // Zed's components read their font through this, and chartr has no
        // settings file for the `theme_settings` crate to read one from.
        fonts::install(settings.resolved(), cx);
        actions::init(&keymap, cx);
        text_input::init(cx);
        prompts_plugin::init(cx);
        settings_window::init(&keymap, cx);
        cx.set_global(settings.clone());
        cx.set_global(keymap);

        // A build whose platform layer cannot rasterise glyphs paints every
        // quad and icon correctly and shows not one character. Saying so is
        // better than opening that window.
        if !fonts::text_renders(cx) {
            eprintln!(
                "chartr cannot render text: this build's GPUI platform layer has no font \
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
                titlebar: Some(title_bar::options("chartr")),
                app_owns_titlebar_drag: title_bar::app_owns_drag(),
                ..Default::default()
            },
            |window, cx| {
                let view = cx.new(|cx| {
                    app::WorkspaceWindow::new(cwd.clone(), opened_path.clone(), window, cx)
                });
                window.focus(&view.read(cx).focus_handle(cx), cx);
                view
            },
        );

        let window = match window {
            Ok(window) => window,
            Err(err) => {
                eprintln!("chartr could not open a window: {err}");
                cx.quit();
                return;
            }
        };
        cx.on_app_quit(move |cx| {
            native_plugin::prepare_to_quit();
            let _ = window.update(cx, |chartr, _, cx| {
                chartr.apply_exit_policy(cx);
                chartr.flush_state(cx);
            });
            async {}
        })
        .detach();
        cx.activate(true);
    });
    native_plugin::shutdown();
}

/// Return only a folder explicitly passed to chartr.
///
/// A desktop launcher controls the process working directory; on macOS that is
/// commonly `/`. It is therefore never evidence that the operator opened a
/// project. Relative command-line paths still resolve against the shell's
/// working directory, as users expect from `chartr .`.
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
        assert_eq!(opened_path_from_args([OsString::from("chartr")], Path::new("/")), None);
        assert_eq!(
            opened_path_from_args(
                [OsString::from("chartr"), OsString::from("-psn_0_12345")],
                Path::new("/"),
            ),
            None
        );
    }

    #[test]
    fn an_explicit_relative_path_is_resolved_from_the_shell_directory() {
        assert_eq!(
            opened_path_from_args(
                [OsString::from("chartr"), OsString::from("project")],
                Path::new("/work"),
            ),
            Some(PathBuf::from("/work/project"))
        );
        assert_eq!(
            opened_path_from_args(
                [OsString::from("chartr"), OsString::from("--"), OsString::from(".")],
                Path::new("/work"),
            ),
            Some(PathBuf::from("/work/."))
        );
    }
}
