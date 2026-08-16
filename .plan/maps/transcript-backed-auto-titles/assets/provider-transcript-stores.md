# Transcript stores: Codex, OpenCode, Pi, Kimi, Grok

Findings for ticket 04 of the [transcript-backed-auto-titles](../map.md) map.
Measured first-hand on this host on 2026-08-16 against the installed versions
listed below. These are operational dependencies with no stable published
schema, so every claim is tied to the version it was observed on.

No transcript body, title text, prompt, credential, reasoning or tool output is
reproduced here. Where a value mattered it is described by shape, length, or the
predicate that recognises it.

## Method

Two kinds of primary source were used, and each claim below says which:

- **Store**, meaning records actually on disk in this user's own state roots,
  read structurally (record type censuses, key sets, value *shapes*), never as
  prose.
- **Binary/source**, meaning the installed program itself: readable JavaScript
  for Pi, bun-bundled JavaScript recoverable from the OpenCode and Kimi
  executables, embedded help and documentation in the Codex and Grok
  executables, and Grok's shipped `README.md`.

Where the two disagree or the store predates the installed build, that is said
explicitly. Nothing here comes from a third-party write-up, a blog post, or a
model's recollection of a schema.

## Installed versions

| Provider | Version observed | How obtained | Store corpus read |
| --- | --- | --- | --- |
| Codex | `codex-cli 0.147.0` (`@openai/codex` npm, Rust binary) | `codex --version` | 155 rollouts, written by 0.133.0 / 0.146.0 / 0.147.0 |
| OpenCode | `1.2.27` (Homebrew, bun-compiled) | `opencode --version` | 40 sessions, 467 messages, 1901 parts, all stamped `1.2.27` |
| Pi | `0.78.0` (`@earendil-works/pi-coding-agent`) | `pi --version` | 362 sessions, all session format `version: 3` |
| Kimi | `0.36.1` (Kimi Code) | `kimi --version` | 85 wire logs, `protocol_version` 1.4 (65) and 1.5 (18) |
| Grok | `grok 1.0.0 (3cd0d0cbcebe)` | `grok --version` | 472 sessions, `chat_format_version: 1`; only 5 carry a full conversation, and those were written by the immediately preceding build |

The Grok caveat is the one real gap: 1.0.0 was installed after this host's only
substantive Grok conversations were recorded. Every record type and field named
below is present as a symbol in the 1.0.0 executable and matches its embedded
documentation, but the *instances* were produced by the previous build. Ticket
05 should re-check Grok against a live 1.0.0 session before trusting the parser.

---

## Codex

### Where sessions live

- **Env allowlist:** `CODEX_HOME`. **Default when unset:** `~/.codex`
  (the CLI's own `--config` help names `~/.codex/config.toml` as the file the
  variable relocates; `CODEX_HOME` appears 61 times in the binary as the root
  every other path hangs off).
- **Transcript path:** `$CODEX_HOME/sessions/<YYYY>/<MM>/<DD>/rollout-<local-timestamp>-<session-uuid>.jsonl`.
  The date shard and the filename stamp are **local** time; the `session_meta`
  record inside carries UTC. (Observed: a file under `sessions/2026/08/15/`
  named `…T02-42-41…` whose head record timestamps `2026-08-14T18:43:15Z`, on a
  UTC+8 host.)
- `CODEX_SQLITE_HOME` also exists and relocates the sidecar databases
  (`state_5.sqlite`, `thread_history_1.sqlite`). The rollout JSONL is not under
  it, so an adapter that reads only the rollout does not need it.

### Storage family

**Append-only JSONL.** Byte-offset cursor, tolerate an incomplete trailing
record. Codex itself does exactly this: `thread_history_projection_state` in
`thread_history_1.sqlite` stores `next_rollout_byte_offset` and
`next_rollout_ordinal` per thread, i.e. the provider's own reader is a byte
offset over the same file.

The sidecar SQLite databases are a *projection* of the rollout, not a second
source of truth. An adapter should not read them (see native title, below).

### Record shapes

Every line is `{timestamp, ordinal, type, payload}`. Top-level `type` census over
155 rollouts: `event_msg` (14423), `response_item` (18587), `turn_context` (532),
`world_state` (173), `session_meta` (157), `inter_agent_communication_metadata`
(18), `compacted` (4).

The critical structural fact: **`response_item` is the model wire, `event_msg`
is the UI item stream.** Synthetic user-role material lives only in the former.
Of 2647 `response_item` messages, 132 user-role records carry an
`<environment_context>` envelope and 4 are `role: "developer"`. Of the 194
`event_msg` / `item_completed` records whose item is a `UserMessage`, **zero**
carried any synthetic marker. An adapter must read the `event_msg` stream and
ignore `response_item` entirely.

`session_meta.history_mode` selects between two shapes, and it is **not** a pure
function of version — 0.147.0 writes both:

| `history_mode` | Human turn opens on | Assistant text | Files observed |
| --- | --- | --- | --- |
| `paginated` | `event_msg` / `item_completed` with `item.type == "UserMessage"` | `item.type == "AgentMessage"`, `item.phase == "final_answer"` | 44 |
| `legacy` | `event_msg` / `user_message` | `event_msg` / `agent_message`, `phase == "final_answer"` | 110 |

