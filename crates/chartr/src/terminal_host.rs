//! The single adapter between chartr's workspace model and Zed's terminal UI.
//!
//! `TerminalView` intentionally supports non-workspace hosts, but its public
//! constructor still accepts weak Zed `Workspace` and `Project` handles for
//! optional integrations such as pane actions and assistant context. chartr
//! owns neither type. Invalid weak handles express that absence without
//! manufacturing a partial Zed workspace; disabling workspace actions selects
//! the view's documented non-workspace-host path. chartr uses the maintained
//! host extensions: top grid alignment, balanced cell padding, an overlay
//! scrollbar, and pausing grid resizes during workspace mode animations. All
//! terminal behavior remains Zed's pinned model and view.

use gpui::{
    App, AppContext as _, Div, Entity, Hsla, InteractiveElement as _, ParentElement as _,
    Styled as _, WeakEntity, Window, div,
};

/// Host an existing Zed terminal model in chartr's pane tree.
pub fn new_view(
    terminal: Entity<terminal::Terminal>,
    window: &mut Window,
    cx: &mut App,
) -> Entity<terminal_view::TerminalView> {
    cx.new(|cx| {
        let mut view = terminal_view::TerminalView::new(
            terminal,
            WeakEntity::new_invalid(),
            None,
            WeakEntity::new_invalid(),
            window,
            cx,
        );
        view.set_show_workspace_actions(false, cx);
        view.set_vertical_alignment(terminal_view::TerminalVerticalAlignment::Top, cx);
        view.set_grid_padding(true, cx);
        view
    })
}

