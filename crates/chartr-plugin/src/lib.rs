//! The contract a chartr plugin is written against.
//!
//! All plugin kinds contribute the same thing — a **pane**: a titled surface chartr
//! can show in the sidebar or as a tab. Nothing above the plugin host cares
//! which runtime a pane came from, which is what lets a web plugin and a native
//! one sit side by side in the same tab strip.
//!
//! # Build-time native modules
//!
//! One trait and one manifest. A native plugin's view is an ordinary
//! GPUI [`AnyView`](gpui::AnyView) mounted directly in chartr's element tree, so
//! scrolling, resizing, focus, input, and painting use exactly the same frame
//! path as a built-in view. There is no webview, no Wasm runtime, no synthetic
//! window, no display-list replay, and no UI RPC layer.
//!
//! ```ignore
//! use chartr_plugin::{Host, Plugin, Registrar, gpui};
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
//! processes, and the network. They are linked into chartr at build time;
//! separately compiled GPUI libraries are not an installable package format.
//!
//! # The web tier
//!
//! A web plugin has no Rust in it at all: a manifest and an entry document.
//! chartr hosts it in an OS webview and hands it the same pane slot. See
//! [`manifest::Kind::Web`].
//!
//! # Hosted surfaces
//!
//! A hosted plugin is a declarative, separately installed package that
//! activates an operating-system surface implemented by chartr. It carries no
//! executable plugin code. See [`manifest::Kind::Hosted`].

#![forbid(unsafe_code)]

pub mod manifest;
pub mod services;
pub mod settings;

pub use gpui;
pub use manifest::{Capabilities, Kind, Manifest, Multiplicity, Permissions, ProjectAccess};

use std::{path::PathBuf, rc::Rc};

type TerminalLaunchHandler = dyn Fn(Vec<u8>, &mut gpui::App);
type TerminalPrepareHandler =
    dyn Fn(&mut gpui::App) -> gpui::Task<Result<PreparedTerminal, String>>;
type TerminalSendHandler = dyn Fn(&[u8]) -> Result<(), String>;
type TerminalFocusHandler = dyn Fn(&str, &mut gpui::Window, &mut gpui::App) -> bool;

/// An attached shell with no agent input yet. A plugin may record a claim using
/// its real session id before starting an agent, and can report delivery errors.
pub struct PreparedTerminal {
    pub id: String,
    send: Box<TerminalSendHandler>,
}

impl PreparedTerminal {
    pub fn new(id: String, send: impl Fn(&[u8]) -> Result<(), String> + 'static) -> Self {
        Self { id, send: Box::new(send) }
    }
    pub fn send(&self, input: &[u8]) -> Result<(), String> {
        (self.send)(input)
    }
}

/// A space-scoped capability for opening a chartr-owned terminal.
///
/// Native plugins are linked into chartr and trusted, but creating a terminal
/// still belongs to the host: it must be inserted into the pane's owning space
/// and follow the same persistence and close lifecycle as a user-created one.
#[derive(Clone)]
pub struct TerminalLauncher {
    launch: Rc<TerminalLaunchHandler>,
    prepare: Option<Rc<TerminalPrepareHandler>>,
    focus: Option<Rc<TerminalFocusHandler>>,
}

impl TerminalLauncher {
    pub fn new(launch: impl Fn(Vec<u8>, &mut gpui::App) + 'static) -> Self {
        Self { launch: Rc::new(launch), prepare: None, focus: None }
    }

    pub fn with_prepare(
        mut self,
        prepare: impl Fn(&mut gpui::App) -> gpui::Task<Result<PreparedTerminal, String>> + 'static,
    ) -> Self {
        self.prepare = Some(Rc::new(prepare));
        self
    }

    pub fn prepare(&self, cx: &mut gpui::App) -> gpui::Task<Result<PreparedTerminal, String>> {
        match &self.prepare {
            Some(prepare) => prepare(cx),
            None => gpui::Task::ready(Err("This view cannot create a terminal.".into())),
        }
    }

    /// Add navigation to a session already owned by this space.
    pub fn with_focus(
        mut self,
        focus: impl Fn(&str, &mut gpui::Window, &mut gpui::App) -> bool + 'static,
    ) -> Self {
        self.focus = Some(Rc::new(focus));
        self
    }

    pub fn focus(&self, session: &str, window: &mut gpui::Window, cx: &mut gpui::App) -> bool {
        self.focus.as_ref().is_some_and(|focus| focus(session, window, cx))
    }

    /// Open a terminal and queue the bytes it should receive first.
    pub fn launch(&self, initial_input: Vec<u8>, cx: &mut gpui::App) {
        (self.launch)(initial_input, cx);
    }
}

impl std::fmt::Debug for TerminalLauncher {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("TerminalLauncher(..)")
    }
}

impl PartialEq for TerminalLauncher {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.launch, &other.launch)
    }
}

impl Eq for TerminalLauncher {}

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
    /// Stable persistence key for the owning space.
    pub space: String,
    /// User-visible name of the owning space.
    pub space_name: String,
    pub project_dir: Option<PathBuf>,
    pub bound_session: Option<String>,
    /// Host capability available to trusted native panes.
    pub terminal: TerminalLauncher,
    /// Live services exported by enabled native plugins in this catalog.
    pub services: services::Services,
    pub plugin_settings: services::PluginSettings,
}

/// What a plugin declares during [`Plugin::activate`].
///
/// Declaring is separate from building. Activation runs once, and chartr calls
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

/// What chartr hands a plugin at construction.
///
/// Deliberately small. Every field here is a promise chartr has to keep across
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
/// A persistent background service, independent of any open plugin pane.
/// Keep this query cheap: the host reads it periodically on the UI thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackgroundStatus {
    pub label: String,
    pub detail: String,
    pub state: BackgroundState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackgroundState {
    Idle,
    Running,
    Error,
}

pub trait Plugin: Sized + 'static {
    /// The reverse-DNS id, which must equal the manifest's.
    const ID: &'static str;

    fn new(host: Host, cx: &mut gpui::App) -> Self;

    /// Declare what this plugin contributes. Called once, at load.
    fn activate(&mut self, registrar: &mut Registrar, cx: &mut gpui::App);

    fn background_status(&self, _cx: &gpui::App) -> Option<BackgroundStatus> {
        None
    }

    fn services(&self) -> Vec<services::ServiceExport> {
        Vec::new()
    }

    /// Connect background consumers to this catalog without opening a pane.
    fn connect_services(&mut self, _services: services::Services, _cx: &mut gpui::App) {}

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

/// The object-safe face of [`Plugin`] used by chartr's build-time registry.
pub trait PluginObject {
    fn background_status(&self, cx: &gpui::App) -> Option<BackgroundStatus>;
    fn id(&self) -> &str;
    fn services(&self) -> Vec<services::ServiceExport>;
    fn connect_services(&mut self, services: services::Services, cx: &mut gpui::App);
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
    fn connect_services(&mut self, services: services::Services, cx: &mut gpui::App) {
        Plugin::connect_services(self, services, cx);
    }

    fn background_status(&self, cx: &gpui::App) -> Option<BackgroundStatus> {
        Plugin::background_status(self, cx)
    }

    fn services(&self) -> Vec<services::ServiceExport> {
        Plugin::services(self)
    }

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
