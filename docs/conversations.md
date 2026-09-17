# Inbox

Choose **Chats** in the window’s view selector, or **Inbox** in the command palette
(`Cmd+Shift+3` on macOS, `Ctrl+Shift+3` on Linux). Existing Conversations view
preferences migrate to Inbox with the same selected history entry. Existing custom `workspace.conversation_mode` shortcuts remain valid.

Inbox keeps a history sidebar with **Inbox** and **Archive** tabs, the agent
launcher, and automatically detected conversation titles. Conversation renaming
is unavailable in Chartr. The body is
the session’s original terminal, including its input, scrollback, selection,
clipboard, terminal search, and interactive agent UI. There is no separate chat
renderer, composer, prompt injection, or chat approval interface.

Inbox and Sidebar share the same resizable pane and saved width. Drag its right
edge to resize it. Inbox needs at least 120 px; entering Inbox from a narrower
Sidebar smoothly expands the pane. The expanded width stays when returning to
Sidebar, where the pane can be narrowed again. Reduce motion disables the animation.
During view transitions and automatic sidebar expansion, terminal grids keep their
current size and resize once to the final dimensions when the animation finishes.

Selecting a live entry mounts the same terminal used by Sidebar and Tabbed.
Switching views does not start, resume, or stop an agent. When its terminal is
no longer available, a successful runtime refresh automatically moves the entry
to **Archive** with a **Session ended** state, preserving its title, recency, and
saved history. This also archives older ended entries after the first successful
refresh on startup. Idle sessions remain in Inbox; a backend disconnection alone
does not archive anything. If the selected Inbox entry ends, its selection clears
without switching the history filter. Inbox does not resume an ended session or
show a reconstructed transcript. If the
same terminal starts a different native conversation, the old entry cannot
control it. Mobile Companion is excluded from the current desktop build.

For ended sessions, **Open session log** locates the provider's log and opens it
in the system's associated application. Lookup runs in the background, matches
the exact native session ID, and reports missing or ambiguous files. Chartr does
not render the log. The provider determines what the original file contains;
Chartr does not reconstruct omitted tool output, attachments or compacted turns.
On macOS, an unsuccessful file open falls back to the default text editor, then
TextEdit. If all attempts fail, Inbox displays the error.

| Provider | Log opened |
| --- | --- |
| Codex | Exact rollout JSONL in `sessions` or `archived_sessions` |
| Claude | `projects/<project>/<id>.jsonl` |
| Pi / OMP | Exact reported JSONL path, or `sessions/<project>/<timestamp>_<id>.jsonl` |
| Kimi | Verified session's `agents/main/wire.jsonl` |
| Grok Build | `sessions/<project>/<id>/updates.jsonl` |
| Cursor | `projects/<project>/agent-transcripts/<id>/<id>.jsonl`, or older flat `<id>.jsonl` / `<id>.txt` |
| Antigravity | `brain/<id>/.system_generated/logs/transcript.jsonl` in the CLI or desktop data root |
| OpenCode | Full JSON from `opencode export <id>`, opened as an owner-only temporary file |

OpenCode uses the installed executable from PATH or `~/.opencode/bin/opencode`,
and the recorded project directory when it still exists. Exports time out after
30 seconds and are checked for the requested session ID before opening. Successful
temporary exports remain available for the external editor and OS temp cleanup;
failed exports are removed. No agent conversation is started.

Grok uses `$GROK_HOME` or `~/.grok`; Cursor uses `$CURSOR_CONFIG_DIR` or `~/.cursor`.
OMP uses its reported file path (including profiles/custom session directories),
with ID-only fallback under `$PI_CODING_AGENT_DIR` or `$OMP_CONFIG_DIR/agent`
(default `~/.omp/agent`). Antigravity checks `~/.gemini/antigravity-cli` and
`~/.gemini/antigravity`. The pinned Herdr integrations retain IDs but discard
Cursor/Antigravity transcript paths, so nonstandard data locations are not yet
discoverable. Disabled Cursor transcripts and legacy database-only histories
report an unavailable log.

OMP, Cursor (`cursor-agent`) and Antigravity CLI (`agy`) use the bundled Herdr
discovery installers. Their native identities reach Inbox and survive archive
and restart. Desktop IDE conversation discovery is outside the terminal-based
Inbox workflow. Cursor, Antigravity and OMP have fixture coverage but still need
live verification with functioning installations.

Inbox always lists conversations from all spaces, including Free sessions,
with newest conversations first. Sidebar and Inbox have no title-bar space
picker. Each compact row shows its status, title, and timestamp on one line.
Hover for the full title, agent adapter, owning space, and working directory.

Ordinary shells, lazygit, and other tools keep running without becoming history
entries. Archiving a conversation, including with `Cmd/Ctrl+W` in Inbox, keeps
its process running. The **Inbox** tab shows **Recent chats** with a **+** launcher;
the **Archive** tab shows **Archived chats** without the launcher. The conversation
header’s archive button restores an archived entry.
Pane move/join/close commands do not affect hidden terminal panes. Explicitly
creating a terminal or plugin returns to the previous terminal layout.

## Launching agents

