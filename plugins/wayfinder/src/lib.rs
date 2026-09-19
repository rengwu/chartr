//! The web pane's narrow host bridge. No native Wayfinder UI lives here.
mod model;
mod prompt;
#[cfg(test)]
mod tests;

use chartr_plugin::{
    InstanceContext,
    services::{AGENT_SERVICE, Agents, SKILLS_SERVICE, SkillCatalog, Skills},
};
use gpui::{AnyWindowHandle, AsyncApp};
use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(tag = "action")]
pub enum Action {
    #[serde(rename = "wayfinder.snapshot")]
    Snapshot,
    #[serde(rename = "wayfinder.preview")]
    Preview {
        #[serde(flatten)]
        selection: Selection,
    },
    #[serde(rename = "wayfinder.launch")]
    Launch { preview: u64, agent: String },
    #[serde(rename = "wayfinder.release")]
    Release { slug: String, ticket: u32, session: String },
    #[serde(rename = "wayfinder.focus")]
    Focus { slug: String, ticket: u32 },
    #[serde(rename = "wayfinder.open")]
    Open { slug: String, ticket: Option<u32>, target: Option<String> },
    #[serde(rename = "wayfinder.settings")]
    Settings { provider: String },
}

#[derive(Clone, Deserialize)]
pub struct Selection {
    slug: String,
    ticket: u32,
    method: Option<String>,
    #[serde(default)]
    note: String,
}

struct Preview {
    id: u64,
    document: String,
    selection: Selection,
    map: model::Map,
    prompt: prompt::Prompt,
}

pub struct Bridge {
    context: InstanceContext,
    preview: Option<Preview>,
    next_preview: u64,
}

impl Bridge {
    /// No bundled-plugin exemption: installed documents need the same grant.
    pub fn for_web(
        context: Option<InstanceContext>,
        permissions: &chartr_plugin::manifest::Permissions,
    ) -> Option<Self> {
        context.filter(|_| permissions.wayfinder).map(Self::new)
    }

    fn new(context: InstanceContext) -> Self {
        Self { context, preview: None, next_preview: 0 }
    }

    fn root(&self) -> Result<PathBuf, String> {
        self.context.project_dir.clone().ok_or("Open a folder space to use Wayfinder.".into())
    }

    async fn maps(&self, cx: &AsyncApp) -> Result<Vec<model::Map>, String> {
        let root = self.root()?;
        cx.background_executor()
            .spawn(async move { model::discover(&root).map_err(|error| format!("{error:#}")) })
            .await
    }

    async fn skills(&self, cx: &mut AsyncApp) -> Result<SkillCatalog, String> {
        cx.update(|cx| {
            self.context
                .services
                .get::<Skills>(SKILLS_SERVICE)
                .map(|service| service.scan(cx))
                .ok_or("Enable Skills and add a skill source.")
        })?
        .await
    }

