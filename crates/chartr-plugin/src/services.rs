//! Typed, catalog-scoped services between trusted native plugins.
//!
//! Providers own their exports. Consumers discover them on demand; disabling a
//! provider removes it from the directory without closing unrelated panes.
use std::{
    any::{Any, TypeId},
    cell::RefCell,
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
};

use crate::gpui;

#[derive(Clone)]
pub struct ServiceExport(Rc<dyn Any>);

impl ServiceExport {
    pub fn new<T: 'static>(service: T) -> Self {
        Self(Rc::new(service))
    }
}

type Directory = HashMap<(String, TypeId), Rc<dyn Any>>;

#[derive(Clone, Default)]
pub struct Services(Rc<RefCell<Directory>>);

impl Services {
    pub fn publish(&self, plugin: &str, exports: Vec<ServiceExport>) {
        self.remove(plugin);
        for ServiceExport(service) in exports {
            self.0.borrow_mut().insert((plugin.to_owned(), service.as_ref().type_id()), service);
        }
    }

    pub fn remove(&self, plugin: &str) {
        self.0.borrow_mut().retain(|(id, _), _| id != plugin);
    }

    /// Discover enabled providers of a common contract, in stable plugin-ID order.
    pub fn all<T: 'static>(&self) -> Vec<(String, Rc<T>)> {
        let mut providers: Vec<_> = self
            .0
            .borrow()
            .iter()
            .filter_map(|((id, kind), value)| {
                (*kind == TypeId::of::<T>())
                    .then(|| (id.clone(), value.clone().downcast::<T>().unwrap()))
            })
            .collect();
        providers.sort_by(|a, b| a.0.cmp(&b.0));
        providers
    }

    pub fn get<T: 'static>(&self, plugin: &str) -> Option<Rc<T>> {
        self.0.borrow().get(&(plugin.to_owned(), TypeId::of::<T>()))?.clone().downcast().ok()
    }
}

impl std::fmt::Debug for Services {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Services(..)")
    }
}
impl PartialEq for Services {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Services {}

pub const AGENT_SERVICE: &str = "com.chartr.agent";
pub const SKILLS_SERVICE: &str = "com.chartr.skills";
pub const PROMPTS_SERVICE: &str = "com.chartr.prompts";

/// A reusable prompt. Titles are display metadata; only `prompt` is injected.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SavedPrompt {
    pub id: String,
    pub title: String,
    pub prompt: String,
}

type PromptList = dyn Fn(&gpui::App) -> Result<Vec<SavedPrompt>, String>;

/// Prompts owns persistence. Consumers resolve stable IDs against a fresh list
/// and use the body verbatim, without adding the title or other formatting.
pub struct Prompts(Box<PromptList>);

impl Prompts {
    pub fn new(list: impl Fn(&gpui::App) -> Result<Vec<SavedPrompt>, String> + 'static) -> Self {
        Self(Box::new(list))
    }

    pub fn list(&self, cx: &gpui::App) -> Result<Vec<SavedPrompt>, String> {
        (self.0)(cx)
    }

    pub fn resolve(&self, id: &str, cx: &gpui::App) -> Result<SavedPrompt, String> {
        self.list(cx)?
            .into_iter()
            .find(|prompt| prompt.id == id)
            .ok_or_else(|| "The saved prompt no longer exists.".into())
    }
}

type Configure = dyn Fn(Option<&str>, &mut gpui::Window, &mut gpui::App);

/// Navigate to a provider's settings, or the plugin list when it is disabled.
#[derive(Clone)]
pub struct PluginSettings(Rc<Configure>);
impl PluginSettings {
    pub fn new(open: impl Fn(Option<&str>, &mut gpui::Window, &mut gpui::App) + 'static) -> Self {
        Self(Rc::new(open))
    }
    pub fn open(&self, plugin: Option<&str>, window: &mut gpui::Window, cx: &mut gpui::App) {
        (self.0)(plugin, window, cx)
    }
}
impl std::fmt::Debug for PluginSettings {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PluginSettings(..)")
    }
}
impl PartialEq for PluginSettings {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for PluginSettings {}

type AgentList = dyn Fn(&gpui::App) -> Result<Vec<String>, String>;
type AgentInput = dyn Fn(&str, &str, &gpui::App) -> Result<Vec<u8>, String>;

/// Agent owns validation, adapters, quoting and prompt delivery.
pub struct Agents {
    list: Box<AgentList>,
    prepare: Box<AgentInput>,
}

impl Agents {
    pub fn new(
        list: impl Fn(&gpui::App) -> Result<Vec<String>, String> + 'static,
        prepare: impl Fn(&str, &str, &gpui::App) -> Result<Vec<u8>, String> + 'static,
    ) -> Self {
        Self { list: Box::new(list), prepare: Box::new(prepare) }
    }

