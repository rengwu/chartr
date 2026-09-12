use super::*;

impl Reader {
    pub(super) fn read_kimi(
        &mut self,
        native: &NativeSession,
        paths: &ProviderPaths,
    ) -> Result<Transcript> {
        // Match sessions/<workDirKey>/<sessionId> by reported identity, never recency.
        let mut matches = Vec::new();
        for (index, entry) in fs::read_dir(paths.kimi.join("sessions"))?.enumerate() {
            ensure!(index < 50_000, "Kimi session index is too large to scan safely");
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                let session = entry.path().join(&native.id);
                if session.is_dir() {
                    matches.push(session);
                }
            }
        }
        ensure!(matches.len() == 1, "No unique Kimi session matches the native identity");
        let session = matches.remove(0);
        let state_file = fs::File::open(session.join("state.json"))?;
        ensure!(state_file.metadata()?.len() <= 1024 * 1024, "Kimi session metadata is too large");
        let state: Value = serde_json::from_reader(state_file.take(1024 * 1024))?;
        ensure!(state["id"] == native.id, "Kimi metadata belongs to another conversation");

        // Completion events and state.updatedAt also change on output. Only
        // accepted prompts in the main agent log represent new user activity.
        let path = session.join("agents/main/wire.jsonl");
        let metadata = fs::metadata(&path)?;
        let modified = metadata.modified()?;
        let key = (Provider::Kimi, native.id.clone(), path.clone());
        if let Some((time, len, transcript)) = self.cache.get(&key)
            && *time == modified
            && *len == metadata.len()
        {
            return Ok(transcript.clone());
        }
        ensure!(metadata.len() <= MAX_TRANSCRIPT_BYTES, "Kimi event log is too large to read");
        let file = fs::File::open(path)?;
        let mut updated = None;
        for line in BufReader::new(file.take(MAX_TRANSCRIPT_BYTES)).lines() {
            let line = line?;
            // Concurrent appends may leave an incomplete final record.
            let Ok(event) = serde_json::from_str::<Value>(&line) else { continue };
            if event["type"] == "prompt.accepted"
                && event["agentId"] == "main"
                && event["promptId"].as_str().is_some_and(|id| !id.is_empty())
                && let Some(time) = event["time"].as_u64().filter(|time| *time > 0)
            {
                updated = Some(updated.unwrap_or(0).max(time));
            }
        }
        // No prompt bodies need to be copied into history for recency tracking.
        let transcript = Transcript { updated, ..Transcript::default() };
        self.cache.insert(key, (modified, metadata.len(), transcript.clone()));
        Ok(transcript)
    }
}