    /// Executed in the pane's ordered queue. `alive` is checked again before any
    /// launch input; the worker remains alive long enough to roll back on close.
    pub async fn handle(
        &mut self,
        action: Action,
        document: &str,
        window: AnyWindowHandle,
        alive: impl Fn() -> bool,
        cx: &mut AsyncApp,
    ) -> Result<Value, String> {
        if !alive() {
            return Err("Wayfinder was closed or navigated away.".into());
        }
        match action {
            Action::Snapshot => {
                let agents = cx.update(|cx| {
                    self.context
                        .services
                        .get::<Agents>(AGENT_SERVICE)
                        .ok_or("Enable Agent and register an agent.")?
                        .list(cx)
                });
                let skills = self.skills(cx).await;
                let (agents, agent_error) = match agents {
                    Ok(names) => (names, None),
                    Err(error) => (Vec::new(), Some(error)),
                };
                let (skills, skill_error) = match skills {
                    Ok(catalog) => (catalog, None),
                    Err(error) => (SkillCatalog::default(), Some(error)),
                };
                let maps = if self.context.project_dir.is_some() {
                    self.maps(cx).await?
                } else {
                    Vec::new()
                };
                // Markdown conversion, like file discovery, stays off the UI thread.
                let maps = cx
                    .background_executor()
                    .spawn(async move { maps.iter().map(map_json).collect::<Vec<_>>() })
                    .await;
                Ok(json!({ "space": self.context.space_name, "folder": self.context.project_dir,
                    "maps": maps, "agents": agents, "agent_error": agent_error,
                    "skills": skills.skills.iter().map(|skill| skill.reference()).collect::<Vec<_>>(),
                    "skill_error": skill_error, "warnings": skills.warnings }))
            }
            Action::Preview { selection } => {
                // A failed preview must not leave an older one launchable.
                self.preview = None;
                self.root()?;
                if selection.note.len() > 16 * 1024 {
                    return Err("Direction exceeds 16 KiB.".into());
                }
                let map = find_map(self.maps(cx).await?, &selection.slug)?;
                let ticket = map.ticket(selection.ticket).ok_or("The ticket was removed.")?;
                if !ticket.frontier || ticket.method().is_none() {
                    return Err("This ticket is not ready to launch.".into());
                }
                let skills = self.skills(cx).await?;
                let prompt = prompt::compose(
                    &map,
                    ticket,
                    &skills,
                    selection.method.as_deref(),
                    &selection.note,
                )?;
                self.next_preview += 1;
                let result = json!({ "preview": self.next_preview, "text": prompt.text, "sources": prompt.sources });
                self.preview = Some(Preview {
                    id: self.next_preview,
                    document: document.into(),
                    selection,
                    map,
                    prompt,
                });
                Ok(result)
            }
            Action::Launch { preview, agent } => {
                let expected = self
                    .preview
                    .take()
                    .filter(|p| p.id == preview && p.document == document)
                    .ok_or("Preview the current prompt before launching.")?;
                self.launch(expected, &agent, alive, cx).await.map(|id| json!({ "session": id }))
            }
            Action::Release { slug, ticket, session } => {
                if session.is_empty() {
                    return Err("No claim to release.".into());
                }
                let map = find_map(self.maps(cx).await?, &slug)?;
                let path = map.ticket(ticket).ok_or("The ticket was removed.")?.path.clone();
                let root = self.root()?;
                if !alive() {
                    return Err("Wayfinder was closed.".into());
                }
                cx.background_executor()
                    .spawn(async move {
                        model::release(&root, &path, &session).map_err(|error| format!("{error:#}"))
                    })
                    .await?;
                Ok(json!(true))
            }
            Action::Focus { slug, ticket } => {
                let map = find_map(self.maps(cx).await?, &slug)?;
                let session = &map.ticket(ticket).ok_or("The ticket was removed.")?.claimed_by;
                if session.is_empty() || !alive() {
                    return Err("No claimed session to open.".into());
                }
                let focused = window
                    .update(cx, |_, window, cx| self.context.terminal.focus(session, window, cx))
                    .map_err(|error| error.to_string())?;
                if !focused {
                    return Err("That session is no longer open in this space. Release its claim only if the work has stopped.".into());
                }
                Ok(json!(true))
            }
            Action::Settings { provider } => {
                let id = match provider.as_str() {
                    "agent" => AGENT_SERVICE,
                    "skills" => SKILLS_SERVICE,
                    _ => return Err("Unknown setup provider.".into()),
                };
                window
                    .update(cx, |_, window, cx| {
                        self.context.plugin_settings.open(Some(id), window, cx)
                    })
                    .map_err(|error| error.to_string())?;
                Ok(json!(true))
            }
            Action::Open { slug, ticket, target } => {
                let map = find_map(self.maps(cx).await?, &slug)?;
                let base = match ticket {
                    Some(number) => {
                        map.ticket(number).ok_or("The ticket was removed.")?.path.clone()
                    }
                    None => map.directory.join("map.md"),
                };
                let root = self.root()?;
                let url = cx
                    .background_executor()
                    .spawn(async move { open_target(&root, &base, target.as_deref()) })
                    .await?;
                if !alive() {
                    return Err("Wayfinder was closed.".into());
                }
                cx.update(|cx| cx.open_url(&url));
                Ok(json!(true))
            }
        }
    }

    async fn launch(
        &self,
        expected: Preview,
        agent: &str,
        alive: impl Fn() -> bool,
        cx: &mut AsyncApp,
    ) -> Result<String, String> {
        let root = self.root()?;
        let mut claimed = None;
        let result = async {
            let catalog = self.skills(cx).await?;
            self.revalidate(&expected, &catalog)?;
            // Refuse an unavailable agent before opening even an idle terminal.
            cx.update(|cx| {
                self.context
                    .services
                    .get::<Agents>(AGENT_SERVICE)
                    .ok_or("Agent is disabled.")?
                    .prepare(agent, &expected.prompt.text, cx)
            })?;
            if !alive() {
                return Err("Wayfinder was closed.".into());
            }
            let terminal = cx.update(|cx| self.context.terminal.prepare(cx)).await?;
            if !alive() {
                return Err("Wayfinder was closed.".into());
            }
            {
                let map = &expected.map;
                let number = expected.selection.ticket;
                let map_copy = map.clone();
                let root = root.clone();
                let session = terminal.id.clone();
                cx.background_executor()
                    .spawn(async move {
                        model::claim(&root, &map_copy, number, &session)
                            .map_err(|error| format!("{error:#}"))
                    })
                    .await?;
                claimed = Some((map.ticket(number).unwrap().path.clone(), terminal.id.clone()));
            }
            let catalog = self.skills(cx).await?;
            self.revalidate(&expected, &catalog)?;
            if !alive() {
                return Err("Wayfinder was closed or navigated away.".into());
            }
            let input = cx.update(|cx| {
                if self.context.services.get::<Skills>(SKILLS_SERVICE).is_none() {
                    return Err("Skills was disabled before launch.".into());
                }
                self.context
                    .services
                    .get::<Agents>(AGENT_SERVICE)
                    .ok_or("Agent was disabled before launch.")?
                    .prepare(agent, &expected.prompt.text, cx)
            })?;
            terminal.send(&input)?;
            Ok(terminal.id)
        }
        .await;
        if result.is_err()
            && let Some((path, session)) = claimed
        {
            let cleanup = cx
                .background_executor()
                .spawn(async move {
                    model::release(&root, &path, &session).map_err(|error| format!("{error:#}"))
                })
                .await;
            if let Err(cleanup) = cleanup {
                return Err(format!(
                    "{} Claim cleanup also failed: {cleanup}",
                    result.unwrap_err()
                ));
            }
        }
        result
    }

