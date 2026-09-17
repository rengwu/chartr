use super::*;

pub(super) fn non_blank(value: &Value) -> Option<String> {
    value.as_str().map(str::trim).filter(|s| !s.is_empty()).map(str::to_owned)
}

impl Reader {
    /// Best-effort, read-only metadata lookup, independent of transcript caches.
    /// A missing or incompatible provider store leaves the existing fallback usable.
    pub(crate) fn native_title(
        &mut self,
        provider: Provider,
        native: &NativeSession,
        paths: &ProviderPaths,
    ) -> Option<String> {
        validate_native_id(&native.id).ok()?;
        match provider {
            Provider::Codex => self
                .codex_index_title(&paths.codex.join("session_index.jsonl"), &native.id)
                .ok()
                .flatten()
                .or_else(|| codex_database_title(&paths.codex, &native.id).ok().flatten()),
            Provider::Kimi => {
                let directory = kimi::session_directory(paths, native).ok()?;
                let state = read_metadata(&directory.join("state.json")).ok()?;
                (state["id"] == native.id).then(|| non_blank(&state["title"]))?
            }
            Provider::Grok => grok_title(&paths.grok, &native.id).ok().flatten(),
            // These providers keep names in the transcript/database reader, or
            // report them through the observed terminal title.
            _ => None,
        }
    }

    fn codex_index_title(&mut self, path: &Path, id: &str) -> Result<Option<String>> {
        let metadata = fs::metadata(path)?;
        let modified = metadata.modified()?;
        if let Some((time, len, titles)) = self.title_indexes.get(path)
            && *time == modified
            && *len == metadata.len()
        {
            return Ok(titles.get(id).cloned());
        }
        ensure!(metadata.len() <= MAX_TRANSCRIPT_BYTES, "Codex title index is too large");
        let mut titles = HashMap::new();
        for line in BufReader::new(fs::File::open(path)?.take(MAX_TRANSCRIPT_BYTES)).lines() {
            let line = line?;
            // Ignore partial appends; later complete records replace earlier names.
            let Ok(value) = serde_json::from_str::<Value>(&line) else { continue };
            if let Some(id) = value["id"].as_str() {
                if let Some(title) = non_blank(&value["thread_name"]) {
                    titles.insert(id.to_owned(), title);
                } else if value.get("thread_name").is_some() {
                    titles.remove(id);
                }
            }
        }
        let title = titles.get(id).cloned();
        self.title_indexes.insert(path.to_owned(), (modified, metadata.len(), titles));
        Ok(title)
    }
}

fn read_metadata(path: &Path) -> Result<Value> {
    let file = fs::File::open(path)?;
    ensure!(file.metadata()?.len() <= 1024 * 1024, "Session metadata is too large");
    Ok(serde_json::from_reader(file.take(1024 * 1024))?)
}

fn codex_database_title(root: &Path, id: &str) -> Result<Option<String>> {
    // Codex versions its state database. Prefer the highest installed schema,
    // never create a database or migrate the provider's files.
    let mut databases = Vec::new();
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(version) = name
            .to_str()
            .and_then(|s| s.strip_prefix("state_"))
            .and_then(|s| s.strip_suffix(".sqlite"))
            .and_then(|s| s.parse::<u32>().ok())
        else {
            continue;
        };
        databases.push((version, entry.path()));
    }
    databases.sort_by(|a, b| b.0.cmp(&a.0));
    let Some((_, path)) = databases.first() else { return Ok(None) };
    let db = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )?;
    db.busy_timeout(std::time::Duration::from_millis(200))?;
    let has_name = db
        .prepare("PRAGMA table_info(threads)")?
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?
        .iter()
        .any(|column| column == "name");
    let query = if has_name {
        "SELECT COALESCE(NULLIF(TRIM(name), ''), title) FROM threads WHERE id = ?1"
    } else {
        "SELECT title FROM threads WHERE id = ?1"
    };
    use rusqlite::OptionalExtension;
    Ok(db
        .query_row(query, [id], |row| row.get::<_, String>(0))
        .optional()?
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty()))
}

