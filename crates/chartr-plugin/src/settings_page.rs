//! Content mounted on the Settings window's surface.

use gpui::{
    AnyElement, AnyView, App, AppContext, Context, ElementId, Entity, InteractiveElement,
    IntoElement, ParentElement, Render, RenderOnce, StatefulInteractiveElement, Styled,
    Subscription, Window, div,
};

/// Native settings cannot supply an arbitrary pane view as their page shell.
pub trait RenderSettings: 'static + Sized {
    fn render_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) -> SettingsPage;
}

#[derive(Clone)]
pub struct SettingsView(AnyView);

impl SettingsView {
    pub fn new<V: RenderSettings>(view: Entity<V>, cx: &mut App) -> Self {
        Self(
            cx.new(|cx| {
                let subscription = cx.observe(&view, |_, _, cx| cx.notify());
                SettingsMount { view, _subscription: subscription }
            })
            .into(),
        )
    }

    pub fn into_view(self) -> AnyView {
        self.0
    }
}

struct SettingsMount<V> {
    view: Entity<V>,
    _subscription: Subscription,
}

impl<V: RenderSettings> Render for SettingsMount<V> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.view.update(cx, |view, cx| view.render_settings(window, cx))
    }
}

/// A transparent settings page. The host owns the page background; plugins own
/// controls, cards, and dialogs within it. Unlike a `Div`, this shell deliberately
/// does not implement `Styled`, so it cannot acquire a separate background.
///
/// ```compile_fail
/// use chartr_plugin::{SettingsPage, gpui::Styled};
/// SettingsPage::flow("settings").bg(chartr_plugin::gpui::rgb(0));
/// ```
#[derive(IntoElement)]
pub struct SettingsPage {
    id: ElementId,
    layout: Layout,
    children: Vec<AnyElement>,
}

enum Layout {
    Flow,
    Fill,
    Scroll,
}

impl SettingsPage {
    /// Content whose height participates in the host's scrolling page.
    pub fn flow(id: impl Into<ElementId>) -> Self {
        Self { id: id.into(), layout: Layout::Flow, children: Vec::new() }
    }

    /// A management page with its own list sizing and scrolling.
    pub fn fill(id: impl Into<ElementId>) -> Self {
        Self { layout: Layout::Fill, ..Self::flow(id) }
    }

    /// A management page that scrolls as a whole within the host's panel.
    pub fn scroll(id: impl Into<ElementId>) -> Self {
        Self { layout: Layout::Scroll, ..Self::flow(id) }
    }
}

impl ParentElement for SettingsPage {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for SettingsPage {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let page = div().id(self.id).flex().flex_col().w_full().gap_4();
        let page = match self.layout {
            Layout::Flow => page,
            Layout::Fill => page.h_full().min_h_0(),
            Layout::Scroll => page.h_full().min_h_0().overflow_y_scroll(),
        };
        page.children(self.children)
    }
}
