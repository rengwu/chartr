//! Exercise actual catalog contributions so new bundled settings join this check.

use super::*;
use gpui::{AnyView, Render, TestAppContext, point, rgb};
use ui::prelude::*;

struct SettingsHarness {
    content: Option<AnyView>,
    background: gpui::Hsla,
}

impl Render for SettingsHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let font = crate::fonts::Fonts::setup_ui(window, cx);
        div().size_full().p_6().font(font).bg(self.background).children(self.content.clone())
    }
}

#[gpui::test]
fn plugin_settings_inherit_the_host_surface_after_theme_changes(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::settings::init_themes(&crate::settings::ResolvedSettings::default(), cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
    });
    let scratch = tempfile::tempdir().unwrap();
    let paths = Paths::under(scratch.path());
    let mut catalog = cx.update(|cx| load_plugin_catalog_at(&SettingsStore::bare(), &paths, cx));
    // Include disabled-by-default bundles without starting any of their services.
    cx.update(|cx| catalog.enable_requested(&paths, |_| true, cx));
    assert!(catalog.rejected.is_empty());
    assert!(catalog.disabled.is_empty());
    let (harness, cx) = cx.add_window_view(|_, _| SettingsHarness {
        content: None,
        background: rgb(0x172331).into(),
    });
    let mut contributions = Vec::new();
    for (id, plugin) in &mut catalog.loaded {
        let source = cx.update(|window, cx| plugin.settings(window, cx));
        if let Some(source) = source {
            let view = cx.update(|_, cx| match source {
                chartr_plugin_host::SettingsSource::Native(view) => view.into_view(),
                chartr_plugin_host::SettingsSource::Declarative(schema) => {
                    crate::plugin_settings::view(schema, paths.data.join(id), cx)
                }
            });
            contributions.push((id.clone(), view));
        }
    }
    // The portable-form path is also part of the same surface contract.
    let clock = chartr_plugin::Manifest::parse(include_str!(
        "../../../../../examples/plugins/clock/chartr-plugin.toml"
    ))
    .unwrap();
    let clock_data = scratch.path().join("clock");
    std::fs::create_dir_all(&clock_data).unwrap();
    let clock =
        cx.update(|_, cx| crate::plugin_settings::view(clock.settings.unwrap(), clock_data, cx));
    contributions.push(("declarative clock".into(), clock));
    assert!(contributions.len() >= 4, "native and declarative settings must be exercised");

    for (id, view) in contributions {
        for (theme_name, background, font_size) in [
            (crate::settings::DEFAULT_DARK_THEME, 0x172331, 12.),
            (crate::settings::DEFAULT_LIGHT_THEME, 0xe7edf3, 12.),
            (crate::settings::DEFAULT_DARK_THEME, 0x172331, 18.),
            (crate::settings::DEFAULT_LIGHT_THEME, 0xe7edf3, 18.),
        ] {
            cx.update(|_, cx| {
                crate::fonts::install(
                    &crate::settings::ResolvedSettings {
                        ui_font_size: font_size,
                        ..Default::default()
                    },
                    cx,
                );
                let theme = theme::ThemeRegistry::global(cx).get(theme_name).unwrap();
                theme::GlobalTheme::update_theme(cx, theme);
                harness.update(cx, |harness, cx| {
                    harness.content = Some(view.clone());
                    // Deliberately differs from theme tokens: content must be
                    // transparent, not merely choose a matching token today.
                    harness.background = rgb(background).into();
                    cx.notify();
                });
            });
            cx.run_until_parked();
            cx.update(|window, _| {
                let viewport = window.viewport_size();
                for x in [px(32.), viewport.width / 2.] {
                    let probe = point(x, viewport.height - px(32.)).scale(window.scale_factor());
                    let painted: Vec<_> = window
                        .painted_quads()
                        .into_iter()
                        .filter(|quad| {
                            quad.bounds.contains(&probe)
                                && quad.content_mask.bounds.contains(&probe)
                                && quad.background != gpui::transparent_black().into()
                        })
                        .collect();
                    assert_eq!(
                        painted.len(),
                        1,
                        "{id} in {theme_name} painted over the host surface"
                    );
                    assert_eq!(
                        painted[0].background,
                        rgb(background).into(),
                        "{id} in {theme_name}"
                    );
                }
            });
        }
    }
}

// A settings contributor need not implement Render: the host must use the
// dedicated settings renderer and retain notification and event routing.
struct CounterSettings {
    count: usize,
}
impl chartr_plugin::RenderSettings for CounterSettings {
    fn render_settings(
        &mut self,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> chartr_plugin::SettingsPage {
        let count = self.count;
        chartr_plugin::SettingsPage::flow("counter-settings")
            .child(div().debug_selector(|| "SETTINGS_INCREMENT".into()).child(
                chartr_plugin::ui::action("increment-settings", "Increment").on_click(cx.listener(
                    |this, _, _, cx| {
                        this.count += 1;
                        cx.notify();
                    },
                )),
            ))
            .child(
                div()
                    .debug_selector(move || format!("SETTINGS_VALUE_{count}"))
                    .child(chartr_plugin::ui::label(count.to_string())),
            )
    }
}

#[gpui::test]
fn settings_mount_routes_actions_and_repaints_on_contributor_notifications(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
    });
    let state = cx.new(|_| CounterSettings { count: 0 });
    let content = cx.update(|cx| chartr_plugin::SettingsView::new(state.clone(), cx).into_view());
    let (_, cx) = cx.add_window_view(|_, _| SettingsHarness {
        content: Some(content),
        background: rgb(0x172331).into(),
    });
    cx.run_until_parked();
    assert!(cx.debug_bounds("SETTINGS_VALUE_0").is_some());
    let button = cx.debug_bounds("SETTINGS_INCREMENT").unwrap();
    cx.simulate_click(button.origin + point(px(8.), px(8.)), gpui::Modifiers::none());
    cx.run_until_parked();
    assert_eq!(state.read_with(cx, |state, _| state.count), 1);
    assert!(cx.debug_bounds("SETTINGS_VALUE_1").is_some());
}
