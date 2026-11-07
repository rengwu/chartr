use std::time::Duration;

use gpui::{Context, Modifiers, MouseButton, Render, ScrollHandle, TestAppContext, point};
use ui::{ScrollAxes, Scrollbars, WithScrollbar, prelude::*};

struct ScrollbarHarness {
    handle: ScrollHandle,
}

impl Render for ScrollbarHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .relative()
            .size(px(200.))
            .child(
                div()
                    .id("sessions")
                    .size_full()
                    .overflow_y_scroll()
                    .track_scroll(&self.handle)
                    // A space heading must not steal the thumb's mouse-down event.
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(div().h(px(1000.)).w_full()),
            )
            .custom_scrollbars(
                Scrollbars::on_hover(ScrollAxes::Vertical)
                    .id("spaces-scrollbar")
                    .tracked_scroll_handle(&self.handle)
                    .notify_content(),
                window,
                cx,
            )
    }
}

#[gpui::test]
fn sidebar_scrollbar_drags_past_the_container_and_releases(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
    });
    let handle = ScrollHandle::new();
    let (_, cx) = cx.add_window_view(|_, _| ScrollbarHarness { handle: handle.clone() });
    cx.simulate_mouse_move(point(px(250.), px(250.)), None, Modifiers::none());
    cx.executor().advance_clock(Duration::from_millis(900));
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
    let hidden_quad_count = cx.update(|window, _| window.painted_quads().len());
    cx.simulate_mouse_move(point(px(50.), px(50.)), None, Modifiers::none());
    assert_eq!(
        cx.update(|window, _| window.painted_quads().len()),
        hidden_quad_count + 1,
        "entering the container must paint the thumb",
    );
    let thumb = point(px(193.), px(20.));
    cx.simulate_mouse_move(thumb, None, Modifiers::none());
    cx.simulate_mouse_down(thumb, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(point(px(193.), px(60.)), MouseButton::Left, Modifiers::none());
    let inside_offset = handle.offset().y;
    assert!(inside_offset < px(-100.), "thumb drag must scroll inside the container");

    let outside = point(px(250.), px(150.));
    cx.simulate_mouse_move(outside, MouseButton::Left, Modifiers::none());
    assert!(handle.offset().y < inside_offset, "drag must continue outside the container");
    assert_eq!(
        cx.update(|window, _| window.painted_quads().len()),
        hidden_quad_count + 1,
        "the thumb must remain visible throughout the drag",
    );
    cx.simulate_mouse_up(outside, MouseButton::Left, Modifiers::none());
    assert_eq!(
        cx.update(|window, _| window.painted_quads().len()),
        hidden_quad_count + 1,
        "releasing outside starts a fade instead of hiding the thumb immediately",
    );
    let released_offset = handle.offset();
    cx.simulate_mouse_move(point(px(250.), px(50.)), None, Modifiers::none());
    assert_eq!(handle.offset(), released_offset, "release must end scrolling");
    cx.executor().advance_clock(Duration::from_millis(900));
    cx.update(|window, _| window.refresh());
    cx.run_until_parked();
    assert_eq!(cx.update(|window, _| window.painted_quads().len()), hidden_quad_count);
}

#[gpui::test]
fn sidebar_scrollbar_has_contrast_in_every_theme(cx: &mut TestAppContext) {
    fn luminance(color: gpui::Hsla) -> f32 {
        let color = gpui::Rgba::from(color);
        let linear =
            |v: f32| if v <= 0.04045 { v / 12.92 } else { ((v + 0.055) / 1.055).powf(2.4) };
        0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
    }
    cx.update(|cx| {
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::settings::init_themes(&crate::settings::ResolvedSettings::default(), cx);
        let registry = theme::ThemeRegistry::global(cx);
        for name in registry.list_names() {
            let theme = registry.get(&name).unwrap();
            let colors = &theme.styles.colors;
            let sidebar = crate::settings::sidebar_theme_colors(&theme);
            for thumb in crate::components::scrollbar_thumb_colors(colors) {
                assert!(thumb.a > 0. && thumb.a < 1., "{name}: thumbs must be translucent");
                for background in
                    [colors.panel_background, sidebar.card_active, sidebar.card_inactive]
                {
                    let a = luminance(background.blend(thumb));
                    let b = luminance(background);
                    let contrast = (a.max(b) + 0.05) / (a.min(b) + 0.05);
                    assert!(contrast >= 3., "{name}: scrollbar contrast is only {contrast:.2}:1");
                }
            }
        }
    });
}

