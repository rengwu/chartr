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
        hidden_quad_count,
        "releasing outside must hide the thumb",
    );
    let released_offset = handle.offset();
    cx.simulate_mouse_move(point(px(250.), px(50.)), None, Modifiers::none());
    assert_eq!(handle.offset(), released_offset, "release must end scrolling");
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
                assert_eq!(thumb.a, 1.);
                for background in
                    [colors.panel_background, sidebar.card_active, sidebar.card_inactive]
                {
                    let a = luminance(thumb);
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

impl Render for TreeHarness {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let on = cx.listener(|this, action: &super::Action, _, cx| {
            if let super::Action::BeginSpaceDrag { at } = action {
                this.sorter.press(*at);
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
fn sidebar_tree_collapses_adds_and_sorts_free_sessions(cx: &mut TestAppContext) {
    use super::{Action, SpaceEntries};
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
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
    let expanded = bounds(cx, 0);
    let project_before = bounds(cx, 1);
    // Free sessions is a regular child in the shared sorter scroll list.
    assert!(project_before.top() > expanded.bottom());
    cx.simulate_click(expanded.origin + point(px(60.), px(12.)), Modifiers::none());
    cx.run_until_parked();
    let collapsed = bounds(cx, 0);
    assert!(collapsed.size.height < expanded.size.height);
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
        assert!(
            view.actions
                .iter()
                .all(|action| !matches!(action, Action::ToggleSpaceCollapsed { .. }))
        );
        assert!(!view.spaces[1].collapsed, "dropping a title must not collapse it");
    });
}