The **+** beside **Recent chats** opens the compact **New conversation** panel.
Choose a registered **Agent** and press **Launch** to start it directly in a new
terminal in Inbox.
There is no opening-message step: type in the agent’s own terminal. Use the
panel’s **Space** picker to choose where the new conversation runs. **Manage agents…** opens the Agent registry settings.

Launch uses the registered executable, arguments, and environment as saved.
Known providers have their Herdr discovery integration installed/verified
before launch. The profile and destination are rechecked during asynchronous
startup; changing the selected space does not redirect an in-flight launch.
The allocated terminal appears immediately, including authentication/setup
screens, and its detected history entry is selected when available. Setup or
startup failures are shown inline, with no automatic retry.

OpenCode no longer receives extra listener options or API-driven session
creation. Saved session, continuation, model, and agent arguments go to the CLI
unchanged. Registered custom commands can still launch even when Herdr cannot
identify them as a supported history provider.

## Discovery and persistence

`chartr-conversations` owns the existing SQLite history index, independent of
GPUI panes. Identity is provider + provider data namespace + native session ID.
Runtime bindings use Herdr pane and terminal identities; rearranging panes does
not rename or duplicate a conversation. Provisional rows merge transactionally
when an identity arrives. Manually resuming an existing native ID reconnects its
history entry.

Herdr detects Codex, Claude Code, OpenCode, Grok, Kimi, Pi, OMP, Cursor and Antigravity CLI sessions. Detection alone
produces a provisional entry whose terminal is already usable. A missing native
session ID does not add setup guidance above the terminal. Launching through
Inbox installs/verifies discovery hooks automatically; existing processes do
not retroactively load new hooks.

Local, read-only provider metadata and transcript readers supply titles and
recency. Titles prefer a previously saved manual Chartr name, then the provider's saved name,
then its observed terminal title, then the existing first-prompt excerpt
(whitespace collapsed, limited to 100 characters). Missing, blank or unreadable
native metadata leaves the fallback available. A prompt excerpt never replaces
an available provider title.

| Provider | Saved title source |
| --- | --- |
| Codex | Latest matching `thread_name` in `session_index.jsonl`; otherwise `name`/`title` in the latest versioned `state_*.sqlite` |
| Claude | `custom-title` first, then `ai-title` in the exact session JSONL |
| Pi | Latest `session_info.name` in the verified session JSONL |
| OMP | Current `title` header, with older `title_change`/session title support |
| OpenCode | Session database title, excluding new/child-session placeholders |
| Kimi Code | Verified session's `state.json` title |
| Grok | Verified session's `summary.json` generated title or legacy session summary |
| Cursor / Antigravity | Observed terminal title |

Separate metadata titles refresh even when the transcript has not changed;
provider title changes do not change conversation recency. Provider stores are never written
or migrated. The running CLI’s terminal title also supplies titles before native
identity arrives. Kimi Code recency
uses the last main-agent `prompt.accepted` timestamp from the exact native
session's `agents/main/wire.jsonl`, after verifying `state.json` identity.
Its data root is `$KIMI_CODE_HOME` or `~/.kimi-code`, following the
[Kimi Code storage layout](https://github.com/MoonshotAI/kimi-code/blob/main/docs/en/guides/sessions.md).
This catches turns completed between refreshes and messages sent while Chartr
was closed, without treating replies, tools, or subagent events as new prompts.
Legacy Python Kimi logs are not read by this adapter. Pi uses its exact reported JSONL path and a
matching session header. The launcher adapts Pi’s managed hook for older
`hasUI`/`agent_end` and newer `mode`/`agent_settled` extension APIs. Existing Pi
terminals need `/reload` after an integration update to load the corrected hook.
No reader guesses the newest
transcript in a project: native identity must match exactly. Missing or
unsupported transcript data does not prevent terminal use. Provider data paths
are inherited by Chartr; remote hosts and separate per-terminal data namespaces
need additional discovery support.

Each observation records the terminal’s owning space separately from its cwd.
Ownership survives identity promotion, exit, and restart. Renamed spaces use
current registered names; removed spaces keep their last recorded label.
Legacy entries without an owner use the most specific registered folder that
contains their cwd. Duplicate space names include paths in labels/pickers.

Previously saved manual titles are retained for compatibility and take precedence
over automatic titles. Search covers title, provider, space,
and project path. New user turns update recency; streaming output does not move
rows. Selection and archive state persist across restart.

History stays in `conversations.sqlite` beside the workspace database, under
`$XDG_STATE_HOME/chartr` or `~/.local/state/chartr`. It is owner-readable/writable
on Unix. Legacy cached messages, drafts, and receipts remain in the existing
index for compatibility; Inbox never displays or sends those drafts/receipts.
Archiving hides an entry without deleting it. There is no bulk export/delete UI.

## Verification

```sh
cargo fmt --all --check
cargo test --workspace --locked --no-fail-fast
cargo build -p chartr --locked
```

Optional local checks (require installed tools and existing sessions; do not start
an agent or open an editor):

```sh
cargo test -p chartr-conversations installed_opencode_exports_a_real_session -- --ignored
cargo test -p chartr-conversations installed_grok_log_matches_its_directory_identity -- --ignored
```

Coverage includes session identity/promotion, ownership and archive persistence,
legacy preference/data compatibility, registered launch arguments, and rejection
of stale terminal bindings. The former chat transport integration fixtures were
removed along with rich chat.
