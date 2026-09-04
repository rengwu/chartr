//! The contract a zeddy plugin is written against.
//!
//! All plugin kinds contribute the same thing — a **pane**: a titled surface zeddy
//! can show in the sidebar or as a tab. Nothing above the plugin host cares
//! which runtime a pane came from, which is what lets a web plugin and a native
//! one sit side by side in the same tab strip.
//!
//! # Build-time native modules
//!
//! One trait and one manifest. A native plugin's view is an ordinary
//! GPUI [`AnyView`](gpui::AnyView) mounted directly in zeddy's element tree, so
//! scrolling, resizing, focus, input, and painting use exactly the same frame
//! path as a built-in view. There is no webview, no Wasm runtime, no synthetic
//! window, no display-list replay, and no UI RPC layer.
//!
//! ```ignore
//! use zeddy_plugin::{Host, Plugin, Registrar, gpui};
//!
//! struct StarMap;
//!
//! impl Plugin for StarMap {
//!     const ID: &'static str = "com.example.starmap";
//!
//!     fn new(_: Host, _: &mut gpui::App) -> Self {
//!         Self
//!     }
//!
//!     fn activate(&mut self, registrar: &mut Registrar, _: &mut gpui::App) {
//!         registrar.add_pane("map", "Star map");
//!     }
//!
//!     fn view(&mut self, _: &PaneKey, _: &InstanceContext, _: &mut gpui::Window, cx: &mut gpui::App) -> gpui::AnyView {
//!         cx.new(|_| MapView::default()).into()
//!     }
//! }
//!
//! ```
//!
//! Native modules may use raw GPUI, compatible crates, the filesystem,
//! processes, and the network. They are linked into Chartr at build time;
//! separately compiled GPUI libraries are not an installable package format.
//!
//! # The web tier
//!
//! A web plugin has no Rust in it at all: a manifest and an entry document.
//! zeddy hosts it in an OS webview and hands it the same pane slot. See
//! [`manifest::Kind::Web`].
//!
//! # Hosted surfaces
//!
//! A hosted plugin is a declarative, separately installed package that
//! activates an operating-system surface implemented by Chartr. It carries no
//! executable plugin code. See [`manifest::Kind::Hosted`].

#![forbid(unsafe_code)]

pub mod manifest;

pub use gpui;
pub use manifest::{Capabilities, Kind, Manifest, Multiplicity, Permissions, ProjectAccess};

use std::path::PathBuf;

/// A pane a plugin contributes, addressed by the plugin's id and its own key.
///
/// Two plugins may both call a pane `"main"`; the id keeps them apart, and the
/// pair is stable across restarts so a saved layout can name one.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PaneKey {
    pub plugin: String,
    pub key: String,
}

impl PaneKey {
    pub fn new(plugin: impl Into<String>, key: impl Into<String>) -> Self {
        Self { plugin: plugin.into(), key: key.into() }
    }
}

/// A pane's declaration: what the sidebar and the tab strip put on it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaneSpec {
    pub key: PaneKey,
    pub title: String,
}

/// Stable ownership handed to one concrete pane instance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceContext {
    /// Stable within the owning space and retained when this item is restored.
    pub instance_id: u64,
    pub space: String,
    pub project_dir: Option<PathBuf>,
    pub bound_session: Option<String>,
}

/// What a plugin declares during [`Plugin::activate`].
///
/// Declaring is separate from building. Activation runs once, and zeddy calls
/// [`Plugin::view`] only when a pane is actually shown — so a plugin that
/// contributes ten panes costs ten strings until one is opened.
#[derive(Debug, Default)]
pub struct Registrar {
    plugin: String,
    panes: Vec<PaneSpec>,
    settings: bool,
}

impl Registrar {
    pub fn new(plugin: impl Into<String>) -> Self {
        Self { plugin: plugin.into(), panes: Vec::new(), settings: false }
    }

    /// Contribute a pane. `key` is this plugin's own name for it.
    pub fn add_pane(&mut self, key: impl Into<String>, title: impl Into<String>) -> &mut Self {
        self.panes
            .push(PaneSpec { key: PaneKey::new(self.plugin.clone(), key), title: title.into() });
        self
    }

    pub fn panes(&self) -> &[PaneSpec] {
        &self.panes
    }

    /// Advertise one user-global settings contribution. Its view remains lazy.
    pub fn add_settings(&mut self) -> &mut Self {
        self.settings = true;
        self
    }

    pub fn has_settings(&self) -> bool {
        self.settings
    }
}

/// What zeddy hands a plugin at construction.
///
/// Deliberately small. Every field here is a promise zeddy has to keep across
/// versions, so the contract grows only when a real plugin needs it to.
#[derive(Debug, Clone)]
pub struct Host {
    /// This plugin's private directory. It survives disablement, replacement,
    /// and upgrade, and is removed only when the user asks for it to be.
    pub data_dir: PathBuf,
    /// The directory the plugin itself was installed into. Read-only by
    /// convention: a reload replaces it.
    pub plugin_dir: PathBuf,
}

/// A native plugin.
pub trait Plugin: Sized + 'static {
    /// The reverse-DNS id, which must equal the manifest's.
    const ID: &'static str;

    fn new(host: Host, cx: &mut gpui::App) -> Self;

    /// Declare what this plugin contributes. Called once, at load.
    fn activate(&mut self, registrar: &mut Registrar, cx: &mut gpui::App);

    /// Build the view for one of the panes declared in [`Plugin::activate`].
    ///
    /// Called when the pane is first shown, and again after a reload.
    fn view(
        &mut self,
        pane: &PaneKey,
        context: &InstanceContext,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> gpui::AnyView;

    /// Build this plugin's user-global Settings contribution on first open.
    fn settings(
        &mut self,
        _window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) -> Option<gpui::AnyView> {
        None
    }
}

/// The object-safe face of [`Plugin`] used by Chartr's build-time registry.
pub trait PluginObject {
    fn id(&self) -> &str;
    fn activate(&mut self, registrar: &mut Registrar, cx: &mut gpui::App);
    fn view(
        &mut self,
        pane: &PaneKey,
        context: &InstanceContext,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> gpui::AnyView;
    fn settings(&mut self, window: &mut gpui::Window, cx: &mut gpui::App) -> Option<gpui::AnyView>;
}

impl<P: Plugin> PluginObject for P {
    fn id(&self) -> &str {
        P::ID
    }

    fn activate(&mut self, registrar: &mut Registrar, cx: &mut gpui::App) {
        Plugin::activate(self, registrar, cx)
    }

    fn view(
        &mut self,
        pane: &PaneKey,
        context: &InstanceContext,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    ) -> gpui::AnyView {
        Plugin::view(self, pane, context, window, cx)
    }

    fn settings(&mut self, window: &mut gpui::Window, cx: &mut gpui::App) -> Option<gpui::AnyView> {
        Plugin::settings(self, window, cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_registrar_stamps_every_pane_with_its_plugin() {
        let mut registrar = Registrar::new("com.example.starmap");
        registrar.add_pane("map", "Star map").add_pane("legend", "Legend");
        assert_eq!(registrar.panes().len(), 2);
        assert!(registrar.panes().iter().all(|p| p.key.plugin == "com.example.starmap"));
    }

    #[test]
    fn settings_contributions_are_opt_in() {
        let mut registrar = Registrar::new("com.example.settings");
        assert!(!registrar.has_settings());
        registrar.add_settings();
        assert!(registrar.has_settings());
    }
}
