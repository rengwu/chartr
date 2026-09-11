//! Versioned prompt storage, independent of the table and future consumers.
use std::{collections::HashSet, path::PathBuf};

use chartr_plugin::services::SavedPrompt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Library {
    version: u32,
    next_id: u64,
    prompts: Vec<SavedPrompt>,
}

impl Default for Library {
    fn default() -> Self {
        Self { version: 1, next_id: 1, prompts: Vec::new() }
    }
}

pub struct Store {
    path: PathBuf,
    library: Library,
}

impl Store {
    pub fn load(path: PathBuf) -> Result<Self, String> {
        let library = Self::read(&path)?;
        Ok(Self { path, library })
    }

    fn read(path: &std::path::Path) -> Result<Library, String> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Library::default());
            }
            Err(error) => return Err(format!("Could not read saved prompts: {error}")),
        };
        let library: Library = serde_json::from_slice(&bytes)
            .map_err(|error| format!("Could not read saved prompts: {error}"))?;
        if library.version != 1 {
            return Err(format!("Unsupported prompt library version: {}.", library.version));
        }
        let mut ids = HashSet::new();
        for prompt in &library.prompts {
            let number = prompt.id.strip_prefix("prompt-").and_then(|id| id.parse::<u64>().ok());
            if !number.is_some_and(|id| id > 0 && id < library.next_id)
                || !ids.insert(&prompt.id)
                || validate(&prompt.title, &prompt.prompt).is_err()
            {
                return Err(
                    "The saved prompt library contains invalid or duplicate records.".into()
                );
            }
        }
        if library.next_id == 0 {
            return Err("The saved prompt library has an invalid ID counter.".into());
        }
        Ok(library)
    }

    pub fn prompts(&self) -> &[SavedPrompt] {
        &self.library.prompts
    }

    /// Compare the record opened by the editor, so another pane cannot silently
    /// overwrite it. Unrelated edits are retained by modifying the live library.
    pub fn save(
        &mut self,
        original: Option<&SavedPrompt>,
        title: String,
        prompt: String,
    ) -> Result<(), String> {
        validate(&title, &prompt)?;
        let mut next = self.library.clone();
        if let Some(original) = original {
            let slot = next.prompts.iter_mut().find(|item| item.id == original.id).ok_or(
                "This prompt was deleted in another pane. Cancel and create a new prompt.",
            )?;
            if slot != original {
                return Err("This prompt changed in another pane. Copy your draft, then reopen it to see the latest version.".into());
            }
            slot.title = title.trim().to_owned();
            slot.prompt = prompt;
        } else {
            let id = format!("prompt-{}", next.next_id);
            next.next_id = next.next_id.checked_add(1).ok_or("Prompt IDs are exhausted.")?;
            next.prompts.push(SavedPrompt { id, title: title.trim().to_owned(), prompt });
        }
        self.commit(next)
    }

    pub fn delete(&mut self, original: &SavedPrompt) -> Result<(), String> {
        let mut next = self.library.clone();
        let index = next
            .prompts
            .iter()
            .position(|item| item.id == original.id)
            .ok_or("This prompt has already been deleted.")?;
        if &next.prompts[index] != original {
            return Err("This prompt changed in another pane. Review it before deleting.".into());
        }
        next.prompts.remove(index);
        self.commit(next)
    }

    fn commit(&mut self, next: Library) -> Result<(), String> {
        // Also protect manual edits on disk. Reload is explicit, and bad data is
        // never treated as an empty library that a save can overwrite.
        if Self::read(&self.path)? != self.library {
            return Err(
                "The prompt library changed on disk. Copy any draft and reload the library.".into(),
            );
        }
        let write = || -> anyhow::Result<()> {
            let mut encoded = serde_json::to_vec_pretty(&next)?;
            encoded.push(b'\n');
            chartr_storage::write_atomic(&self.path, &encoded)?;
            Ok(())
        };
        write().map_err(|error| format!("Could not save prompts: {error:#}"))?;
        self.library = next;
        Ok(())
    }
}

fn validate(title: &str, prompt: &str) -> Result<(), String> {
    if title.trim().is_empty() {
        return Err("Give this prompt a title.".into());
    }
    if title.contains(['\n', '\r']) {
        return Err("Keep the title on one line.".into());
    }
    if prompt.trim().is_empty() {
        return Err("Write some prompt text before saving.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_body_and_identity_after_rename_and_delete() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("prompts.json");
        let mut store = Store::load(path.clone()).unwrap();
        assert!(!path.exists());
        let body = "  Review this code.\n\n\t保留 whitespace 🦀\n";
        store.save(None, "Review".into(), body.into()).unwrap();
        let original = store.prompts()[0].clone();
        store.save(Some(&original), "Code review".into(), body.into()).unwrap();
        let mut store = Store::load(path).unwrap();
        assert_eq!(store.prompts()[0].id, original.id);
        assert_eq!(store.prompts()[0].prompt, body);
        let renamed = store.prompts()[0].clone();
        store.delete(&renamed).unwrap();
        store.save(None, "Review".into(), "New body".into()).unwrap();
        assert_ne!(store.prompts()[0].id, original.id);
    }

    #[test]
    fn stale_edits_and_deletes_fail_without_losing_other_prompts() {
        let root = tempfile::tempdir().unwrap();
        let mut store = Store::load(root.path().join("prompts.json")).unwrap();
        store.save(None, "One".into(), "Original".into()).unwrap();
        let original = store.prompts()[0].clone();
        store.save(None, "Two".into(), "Other".into()).unwrap();
        store.save(Some(&original), "Renamed".into(), "Latest".into()).unwrap();
        assert!(store.save(Some(&original), "Stale".into(), "Stale".into()).is_err());
        assert!(store.delete(&original).is_err());
        assert_eq!(store.prompts().len(), 2);
        assert_eq!(store.prompts()[0].prompt, "Latest");
    }

    #[test]
    fn unreadable_or_newer_data_is_not_overwritten() {
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("prompts.json");
        let mut store = Store::load(path.clone()).unwrap();
        for bytes in ["broken", r#"{"version":2,"next_id":1,"prompts":[]}"#] {
            std::fs::write(&path, bytes).unwrap();
            assert!(Store::load(path.clone()).is_err());
            assert!(store.save(None, "Title".into(), "Body".into()).is_err());
            assert_eq!(std::fs::read_to_string(&path).unwrap(), bytes);
            assert!(store.prompts().is_empty());
        }
    }
}
