use super::*;
use gpui::{Modifiers, TestAppContext};
use std::time::Duration;

struct MenuHarness {
    invoked: Rc<Cell<bool>>,
}

impl Render for MenuHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let invoked = self.invoked.clone();
        PopupMenu::new("popup-test").trigger(ui::Button::new("popup-test-trigger", "Open")).menu(
            move |window, cx| {
                let invoked = invoked.clone();
                Some(ContextMenu::build_popup(window, cx, move |menu| {
                    menu.entry("Run", None, move |_, _| invoked.set(true))
                }))
            },
        )
    }
}

struct TooltipMenuHarness {
    tooltip_built: Rc<Cell<bool>>,
}

struct NativeTooltipHarness;

struct NativeModalHarness;

struct NativeModalBody;

impl Render for NativeTooltipHarness {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(14.));
        div().size_full().child(
            ui::Button::new("native-tooltip-trigger", "Hover")
                .tooltip(ui::Tooltip::text("Native tooltip")),
        )
    }
}

impl Render for NativeModalHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().id("native-modal-trigger").size_full().on_mouse_down(
            MouseButton::Left,
            |_, window, cx| {
                open_native_modal(window, cx, |_, cx| cx.new(|_| NativeModalBody)).unwrap();
            },
        )
    }
}

impl Render for NativeModalBody {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full().debug_selector(|| "NATIVE_MODAL_BODY".into())
    }
}

struct ContentSizedMenuHarness {
    rows: usize,
}

impl Render for ContentSizedMenuHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let rows = self.rows;
        PopupMenu::new("content-sized-popup-test")
            .trigger(ui::Button::new("content-sized-popup-trigger", "Open"))
            .menu(move |window, cx| {
                Some(ContextMenu::build_popup(window, cx, move |menu| {
                    let mut menu = menu.popup_width(px(320.));
                    for _ in 0..rows {
                        menu = menu
                            .custom_row(|_, _| div().h(px(120.)).child("Tall").into_any_element());
                    }
                    menu
                }))
            })
    }
}

struct TestTooltip;

impl Render for TestTooltip {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child("Open menu")
    }
}

impl Render for TooltipMenuHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let tooltip_built = self.tooltip_built.clone();
        PopupMenu::new("tooltip-popup-test")
            .trigger_with_tooltip(
                ui::Button::new("tooltip-popup-test-trigger", "Open"),
                move |_, cx| {
                    tooltip_built.set(true);
                    cx.new(|_| TestTooltip).into()
                },
            )
            .menu(|window, cx| {
                Some(ContextMenu::build_popup(window, cx, |menu| {
                    menu.entry("Run", None, |_, _| {})
                }))
            })
    }
}

#[gpui::test]
fn anchored_menu_entries_receive_clicks_and_close(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
    });

    let invoked = Rc::new(Cell::new(false));
    let invoked_for_view = invoked.clone();
    let (_, cx) = cx.add_window_view(|_, _| MenuHarness { invoked: invoked_for_view });
    let parent = cx.window_handle();

    cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
    let popup = cx
        .windows()
        .into_iter()
        .find(|window| *window != parent)
        .expect("clicking the trigger should open an anchored menu window");

    let mut popup = gpui::VisualTestContext::from_window(popup, cx);
    popup.run_until_parked();
    let entry = popup
        .debug_bounds("MENU_ITEM-Run")
        .expect("the stock context-menu entry should render in the popup");
    popup.simulate_click(entry.center(), Modifiers::none());

    assert!(invoked.get(), "clicking a popup entry should invoke its parent-window handler");
    assert_eq!(popup.windows(), vec![parent], "confirming an entry should close the popup");
}

#[gpui::test]
fn clicking_an_open_popup_trigger_does_not_reopen_it(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
    });

    let invoked = Rc::new(Cell::new(false));
    let invoked_for_view = invoked.clone();
    let (_, cx) = cx.add_window_view(|_, _| MenuHarness { invoked: invoked_for_view });
    let parent = cx.window_handle();
    cx.update(|window, _| window.activate_window());
    cx.run_until_parked();

    cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
    assert_eq!(cx.windows().len(), 2, "the first click should open the popup");

    cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
    cx.run_until_parked();

    assert_eq!(cx.windows(), vec![parent], "the second click should only close the popup");
}