One further file (0.133.0) has no `history_mode` key at all and writes the
legacy shape; an absent value should be treated as unknown and fail closed
rather than defaulting to legacy.

- **Top-level human text turn.** *Paginated:* `item.content` is an array whose
  entries are `{type:"text", text, text_elements}`; a `local_image` entry
  appears for image prompts (6 of 200 content entries) and disqualifies the turn
  as non-text-only. The record carries `turn_id`. *Legacy:* `payload.message` is
  the text, with sibling `images`, `local_images`, `audio`, `local_audio` arrays
  that must all be empty for a text-only turn; there is **no `turn_id`** on this
  record, so identity has to come from the closing record.
- **Turn completion and final visible assistant text, in one record.**
  `event_msg` / `task_complete` carries `{turn_id, last_agent_message, started_at,
  completed_at, duration_ms}`. In all 166 turns where both were present,
  `last_agent_message` was byte-identical to the last `AgentMessage` item of that
  `turn_id`. 10 of 443 `task_complete` records had an empty
  `last_agent_message` — those are turns that produced no visible text and must
  emit nothing.
- **Interruption.** `event_msg` / `turn_aborted` with `{turn_id, reason}`
  (21 observed). A turn that aborts never gets a `task_complete`.
- **Stable turn identity.** `turn_id` (uuid) from `task_complete`, matched to the
  opening `UserMessage`'s own `turn_id` in paginated mode, or to the most recent
  unclosed user message in legacy mode.

**Record kinds to ignore.** All `response_item` records without exception
(`message`, `reasoning`, `function_call`, `function_call_output`,
`custom_tool_call`, `custom_tool_call_output`, `tool_search_call`,
`tool_search_output`, `agent_message`); `turn_context`; `world_state`;
`compacted`; `inter_agent_communication_metadata`; and the `event_msg` types
`task_started`, `token_count`, `thread_settings_applied`, `web_search_end`,
`patch_apply_end`, `mcp_tool_call_end`, `sub_agent_activity`,
`context_compacted`. Within `item_completed`, ignore item types `Reasoning`,
`CommandExecution`, `FileChange`, `McpToolCall`, `ContextCompaction`,
`Extension`, `ImageView`. `AgentMessage` items with `phase == "commentary"`
(432 of 602) are intermediate progress, not the answer.

**Whole files to ignore.** `session_meta.thread_source` is `"user"` or
`"subagent"`; subagent threads get their own rollout in the same tree with the
same `cwd` (29 of 155 files). `session_meta.originator` is `"codex-tui"` for the
interactive CLI and `"codex_exec"` for `codex exec` (13 files), and
`session_meta.source` is `"cli"`, `"exec"`, or an object for subagents. A binding
candidate must have `thread_source == "user"` and `originator == "codex-tui"`.

### Failing closed

Sniff, in order: a complete line parses as JSON; the object has a string `type`;
a `session_meta` head record exists and has `cli_version`, `cwd`, and
`history_mode`; `history_mode` is one of the two known values. An unrecognised
`history_mode`, a missing `payload.type`, or a `UserMessage` item whose `content`
is not an array of typed parts ends the binding. New `event_msg` types and new
`item.type` values are ordinary and skipped — Codex adds them often (this corpus
alone spans three versions and shows the `item_completed` family appearing).

### Native title

**None usable.** There is no title record anywhere in the rollout — a scan of
all 155 files for any key containing "title" found none. The sidecar
`state_5.sqlite` `threads` table does have `title`, `preview` and `name`
columns, and they resolve as follows across all 155 rows:

- `title == preview == first_user_message` for **every** row, with lengths from
  2 to 53,561 characters (mean 6,539). It is a verbatim copy of the first user
  message, not a summary. Displaying it would put a whole prompt in the tab
  title and, because a native title blocks paid generation for the life of the
  session, would pin every Codex tab to that forever.
- `name` is a genuine user-assigned session name (the TUI has a rename; the
  binary carries `Session renamed to …`, and `codex resume`/`archive`/`delete`
  accept "session id or session name"). It is `NULL` on all 155 rows here, is
  never auto-populated, and lives in a separate SQLite state database that an
  otherwise-JSONL adapter would have to open specially.

Recommendation: Codex exposes **no native title**. Reading `threads.name` is a
possible later addition, but it buys an optional field that is unset by default
at the cost of importing a second storage family into the adapter.

### Discovery and binding

- **Metadata available before any message body:** the `session_meta` head record
  gives `session_id`, `cwd`, `cli_version`, `originator`, `source`,
  `thread_source`, `history_mode`. One 512-byte read of the file head is enough
  to accept or reject a candidate.
- **Direct process-to-session registry: none.** No pid appears anywhere under
  `$CODEX_HOME`. Live threads *are* identifiable — `$CODEX_HOME/thread-writer-locks/<thread-uuid>.lock`
  holds one advisory lock per live writer — but the lock names the thread, not
  the holder, and resolving the holder needs an fd-table walk (`lsof`-class
  syscalls) that this project has no reason to acquire. Treat Codex as having no
  registry.