/// Exercise the real sidebar so bubbling clicks and collapsed layout are tested
/// together with the same scroll geometry used for space sorting.
struct TreeHarness {
    spaces: Vec<super::SpaceEntries>,
    sorter: super::sidebar::SpaceSorter,
    actions: Vec<super::Action>,
}

struct TabDragHarness {
    space: super::SpaceEntries,
    sidebar: bool,
    sorter: super::sidebar::SpaceSorter,
}

impl Render for TabDragHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let on: super::Emit = std::rc::Rc::new(|_, _, _| {});
        let chrome = if self.sidebar {
            super::sidebar::render(std::slice::from_ref(&self.space), on, &self.sorter, window, cx)
                .into_any_element()
        } else {
            super::tabs::render(
                &self.space.entries,
                None,
                div().into_any_element(),
                div().into_any_element(),
                on,
                window,
                cx,
            )
            .into_any_element()
        };
        div().w(px(if self.sidebar { 280. } else { 640. })).h(px(400.)).child(chrome)
    }
}

#[gpui::test]
fn dragged_tabs_paint_a_solid_rounded_surface_without_changing_size(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        cx.set_global(crate::settings::SettingsStore::bare());
    });
    for sidebar in [true, false] {
        for selected in [false, true] {
            let (_, cx) = cx.add_window_view(|_, _| TabDragHarness {
                sidebar,
                sorter: super::sidebar::SpaceSorter::new(super::sidebar::CARD_GAP),
                space: super::SpaceEntries {
                    id: 10_u64.into(),
                    name: "Project".into(),
                    collapsed: false,
                    active: true,
                    removable: true,
                    available: true,
                    entries: (1..=3)
                        .map(|id| super::Entry {
                            space: 10_u64.into(),
                            space_key: "test".into(),
                            key: serde_json::from_value(serde_json::json!(id)).unwrap(),
                            tab: serde_json::from_value(serde_json::json!(id)).unwrap(),
                            pane: serde_json::from_value(serde_json::json!(1)).unwrap(),
                            index: id - 1,
                            title: format!("Session {id}"),
                            title_detail: None,
                            terminal: true,
                            manual_name: false,
                            icon_path: None,
                            status: None,
                            process_running: false,
                            ended: false,
                            bell: false,
                            unread: false,
                            selected: id == if selected { 2 } else { 1 },
                            closable: true,
                            grouped: false,
                        })
                        .collect(),
                },
            });
            cx.run_until_parked();
            let selector = if sidebar { "SESSION_TAB_2" } else { "WORKSPACE_TAB_2" };
            let original = cx.debug_bounds(selector).unwrap();
            let source = original.center();
            let target =
                source + if sidebar { point(px(0.), px(-18.)) } else { point(px(-45.), px(0.)) };
            cx.simulate_mouse_down(source, MouseButton::Left, Modifiers::none());
            cx.simulate_mouse_move(
                source + point(px(8.), px(0.)),
                MouseButton::Left,
                Modifiers::none(),
            );
            cx.simulate_mouse_move(target, MouseButton::Left, Modifiers::none());
            cx.run_until_parked();
            let held = cx.debug_bounds(selector).unwrap();
            assert_eq!(held.size, original.size, "sidebar={sidebar}, selected={selected}");
            assert!(held.contains(&target));
            cx.update(|window, cx| {
                let colors = cx.theme().colors();
                let overlay = if sidebar {
                    let colors = crate::settings::sidebar_theme_colors(cx.theme());
                    if selected { colors.session_active } else { colors.session_hover }
                } else if selected {
                    colors.ghost_element_selected
                } else {
                    colors.ghost_element_hover
                };
                let background = colors.background.blend(colors.panel_background).blend(overlay);
                assert_eq!(background.a, 1.);
                assert!(window.painted_quads().iter().any(|quad| {
                    quad.bounds == held.scale(window.scale_factor())
                        && quad.background == gpui::solid_background(background)
                        && quad.corner_radii.top_left > gpui::ScaledPixels(0.)
                }), "dragged tab must cover underlying labels within its own rounded bounds: sidebar={sidebar}, selected={selected}");
                cx.stop_active_drag(window);
                window.refresh();
            });
            cx.simulate_mouse_up(target, MouseButton::Left, Modifiers::none());
            cx.run_until_parked();
            assert_eq!(cx.debug_bounds(selector).unwrap(), original);
        }
    }
}