    fn revalidate(&self, expected: &Preview, catalog: &SkillCatalog) -> Result<(), String> {
        let ticket =
            expected.map.ticket(expected.selection.ticket).ok_or("The ticket was removed.")?;
        let current = prompt::compose(
            &expected.map,
            ticket,
            catalog,
            expected.selection.method.as_deref(),
            &expected.selection.note,
        )?;
        if current != expected.prompt {
            return Err(
                "Skill sources changed after preview. Review the prompt and try again.".into()
            );
        }
        Ok(())
    }
}

fn find_map(maps: Vec<model::Map>, slug: &str) -> Result<model::Map, String> {
    maps.into_iter()
        .find(|map| map.slug == slug)
        .ok_or("The map was removed. Refresh and try again.".into())
}

fn map_json(map: &model::Map) -> Value {
    json!({ "slug": map.slug, "title": map.title, "html": markdown(&map.body),
        "destination": map.destination, "finished": map.finished(), "frontier": map.frontier(),
        "warnings": map.warnings, "fog": map.fog.iter().map(|fog| json!({"title": fog.title, "clears_with": fog.clears_with})).collect::<Vec<_>>(),
        "tickets": map.tickets.iter().map(|t| json!({ "number": t.number, "title": t.title,
            "kind": t.kind, "state": t.state(), "frontier": t.frontier, "html": markdown(&t.body),
            "answer_html": markdown(&t.answer), "blockers": t.blockers, "assets": t.assets,
            "claimed_by": t.claimed_by, "warnings": t.warnings })).collect::<Vec<_>>() })
}

/// Markdown is data, never executable web content. Raw HTML is shown as text;
/// images are links (no external fetches), and the UI brokers link activation.
fn markdown(source: &str) -> String {
    use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd, html};
    let events = Parser::new_ext(
        source,
        Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS,
    )
    .map(|event| match event {
        Event::Html(text) | Event::InlineHtml(text) => Event::Text(text),
        Event::Start(Tag::Link { link_type, dest_url, title, id })
        | Event::Start(Tag::Image { link_type, dest_url, title, id }) => Event::Start(Tag::Link {
            link_type,
            dest_url: if allowed_link(&dest_url) { dest_url } else { "".into() },
            title,
            id,
        }),
        Event::End(TagEnd::Image) => Event::End(TagEnd::Link),
        event => event,
    });
    let mut html = String::new();
    html::push_html(&mut html, events);
    html
}

fn allowed_link(link: &str) -> bool {
    !link.chars().any(char::is_control)
        && !link.starts_with("//")
        && (!link.contains(':')
            || url::Url::parse(link).is_ok_and(|u| matches!(u.scheme(), "http" | "https")))
}

fn open_target(root: &Path, base: &Path, target: Option<&str>) -> Result<String, String> {
    if let Some(target) = target {
        if !allowed_link(target) {
            return Err("Only project files and HTTP(S) links can be opened.".into());
        }
        if let Ok(url) = url::Url::parse(target) {
            return Ok(url.into());
        }
    }
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let path = match target {
        Some(target) => {
            let target = target.split('#').next().unwrap_or_default();
            let target = percent_encoding::percent_decode_str(target)
                .decode_utf8()
                .map_err(|error| error.to_string())?;
            base.parent().ok_or("Missing file directory")?.join(target.as_ref())
        }
        None => base.to_owned(),
    }
    .canonicalize()
    .map_err(|error| error.to_string())?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err("The file must be inside this space.".into());
    }
    url::Url::from_file_path(path).map(String::from).map_err(|_| "Invalid file path.".into())
}
