//! The smallest complete native chartr plugin.
//!
//! Its view is an ordinary GPUI view mounted directly in chartr's element tree:
//! the same frame path, input, and painting as the terminal in the next tab.
//! Nothing here is a shim — `div()` is GPUI's own `div()`, and the pane it
//! returns is an `AnyView` chartr renders without a renderer in between.

use chartr_plugin::{
    Host, PaneKey, Plugin, PluginObject, Registrar, gpui,
    gpui::{Context, IntoElement, Window, div, prelude::*, px, rgb},
};

pub struct Hello {
    host: Host,
}

/// Construct the same plugin object when the example is bundled with chartr.
pub fn bundled(host: Host, cx: &mut gpui::App) -> Box<dyn PluginObject> {
    Box::new(Hello::new(host, cx))
}

impl Plugin for Hello {
    const ID: &'static str = "com.example.hello";

    fn new(host: Host, _: &mut gpui::App) -> Self {
        Self { host }
    }

    fn activate(&mut self, registrar: &mut Registrar, _: &mut gpui::App) {
        // Declaring is not building: chartr calls `view` only when the pane is
        // actually shown, so contributing a pane costs a string until then.
        registrar.add_pane("main", "Hello");
    }

    fn view(
        &mut self,
        _: &PaneKey,
        _: &chartr_plugin::InstanceContext,
        _: &mut Window,
        cx: &mut gpui::App,
    ) -> gpui::AnyView {
        let data_dir = self.host.data_dir.display().to_string();
        cx.new(|_| HelloView { data_dir }).into()
    }
}

struct HelloView {
    data_dir: String,
}

impl Render for HelloView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .items_center()
            .justify_center()
            .text_color(rgb(0xd0d0d0))
            .child("Hello from a native plugin.")
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(rgb(0x808080))
                    .child(format!("data: {}", self.data_dir)),
            )
    }
}