fn grok_title(root: &Path, id: &str) -> Result<Option<String>> {
    let mut matches = Vec::new();
    for (index, entry) in fs::read_dir(root.join("sessions"))?.enumerate() {
        ensure!(index < 50_000, "Grok session index is too large");
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let path = entry.path().join(id).join("summary.json");
            if path.is_file() {
                matches.push(path);
            }
        }
    }
    ensure!(matches.len() == 1, "No unique Grok session matches the native identity");
    let value = read_metadata(&matches[0])?;
    ensure!(value["info"]["id"] == id, "Grok metadata belongs to another conversation");
    Ok(non_blank(&value["generated_title"]).or_else(|| non_blank(&value["session_summary"])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_index_tracks_exact_ids_latest_names_and_partial_appends() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("session_index.jsonl");
        let mut reader = Reader::default();
        assert!(reader.codex_index_title(&path, "a").is_err());
        fs::write(
            &path,
            concat!(
                "{\"id\":\"a\",\"thread_name\":\"First name\"}\n",
                "{\"id\":\"other\",\"thread_name\":\"Unrelated\"}\n",
                "{\"id\":\"a\",\"thread_name\":\"New name\"}\n",
                "{\"id\":\"a\",\"thread_name\":"
            ),
        )
        .unwrap();
        assert_eq!(reader.codex_index_title(&path, "a").unwrap().as_deref(), Some("New name"));
        assert_eq!(reader.codex_index_title(&path, "missing").unwrap(), None);
        fs::write(&path, "{\"id\":\"a\",\"thread_name\":\"  \"}\n").unwrap();
        assert_eq!(reader.codex_index_title(&path, "a").unwrap(), None);
    }

    #[test]
    fn codex_database_prefers_readable_name_and_supports_older_schemas() {
        let dir = tempfile::tempdir().unwrap();
        assert_eq!(codex_database_title(dir.path(), "a").unwrap(), None);
        let old = Connection::open(dir.path().join("state_4.sqlite")).unwrap();
        old.execute_batch(
            "CREATE TABLE threads(id TEXT, title TEXT);
            INSERT INTO threads VALUES ('a', 'Legacy title');",
        )
        .unwrap();
        assert_eq!(codex_database_title(dir.path(), "a").unwrap().as_deref(), Some("Legacy title"));
        let current = Connection::open(dir.path().join("state_5.sqlite")).unwrap();
        current
            .execute_batch(
                "CREATE TABLE threads(id TEXT, title TEXT, name TEXT);
            INSERT INTO threads VALUES ('a', 'Raw prompt title', 'Readable generated name');",
            )
            .unwrap();
        assert_eq!(
            codex_database_title(dir.path(), "a").unwrap().as_deref(),
            Some("Readable generated name")
        );
        assert_eq!(codex_database_title(dir.path(), "other").unwrap(), None);
        current.execute("UPDATE threads SET name = ' '", []).unwrap();
        assert_eq!(
            codex_database_title(dir.path(), "a").unwrap().as_deref(),
            Some("Raw prompt title")
        );
        // Missing or malformed optional metadata must not disable title fallbacks.
        let mut paths = super::super::tests::log_paths(dir.path());
        paths.codex = dir.path().to_owned();
        fs::write(dir.path().join("session_index.jsonl"), "malformed\n").unwrap();
        let native = NativeSession { id: "a".into(), path: None };
        assert_eq!(
            Reader::default().native_title(Provider::Codex, &native, &paths).as_deref(),
            Some("Raw prompt title")
        );
    }

    #[test]
    fn kimi_and_grok_titles_refresh_without_logs_and_reject_wrong_identities() {
        let dir = tempfile::tempdir().unwrap();
        let paths = super::super::tests::log_paths(dir.path());
        let mut reader = Reader::default();
        let native = NativeSession { id: "a".into(), path: None };
        let kimi = paths.kimi.join("sessions/workspace/a");
        fs::create_dir_all(&kimi).unwrap();
        fs::write(kimi.join("state.json"), r#"{"id":"a","title":"Kimi name"}"#).unwrap();
        assert_eq!(
            reader.native_title(Provider::Kimi, &native, &paths).as_deref(),
            Some("Kimi name")
        );
        fs::write(kimi.join("state.json"), r#"{"id":"a","title":"Renamed Kimi session"}"#).unwrap();
        assert_eq!(
            reader.native_title(Provider::Kimi, &native, &paths).as_deref(),
            Some("Renamed Kimi session")
        );
        fs::write(kimi.join("state.json"), r#"{"id":"other","title":"Wrong session"}"#).unwrap();
        assert_eq!(reader.native_title(Provider::Kimi, &native, &paths), None);

        let grok = paths.grok.join("sessions/workspace/a");
        fs::create_dir_all(&grok).unwrap();
        fs::write(grok.join("summary.json"), r#"{"info":{"id":"a"},"generated_title":"Grok name","session_summary":"Older summary"}"#).unwrap();
        assert_eq!(
            reader.native_title(Provider::Grok, &native, &paths).as_deref(),
            Some("Grok name")
        );
        fs::write(
            grok.join("summary.json"),
            r#"{"info":{"id":"a"},"session_summary":"Legacy session name"}"#,
        )
        .unwrap();
        assert_eq!(
            reader.native_title(Provider::Grok, &native, &paths).as_deref(),
            Some("Legacy session name")
        );
        fs::write(
            grok.join("summary.json"),
            r#"{"info":{"id":"other"},"generated_title":"Wrong session"}"#,
        )
        .unwrap();
        assert_eq!(reader.native_title(Provider::Grok, &native, &paths), None);
    }
}