- **Route to use:** the same fallback Claude has. Rollouts written since the
  agent started, under today's (and, across a local midnight, yesterday's) date
  shard, whose head record names this working directory and is
  `thread_source: "user"` / `originator: "codex-tui"` — one candidate or none.

### One-shot generation

**Yes, and already shipped.** `codex exec [--model <id>] <prompt>` is the
non-interactive path, and `internal/adapter/titlegen.go` already carries the
Codex row (`gpt-5-nano`, `gpt-5-mini`, then the account default). No new work.

Note the interaction: a `codex exec` run writes its own rollout into the same
`sessions/` tree with the same `cwd`. It is excluded by the
`originator == "codex-tui"` filter above, which the adapter needs anyway for
subagents.

---

## OpenCode

### Where sessions live

- **Env allowlist:** `XDG_DATA_HOME`, **with the fixed suffix `/opencode`**.
  **Default when unset:** `~/.local/share/opencode`. From the bundled source
  (`src/global/index.ts` and its `xdg-basedir` init):
  `xdgData = env.XDG_DATA_HOME || join(homedir(), ".local", "share")`, then
  `data = join(xdgData, "opencode")`.
- **Store path:** `<data>/opencode.db`, resolved as
  `path.join(Global.Path.data, "opencode.db")`.
- `OPENCODE_CONFIG_DIR` and `OPENCODE_CONFIG` select *configuration*, not data,
  and must not be in the allowlist. `OPENCODE_TEST_HOME` overrides only
  `Global.Path.home`, not `Global.Path.data`.

> **This breaks an assumption in `internal/adapter/stateroot.go`.** That table
> returns an environment variable's value as the root verbatim. OpenCode is the
> first provider where the variable names a *parent* and the provider appends a
> fixed segment: `XDG_DATA_HOME=/foo` means the root is `/foo/opencode`, not
> `/foo`. `stateRootSpec` needs a per-variable suffix (and the `fallback`
> already encodes the same idea home-relative). Pi has the same shape for its
> secondary variable. This is a small, contained change to ticket 05's data
> table, not to any caller.

### Storage family

**Database-backed.** SQLite via Drizzle, with `-wal` and `-shm` sidecars present,
so the agent is writing under WAL while chartr reads. Open read-only
(`file:opencode.db?mode=ro`), never migrate, never write. Relevant tables:

- `session(id, project_id, parent_id, slug, directory, title, version, …, time_created, time_updated, time_archived, workspace_id)`
- `message(id, session_id, time_created, time_updated, data)` — `data` is the
  message JSON minus the id/session columns
- `part(id, message_id, session_id, time_created, time_updated, data)` — indexed
  `part_session_idx (session_id)`

Cursoring: `part` and `message` are ordinary rowid tables, so `WHERE session_id = ?
AND rowid > ? ORDER BY rowid LIMIT n` is the incremental read, with
`session_project_idx` / `part_session_idx` keeping it off a full scan. Note that
parts are **updated in place** while a response streams (`time_updated` moves,
rowid does not), so a completed-turn reader must re-read the parts of the
message it is closing rather than trusting a rowid high-water mark alone.

### Record shapes

Taken from the bundled `src/session/message-v2.ts` zod schemas, which are the
authoritative definition, and confirmed against the store.

- **Top-level human text turn.** `message.data.role == "user"`. The message
  carries `{role, time:{created}, agent, model:{providerID, modelID}}` and
  optionally `format`, `summary`, `system`, `tools`, `variant`. Its parts must
  be `type: "text"` with **neither `synthetic` nor `ignored` set** — OpenCode's
  own first-real-user-message predicate is exactly
  `part.type !== "text" || part.ignored || part.synthetic`, and its title
  generator skips messages whose parts are *all* synthetic.
- **Final visible assistant text.** Assistant messages carry
  `parentID` = the user message id, so a turn is one user message plus every
  assistant message pointing at it (observed ratio ≈ 4 assistant messages per
  user message: 374 to 93). The final visible text is the `text` parts of the
  **last** such assistant message.
- **Turn completion.** `assistant.time.completed` is set and `assistant.error`
  is absent. Observed: 370 of 374 completed, 12 carrying an `error`
  (`APIError`, `ProviderAuthError`, `MessageAbortedError`,
  `MessageOutputLengthError`, `StructuredOutputError`, `ContextOverflowError`,
  `NamedError.Unknown` are the whole union), 4 still open.
- **Stable turn identity.** The user message `id` (`msg_…`, monotonically
  ordered), which is also the assistant messages' `parentID`.

**Part kinds to ignore.** The `Part` discriminated union is exactly
`text | subtask | reasoning | file | tool | step-start | step-finish | snapshot |
patch | agent | retry | compaction`. Everything but a non-synthetic,
non-ignored `text` is ignored. `subtask` is subagent dispatch; `compaction` and
assistant messages with `summary: true` are summaries; `tool` state carries
`input`/`output` and never crosses the seam.

**Whole sessions to ignore.** `parent_id IS NOT NULL` marks a child/subagent
session (4 of 40 here); they have their own real titles and their own directory,
and must never be a binding candidate for a tab.

### Failing closed

