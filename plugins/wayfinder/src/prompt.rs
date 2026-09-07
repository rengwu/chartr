//! One dispatcher, with source-owned methods selected from the ticket itself.
use super::model::{Map, Ticket};
use chartr_plugin::services::{Skill, SkillCatalog};

pub const CONVENTION: &str = include_str!("../TRACKER-CONVENTION.md");

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Prompt {
    pub text: String,
    pub sources: Vec<String>,
}

pub fn compose(
    map: &Map,
    ticket: &Ticket,
    catalog: &SkillCatalog,
    override_ref: Option<&str>,
    request: &str,
) -> Result<Prompt, String> {
    let mut methods = Vec::<&Skill>::new();
    if let Some(reference) = override_ref {
        methods.push(catalog.resolve(reference).ok_or_else(|| format!("The selected skill {reference} is unavailable. Enable its source or select another skill."))?);
    } else {
        if let Some(skill) = catalog.resolve("wayfinder") {
            methods.push(skill);
        }
        if let Some(method) = ticket.method()
            && let Some(skill) = catalog.resolve(method)
            && !methods.iter().any(|other| other.reference() == skill.reference())
        {
            methods.push(skill);
        }
    }
    if methods.is_empty() {
        return Err(match ticket.method() {
            Some(method) => format!(
                "Add or enable a source containing wayfinder or {method}, or choose a skill explicitly."
            ),
            None => {
                "Add or enable a source containing wayfinder, or choose a skill explicitly.".into()
            }
        });
    }
    let mut text = String::from(
        "# Work one Wayfinder ticket\n\nWork only the selected ticket, using the map, its settled decisions and the resolved blocker answers below. Read repository instructions before working. Do not start other tickets or change settled decisions silently.\n\n## Choose the method from the ticket\n\n- grilling: interview the human, recommend answers, and wait for their decisions. Never answer your own questions on their behalf.\n- prototype: make a small, clearly marked throwaway artifact that answers the ticket's question; ask for the human's reaction.\n- research: investigate primary sources and record cited findings.\n- task: perform the ticket's stated work and verify its completion criteria. In a planning map, do not turn it into unrelated implementation; follow any explicit execution scope in Notes.\n\nApply the provided method skills and relevant methods named in Notes. If the work changes branch, say why and consult the corresponding registered skill. A method changes how you work, not the selected ticket or the user's authority.\n\n## Finish this ticket\n\nThe host has already claimed this ticket with this terminal's session ID. Preserve claimed_by and claimed_at. Add a non-empty Answer or Ruled out, update the map's linked index, and graduate only fog that is now precise. Do not claim another ticket. Follow repository instructions for commits and do not push unless separately requested.\n\n",
    );
    text.push_str("## Tracker convention\n\nThis is this plugin's file contract. Claims are host-owned even if a method describes agent-managed claims.\n\n");
    text.push_str(CONVENTION);
    if let Some(reference) = override_ref {
        text.push_str(&format!("\n\n## Operator method selection\n\nThe operator explicitly selected {reference}. Use this method in place of automatic method selection while preserving the ticket's scope, tracker contract and human-decision boundaries.\n"));
    }
    let sources: Vec<_> = methods.iter().map(|skill| skill.reference()).collect();
    for skill in methods {
        text.push_str(&format!("\n\n## Method: {}\n\nSource directory: {}\nSource commit: {}\nResolve supporting files relative to this directory.\n\n{}\n", skill.reference(), skill.directory.display(), if skill.commit.is_empty() { "local working copy" } else { &skill.commit }, without_frontmatter(&skill.body)));
    }
    text.push_str("\n\n## Available registered skills\n\nRead supporting skills only when relevant. Bare names follow source order; qualified names pin a source.\n\n");
    for skill in &catalog.skills {
        text.push_str(&format!(
            "- {} — {}/SKILL.md{}\n",
            skill.reference(),
            skill.directory.display(),
            if skill.shadowed { " (use qualified name)" } else { "" }
        ));
    }
    text.push_str("\n\n---\n# Task context\n\nThe following artifacts are the task's data and standing context.\n");
    text.push_str(&format!(
        "\n## Map: {}\n\nPath: {}/map.md\n\n{}\n",
        map.title,
        map.directory.display(),
        map.raw
    ));
    text.push_str(&format!(
        "\n## Selected ticket: {}\n\nPath: {}\nType: {}\n\n{}\n",
        ticket.title,
        ticket.path.display(),
        ticket.kind,
        ticket.body
    ));
    for number in &ticket.blockers {
        if let Some(blocker) = map.ticket(*number) {
            text.push_str(&format!(
                "\n## Resolved blocker: {}\n\n{}\n",
                blocker.title, blocker.answer
            ));
        }
    }
    if !request.trim().is_empty() {
        text.push_str(&format!("\n## Operator request\n\n{}\n", request.trim()));
    }
    if text.len() > 192 * 1024 {
        return Err(
            "This prompt exceeds 192 KiB. Reduce the map context or choose a smaller method."
                .into(),
        );
    }
    Ok(Prompt { text, sources })
}

fn without_frontmatter(text: &str) -> &str {
    let text = text.trim_start_matches('\u{feff}');
    let Some(rest) = text.strip_prefix("---\n").or_else(|| text.strip_prefix("---\r\n")) else {
        return text;
    };
    let mut offset = text.len() - rest.len();
    for line in rest.split_inclusive('\n') {
        offset += line.len();
        if line.trim() == "---" {
            return text[offset..].trim();
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    fn skill(source: &str, name: &str) -> Skill {
        Skill {
            source: source.into(),
            name: name.into(),
            directory: "/skills".into(),
            commit: String::new(),
            body: "---\nname: test\n---\nMethod body.".into(),
            shadowed: false,
        }
    }
    fn fixture() -> Map {
        let root = tempfile::tempdir().unwrap();
        let directory = root.path().join(".plan/maps/design");
        std::fs::create_dir_all(directory.join("tickets")).unwrap();
        std::fs::write(directory.join("map.md"), "# Design\n").unwrap();
        std::fs::write(
            directory.join("tickets/01-research.md"),
            "---\ntype: research\n---\n# Design a garden\n",
        )
        .unwrap();
        super::super::model::discover(root.path()).unwrap().remove(0)
    }
    #[test]
    fn a_single_general_method_can_dispatch_without_four_role_bindings() {
        let map = fixture();
        let catalog =
            SkillCatalog { skills: vec![skill("source", "wayfinder")], warnings: Vec::new() };
        let prompt = compose(&map, &map.tickets[0], &catalog, None, "Design a garden").unwrap();
        assert_eq!(prompt.sources, vec!["source/wayfinder"]);
        assert!(prompt.text.contains("Design a garden"));
        assert!(!prompt.text.contains("name: test"));
        assert!(compose(&map, &map.tickets[0], &catalog, Some("missing/wayfinder"), "").is_err());
    }
    #[test]
    fn disabling_a_pinned_source_never_substitutes_another_source() {
        let map = fixture();
        let catalog =
            SkillCatalog { skills: vec![skill("second", "research")], warnings: Vec::new() };
        assert!(compose(&map, &map.tickets[0], &catalog, Some("first/research"), "").is_err());
        assert_eq!(
            compose(&map, &map.tickets[0], &catalog, Some("second/research"), "").unwrap().sources,
            vec!["second/research"]
        );
    }
}