impl Render for TreeHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let on = cx.listener(|this, action: &super::Action, _, cx| {
            if let super::Action::BeginSpaceDrag { at } = action {
                this.sorter.press(*at);
            }
            if let super::Action::ActivateSpace { space } = action {
                for entry in &mut this.spaces {
                    entry.active = entry.id == *space;
                }
            }
            if let super::Action::ToggleSpaceCollapsed { space } = action {
                let space = this.spaces.iter_mut().find(|entry| entry.id == *space).unwrap();
                space.collapsed = !space.collapsed;
            }
            this.actions.push(action.clone());
            cx.notify();
        });
        let mut spaces = self.spaces.clone();
        self.sorter.arrange(&mut spaces, |space| space.id);
        div()
            .w(px(280.))
            .h(px(400.))
            .on_drag_move::<super::DraggedSpace>(cx.listener(
                |this, event: &gpui::DragMoveEvent<super::DraggedSpace>, window, cx| {
                    let order = this.spaces.iter().map(|space| space.id).collect();
                    if this.sorter.drag_move(
                        event.drag(cx).0,
                        order,
                        event.event.position,
                        window.rem_size(),
                        cx.background_executor().now(),
                        true,
                    ) {
                        cx.notify();
                    }
                },
            ))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, event: &gpui::MouseUpEvent, window, cx| {
                    if let Some((space, target)) = this.sorter.drop_at(
                        event.position.y,
                        window.rem_size(),
                        cx.background_executor().now(),
                        cx.reduce_motion(),
                    ) {
                        let from = this.spaces.iter().position(|entry| entry.id == space).unwrap();
                        let moved = this.spaces.remove(from);
                        this.spaces.insert(target, moved);
                        this.sorter.accept_drop(cx.background_executor().now(), cx.reduce_motion());
                        cx.notify();
                    }
                }),
            )
            .child(super::sidebar::render(
                &spaces,
                std::rc::Rc::new(move |action, window, cx| on(&action, window, cx)),
                &self.sorter,
                window,
                cx,
            ))
    }
}

