# Inbox

Choose **Inbox** in the window’s view selector or command palette
(`Cmd+Shift+3` on macOS, `Ctrl+Shift+3` on Linux). Existing Conversations view
preferences migrate to Inbox, with the same selected history entry and space
scope. Existing custom `workspace.conversation_mode` shortcuts remain valid.

Inbox keeps a history sidebar with **Inbox** and **Archive** tabs, the agent
launcher, and a header with the conversation title, owning space, rename, and
archive actions. The body is
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
no longer available, the entry remains with a **Session ended** state. Inbox
does not resume an ended session or show a reconstructed transcript. If the
same terminal starts a different native conversation, the old entry cannot
control it. When mobile owns a session’s terminal geometry, Inbox displays its
mobile status until control returns to desktop.

Inbox uses a simple chat list, with newest conversations first. **All spaces**
mixes conversations from every space, including Free sessions, in that same
recency order. Choosing a single space filters the list. Each compact row shows
its status, title, and timestamp on one line. Hover for the full title, agent
adapter, owning space, and working directory. All spaces is remembered across
restart and view changes without replacing the active terminal space.

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
There is no opening-message step: type in the agent’s own terminal. In All
spaces, the panel also offers a **Space** picker; in a single space, it uses
that space. **Manage agents…** opens the Agent registry settings.

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

Herdr detects Codex, Claude Code, OpenCode, Grok, Kimi, and Pi sessions. Detection alone
produces a provisional entry whose terminal is already usable. A missing native
session ID does not add setup guidance above the terminal. Launching through
Inbox installs/verifies discovery hooks automatically; existing processes do
not retroactively load new hooks.

Local, read-only Codex/Claude/Pi JSONL and OpenCode database readers supply titles
and recency. The running CLI’s terminal title also supplies titles before native
identity arrives; Grok and Kimi use this observed metadata. Pi uses its saved
session name or first user prompt, with its exact reported JSONL path and a
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
current registered names; removed spaces keep their last label in All spaces.
Legacy entries without an owner use the most specific registered folder that
contains their cwd. Duplicate space names include paths in labels/pickers.

Manual titles win over automatic titles. Search covers title, provider, space,
and project path. New user turns update recency; streaming output does not move
rows. Selection and archive state persist across restart.

History stays in `conversations.sqlite` beside the workspace database, under
`$XDG_STATE_HOME/chartr` or `~/.local/state/chartr`. It is owner-readable/writable
on Unix. Legacy cached messages, drafts, and receipts remain in the existing
index for compatibility; Inbox never displays or sends those drafts/receipts.
Archiving hides an entry without deleting it. There is no export/delete UI.

## Verification

```sh
cargo fmt --all --check
cargo test --workspace --locked --no-fail-fast
cargo build -p chartr --locked
```

Coverage includes session identity/promotion, ownership and archive persistence,
legacy preference/data compatibility, registered launch arguments, and rejection
of stale terminal bindings. The former chat transport integration fixtures were
removed along with rich chat.