    pub fn list(&self, cx: &gpui::App) -> Result<Vec<String>, String> {
        (self.list)(cx)
    }
    pub fn prepare(&self, name: &str, prompt: &str, cx: &gpui::App) -> Result<Vec<u8>, String> {
        (self.prepare)(name, prompt, cx)
    }
}

/// Optional chat-launch preparation. Agent still owns saved arguments, environment
/// and quoting; the host owns terminal creation and conversation observation.
pub struct ConversationLaunch {
    pub input: Vec<u8>,
    pub integration: Option<String>,
    pub opencode: Option<OpenCodeConversation>,
}

pub struct OpenCodeConversation {
    pub prompt: String,
    /// A saved continuation/session option must not be replaced by a fresh session.
    pub reuse: bool,
    pub model: Option<String>,
    pub agent: Option<String>,
}

type ConversationInput = dyn Fn(&str, &str, &gpui::App) -> Result<ConversationLaunch, String>;
pub struct ConversationAgents(Box<ConversationInput>);
impl ConversationAgents {
    pub fn new(
        prepare: impl Fn(&str, &str, &gpui::App) -> Result<ConversationLaunch, String> + 'static,
    ) -> Self {
        Self(Box::new(prepare))
    }
    pub fn prepare(
        &self,
        name: &str,
        prompt: &str,
        cx: &gpui::App,
    ) -> Result<ConversationLaunch, String> {
        (self.0)(name, prompt, cx)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skill {
    pub source: String,
    pub name: String,
    pub directory: PathBuf,
    pub commit: String,
    pub body: String,
    pub shadowed: bool,
}

impl Skill {
    pub fn reference(&self) -> String {
        format!("{}/{}", self.source, self.name)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SkillCatalog {
    pub skills: Vec<Skill>,
    pub warnings: Vec<String>,
}

impl SkillCatalog {
    /// Qualified references are exact pins; bare names follow enabled source order.
    pub fn resolve(&self, reference: &str) -> Option<&Skill> {
        self.skills.iter().find(|skill| {
            if reference.contains('/') {
                skill.reference().eq_ignore_ascii_case(reference)
            } else {
                !skill.shadowed && skill.name.eq_ignore_ascii_case(reference)
            }
        })
    }
}

type SkillScan = dyn Fn(&mut gpui::App) -> gpui::Task<Result<SkillCatalog, String>>;

/// Skills owns source order, enablement, discovery and reading registered sources.
pub struct Skills(Box<SkillScan>);

impl Skills {
    pub fn new(
        scan: impl Fn(&mut gpui::App) -> gpui::Task<Result<SkillCatalog, String>> + 'static,
    ) -> Self {
        Self(Box::new(scan))
    }
    pub fn scan(&self, cx: &mut gpui::App) -> gpui::Task<Result<SkillCatalog, String>> {
        (self.0)(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn service_removal_and_replacement_reach_existing_consumers() {
        let directory = Services::default();
        let consumer = directory.clone();
        directory.publish("provider", vec![ServiceExport::new(7u32)]);
        assert_eq!(*consumer.get::<u32>("provider").unwrap(), 7);
        directory.publish("provider", vec![ServiceExport::new(9u32)]);
        assert_eq!(*consumer.get::<u32>("provider").unwrap(), 9);
        directory.remove("provider");
        assert!(consumer.get::<u32>("provider").is_none());
    }
}

/// A common asynchronous template contract. IDs are stable within the exporting
/// plugin; consumers persist (plugin ID, template ID), never display titles.
/// Bodies are literal Markdown, not recursively evaluated or executed.
/// Providers own their data and may use the requesting project to build content.
type TemplateList =
    dyn Fn(Option<PathBuf>, &mut gpui::App) -> gpui::Task<Result<Vec<SavedPrompt>, String>>;
pub struct PromptTemplates(Box<TemplateList>);
impl PromptTemplates {
    pub fn new(
        list: impl Fn(Option<PathBuf>, &mut gpui::App) -> gpui::Task<Result<Vec<SavedPrompt>, String>>
        + 'static,
    ) -> Self {
        Self(Box::new(list))
    }
    pub fn list(
        &self,
        project: Option<PathBuf>,
        cx: &mut gpui::App,
    ) -> gpui::Task<Result<Vec<SavedPrompt>, String>> {
        (self.0)(project, cx)
    }
}

/// Provider-owned content changed. Consumers subscribe without keeping provider panes open.
pub struct TemplateChanges;
struct TemplateChangeBus(gpui::Entity<TemplateChanges>);
impl gpui::Global for TemplateChangeBus {}
impl PromptTemplates {
    pub fn changes(cx: &mut gpui::App) -> gpui::Entity<TemplateChanges> {
        use gpui::AppContext;
        if let Some(bus) = cx.try_global::<TemplateChangeBus>() {
            return bus.0.clone();
        }
        let signal = cx.new(|_| TemplateChanges);
        cx.set_global(TemplateChangeBus(signal.clone()));
        signal
    }
    pub fn changed(cx: &mut gpui::App) {
        Self::changes(cx).update(cx, |_, cx| cx.notify());
    }
}