#[gpui::test]
fn spaces_scroll_under_the_fixed_header_and_fade_from_the_first_pixel(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        cx.set_global(crate::settings::SettingsStore::bare());
    });
    let (view, cx) = cx.add_window_view(|_, _| TreeHarness {
        spaces: (0..20)
            .map(|index| super::SpaceEntries {
                id: (index + 10).into(),
                name: format!("Space {index}"),
                collapsed: true,
                active: index == 0,
                removable: false,
                available: true,
                entries: vec![],
            })
            .collect(),
        sorter: super::sidebar::SpaceSorter::new(super::sidebar::CARD_GAP),
        actions: vec![],
    });
    for rem_size in [14., 20.] {
        cx.update(|window, _| {
            window.set_rem_size(px(rem_size));
            window.refresh();
        });
        cx.run_until_parked();
        let handle = view.read_with(cx, |view, _| view.sorter.scroll_handle().clone());
        let header = cx.debug_bounds("SPACES_HEADER").unwrap();
        assert_eq!(handle.bounds().top(), header.top(), "scrolling must extend behind the header");
        assert_eq!(handle.children_count(), 20, "top padding must not become a sortable item");
        let first = handle.bounds_for_item(0).unwrap();
        assert_eq!(first.top(), header.bottom(), "the first card starts below the fade at rest");

        for offset in [0., 1., 8., 30., 80., 0.] {
            handle.set_offset(point(px(0.), px(-offset)));
            cx.update(|window, _| window.refresh());
            cx.run_until_parked();
            assert_eq!(cx.debug_bounds("SPACES_HEADER").unwrap(), header);
            cx.update(|window, cx| {
                let background = cx.theme().colors().panel_background;
                let progress = (offset / 32_f32).clamp(0., 1.);
                let fade_height = header.size.height + px(16.) * progress;
                let opaque_height = header.size.height * (0.3 + 0.3 * progress);
                let gradient = gpui::linear_gradient(
                    180.,
                    gpui::linear_color_stop(background, opaque_height / fade_height),
                    gpui::linear_color_stop(background.opacity(0.), 1.),
                );
                let quads = window.painted_quads();
                let fade = quads
                    .iter()
                    .find(|quad| quad.background == gradient)
                    .expect("the header fade must stay active, including at scroll offset zero");
                assert_eq!(fade.bounds.origin, header.origin.scale(window.scale_factor()));
                assert_eq!(fade.bounds.size.width, header.size.width.scale(window.scale_factor()));
                assert!(
                    (fade.bounds.size.height.as_f32()
                        - fade_height.scale(window.scale_factor()).as_f32())
                    .abs()
                        <= 1.,
                    "the growing fade height may round to the nearest device pixel"
                );
                assert!(
                    opaque_height >= header.size.height * 0.3
                        && opaque_height <= header.size.height * 0.6,
                    "the opaque region stays within the tuned 30–60% of the header"
                );
                if offset == 0. {
                    assert_eq!(
                        fade.bounds.bottom(),
                        first.top().scale(window.scale_factor()),
                        "the resting fade must not dim the first card"
                    );
                } else {
                    assert!(
                        fade.bounds.bottom() > header.bottom().scale(window.scale_factor()),
                        "scrolling extends the soft fade below the header"
                    );
                }
                let card_color = background.blend(
                    crate::settings::sidebar_theme_colors(cx.theme()).card_active.opacity(0.4),
                );
                if offset < 30. {
                    let card =
                        quads.iter().find(|quad| quad.background == card_color.into()).unwrap();
                    assert_eq!(
                        card.bounds.top(),
                        (first.top() - px(offset)).scale(window.scale_factor())
                    );
                    assert_eq!(
                        card.content_mask.bounds.top(),
                        header.top().scale(window.scale_factor()),
                        "cards must not clip at the old bottom-of-header boundary"
                    );
                    if offset > 0. {
                        assert!(
                            fade.order > card.order,
                            "the fade paints over the overlapping card at offset {offset}"
                        );
                    }
                }
            });
        }

        // The header lets wheel input reach the list, but hides its covered
        // cards from clicks. Its own add button remains usable after scrolling.
        cx.simulate_event(gpui::ScrollWheelEvent {
            position: header.center(),
            delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-20.))),
            ..Default::default()
        });
        cx.run_until_parked();
        assert!(handle.offset().y < px(0.), "scrolling over the header must move the list");
        view.update(cx, |view, _| view.actions.clear());
        cx.simulate_click(header.center(), Modifiers::none());
        assert!(view.read_with(cx, |view, _| view.actions.is_empty()));
        let add = cx.update(|window, cx| {
            let button_size = IconSize::Small.square(window, cx);
            point(
                header.right() - window.rem_size() * 0.5 - px(4.) - button_size * 0.5,
                header.top() + button_size * 0.5,
            )
        });
        cx.simulate_click(add, Modifiers::none());
        assert_eq!(
            view.read_with(cx, |view, _| view.actions.clone()),
            vec![super::Action::NewSpace]
        );
        handle.set_offset(point(px(0.), px(0.)));
    }
    // Removing overflow must restore the resting geometry and scroll range.
    view.update(cx, |view, cx| {
        view.spaces.truncate(1);
        cx.notify();
    });
    cx.run_until_parked();
    let handle = view.read_with(cx, |view, _| view.sorter.scroll_handle().clone());
    assert_eq!(handle.max_offset().y, px(0.));
    assert_eq!(handle.offset().y, px(0.));
}