/// Mount a TerminalView exactly as Zed mounts it: it fills the available pane
/// without an additional product-level inset, and the TerminalElement remains
/// the innermost mouse target. The view itself owns its balanced grid gutter.
pub fn element(view: Entity<terminal_view::TerminalView>, background: Hsla) -> Div {
    let drop_view = view.clone();
    div()
        .size_full()
        .bg(background)
        .can_drop(|value, _, _| value.downcast_ref::<gpui::ExternalPaths>().is_some())
        .on_drop(move |paths: &gpui::ExternalPaths, window, cx| {
            drop_view.update(cx, |view, cx| view.add_paths_to_terminal(paths.paths(), window, cx));
        })
        .child(view)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{
        FocusHandle, Focusable as _, Modifiers, Render, TestAppContext, point, px, size,
        transparent_black,
    };
    use terminal::{
        TerminalBuilder,
        terminal_settings::{AlternateScroll, CursorShape},
    };
    use util::paths::PathStyle;

    struct TestHost {
        terminal: Entity<terminal::Terminal>,
        view: Entity<terminal_view::TerminalView>,
        other_focus: FocusHandle,
        resize_paused: bool,
        inset: gpui::Point<gpui::Pixels>,
    }

    impl Render for TestHost {
        fn render(
            &mut self,
            _: &mut Window,
            cx: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            self.view.update(cx, |view, cx| view.set_resize_paused(self.resize_paused, cx));
            gpui::div()
                .size_full()
                .pl(self.inset.x)
                .pt(self.inset.y)
                .overflow_hidden()
                .track_focus(&self.other_focus)
                .child(element(self.view.clone(), transparent_black()))
        }
    }

    fn init_test(cx: &mut TestAppContext) {
        cx.update(|cx| {
            ::settings::init(cx);
            theme::init(theme::LoadThemes::JustBase, cx);
            crate::fonts::install(&crate::settings::ResolvedSettings::default(), cx);
        });
    }

    #[gpui::test]
    fn mounts_zeds_terminal_view_as_a_stable_chartr_view(cx: &mut TestAppContext) {
        init_test(cx);
        let terminal = cx.new(|cx| {
            TerminalBuilder::new_display_only(
                CursorShape::default(),
                AlternateScroll::On,
                None,
                0,
                cx.background_executor(),
                PathStyle::local(),
            )
            .subscribe(cx)
        });
        let terminal_for_view = terminal.clone();
        let terminal_for_host = terminal.clone();
        let (host, cx) = cx.add_window_view(|window, cx| TestHost {
            terminal: terminal_for_host,
            view: new_view(terminal_for_view, window, cx),
            other_focus: cx.focus_handle(),
            resize_paused: false,
            inset: point(px(0.), px(0.)),
        });
        let view = host.read_with(cx, |host, _| host.view.clone());

        host.update_in(cx, |host, window, cx| {
            window.focus(&host.other_focus, cx);
        });
        terminal.update(cx, |terminal, cx| terminal.write_output(b"\x1b[?1049hhello", cx));
        cx.simulate_resize(size(px(400.), px(201.)));
        cx.run_until_parked();

        cx.simulate_click(point(px(100.), px(100.)), Modifiers::none());
        view.update_in(cx, |view, window, cx| {
            assert!(view.focus_handle(cx).is_focused(window));
        });

        assert!(terminal.read_with(cx, |terminal, _| terminal.used_lines()) >= 1);
        let initial_line_height = terminal.read_with(cx, |terminal, _| {
            let bounds = terminal.last_content().terminal_bounds;
            assert_balanced_padding(bounds, size(px(400.), px(201.)));
            bounds.line_height
        });

        cx.simulate_resize(size(px(400.), px(202.)));
        cx.run_until_parked();
        terminal.read_with(cx, |terminal, _| {
            assert_balanced_padding(
                terminal.last_content().terminal_bounds,
                size(px(400.), px(202.)),
            );
        });

        let mut larger_typography = crate::settings::ResolvedSettings::default();
        larger_typography.terminal_font_size = 19.;
        cx.update(|_, cx| crate::fonts::install(&larger_typography, cx));
        cx.run_until_parked();
        let larger_line_height = terminal
            .read_with(cx, |terminal, _| terminal.last_content().terminal_bounds.line_height);
        assert!(larger_line_height > initial_line_height);

        assert_eq!(host.read_with(cx, |host, _| host.terminal.entity_id()), terminal.entity_id());
    }

    #[gpui::test]
    fn animated_panes_keep_the_grid_until_the_final_layout(cx: &mut TestAppContext) {
        init_test(cx);
        let terminal = cx.new(|cx| {
            TerminalBuilder::new_display_only(
                CursorShape::default(),
                AlternateScroll::On,
                None,
                0,
                cx.background_executor(),
                PathStyle::local(),
            )
            .subscribe(cx)
        });
        let (host, cx) = cx.add_window_view(|window, cx| TestHost {
            terminal: terminal.clone(),
            view: new_view(terminal.clone(), window, cx),
            other_focus: cx.focus_handle(),
            resize_paused: false,
            inset: point(px(220.), px(0.)),
        });
        cx.simulate_resize(size(px(800.), px(600.)));
        terminal.update(cx, |terminal, cx| terminal.write_output(b"\x1b[?1049hhello", cx));
        cx.run_until_parked();
        let mut output = String::from("hello");

        // Shrinking and expanding slots, including a reversal mid-animation.
        // The same terminal remains live and its origin follows the pane.
        for frames in [
            vec![(150., 10.), (70., 20.), (160., 8.), (30., 28.), (0., 32.)],
            vec![(80., 22.), (160., 12.), (220., 0.)],
        ] {
            let before =
                terminal.read_with(cx, |terminal, _| terminal.last_content().terminal_bounds);
            for (left, top) in frames {
                host.update(cx, |host, cx| {
                    host.resize_paused = true;
                    host.inset = point(px(left), px(top));
                    cx.notify();
                });
                terminal.update(cx, |terminal, cx| terminal.write_output(b"!", cx));
                output.push('!');
                cx.run_until_parked();
                terminal.read_with(cx, |terminal, _| {
                    let bounds = terminal.last_content().terminal_bounds;
                    assert_eq!(bounds.bounds.size, before.bounds.size);
                    assert_eq!(bounds.num_columns(), before.num_columns());
                    assert_eq!(bounds.num_lines(), before.num_lines());
                    assert!(
                        f32::from(bounds.bounds.origin.x - px(left) - bounds.cell_width).abs()
                            <= 1.
                    );
                    assert!(
                        f32::from(bounds.bounds.origin.y - px(top) - bounds.cell_width).abs() <= 1.
                    );
                    assert!(terminal.get_content().contains(&output));
                });
            }

            host.update(cx, |host, cx| {
                host.resize_paused = false;
                cx.notify();
            });
            cx.run_until_parked();
            let settled =
                terminal.read_with(cx, |terminal, _| terminal.last_content().terminal_bounds);
            assert_ne!(settled.num_columns(), before.num_columns());
            assert_ne!(settled.num_lines(), before.num_lines());
            let inset = host.read_with(cx, |host, _| host.inset);
            let mut local = settled;
            local.bounds.origin -= inset;
            assert_balanced_padding(local, size(px(800.) - inset.x, px(600.) - inset.y));
            host.update(cx, |_, cx| cx.notify());
            cx.run_until_parked();
            assert_eq!(
                terminal.read_with(cx, |terminal, _| terminal.last_content().terminal_bounds),
                settled
            );
        }

        // Ordinary window resizes still update immediately after the slide.
        let before = terminal.read_with(cx, |terminal, _| terminal.last_content().terminal_bounds);
        cx.simulate_resize(size(px(950.), px(700.)));
        cx.run_until_parked();
        terminal.read_with(cx, |terminal, _| {
            let bounds = terminal.last_content().terminal_bounds;
            assert!(bounds.num_columns() > before.num_columns());
            assert!(bounds.num_lines() > before.num_lines());
        });
    }

    fn assert_balanced_padding(
        terminal: terminal::TerminalBounds,
        viewport: gpui::Size<gpui::Pixels>,
    ) {
        let left = terminal.bounds.origin.x;
        let top = terminal.bounds.origin.y;
        let grid_right = left + terminal.cell_width * terminal.num_columns() as f32;
        let grid_bottom = top + terminal.line_height * terminal.num_lines() as f32;
        let right = viewport.width - grid_right;
        let bottom = viewport.height - grid_bottom;

        assert!(left > px(0.));
        assert!(f32::from(top - left).abs() <= 1.);
        assert!(right + px(1.) >= left);
        assert!(right - left <= terminal.cell_width + px(1.));
        assert!(bottom + px(1.) >= top);
        assert!(
            bottom - top <= terminal.line_height + px(2.),
            "top={top:?}, bottom={bottom:?}, line_height={:?}, lines={}",
            terminal.line_height,
            terminal.num_lines()
        );
    }
}
