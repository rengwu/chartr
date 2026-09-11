# Conversation mode

Choose **Conversations** in the window’s view selector or command palette
(`Cmd+Shift+3` on macOS, `Ctrl+Shift+3` on Linux).
History replaces the terminal workspace with a searchable conversation list and
native chat surface. Sidebar and Tabbed still show the original terminal layout.
Switching views does not launch, resume, stop, or summarize an agent.

The space picker offers **All spaces** in Conversations mode. It mixes history
from every space in recency order; choosing a single space filters that same list.
Every row shows its space beneath its title in both scopes. Search also matches
space names. All spaces is remembered across restart and view changes without
replacing the active terminal space.

**Show in terminal** reveals the selected conversation’s existing terminal.
Ordinary shells, lazygit and other tools remain running without becoming history
entries. Archiving a conversation, including with `Cmd/Ctrl+W` in this mode,
keeps its process running. Pane move/join/close commands do not affect hidden
terminal panes. Explicitly creating a terminal or plugin returns to terminal mode.

## Current provider support

| Provider | Discovery | Readable history | Rich input |
| --- | --- | --- | --- |
| OpenCode | Herdr native session ID | Local database; live API when available | Text, interruption, single-question choices, allow-once/deny permissions through the same running CLI’s local server |
| Codex | Herdr native session ID | Exact-ID local JSONL rollout adapter | Plain text through the existing terminal, with transcript confirmation |
| Claude Code | Herdr native session ID | Exact-ID local JSONL transcript adapter | Plain text through the existing terminal, with transcript confirmation |
| Grok Build | Herdr detection/session ID | No verified transcript adapter yet | Continue in terminal |

Detection alone produces a provisional row. **Enable integration** invokes
Herdr’s provider integration installer; restart the agent to load newly installed
hooks/plugins. Setup checks the installed state, shows progress and inline errors,
and changes to **integration is enabled** after verification. Installation does
not retroactively attach a running agent.

Upgrades from the older Rust implementation also migrate its generated shell
wrapper, which could restore another terminal's Herdr socket and pane ID. The
original wrapper is retained as `chartr-session-shell.before-conversations`.
Existing processes keep their old environment: open a **fresh terminal tab** and
start or resume the agent there. OpenCode's managed server plugin is also placed
in the config directory actually loaded by these terminals, including the older
wrapper's restored XDG directory; other plugins are preserved.

A session without a verified provider ID is never matched to the
newest file in its project directory. These initial file readers assume the
provider data locations inherited by Chartr. Enable the provider integration for continuous discovery when changing conversations inside a manually launched CLI. Launching a registered known provider from History installs/verifies its integration before starting the process. Separate per-terminal data/config
namespaces and remote hosts need additional integration.

OpenCode **1.2.27** was tested with real TUI processes. Its default TUI uses an
internal transport. To make a manually launched instance reachable from chat, use:

```sh
opencode --port 0 --hostname 127.0.0.1
```

**History → +** opens a compact launch panel with separate **Space** and **Agent**
subpickers and a **Launch** button. All spaces lets you choose the destination;
single-space mode hides the Space field and uses the current space. Agent choices come from the
Agent plugin's registered profiles, including custom names. **Manage agents…**
opens that registry's settings. Launch opens the selected profile's composer;
write its opening message and send to start it in a new ordinary Herdr terminal
in the chosen space. Saved executable, arguments and environment come from the same
registry used in terminal mode; there is no separate conversation-only agent list.
The profile is rechecked before launch, and the detected conversation is selected
automatically. The underlying terminal remains available in Sidebar/Tabbed mode.
Profiles without a supported chat adapter still launch, with an **Open terminal**
route. A setup or startup failure is shown inline; Chartr does not retry the launch
automatically. An opening message retained on the launch screen is currently
in-memory, not a persisted pre-launch draft. These drafts are separate for each
space/profile pair. The chosen destination is retained throughout asynchronous
startup, even if the user changes spaces in the meantime.

For OpenCode, missing listener options are added to the launch command without
changing the saved profile. Chartr creates and selects a conversation through that
exact process's loopback endpoint, waits for the tested TUI to display it, then
verifies the native identity with Herdr before sending the opening message. Saved
model/agent options are used for that message; later messages retain the latest
native user turn's model/agent. Saved continuation/session options are not replaced
with a new conversation: their exact native identity must arrive through Herdr.
The readiness check is version-specific and fails visibly if it cannot establish
that state. The CLI's configuration and authentication remain in use. Servers
requiring additional HTTP authentication are not yet supported by this adapter.