#[gpui::test]
fn sidebar_tree_collapses_adds_and_sorts_free_sessions(cx: &mut TestAppContext) {
    use super::{Action, Entry, SpaceEntries};
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        cx.set_global(crate::settings::SettingsStore::bare());
    });
    let free_id = 10_u64.into();
    let folder_id = 11_u64.into();
    let (view, cx) = cx.add_window_view(|_, _| TreeHarness {
        spaces: vec![
            SpaceEntries {
                id: free_id,
                name: "Free sessions".into(),
                collapsed: false,
                active: true,
                removable: false,
                available: true,
                entries: vec![],
            },
            SpaceEntries {
                id: folder_id,
                name: "Project".into(),
                collapsed: false,
                active: false,
                removable: true,
                available: true,
                entries: vec![],
            },
        ],
        sorter: super::sidebar::SpaceSorter::new(super::sidebar::CARD_GAP),
        actions: vec![],
    });
    cx.run_until_parked();
    let bounds = |cx: &mut gpui::VisualTestContext, index| {
        view.read_with(cx, |view, _| view.sorter.scroll_handle().bounds_for_item(index).unwrap())
    };
    let empty = bounds(cx, 0);
    let project_without_sessions = bounds(cx, 1);

    // An inactive heading selects its space without changing expansion. Its
    // chevron remains an independent collapse control and must not select it.
    cx.simulate_click(project_without_sessions.origin + point(px(12.), px(12.)), Modifiers::none());
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert!(view.spaces[1].collapsed);
        assert!(view.spaces[0].active);
        assert!(!view.spaces[1].active);
    });
    view.update(cx, |view, _| view.actions.clear());
    cx.simulate_click(project_without_sessions.origin + point(px(60.), px(12.)), Modifiers::none());
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert!(view.actions.contains(&Action::ActivateSpace { space: folder_id }));
        assert!(
            !view
                .actions
                .iter()
                .any(|action| matches!(action, Action::ToggleSpaceCollapsed { .. }))
        );
        assert!(view.spaces[1].active && view.spaces[1].collapsed);
        assert!(!view.spaces[0].active);
    });
    cx.simulate_click(empty.origin + point(px(60.), px(12.)), Modifiers::none());
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert!(view.spaces[0].active && !view.spaces[0].collapsed);
        assert!(!view.spaces[1].active);
    });

    // Empty spaces have only a heading: toggling them must not introduce or
    // remove a placeholder row, or shift the neighboring space.
    cx.simulate_click(empty.origin + point(px(60.), px(12.)), Modifiers::none());
    cx.run_until_parked();
    assert!(view.read_with(cx, |view, _| view.spaces[0].collapsed));
    assert_eq!(bounds(cx, 0), empty);
    assert_eq!(bounds(cx, 1), project_without_sessions);

    cx.simulate_click(empty.origin + point(px(12.), px(12.)), Modifiers::none());
    cx.run_until_parked();
    assert!(!view.read_with(cx, |view, _| view.spaces[0].collapsed));
    assert_eq!(bounds(cx, 0), empty);
    assert_eq!(bounds(cx, 1), project_without_sessions);

    // A populated space still collapses its session rows. Keep the subsequent
    // add-button and sorting checks on the expanded tree.
    view.update(cx, |view, cx| {
        view.spaces[0].entries.push(Entry {
            space: free_id,
            space_key: "free".into(),
            key: serde_json::from_value(serde_json::json!(1)).unwrap(),
            tab: serde_json::from_value(serde_json::json!(1)).unwrap(),
            pane: serde_json::from_value(serde_json::json!(1)).unwrap(),
            index: 0,
            title: "Terminal".into(),
            title_detail: None,
            terminal: true,
            manual_name: false,
            icon_path: None,
            status: None,
            process_running: false,
            ended: false,
            bell: false,
            unread: false,
            selected: true,
            closable: true,
            grouped: false,
        });
        cx.notify();
    });
    cx.run_until_parked();
    let expanded = bounds(cx, 0);
    let project_before = bounds(cx, 1);
    // Free sessions is a regular child in the shared sorter scroll list.
    assert!(project_before.top() > expanded.bottom());
    cx.simulate_click(expanded.origin + point(px(60.), px(12.)), Modifiers::none());
    cx.run_until_parked();
    let collapsed = bounds(cx, 0);
    assert!(collapsed.size.height < expanded.size.height);
    assert_eq!(collapsed, empty);
    assert!(bounds(cx, 1).top() < project_before.top());
    assert!(view.read_with(cx, |view, _| view.spaces[0].collapsed));

    // Click the chevron to reopen the same row.
    cx.simulate_click(collapsed.origin + point(px(12.), px(12.)), Modifiers::none());
    cx.run_until_parked();
    assert_eq!(bounds(cx, 0).size.height, expanded.size.height);
    view.update(cx, |view, _| view.actions.clear());

    // The terminal plus is immediately before the surface control. Its press
    // must not begin a space drag and its click must not collapse the space.
    cx.simulate_click(
        point(expanded.right() - px(42.), expanded.top() + px(12.)),
        Modifiers::none(),
    );
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(view.actions, vec![Action::NewInSpace { space: free_id }]);
        assert!(!view.spaces[0].collapsed);
    });

    view.update(cx, |view, _| view.actions.clear());
    let start = expanded.origin + point(px(60.), px(12.));
    let end = point(start.x, project_before.bottom() - px(5.));
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(start + point(px(0.), px(8.)), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
    cx.run_until_parked();
    view.read_with(cx, |view, _| {
        assert_eq!(
            view.spaces.iter().map(|space| space.id).collect::<Vec<_>>(),
            vec![folder_id, free_id]
        );
        assert!(view.actions.iter().all(|action| !matches!(
            action,
            Action::ToggleSpaceCollapsed { .. } | Action::ActivateSpace { .. }
        )));
        assert!(!view.spaces[1].collapsed, "dropping a title must not collapse it");
    });
}