Sniff the columns the reader needs — `session.directory`, `session.title`,
`session.parent_id`, `session.version`, `part.data`, `message.data` — and the
message discriminator `data.role ∈ {user, assistant}`. `session.version` stamps
the OpenCode version that created the row (`Installation.VERSION`; all 40 rows
here read `1.2.27`), which gives a per-session version to record in fixtures. A
missing column, an unparseable `data` blob, or a role outside the union ends the
binding.

### Native title

**Yes — `session.title`, and it is real.** This is the strongest native title of
the five.

- It starts as a **placeholder** and must be filtered: OpenCode's own predicate
  is `^(New session - |Child session - )\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$`,
  exported as `Session.isDefaultTitle`. 10 of 40 sessions here still hold a
  placeholder. **Publishing a placeholder would block paid generation forever
  with a timestamp**, so the adapter must reproduce this regex exactly.
- Once a real title is set it is **generated once and never refreshed**:
  `ensureTitle` returns early unless the session has no parent, still holds a
  default title, and has exactly one non-synthetic user message. Observed real
  titles run 3–59 characters (mean 30).
- It therefore updates exactly once per session, early in the first turn — the
  same free-path-wins-the-race dynamic Claude's `ai-title` has.

### Discovery and binding

- **Metadata:** `session.directory` (the cwd), `session.time_created`,
  `session.parent_id`, `session.workspace_id`, and `project.worktree`.
- **Direct process-to-session registry: none.** OpenCode sets `OPENCODE_PID` into
  its *own* environment for children; nothing on disk maps a pid to a session.
- **Route:** rows with `parent_id IS NULL`, `directory` equal to the agent's
  working directory, and `time_created` after the agent started — one or none.
- **Open question for ticket 05.** Whether the `session` row is inserted at TUI
  start or at first prompt was not established first-hand; all 40 sessions here
  have at least one message, which is consistent with lazy creation on submit.
  If it is lazy, an OpenCode tab can never see its *opening* turn, because the
  row and the first user message appear together and the cursor seats after
  them. This costs little in practice: the native title arrives during that same
  first turn, for free.

### One-shot generation

**Yes.** `opencode run [-m provider/model] [--agent <name>] <message>` is the
documented non-interactive path, with `--format json` available. There is no
`--no-session` equivalent, so a generation run inserts its own `session` row in
the same directory; `--title` can label it. That row is a false candidate only
for a *different* tab that is still unbound in the same directory, where the
"unique or nothing" rule turns it into a missing title rather than a wrong one.

---

## Pi

Pi ships readable JavaScript, TypeScript declarations, and a first-party
`docs/session-format.md`. It is the best-documented store of the five, and the
findings below are from that documentation cross-checked against the corpus.

### Where sessions live

Precedence, from `dist/config.js` and `dist/main.js`:

1. `--session-dir <dir>` (a flag, not visible to an environment reader)
2. `PI_CODING_AGENT_SESSION_DIR` — the session directory **directly**
3. `PI_CODING_AGENT_DIR` — the agent directory; sessions are `<dir>/sessions`
4. `settings.json`'s `sessionDir` (read from the resolved agent directory)
5. Default: `~/.pi/agent/sessions`

The variable names are derived, not literal: `ENV_AGENT_DIR =
`${APP_NAME.toUpperCase()}_CODING_AGENT_DIR``, where `APP_NAME` comes from the
package's `piConfig.name` and is `pi` for this build. A rebranded build would
use a different prefix. Both variables expand a leading `~`.

**Transcript path:** `<sessions>/--<cwd with leading slash stripped and `/`,
`\`, `:` replaced by `-`>--/<ISO-timestamp>_<session-uuid>.jsonl`. The encoding
is computable (`getDefaultSessionDirPath`) but lossy, so the directory should
locate the file and the header's own `cwd` should confirm it.

### Storage family

**Append-only JSONL**, one file per session, entries forming a tree via
`id`/`parentId`.

### Record shapes

- **Header, always the first line:** `{type:"session", version:3, id, timestamp,
  cwd}`, optionally `parentSession` for a `/fork` or `/clone`. All 362 files
  have it first; all are `version: 3`.
- **Top-level human text turn:** `{type:"message", id, parentId, timestamp,
  message:{role:"user", content, timestamp}}`. `content` is documented as a
  string *or* an array of `TextContent | ImageContent`; all 969 observed user
  messages used the array form and were text-only. An `ImageContent` entry
  disqualifies the turn.
- **Final visible assistant text and completion:** the answering
  `{message:{role:"assistant", content, stopReason, …}}` with
  `stopReason == "stop"` and at least one `{type:"text"}` content block.
  `stopReason` is a closed union: `stop | length | toolUse | error | aborted`
  (observed 863 / 0 / 5634 / 39 / 36). `toolUse` continues the turn; `error` and
  `aborted` end it with nothing.
- **Stable turn identity:** the user entry's 8-char hex `id`; the assistant entry
  chains back through `parentId`.