Codex **0.154.0** and Claude Code **2.1.267** use Herdr's input channel to
paste and submit plain text into the original running TUI. No second agent is
started. Sending requires a verified native conversation ID, the observed
foreground process, idle status and a recognized empty terminal prompt. Finish
an approval/menu or clear an existing terminal draft before sending from chat;
a rejected send restores the chat draft. Slash commands and attachments still
use terminal mode. Provider prompt layouts can change, so an unrecognized prompt
fails visibly instead of receiving input.

The composer supports multiline text (`Shift+Enter`), send (`Enter` or
`Cmd/Ctrl+Enter`), drafts, basic Markdown responses, code blocks, copied messages,
and expandable OpenCode tool results. Pending questions with multiple questions,
multiple selection, custom free-text answers, images/attachments and other
provider-specific interactions use **Continue in terminal**. The terminal’s own
unsent draft stays in the terminal; Chartr does not synchronize or overwrite it.

## Identity and persistence

`chartr-conversations` owns a SQLite history index independent of GPUI panes.
Identity is provider + provider data namespace + native conversation ID.
Runtime bindings use Herdr pane/terminal identities; changing a pane group does
not rename or duplicate the conversation. Provisional rows merge transactionally
when an identity arrives. New conversations in reused terminals remain distinct;
resuming an existing ID reconnects its previous history and draft.

Each observation records the terminal's owning space separately from its current
working directory. That owner survives native-ID promotion, terminal exit and
restart. Renamed spaces use their current registered name; history for removed
spaces retains its last known label and remains accessible in All spaces.
Older history without an owner falls back to the most specific registered folder
containing its working directory, using path components rather than string
prefixes. Duplicate registered space names include their paths in labels/pickers.

Titles use provider titles or local prompt excerpts. Manual renaming wins.
Codex history uses its human-message events, excluding injected environment and
AGENTS.md context from messages and automatic titles.
Search covers title, provider, space name and project path. Streaming deltas do not move rows;
new user turns update recency. Selection, drafts and archive state persist;
scroll positions are retained while Chartr remains open. Exit leaves readable
history. Automatic resume after an agent exits is not implemented.

History lives beside the workspace database in `conversations.sqlite`, under
`$XDG_STATE_HOME/chartr` or `~/.local/state/chartr`. The file is owner-readable and
writable on Unix. Provider transcripts are opened read-only. Cached history is
retained until the database is removed; archiving only hides a row. The initial
reader retains the latest 300 rendered messages per conversation, limits source
JSONL reads to 32 MiB, and truncates oversized individual messages with a notice.
There is no export/delete interface or indefinite full-transcript guarantee yet.

Every live input route requires a fresh runtime observation and the exact
native ID. OpenCode additionally verifies a healthy loopback listener owned by
the observed agent. Codex/Claude recheck the pane, terminal, process, idle status
and empty prompt immediately before one bracketed paste plus Enter. Herdr does
not yet offer an atomic conditional write, so this does not eliminate a race
with another client simultaneously changing that terminal.

Questions are revalidated by ID and contents before answering. An uncertain
send is persisted and never automatically retried. Terminal sends stay pending
until a new matching user message appears in the native transcript; a successful
PTY write alone is not considered acceptance. Pending receipts survive restart.
The composer stays blocked until confirmation or explicit unlocking after a
terminal check, which restores the submitted text for terminal sends. Draft
writes are versioned so a late background save cannot overwrite the latest draft.

## Verification

```sh
cargo test --workspace --locked
cargo test -p chartr-conversations --test live_opencode -- --ignored --nocapture
cargo test -p chartr-conversations --test live_terminal_input -- --ignored --nocapture
cargo build -p chartr --locked
```

The opt-in test requires installed OpenCode, lazygit, Python 3 and the built
`target/debug/herdr`. It creates isolated provider directories and a private
Herdr daemon, and serves deterministic model responses locally. No paid model
or user provider credentials are used. It verifies two manual OpenCode launches
in one cwd, distinct native IDs from Herdr, registered model/agent retention across turns, multiline input, streaming responses,
no cross-routing, preserved process identities, an unsent TUI draft, a running
shell and lazygit, a real pending question, stale-answer rejection, and database
reopen with history/drafts.

The second opt-in test requires installed Codex and Claude CLIs. It launches
real TUIs in another private Herdr daemon and serves both providers' model
responses locally. It verifies multiline Unicode submission, transcript
confirmation, replies in the TUI, unchanged native ID/process, preserved terminal
drafts and rejection of a stale conversation identity. Its Claude launch uses
`--bare` and an explicitly assigned test session ID, reported to the test pane;
production discovery still uses Herdr's SessionStart integration.