#[gpui::test]
fn anchored_custom_rows_size_the_popup_from_rendered_content(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
    });

    let (_, cx) = cx.add_window_view(|_, _| ContentSizedMenuHarness { rows: 1 });
    let parent = cx.window_handle();
    let parent_height = cx.update(|window, _| window.viewport_size().height);

    cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
    let popup = cx
        .windows()
        .into_iter()
        .find(|window| *window != parent)
        .expect("clicking the trigger should open an anchored menu window");

    let mut popup = gpui::VisualTestContext::from_window(popup, cx);
    popup.run_until_parked();
    let popup_height = popup.update(|window, _| match window.window_bounds() {
        WindowBounds::Windowed(bounds) => bounds.size.height,
        WindowBounds::Maximized(bounds) => bounds.size.height,
        WindowBounds::Fullscreen(bounds) => bounds.size.height,
    });

    assert!(popup_height > px(120.), "the popup should contain the complete custom row");
    assert!(
        popup_height < parent_height,
        "a short menu should shrink instead of occupying all available height"
    );
}

#[gpui::test]
fn anchored_custom_rows_scroll_only_at_the_parent_window_limit(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
    });

    let (_, cx) = cx.add_window_view(|_, _| ContentSizedMenuHarness { rows: 20 });
    let parent = cx.window_handle();
    let parent_height = cx.update(|window, _| window.viewport_size().height);

    cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
    let popup = cx
        .windows()
        .into_iter()
        .find(|window| *window != parent)
        .expect("clicking the trigger should open an anchored menu window");

    let mut popup = gpui::VisualTestContext::from_window(popup, cx);
    popup.run_until_parked();
    let popup_height = popup.update(|window, _| window.window_bounds().get_bounds().size.height);

    assert_eq!(popup_height, parent_height - POPUP_GAP);
}

#[gpui::test]
fn opening_a_popup_cancels_its_pending_trigger_tooltip(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
    });

    let tooltip_built = Rc::new(Cell::new(false));
    let tooltip_built_for_view = tooltip_built.clone();
    let (_, cx) =
        cx.add_window_view(|_, _| TooltipMenuHarness { tooltip_built: tooltip_built_for_view });

    cx.simulate_mouse_move(point(px(10.), px(10.)), None, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_mouse_move(point(px(11.), px(10.)), None, Modifiers::none());
    cx.run_until_parked();
    cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());
    cx.executor().advance_clock(Duration::from_millis(600));
    cx.run_until_parked();

    assert!(!tooltip_built.get(), "the trigger tooltip must stay hidden while its menu is open");
}

#[gpui::test]
fn tooltip_uses_a_native_popup_and_closes_on_mouse_exit(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
    });

    let (_, cx) = cx.add_window_view(|_, _| NativeTooltipHarness);
    let parent = cx.window_handle();
    let parent_rem_size = cx.update(|window, _| window.rem_size());
    cx.simulate_mouse_move(point(px(10.), px(10.)), None, Modifiers::none());
    cx.run_until_parked();
    cx.executor().advance_clock(Duration::from_millis(600));
    cx.run_until_parked();

    assert_eq!(cx.windows().len(), 2, "the tooltip should open a native window");
    let popup = cx
        .windows()
        .into_iter()
        .find(|window| *window != parent)
        .expect("the tooltip popup should exist");
    {
        let mut popup = gpui::VisualTestContext::from_window(popup, cx);
        popup.run_until_parked();
        let body = popup
            .debug_bounds("NATIVE_TOOLTIP_BODY")
            .expect("the native popup should render the tooltip body");
        let popup_size = popup.update(|window, _| window.viewport_size());
        let popup_rem_size = popup.update(|window, _| window.rem_size());
        assert_eq!(popup_rem_size, parent_rem_size);
        assert_eq!(body.origin, point(px(8.), px(8.)));
        assert_eq!(popup_size.width, body.size.width + px(16.));
        assert_eq!(popup_size.height, body.size.height + px(16.));
    }

    cx.simulate_mouse_move(point(px(300.), px(300.)), None, Modifiers::none());
    cx.run_until_parked();
    assert_eq!(cx.windows(), vec![parent], "the tooltip window should close on mouse exit");
}

#[gpui::test]
fn native_modal_uses_a_parent_sized_window(cx: &mut TestAppContext) {
    cx.update(|cx| {
        ::settings::init(cx);
        theme::init(theme::LoadThemes::JustBase, cx);
        crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
    });

    let (_, cx) = cx.add_window_view(|_, _| NativeModalHarness);
    let parent = cx.window_handle();
    let parent_size = cx.update(|window, _| window.viewport_size());
    cx.simulate_click(point(px(10.), px(10.)), Modifiers::none());

    let popup = cx
        .windows()
        .into_iter()
        .find(|window| *window != parent)
        .expect("opening a modal should create a native child window");
    let mut popup = gpui::VisualTestContext::from_window(popup, cx);
    popup.run_until_parked();

    assert_eq!(popup.update(|window, _| window.viewport_size()), parent_size);
    assert_eq!(
        popup.debug_bounds("NATIVE_MODAL_BODY"),
        Some(Bounds::new(Default::default(), parent_size)),
        "the modal surface should cover the complete parent viewport"
    );
}