**Record kinds to ignore.** Entry types `model_change`, `thinking_level_change`,
`compaction`, `branch_summary`, `custom` (extension state), `custom_message`
(extension-injected, user-shaped, and *does* enter LLM context), `label`. Message
roles `toolResult` (its own role — 8009 records, so tool results never wear the
user role here), `bashExecution`, `custom`, `branchSummary`, `compactionSummary`.
Assistant content blocks of type `thinking` (6357) and `toolCall` (8009).

The census over 362 files is small and total: 4 entry types (`session`,
`model_change`, `thinking_level_change`, `message`), 3 message roles, 5 content
block types. No synthetic or slash-command envelope appeared in any of the 969
user messages.

### Failing closed

Sniff: first line is `type:"session"` with a string `id` and a `version` of 3
(Pi's own `loadEntriesFromFile` rejects a file whose head is not that); every
message entry has `id`, `parentId`, and a `message` object with a known `role`;
assistant `stopReason` is in the documented union. Version 1 and 2 files exist
historically and are auto-migrated on load — an adapter should refuse anything
that is not 3 rather than guess.

### Native title

**Yes, but user-set only.** A `{type:"session_info", id, parentId, timestamp,
name}` entry carries a display name, set by `/name <name>`, `--name`/`-n`, or an
extension's `pi.setSessionName()`. `getSessionName()` reads *the latest*
`session_info` entry, so it can change over a session's life and the adapter
should publish each change. Pi never generates one: zero of the 362 sessions in
this corpus carries a `session_info` entry.

So Pi needs the paid path in practice, with `session_info` as a free override
when an operator names a session.

### Discovery and binding

- **Metadata:** the header's `cwd`, `id`, `timestamp`, read from the first 512
  bytes (Pi's own `readSessionHeader` does exactly that).
- **Direct process-to-session registry: none.**
- **Route:** the computed encoded-cwd directory, files created since the agent
  started, header `cwd` equal to the working directory — one or none.
- **The file appears late, and this matters.** `SessionManager._persist` buffers
  every entry in memory and does not create the file until the first *assistant*
  message exists; at that moment it writes the header, the first user message and
  that assistant entry together with `open(…, "wx")`. There is no header-only
  file (none of the 362 has fewer than 2 entries). A binding therefore always
  arrives after the opening prompt is already on disk, so under the seat-at-end
  rule **Pi's first turn can never be titled** and a single-prompt session stays
  untitled.

  One option for ticket 05, flagged as a judgment call rather than a
  recommendation: seat the cursor at offset zero when chartr *observed the
  store's absence* at bind time — the direct analogue of Claude's
  registry-known-but-unwritten case. Nothing historical can leak, because the
  whole file was written after both the agent started and the watch began. It is
  a deliberate reading of "a session bound before its first write", and it should
  be decided by a human rather than assumed.

### One-shot generation

**Yes, and it is the cleanest of the five.**
`pi --no-session -p "<prompt>" [--model <pattern>]`. `--print`/`-p` is
"non-interactive mode: process prompt and exit", `--model` accepts
`provider/id`, and **`--no-session` makes the run ephemeral**, so a Pi title
generation writes no session file at all and can never become a false binding
candidate.

---

## Kimi

### Where sessions live

- **Env allowlist:** `KIMI_CODE_HOME`. **Default when unset:** `~/.kimi-code`.
  Verbatim from the 0.36.1 bundle:
  `homeDir ?? process.env["KIMI_CODE_HOME"] ?? join(os.homedir(), ".kimi-code")`.
  This confirms at 0.36.1 the row ticket 01 added from a 0.29.0 measurement; the
  variable and default are unchanged.
- **Paths:** `$KIMI_CODE_HOME/sessions/wd_<slug>_<hash12>/session_<uuid>/`
  containing `state.json`, `agents/<agentId>/wire.jsonl`, and `logs/`. The
  bucket prefix is `WORKDIR_KEY_PREFIX = "wd_"` and the name is documented in the
  bundle as "bucket directory name `wd_<slug>_<hash12>` for a workdir path" —
  a slug plus a hash, so it is not reversible and should not be recomputed.
- `$KIMI_CODE_HOME/session_index.jsonl` is an append-only index of
  `{sessionId, sessionDir, workDir}` — 83 lines for 83 sessions here. It removes
  any need to walk the bucket tree.

### Storage family

**Append-only JSONL** (`agents/main/wire.jsonl`) for the conversation, plus a
small **rewritten JSON sidecar** (`state.json`) for session metadata and the
title. Byte-offset cursor on the wire log; a size/mtime check on `state.json`.

### Record shapes

Wire-log type census (85 files): `context.append_loop_event`, `llm.request`,
`usage.record`, `context.append_message`, `turn.prompt`, `turn.ended`,
`tools.update_store`, `permission.set_mode`, `metadata`, `task.started`,
`task.terminated`, `llm.tools_snapshot`, `profile.bind`, `interaction.request`,
`interaction.resolved`, `permission.record_approval_result`,
`plugin.session_start`, `config.update`, `turn.cancel`,
`interruptionReminder.recorded`, `turn.steer`, `plan_mode.enter`,
`plan_mode.cancel`, `tools.set_active_tools`, `full_compaction.begin`,
`context.apply_compaction`, `full_compaction.complete`.

Only four of them matter:

- **Head record:** `{type:"metadata", protocol_version, created_at}`.
- **Top-level human text turn:** `{type:"turn.prompt", input:[…], origin:{kind},
  time}`. `origin.kind` is the human gate — observed `user` (128),
  `skill_activation` (1), `task` (1); only `user` counts. `input` entries are
  `{type:"text", text}` (133) or `{type:"image_url", …}` (5), and an
  `image_url` entry disqualifies the turn as non-text-only. This record carries
  **no** `turnId`.
- **Assistant content:** `{type:"context.append_loop_event", event:{…}, time}`
  where `event.type` is `step.begin | content.part | tool.call | tool.result |
  step.end`. Every one of these except `tool.result` carries `turnId` (a
  *string*) and `step`. A `content.part` has `part.type` of `text` (71) or
  `think` (142); only `text` is visible. `step.end` carries `finishReason`,
  observed `tool_use` (1033) and `end_turn` (97).
- **Turn completion:** `{type:"turn.ended", turnId, reason, durationMs, time}`.
  `reason` is `completed` (34), `failed` (7), or `cancelled` (3). Note `turnId`
  is an **integer** here and a **string** in the loop events — the adapter must
  normalise, and a mismatch in that assumption is worth a fixture.
- **Stable turn identity:** the `turnId` of the first `step.begin` following the
  `turn.prompt`, matched against `turn.ended`.

**Record kinds to ignore.** Everything else in the census, in particular
`context.append_message` (261 records — the same prompts mirrored into the model
context, plus user-role machinery; the `turn.prompt` stream is the clean one),
`llm.request`, `usage.record`, `interruptionReminder.recorded`, `turn.steer`
(mid-turn injection, whose `origin.kind` was `skill_activation`),
`permission.*`, `interaction.*`, `full_compaction.*`, `task.*`, `plugin.*`.

**Whole files to ignore.** Subagents are physically separated: the main
conversation is `agents/main/wire.jsonl` and a subagent's is
`agents/agent-0/wire.jsonl` (2 of 85 files). Reading only `agents/main/` excludes
subagent traffic structurally, with no predicate needed.

### Failing closed

Sniff `metadata.protocol_version` on the head record. Observed values are `1.4`
(65 files) and `1.5` (18, current). Also require `turn.prompt.input` to be an
array of typed parts and `turn.ended.reason` to be in the known set. An
unrecognised `protocol_version` should end the binding rather than be parsed
optimistically — this is the one provider that hands chartr an explicit,
versioned protocol number, so use it.

### Native title

**Present but almost never usable on this install.** `state.json` carries
`{title, titleKind, isCustomTitle}`, and the 0.36.1 bundle defines
`isSessionTitleKind(value) === (value === "replaceable" || value === "generated"
|| value === "custom")` with these meanings:

| `titleKind` | Set by | Usable as a native title |
| --- | --- | --- |
| `replaceable` | `applyPromptMetadataUpdate` → `titleFromPromptMetadataText(text)`, i.e. the prompt itself, capped at `MAX_TITLE_LENGTH = 200` | **No** |
| `generated` | `setGeneratedTitleIfUncustomized`, from a `chat_title` service call | Yes |
| `custom` | `setTitle`, an explicit rename | Yes |

Legacy records without `titleKind` normalise as `isCustomTitle === true →
custom`, otherwise `replaceable`.

Two facts decide the recommendation. First, on this host **every** session with a
`titleKind` has `replaceable` (4 of 4), 80 of 83 have `isCustomTitle: false`,
and the longest title is exactly 200 characters — the truncation cap, i.e. a
prompt, not a title. Second, generation is gated: `generateTitleOnce` returns
early unless the `auto_session_title` feature flag is on, and
`generateAndApply` additionally requires the Kimi Code OAuth provider with a
resolvable token, giving up quietly otherwise.

So the adapter must publish `title` **only** when `titleKind ∈ {generated,
custom}` (or legacy `isCustomTitle === true`), and must expect that to be absent
most of the time. A brand-new session has no `title` key at all, and
`isUntitled` also treats `"New Session"` and whitespace as untitled.

### Discovery and binding

- **Metadata:** `state.json` v2 gives `{id, version:2, cwd, createdAt,
  updatedAt, archived, agents, …}`; the v1 shape (65 files) uses `workDir`
  instead of `cwd` and has no `version`. `session_index.jsonl` gives
  `{sessionId, sessionDir, workDir}` without opening any session.
- **Direct process-to-session registry: none.** No pid appears under
  `$KIMI_CODE_HOME`; the only lock is the search index's.
- **Route:** `session_index.jsonl` entries whose `workDir` is the agent's
  working directory, whose session directory was created after the agent
  started — one or none.

### One-shot generation

**Yes.** `kimi -p "<prompt>" [-m <model alias>] [--output-format text]` — "run
one prompt non-interactively and print the response". There is no ephemeral
flag, so a generation run persists a session in the same working-directory
bucket, with the same bounded consequence described for OpenCode.

---

## Grok

### Where sessions live

- **Env allowlist:** `GROK_HOME`. **Default when unset:** `~/.grok`. The shipped
  `README.md` states it outright — "`GROK_HOME` | Override config directory
  (default: `~/.grok`)" — and the binary carries the matching failure string
  "no user grok home (set $GROK_HOME or $HOME)". Each `summary.json` also records
  the `grok_home` it was written under.
- **Paths:** `$GROK_HOME/sessions/<percent-encoded-cwd>/<session-uuid>/`. The
  encoding is a plain URL encoding of the absolute path
  (`%2FUsers%2F…`) — lossless and computable, unlike Pi's and Kimi's.
- Files in a session directory, per the first-party docs: `summary.json`,
  `updates.jsonl`, `chat_history.jsonl`, `plan.json`, `rewind_points.jsonl`,
  `signals.json`, `feedback.jsonl`, `compaction_checkpoints/`, `subagents/`.

### Storage family

**Append-only JSONL** for the conversation plus a **rewritten JSON sidecar** for
metadata — the same shape as Kimi. The binary's own documentation is explicit:
"Each line in `updates.jsonl` is a self-contained ACP session update event…
Incremental writes (append-only during a session)… JSONL is the source of truth
for session content", with `summary.json`, `plan.json` and `signals.json` called
out as plain JSON.

`session_search.sqlite` under `sessions/` is an FTS5 keyword index for
`grok sessions search`. It is derived and must not be read.

### Record shapes

Each line of `updates.jsonl` is `{timestamp, method, params}`.

- **Top-level human text turn:** `method: "session/update"` with
  `params.update.sessionUpdate == "user_message_chunk"` and
  `update.content = {type:"text", text}`. `update._meta` carries
  `{modelId, promptIndex}`. It arrives in **chunks** — the adapter must
  concatenate consecutive chunks for one `promptIndex`.
- **Final visible assistant text:** `sessionUpdate == "agent_message_chunk"`,
  likewise chunked, with `params._meta.promptId` tying the chunks to the turn.
- **Turn completion:** `method: "_x.ai/session/update"` with
  `sessionUpdate == "turn_completed"`, carrying `{prompt_id, stop_reason,
  usage}`. Observed `stop_reason: "end_turn"`.
- **Stable turn identity:** `promptId` / `prompt_id` (uuid) on the assistant
  side; `promptIndex` (integer, 0-based) on the user side. They are not the same
  key, so the linkage is positional: the user chunks preceding a turn's agent
  chunks and its `turn_completed`. An in-order tail makes that unambiguous.

**Record kinds to ignore.** `agent_thought_chunk` (reasoning), `tool_call`,
`tool_call_update`, and every `_meta` block. `chat_history.jsonl` is the raw
model wire — the direct analogue of Codex's `response_item` stream — and must not
be read; `events.jsonl` (`phase_changed`, `loop_started`, `first_token`,
`turn_started`, `turn_ended`, `tool_started`, `permission_requested`, …) is
telemetry, not conversation.

**Whole directories to ignore.** `subagents/` holds child session directories.
`summary.json` also carries `parent_session_id` for a fork or restore.

### Failing closed

Sniff `summary.json` for `chat_format_version` (observed `1` on all 472
sessions) and `info.id` / `info.cwd`; in `updates.jsonl`, require `method` and
`params.update.sessionUpdate` to be strings and `update.content` to be a typed
object. An unknown `chat_format_version` ends the binding. Unknown
`sessionUpdate` values are ordinary and skipped.

### Native title

**Yes — `summary.json`'s `generated_title`.** The binary's own documentation
calls it "the session summary and its model-generated title", and
`session_summary` mirrors it. Observed on 5 of 472 sessions, 34–44 characters,
with `generated_title == session_summary` in every case.

The distribution is the reassuring part: **every** session with 7 or more chat
messages had a generated title, and **every** session without one had 2 or fewer
— the 467 titleless sessions are stubs, not failures. Grok generates a title
reliably once there is a conversation to title. `/rename <title>` additionally
lets an operator set one by hand (the docs distinguish "a single manually
renamed session" from "auto-generated duplicates" when resolving `--resume` by
title), so the title can change over a session's life and each change should be
published.

Note the key is **absent** rather than empty before generation, and
`session_summary` is `""` in the older shape — so the adapter must treat missing
and empty alike as "no native title yet", and must not confuse the older
`summary.json` shape (13 keys, no `generated_title`) with a failure.

### Discovery and binding

- **Metadata:** `summary.json` gives `info.id`, `info.cwd`, `created_at`,
  `updated_at`, `num_chat_messages`, `current_model_id`, `agent_name`,
  `chat_format_version`, `grok_home`, and `parent_session_id`.
- **Direct process-to-session registry: none.** `$GROK_HOME/active_sessions.json`
  looks like one, but it is `[]` here and the string does not appear anywhere in
  the 1.0.0 binary — it is a leftover from the preceding build.
- **Route:** the computed percent-encoded cwd directory, session directories
  created since the agent started, `info.cwd` confirming the directory — one or
  none.
- **A useful accident:** headless `grok -p` runs write `chat_history.jsonl` but
  **no `updates.jsonl`** (467 of 472 sessions here are exactly that shape). An
  adapter keyed on `updates.jsonl` therefore ignores chartr's own title
  generations for free, which is the cleanest version of that hazard among the
  five.

### One-shot generation

**Yes.** `grok -p "<prompt>"`, documented as headless mode, with
`--output-format json` for a structured result. Model selection is by config or
`--model`.

---

## Cross-cutting findings

**1. The state-root table needs a suffix.** `internal/adapter/stateroot.go`
treats an allowlisted variable's value as the root itself. OpenCode
(`XDG_DATA_HOME` + `/opencode`) and Pi (`PI_CODING_AGENT_DIR` + `/sessions`)
both name a parent instead. `stateRootSpec` should carry a per-variable suffix;
its `fallback` already expresses the same idea home-relative. Codex, Kimi and
Grok are unaffected — their variables name the root directly.

**2. Two providers hand chartr an explicit schema version; use it.** Kimi's
`metadata.protocol_version` (1.4 → 1.5) and Grok's `chat_format_version` (1) are
purpose-built sniff points. Codex's `session_meta.history_mode` is not a version
but selects between two incompatible record families and must be treated the
same way. OpenCode stamps `session.version` with the creating build. Pi's header
`version` is a format version with a documented migration history (1 → 2 → 3).

**3. Subagent traffic is excluded structurally, not by predicate, in three of
five.** Kimi puts it in a sibling `agents/<id>/` directory, Grok in
`subagents/`, OpenCode in rows with a non-null `parent_id`. Codex is the awkward
one: subagent threads are ordinary rollouts in the same tree with the same cwd,
separated only by `session_meta.thread_source`. Pi has no subagents in this
sense.

**4. Headless generation writes a session for four of five.** `codex exec`,
`opencode run`, `kimi -p` and `grok -p` all persist something in the working
directory chartr is binding against; only `pi --no-session -p` writes nothing.
This is bounded rather than dangerous: generation only ever runs for a tab whose
binding is already established, so a generation-created session cannot steal an
existing binding — it can only add ambiguity for a *different* tab in the same
directory that is still unbound, where "unique or nothing" turns it into a
missing title. Two of the four are excluded outright anyway: Codex by
`originator == "codex-tui"`, Grok by keying on `updates.jsonl`.

**5. Only Claude has a process-to-session registry.** None of the five exposes a
pid-keyed mapping. All five bind through the specification's other route:
working directory plus sessions written since the agent started, unique or
nothing. Codex's `thread-writer-locks/` and Grok's stale `active_sessions.json`
were both investigated and rejected above.

**6. Where the first turn is visible.** Claude's registry lets a chartr-launched
tab bind at offset zero and see its opening turn. Of the five here, Pi
definitively cannot (the file materialises only once the first assistant message
exists, carrying the prompt with it), and OpenCode probably cannot (pending the
check noted in its section). Codex, Kimi and Grok create their session store at
session start, before the first prompt is submitted, so a tab bound at that
moment should see its opening turn — but that was not verified against a live
process in this ticket and ticket 05 should confirm it per provider.

> **Corrected in ticket 05, measured against live processes.** All five create
> their store lazily, at first submit — not at session start. Each agent's TUI
> was started in a pty in a fresh working directory, its trust prompt accepted,
> and left at its prompt for forty seconds: none of the five wrote a session a
> binding could find, and OpenCode inserted no `session` row (the open question
> its section records). So no tab on these five providers can bind before its
> first submission, and the first turn arrives already behind the cursor. The
> second turn is the first one that can be titled. This costs little: OpenCode
> and Grok have real native titles that land free during that same first turn.

---

## Assignment

All five providers get an adapter in ticket 05. None of them lacks both a usable
native title and a safe one-shot recipe, so no provider is left permanently
untitled.

| Provider | Storage family | Reader shape | Native title | One-shot recipe | Verdict |
| --- | --- | --- | --- | --- | --- |
| **Codex** | Append-only JSONL | Byte offset over `rollout-*.jsonl` | **None usable** — `threads.title` is the verbatim first user message | `codex exec --model <id> <prompt>` (already in `titlegen.go`) | Adapter, paid path |
| **OpenCode** | **Database (SQLite/WAL)** | Read-only incremental `rowid` queries on `part`/`message` | **Yes** — `session.title`, minus the `New session - `/`Child session - ` placeholder | `opencode run -m <provider/model> <message>` | Adapter, free path usually wins |
| **Pi** | Append-only JSONL | Byte offset over `<ts>_<uuid>.jsonl` | User-set only — latest `session_info.name`; never auto-generated | `pi --no-session -p <prompt> --model <pattern>` | Adapter, paid path |
| **Kimi** | Append-only JSONL + JSON sidecar | Byte offset over `agents/main/wire.jsonl`; poll `state.json` | Conditional — `state.json.title` only when `titleKind ∈ {generated, custom}`; `replaceable` is the prompt and must be rejected | `kimi -p <prompt> -m <alias>` | Adapter, paid path in practice |
| **Grok** | Append-only JSONL + JSON sidecar | Byte offset over `updates.jsonl`; poll `summary.json` | **Yes** — `summary.json.generated_title`, plus `/rename` | `grok -p <prompt> --output-format json` | Adapter, free path usually wins |

One database-backed adapter (OpenCode) and four JSONL adapters. That is the
split ticket 05 should plan its reader work around.
