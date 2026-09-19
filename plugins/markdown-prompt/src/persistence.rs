//! Explicit saves, serialized across panes, with a receipt of the last applied destination.
use super::document::{self, Document};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Applied {
    version: u32,
    project: PathBuf,
    document: Document,
    #[serde(default = "legacy_has_content")]
    has_content: bool,
}
fn legacy_has_content() -> bool {
    true
}
fn applied_path(path: &Path) -> PathBuf {
    path.with_extension("applied.json")
}
fn read(path: &Path) -> Result<Option<Applied>, String> {
    match std::fs::read(applied_path(path)) {
        Ok(bytes) => {
            let applied: Applied = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
            if applied.version != 1 || applied.document.version != 1 {
                return Err("Unsupported applied-composition version.".into());
            }
            Ok(Some(applied))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

pub fn save(
    path: &Path,
    project: &Path,
    mut doc: Document,
    body: &str,
    expected: Option<&[u8]>,
) -> Result<Document, String> {
    document::validate_filename(&doc.filename)?;
    std::fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(path.with_extension("lock"))
        .map_err(|e| e.to_string())?;
    lock.lock().map_err(|e| e.to_string())?;
    if std::fs::read(path).ok().as_deref() != expected {
        return Err("This composition changed in another pane. Reopen Markdown Prompt to load the saved version.".into());
    }
    let previous = read(path)?;
    let project = project.canonicalize().map_err(|e| e.to_string())?;
    if previous.as_ref().is_some_and(|p| p.project != project) {
        return Err("The previously applied composition belongs to another folder.".into());
    }
    // Old drafts can carry whole-file receipts absent from their applied sidecar.
    let saved = document::load(path)?;
    let mut previous_doc = previous.filter(|p| p.has_content).map(|p| p.document);
    if !saved.managed_files.is_empty() {
        let previous = previous_doc.get_or_insert_with(|| saved.clone());
        for (filename, receipt) in saved.managed_files {
            previous.managed_files.entry(filename).or_insert(receipt);
        }
    }
    document::apply(&project, &doc, previous_doc.as_ref(), body)?;
    doc.managed_files.clear();
    let applied = Applied {
        version: 1,
        project,
        document: doc.clone(),
        has_content: !body.trim().is_empty(),
    };
    let encoded = serde_json::to_vec_pretty(&applied).map_err(|e| e.to_string())?;
    chartr_storage::write_atomic(&applied_path(path), &encoded).map_err(|e| e.to_string())?;
    document::save(path, &doc)?;
    Ok(doc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn saves_track_cleanup_across_restarts_and_reject_stale_panes() {
        let root = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();
        let path = data.path().join("composition.json");
        let old = save(&path, root.path(), Document::default(), "one", None).unwrap();
        let expected = fs::read(&path).unwrap();
        let new = Document { filename: "AGENTS.md".into(), ..old.clone() };
        assert!(save(&path, root.path(), new.clone(), "two", None).is_err());
        assert!(root.path().join(&old.filename).exists());
        save(&path, root.path(), new.clone(), "two", Some(&expected)).unwrap();
        assert!(!root.path().join(&old.filename).exists());
        assert_eq!(
            fs::read_to_string(root.path().join(&new.filename)).unwrap(),
            document::appended("", "two").unwrap()
        );
        assert_eq!(document::load(&path).unwrap(), new);
        let expected = fs::read(&path).unwrap();
        save(&path, root.path(), old.clone(), "", Some(&expected)).unwrap();
        assert!(!root.path().join(&new.filename).exists());
        // An empty Save never adopts its unused destination for later cleanup.
        let unrelated = document::appended("", "unrelated").unwrap();
        fs::write(root.path().join(&old.filename), &unrelated).unwrap();
        let expected = fs::read(&path).unwrap();
        save(&path, root.path(), new, "three", Some(&expected)).unwrap();
        assert_eq!(fs::read_to_string(root.path().join(&old.filename)).unwrap(), unrelated);
    }

    #[test]
    fn legacy_state_loads_and_migrates_latest_whole_file_receipts() {
        let root = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();
        let path = data.path().join("composition.json");
        let legacy = serde_json::json!({
            "version": 1, "enabled": false, "append": false, "create_if_missing": false,
            "filename": "CHARTR.md", "parts": [], "managed_files": {"CHARTR.md": "old"}
        });
        fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        let mut active = legacy;
        active["managed_files"]["CHARTR.md"] = "latest".into();
        let applied = serde_json::json!({
            "version": 1, "project": root.path().canonicalize().unwrap(), "document": active
        });
        fs::write(applied_path(&path), serde_json::to_vec(&applied).unwrap()).unwrap();
        fs::write(root.path().join("CHARTR.md"), "latest").unwrap();
        let expected = fs::read(&path).unwrap();
        let doc = document::load(&path).unwrap();
        let doc = save(&path, root.path(), doc, "new", Some(&expected)).unwrap();
        assert!(doc.managed_files.is_empty());
        assert_eq!(
            fs::read_to_string(root.path().join("CHARTR.md")).unwrap(),
            document::appended("", "new").unwrap()
        );
        let saved: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        for removed in ["enabled", "append", "create_if_missing", "managed_files"] {
            assert!(saved.get(removed).is_none());
        }
    }

    #[test]
    fn invalid_saves_preserve_configuration_receipts_and_output() {
        let root = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();
        let path = data.path().join("composition.json");
        save(&path, root.path(), Document::default(), "keep", None).unwrap();
        let expected = fs::read(&path).unwrap();
        let receipt = fs::read(applied_path(&path)).unwrap();
        let bad = Document { filename: "bad.txt".into(), ..Document::default() };
        assert!(save(&path, root.path(), bad, "", Some(&expected)).is_err());
        assert_eq!(fs::read(&path).unwrap(), expected);
        assert_eq!(fs::read(applied_path(&path)).unwrap(), receipt);
        assert_eq!(
            fs::read_to_string(root.path().join("CHARTR.md")).unwrap(),
            document::appended("", "keep").unwrap()
        );
    }
}
