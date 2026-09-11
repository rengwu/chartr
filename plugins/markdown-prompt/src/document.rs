//! Pure composition and conservative project-file updates.
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Component, Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Part {
    Text { text: String },
    Template { provider: String, id: String, title: String },
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub version: u32,
    pub enabled: bool,
    pub append: bool,
    pub filename: String,
    pub parts: Vec<Part>,
    #[serde(default)]
    pub managed_files: HashMap<String, String>,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
            enabled: true,
            append: false,
            filename: "CHARTR.md".into(),
            parts: vec![Part::Text { text: String::new() }],
            managed_files: HashMap::new(),
        }
    }
}
pub type Bodies = HashMap<(String, String), String>;
pub fn compose(parts: &[Part], bodies: &Bodies) -> Result<String, String> {
    let mut output = String::new();
    for part in parts {
        match part {
            Part::Text { text } => output.push_str(text),
            Part::Template { provider, id, .. } => output.push_str(bodies.get(&(provider.clone(), id.clone())).ok_or_else(|| format!("Template {provider}/{id} is unavailable. Enable its plugin or remove the reference."))?),
        }
        if output.len() > 1024 * 1024 {
            return Err("The constructed prompt exceeds 1 MiB.".into());
        }
    }
    Ok(output)
}
const START: &str = "<!--chartr-markdown-prompt-begin-->";
const END: &str = "<!--chartr-markdown-prompt-end-->";
pub fn appended(existing: &str, body: &str) -> Result<String, String> {
    if body.contains(START) || body.contains(END) {
        return Err("The prompt contains reserved section markers.".into());
    }
    let starts: Vec<_> = existing.match_indices(START).collect();
    let ends: Vec<_> = existing.match_indices(END).collect();
    let section = format!("{START}\n{body}\n{END}");
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => Ok(format!("{existing}{}{section}\n", if existing.is_empty() || existing.ends_with("\n\n") { "" } else if existing.ends_with('\n') { "\n" } else { "\n\n" })),
        ([(start, _)], [(end, _)]) if start < end => Ok(format!("{}{}{}", &existing[..*start], section, &existing[end + END.len()..])),
        _ => Err("The file has ambiguous or damaged Markdown Prompt markers. Repair them before applying.".into()),
    }
}
fn target(root: &Path, filename: &str) -> Result<PathBuf, String> {
    let path = Path::new(filename);
    if filename.trim().is_empty()
        || path.components().any(|c| !matches!(c, Component::Normal(_)))
        || !path.extension().is_some_and(|s| s.eq_ignore_ascii_case("md"))
    {
        return Err("Use a project-relative .md filename without '..'.".into());
    }
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let full = root.join(path);
    let parent = full
        .parent()
        .unwrap()
        .canonicalize()
        .map_err(|e| format!("The destination folder must exist: {e}"))?;
    if !parent.starts_with(&root) {
        return Err("The destination must stay inside this project.".into());
    }
    if std::fs::symlink_metadata(&full).is_ok_and(|m| m.file_type().is_symlink()) {
        return Err("Choose a regular Markdown file, not a symbolic link.".into());
    }
    Ok(parent.join(full.file_name().unwrap()))
}
pub fn apply(root: &Path, doc: &Document, body: &str) -> Result<PathBuf, String> {
    if !doc.enabled {
        return Err("Markdown Prompt is disabled.".into());
    }
    let path = target(root, &doc.filename)?;
    let (content, original) = if doc.append {
        let bytes = std::fs::read(&path)
            .map_err(|e| format!("Append needs an existing Markdown file: {e}"))?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err("The destination exceeds 4 MiB.".into());
        }
        let existing =
            std::str::from_utf8(&bytes).map_err(|_| "The destination is not UTF-8 Markdown.")?;
        (appended(existing, body)?, Some(bytes))
    } else {
        match std::fs::read(&path) {
            Ok(bytes) => {
                if !doc.managed_files.get(&doc.filename).is_some_and(|old| old.as_bytes() == bytes)
                {
                    return Err("This file is not owned by Markdown Prompt, or was edited outside it. Choose another filename or use append mode.".into());
                }
                (body.to_owned(), Some(bytes))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (body.to_owned(), None),
            Err(e) => return Err(e.to_string()),
        }
    };
    if original.as_ref().is_some_and(|bytes| bytes == content.as_bytes()) {
        return Ok(path);
    }
    let staged =
        chartr_storage::StagedWrite::new(&path, content.as_bytes()).map_err(|e| e.to_string())?;
    if let Some(original) = original {
        if std::fs::read(&path).map_err(|e| e.to_string())? != original {
            return Err("The destination changed while applying. Try again.".into());
        }
        staged.replace().map_err(|e| e.to_string())?;
    } else {
        staged.create_new().map_err(|e| {
            format!("New file could not be created (existing files are never overwritten): {e}")
        })?;
    }
    Ok(path)
}
pub fn load(path: &Path) -> Result<Document, String> {
    match std::fs::read(path) {
        Ok(bytes) => {
            let doc: Document = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if doc.version != 1 {
                return Err("Unsupported Markdown Prompt configuration version.".into());
            }
            Ok(doc)
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Document::default()),
        Err(e) => Err(e.to_string()),
    }
}
pub fn save(path: &Path, doc: &Document) -> Result<(), String> {
    let encoded = serde_json::to_vec_pretty(doc).map_err(|e| e.to_string())?;
    chartr_storage::write_atomic(path, &encoded).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn references_are_exact_and_bodies_are_literal() {
        let parts = vec![
            Part::Text { text: "Before ".into() },
            Part::Template { provider: "a".into(), id: "one".into(), title: "Old title".into() },
            Part::Text { text: " after".into() },
        ];
        let mut bodies = Bodies::new();
        bodies.insert(("b".into(), "one".into()), "wrong".into());
        assert!(compose(&parts, &bodies).is_err());
        bodies.insert(("a".into(), "one".into()), "{{literal}}".into());
        assert_eq!(compose(&parts, &bodies).unwrap(), "Before {{literal}} after");
    }
    #[test]
    fn append_is_idempotent_and_preserves_surrounding_text() {
        let first = appended("# Existing\n", "hello").unwrap();
        assert_eq!(appended(&first, "hello").unwrap(), first);
        let with_tail = format!("{first}User tail\n");
        let changed = appended(&with_tail, "updated").unwrap();
        assert!(changed.starts_with("# Existing\n\n"));
        assert!(changed.ends_with("User tail\n"));
        assert!(!changed.contains("hello"));
        assert!(appended(START, "x").is_err());
        assert!(appended(&format!("{first}{first}"), "x").is_err());
        assert!(appended("", END).is_err());
    }
    #[test]
    fn writes_respect_mode_project_boundary_and_disabled_state() {
        let root = tempfile::tempdir().unwrap();
        let mut doc = Document::default();
        let path = apply(root.path(), &doc, "one").unwrap();
        assert!(apply(root.path(), &doc, "two").is_err());
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "one");
        doc.append = true;
        apply(root.path(), &doc, "two").unwrap();
        apply(root.path(), &doc, "three").unwrap();
        let content = std::fs::read_to_string(path).unwrap();
        assert!(content.starts_with("one\n\n"));
        assert!(!content.contains("two"));
        doc.filename = "../outside.md".into();
        assert!(apply(root.path(), &doc, "x").is_err());
        doc.filename = "missing.md".into();
        assert!(apply(root.path(), &doc, "x").is_err());
        doc.enabled = false;
        doc.append = false;
        assert!(apply(root.path(), &doc, "x").is_err());
    }
    #[cfg(unix)]
    #[test]
    fn rejects_symlink_destinations_and_parent_escape() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        std::fs::write(outside.path().join("a.md"), "private").unwrap();
        std::os::unix::fs::symlink(outside.path().join("a.md"), root.path().join("a.md")).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();
        let mut doc = Document { append: true, filename: "a.md".into(), ..Document::default() };
        assert!(apply(root.path(), &doc, "x").is_err());
        doc.filename = "escape/a.md".into();
        assert!(apply(root.path(), &doc, "x").is_err());
        assert_eq!(std::fs::read_to_string(outside.path().join("a.md")).unwrap(), "private");
    }
}
