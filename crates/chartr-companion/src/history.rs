use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;

const PAGE_BYTES: usize = 256 * 1024;

/// Immutable transfers let readers fetch all retained scrollback while output continues.
#[derive(Default)]
pub struct History {
    snapshots: VecDeque<(String, String, String)>,
}

impl History {
    pub fn read(
        &mut self,
        session: &str,
        snapshot: Option<&str>,
        offset: usize,
        known: Option<&str>,
        content: impl FnOnce() -> String,
    ) -> Result<Value, String> {
        let id = if let Some(id) = snapshot {
            id.to_owned()
        } else {
            let text = content();
            let id = super::hex(&Sha256::digest(text.as_bytes()));
            if known == Some(id.as_str()) {
                return Ok(json!({"unchanged":true,"snapshot":id}));
            }
            if !self.snapshots.iter().any(|(s, i, _)| s == session && i == &id) {
                // Bound retained copies; a slow reader can restart an evicted transfer.
                while self.snapshots.len() >= 4 {
                    self.snapshots.pop_front();
                }
                self.snapshots.push_back((session.to_owned(), id.clone(), text));
            }
            id
        };
        let (_, _, text) = self
            .snapshots
            .iter()
            .find(|(s, i, _)| s == session && i == &id)
            .ok_or("Scrollback snapshot expired. Read it again.")?;
        if offset > text.len() || !text.is_char_boundary(offset) {
            return Err("Invalid scrollback offset.".into());
        }
        let mut end = offset.saturating_add(PAGE_BYTES).min(text.len());
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        Ok(json!({"text": &text[offset..end], "snapshot":id, "offset":offset,
            "next": if end < text.len() { Some(end) } else { None }, "total":text.len()}))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_history_is_stable_paged_and_preserves_whitespace_and_unicode() {
        let text = format!("first\n\n{}\nlast\n", "界 café\n".repeat(60_000));
        let mut history = History::default();
        let mut page = history.read("s", None, 0, None, || text.clone()).unwrap();
        let id = page["snapshot"].as_str().unwrap().to_owned();
        let mut result = String::new();
        loop {
            assert!(page["text"].as_str().unwrap().len() <= PAGE_BYTES);
            result.push_str(page["text"].as_str().unwrap());
            let Some(next) = page["next"].as_u64() else {
                break;
            };
            page = history
                .read("s", Some(&id), next as usize, None, || {
                    panic!("Continuation must use the immutable snapshot")
                })
                .unwrap();
        }
        assert_eq!(result, text);
        assert_eq!(history.read("s", None, 0, Some(&id), || text).unwrap()["unchanged"], true);
    }

    #[test]
    fn invalid_offsets_and_evicted_or_cross_session_snapshots_are_rejected() {
        let mut history = History::default();
        let first = history.read("s", None, 0, None, || "界".into()).unwrap();
        let id = first["snapshot"].as_str().unwrap();
        for offset in [1, 100] {
            assert!(history.read("s", Some(id), offset, None, String::new).is_err());
        }
        assert!(history.read("other", Some(id), 0, None, String::new).is_err());
        for i in 0..4 {
            history.read("s", None, 0, None, || i.to_string()).unwrap();
        }
        assert!(history.read("s", Some(id), 0, None, String::new).is_err());
    }
}
