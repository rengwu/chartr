//! Appearance controls and theme selection.

use super::*;

impl SettingsWindow {
    fn theme_dropdown(
        &self,
        id: &'static str,
        label: &'static str,
        current: String,
        target: ThemeTarget,
        themes: Vec<theme::ThemeMeta>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let weak = cx.weak_entity();
        let selected = current.clone();
        let menu = ContextMenu::build(window, cx, move |menu, _, _| {
            let mut menu = menu;
            let has_dark = themes.iter().any(|theme| theme.appearance == theme::Appearance::Dark);
            let has_light = themes.iter().any(|theme| theme.appearance == theme::Appearance::Light);

            for (appearance, heading) in [
                (theme::Appearance::Dark, "Dark themes"),
                (theme::Appearance::Light, "Light themes"),
            ] {
                let choices: Vec<_> =
                    themes.iter().filter(|theme| theme.appearance == appearance).collect();
                if choices.is_empty() {
                    continue;
                }
                if has_dark && has_light {
                    if appearance == theme::Appearance::Light {
                        menu = menu.separator();
                    }
                    menu = menu.header(heading);
                }
                for choice in choices {
                    let name = choice.name.to_string();
                    let checked = name == selected;
                    let update = weak.clone();
                    menu = menu.toggleable_entry(
                        name.clone(),
                        checked,
                        IconPosition::End,
                        None,
                        move |_, cx| {
                            let name = name.clone();
                            let _ = update.update(cx, |this, cx| this.set_theme(target, name, cx));
                        },
                    );
                }
            }
            menu
        });

        DropdownMenu::new(id, current, menu)
            .style(DropdownStyle::Outlined)
            .trigger_size(FORM_CONTROL_SIZE)
            .attach(Anchor::BottomLeft)
            .aria_label(label)
            .into_any_element()
    }

    pub(super) fn appearance_page(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let settings = self.settings(cx);
        let mode = settings.theme_mode;
        let mut themes = theme::ThemeRegistry::global(cx).list();
        themes.sort_unstable_by(|a, b| {
            a.appearance.is_light().cmp(&b.appearance.is_light()).then(a.name.cmp(&b.name))
        });
        let light_themes = themes
            .iter()
            .filter(|theme| theme.appearance == theme::Appearance::Light)
            .cloned()
            .collect();
        let dark_themes = themes
            .iter()
            .filter(|theme| theme.appearance == theme::Appearance::Dark)
            .cloned()
            .collect();
        let fixed_mode = cx.listener(|this, _, _, cx| this.set_theme_mode(ThemeMode::Fixed, cx));
        let system_mode = cx.listener(|this, _, _, cx| this.set_theme_mode(ThemeMode::System, cx));
        let reduce_motion = settings.reduce_motion;
        let reduce_motion_setting = cx.weak_entity();
        let font = cx.weak_entity();
        let smaller = cx.listener(|this, _, _, cx| this.adjust_ui_font_size(-1., cx));
        let larger = cx.listener(|this, _, _, cx| this.adjust_ui_font_size(1., cx));
        let fixed_picker = self.theme_dropdown(
            "fixed-theme-menu",
            "Theme",
            settings.fixed_theme.clone(),
            ThemeTarget::Fixed,
            themes,
            window,
            cx,
        );
        let light_picker = self.theme_dropdown(
            "light-theme-menu",
            "Light theme",
            settings.light_theme.clone(),
            ThemeTarget::Light,
            light_themes,
            window,
            cx,
        );
        let dark_picker = self.theme_dropdown(
            "dark-theme-menu",
            "Dark theme",
            settings.dark_theme.clone(),
            ThemeTarget::Dark,
            dark_themes,
            window,
            cx,
        );
        let font_picker = PopoverMenu::new("ui-font-menu")
            .trigger(
                settings_button("ui-font-family", settings.ui_font_family)
                    .end_icon(Icon::new(IconName::ChevronDown)),
            )
            .anchor(Anchor::BottomLeft)
            .menu(move |window, cx| {
                let font = font.clone();
                Some(ContextMenu::build(window, cx, move |menu, _, _| {
                    fonts::UI_FONTS.iter().fold(menu, |menu, ui_font| {
                        let family = ui_font.family;
                        let set = font.clone();
                        menu.entry(family, None, move |_, cx| {
                            let _ =
                                set.update(cx, |this, cx| this.set_ui_font(family.to_owned(), cx));
                        })
                    })
                }))
            });
        let font_size = number_field(
            "ui-font-size",
            "Interface font size",
            "Adjust the size of interface text.",
            self.ui_font_size_input.clone(),
            smaller,
            larger,
            cx,
        );

        let mut fields = vec![setting_field(
            "Theme mode",
            "Use one theme at all times or follow the system appearance.",
            SegmentedControl::new(
                "Theme mode",
                [
                    SegmentedControlOption::new(
                        "theme-fixed",
                        "Fixed",
                        mode == ThemeMode::Fixed,
                        fixed_mode,
                    ),
                    SegmentedControlOption::new(
                        "theme-system",
                        "System",
                        mode == ThemeMode::System,
                        system_mode,
                    ),
                ],
            ),
        )];
        match mode {
            ThemeMode::Fixed => fields.push(setting_field(
                "Theme",
                "Choose the theme used throughout the interface.",
                fixed_picker,
            )),
            ThemeMode::System => {
                fields.push(setting_field(
                    "Light theme",
                    "Choose the theme used while the system is in light mode.",
                    light_picker,
                ));
                fields.push(setting_field(
                    "Dark theme",
                    "Choose the theme used while the system is in dark mode.",
                    dark_picker,
                ));
            }
        }
        fields.extend([
            setting_field(
                "Font family",
                "Choose the typeface used throughout the interface.",
                font_picker,
            ),
            setting_field("Font size", "Adjust the size of interface text.", font_size),
            setting_field(
                "Reduce motion",
                "Disable movement animations when switching views, resizing sidebars, and rearranging items.",
                Switch::new("reduce-motion", reduce_motion.into())
                    .tab_index(0isize)
                    .aria_label("Reduce motion")
                    .aria_description("Disable movement animations when switching views, resizing sidebars, and rearranging items.")
                    .on_click(move |state, _, cx| {
                        let reduce_motion = state.selected();
                        let _ = reduce_motion_setting
                            .update(cx, |this, cx| this.set_reduce_motion(reduce_motion, cx));
                    }),
            ),
        ]);

        settings_fields(fields, cx.theme().colors().border_variant)
    }
}
