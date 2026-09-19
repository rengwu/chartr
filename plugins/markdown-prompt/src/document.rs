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
    pub filename: String,
    pub parts: Vec<Part>,
    // Legacy whole-file receipts, used only to migrate older compositions safely.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub managed_files: HashMap<String, String>,
}
impl Default for Document {
    fn default() -> Self {
        Self {
            version: 1,
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
fn marked_range(existing: &str) -> Result<Option<std::ops::Range<usize>>, String> {
    let starts: Vec<_> = existing.match_indices(START).collect();
    let ends: Vec<_> = existing.match_indices(END).collect();
    match (starts.as_slice(), ends.as_slice()) {
        ([], []) => Ok(None),
        ([(start, _)], [(end, _)]) if start < end => Ok(Some(*start..end + END.len())),
        _ => Err("The file has ambiguous or damaged Markdown Prompt markers. Repair them before applying.".into()),
    }
}
pub fn appended(existing: &str, body: &str) -> Result<String, String> {
    if body.contains(START) || body.contains(END) {
        return Err("The prompt contains reserved section markers.".into());
    }
    let section = format!("{START}\n{body}\n{END}");
    match marked_range(existing)? {
        None => Ok(format!(
            "{existing}{}{section}\n",
            if existing.is_empty() || existing.ends_with("\n\n") {
                ""
            } else if existing.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            }
        )),
        Some(range) => {
            Ok(format!("{}{}{}", &existing[..range.start], section, &existing[range.end..]))
        }
    }
}
pub fn validate_filename(filename: &str) -> Result<(), String> {
    let path = Path::new(filename);
    if filename.trim().is_empty()
        || path.components().any(|c| !matches!(c, Component::Normal(_)))
        || !path.extension().is_some_and(|s| s.eq_ignore_ascii_case("md"))
    {
        return Err("Use a project-relative .md filename without '..'.".into());
    }
    Ok(())
}
fn target(root: &Path, filename: &str) -> Result<PathBuf, String> {
    validate_filename(filename)?;
    let path = Path::new(filename);
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
struct Change {
    path: PathBuf,
    original: Option<Vec<u8>>,
    content: Option<String>,
}
fn read_file(path: &Path) -> Result<Option<Vec<u8>>, String> {
    match std::fs::read(path) {
        Ok(bytes) if bytes.len() > 4 * 1024 * 1024 => Err("The destination exceeds 4 MiB.".into()),
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}
fn markdown(bytes: &Option<Vec<u8>>) -> Result<&str, String> {
    std::str::from_utf8(bytes.as_deref().unwrap_or_default())
        .map_err(|_| "The destination is not UTF-8 Markdown.".into())
}
fn cleaned(existing: &str, receipt: Option<&String>) -> Result<Option<String>, String> {
    if let Some(range) = marked_range(existing)? {
        // Include the marker's trailing line ending, but preserve surrounding user text.
        let tail = &existing[range.end..];
        let tail = tail.strip_prefix("\r\n").or_else(|| tail.strip_prefix('\n')).unwrap_or(tail);
        return Ok(Some(format!("{}{tail}", &existing[..range.start])));
    }
    if let Some(receipt) = receipt {
        if receipt != existing {
            return Err("A previously managed file was edited outside Markdown Prompt. Restore its last applied content or add section markers around the content to replace.".into());
        }
        return Ok(Some(String::new()));
    }
    Ok(None)
}
/// Plan and validate every file before touching any output. For the same target,
/// replace in place so the section keeps its position and repeated saves are stable.
pub fn apply(
    root: &Path,
    doc: &Document,
    previous: Option<&Document>,
    body: &str,
) -> Result<PathBuf, String> {
    validate_filename(&doc.filename)?;
    let empty = body.trim().is_empty();
    let path = if empty { root.join(&doc.filename) } else { target(root, &doc.filename)? };
    if body.contains(START) || body.contains(END) {
        return Err("The prompt contains reserved section markers.".into());
    }
    let mut changes = Vec::new();
    let mut old_files = std::collections::BTreeMap::new();
    if let Some(previous) = previous {
        old_files.insert(previous.filename.clone(), None);
        for (filename, receipt) in &previous.managed_files {
            old_files.insert(filename.clone(), Some(receipt));
        }
    }
    let mut destination_receipt = None;
    for (filename, receipt) in old_files {
        validate_filename(&filename)?;
        if std::fs::symlink_metadata(root.join(&filename))
            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        {
            continue;
        }
        let old_path = target(root, &filename)?;
        if !empty && old_path == path {
            destination_receipt = receipt;
            continue;
        }
        let original = read_file(&old_path)?;
        if original.is_none() {
            continue;
        }
        if let Some(content) = cleaned(markdown(&original)?, receipt)? {
            let content = (!content.trim().is_empty()).then_some(content);
            changes.push(Change { path: old_path, original, content });
        }
    }
    if !empty {
        let original = read_file(&path)?;
        let existing = markdown(&original)?;
        let existing = if original.is_some()
            && destination_receipt.is_some()
            && marked_range(existing)?.is_none()
        {
            cleaned(existing, destination_receipt)?;
            ""
        } else {
            existing
        };
        let content = Some(appended(existing, body)?);
        changes.push(Change { path: path.clone(), original, content });
    }
    // Stage all writes before cleanup, catching permissions and staging failures early.
    let mut staged = Vec::new();
    for change in changes {
        if change.original.as_deref() == change.content.as_ref().map(|s| s.as_bytes()) {
            continue;
        }
        let write = change
            .content
            .as_ref()
            .map(|content| {
                chartr_storage::StagedWrite::new(&change.path, content.as_bytes())
                    .map_err(|e| e.to_string())
            })
            .transpose()?;
        staged.push((change, write));
    }
    for (change, write) in staged {
        if read_file(&change.path)? != change.original {
            return Err("A Markdown file changed while applying. Try again.".into());
        }
        match write {
            Some(write) if change.original.is_some() => {
                write.replace().map_err(|e| e.to_string())?
            }
            Some(write) => write.create_new().map_err(|e| e.to_string())?,
            None => std::fs::remove_file(&change.path).map_err(|e| e.to_string())?,
        }
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
    use std::fs;

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
    fn creates_appends_and_updates_in_place_without_rewriting_unchanged_output() {
        let root = tempfile::tempdir().unwrap();
        let doc = Document::default();
        let path = apply(root.path(), &doc, None, "one").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), appended("", "one").unwrap());
        let existing =
            format!("# User instructions\r\n\r\n{}User tail\r\n", appended("", "one").unwrap());
        fs::write(&path, &existing).unwrap();
        apply(root.path(), &doc, Some(&doc), "two").unwrap();
        let expected = existing.replace("one", "two");
        assert_eq!(fs::read_to_string(&path).unwrap(), expected);
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        apply(root.path(), &doc, Some(&doc), "two").unwrap();
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
        fs::write(&path, "Unmarked instructions").unwrap();
        apply(root.path(), &doc, Some(&doc), "three").unwrap();
        assert_eq!(
            fs::read_to_string(&path).unwrap(),
            appended("Unmarked instructions", "three").unwrap()
        );
    }

    #[test]
    fn changing_filename_cleans_previous_and_preserves_surrounding_text() {
        let root = tempfile::tempdir().unwrap();
        let old = Document::default();
        let old_path = apply(root.path(), &old, None, "old").unwrap();
        let old_section = fs::read_to_string(&old_path).unwrap();
        fs::write(&old_path, format!("Before\n{old_section}After\n")).unwrap();
        let new = Document { filename: "AGENTS.md".into(), ..Document::default() };
        fs::write(root.path().join(&new.filename), appended("Existing", "stale").unwrap()).unwrap();
        let new_path = apply(root.path(), &new, Some(&old), "new").unwrap();
        assert_eq!(fs::read_to_string(&old_path).unwrap(), "Before\nAfter\n");
        assert_eq!(fs::read_to_string(&new_path).unwrap(), appended("Existing", "new").unwrap());
        apply(root.path(), &old, Some(&new), "back").unwrap();
        assert_eq!(fs::read_to_string(&new_path).unwrap(), "Existing\n\n");
        assert_eq!(
            fs::read_to_string(&old_path).unwrap(),
            appended("Before\nAfter\n", "back").unwrap()
        );
    }

    #[test]
    fn cleanup_deletes_empty_files_and_empty_prompts_stop_before_destination() {
        let root = tempfile::tempdir().unwrap();
        let old = Document::default();
        let path = apply(root.path(), &old, None, "old").unwrap();
        let new = Document { filename: "missing/future.md".into(), ..Document::default() };
        apply(root.path(), &new, Some(&old), " \n\t").unwrap();
        assert!(!path.exists());
        assert!(!root.path().join("missing").exists());
        apply(root.path(), &old, None, "old").unwrap();
        let untouched = root.path().join("unrelated.md");
        fs::write(&untouched, START).unwrap();
        let new = Document { filename: "unrelated.md".into(), ..Document::default() };
        apply(root.path(), &new, Some(&old), "").unwrap();
        assert!(!path.exists());
        assert_eq!(fs::read_to_string(&untouched).unwrap(), START);
        // Unmarked user content (including empty files) is never deleted by cleanup.
        fs::write(&path, "").unwrap();
        apply(root.path(), &old, Some(&old), "").unwrap();
        assert!(path.exists());
    }

    #[test]
    fn invalid_filenames_and_content_leave_last_applied_file_untouched() {
        let root = tempfile::tempdir().unwrap();
        let old = Document::default();
        let path = apply(root.path(), &old, None, "keep").unwrap();
        let original = fs::read(&path).unwrap();
        for filename in
            ["", "bad.txt", "file.md.txt", "../outside.md", "/absolute.md", "missing/file.md"]
        {
            let new = Document { filename: filename.into(), ..Document::default() };
            assert!(apply(root.path(), &new, Some(&old), "new").is_err(), "{filename}");
            assert_eq!(fs::read(&path).unwrap(), original);
        }
        let new = Document { filename: "new.md".into(), ..Document::default() };
        for content in
            [START.to_owned(), format!("{END}{START}"), appended("", "one").unwrap().repeat(2)]
        {
            fs::write(root.path().join(&new.filename), &content).unwrap();
            assert!(apply(root.path(), &new, Some(&old), "new").is_err());
            assert_eq!(fs::read(&path).unwrap(), original);
            assert_eq!(fs::read_to_string(root.path().join(&new.filename)).unwrap(), content);
        }
        fs::write(root.path().join(&new.filename), [0xff]).unwrap();
        assert!(apply(root.path(), &new, Some(&old), "new").is_err());
        assert!(apply(root.path(), &old, Some(&old), END).is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
        let bad = Document { filename: "invalid.txt".into(), ..Document::default() };
        assert!(apply(root.path(), &bad, Some(&old), "").is_err());
        assert_eq!(fs::read(&path).unwrap(), original);
    }

    #[test]
    fn malformed_previous_markers_block_cleanup_and_missing_previous_folder_is_ok() {
        let root = tempfile::tempdir().unwrap();
        let old = Document { filename: "old/file.md".into(), ..Document::default() };
        fs::create_dir(root.path().join("old")).unwrap();
        fs::write(root.path().join(&old.filename), START).unwrap();
        let new = Document::default();
        assert!(apply(root.path(), &new, Some(&old), "new").is_err());
        assert!(!root.path().join(&new.filename).exists());
        fs::remove_dir_all(root.path().join("old")).unwrap();
        apply(root.path(), &new, Some(&old), "new").unwrap();
    }

    #[test]
    fn legacy_owned_files_migrate_only_when_receipts_match() {
        let root = tempfile::tempdir().unwrap();
        let mut old = Document::default();
        old.managed_files.insert(old.filename.clone(), "legacy body".into());
        let path = root.path().join(&old.filename);
        fs::write(&path, "external edit").unwrap();
        assert!(apply(root.path(), &old, Some(&old), "new").is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "external edit");
        fs::write(&path, "legacy body").unwrap();
        apply(root.path(), &old, Some(&old), "new").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), appended("", "new").unwrap());
        fs::write(&path, "legacy body").unwrap();
        let new = Document { filename: "new.md".into(), ..Document::default() };
        apply(root.path(), &new, Some(&old), "moved").unwrap();
        assert!(!path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks_in_both_destination_and_cleanup() {
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let private = outside.path().join("private.md");
        fs::write(&private, appended("", "private").unwrap()).unwrap();
        std::os::unix::fs::symlink(&private, root.path().join("link.md")).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("escape")).unwrap();
        let doc = Document::default();
        for filename in ["link.md", "escape/private.md"] {
            let link = Document { filename: filename.into(), ..Document::default() };
            assert!(apply(root.path(), &link, None, "new").is_err());
            assert!(apply(root.path(), &doc, Some(&link), "").is_err());
        }
        assert_eq!(fs::read_to_string(&private).unwrap(), appended("", "private").unwrap());
    }
}
