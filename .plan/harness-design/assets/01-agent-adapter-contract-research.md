# Agent adapter contract — CLI capability survey

Research for [ticket 01](../tickets/01-the-agent-adapter-contract.md). Investigates the four
candidate agent CLIs — **claude, codex, opencode, pi** — against the four things the harness
needs from each (launch, prompt injection, context bundle, observation), plus model selection,
resume, stop, and cost reporting. Ends with **the narrowest contract every agent can satisfy** —
the adapter interface (ADR 0002) — and what degrades when an optional capability is absent.

All flag names below are quoted from primary sources; each agent's section lists the sources it draws on.

---

## claude — Claude Code

Sources: [CLI reference](https://code.claude.com/docs/en/cli-reference) (code.claude.com).

1. **Launch (PTY, given cwd, no human for first-run).** `--print`/`-p` runs non-interactively.
   The process cwd is the session's working directory (no flag); `--add-dir` grants extra dirs.
   The first-run/permission prompt is silenced with `--dangerously-skip-permissions`
   (= `--permission-mode bypassPermissions`), or handled programmatically with
   `--permission-prompt-tool <mcp_tool>` (waits up to `MCP_TIMEOUT` 30s for the server).
   `--permission-mode` also accepts `default|acceptEdits|plan|auto|dontAsk|bypassPermissions|manual`.
2. **Inject an opening prompt / wire a role.** Positional prompt: `claude -p "query"`; or piped stdin
   (`cat f | claude -p "query"`). Role wiring has dedicated flags:
   `--system-prompt` / `--system-prompt-file` (replace), `--append-system-prompt` /
   `--append-system-prompt-file` (append).
3. **Load a context bundle.** Channels: the prompt argument; stdin (`--input-format text|stream-json`,
   where `stream-json` streams multiple user messages); files the prompt tells the agent to read
   (plus `CLAUDE.md`); `--add-dir` extra roots; MCP servers; settings files. No documented hard size
   limit on the prompt beyond the model context window; `stream-json` exists precisely for large/multi-turn input.
4. **Be observed.** `--output-format text|json|stream-json`; `--verbose`,
   `--include-partial-messages`, `--include-hook-events` enrich the stream (all require
   stream-json). Non-zero exit on error or when `--max-turns` is hit. The final `result` object in
   `--output-format json` carries usage and cost fields (see cost, below).

- **Model selection:** `--model <alias|full-id>` (aliases `sonnet|opus|haiku|fable`), overrides
  `ANTHROPIC_MODEL`; `--fallback-model a,b` on overload.
- **Resume:** `--resume`/`-r <id|name>`, `--continue`/`-c` (most recent in cwd),
  `--fork-session`. `--no-session-persistence` disables saving.
- **Stop:** `--max-turns N` and `--max-budget-usd N` exit with an error at the limit (print mode);
  otherwise signal the process.
- **Cost:** native. `--max-budget-usd` caps spend; the JSON `result` reports `total_cost_usd`,
  `usage`, `num_turns`, `duration_ms` (surfaced by print mode / the Agent SDK result message).
  The CLI-reference page does not itemise the JSON schema, so treat the exact field set as
  SDK-version-dependent, but dollar cost *is* first-party.

## codex — OpenAI Codex CLI

Sources: [non-interactive mode](https://learn.chatgpt.com/docs/non-interactive-mode),
[configuration](https://learn.chatgpt.com/docs/configuration) (learn.chatgpt.com, the redirect
target of developers.openai.com/codex).

1. **Launch.** `codex exec "prompt"` is the non-interactive entry point. Working dir `--cd`/`-C`.
   Sandbox `--sandbox read-only|workspace-write|danger-full-access`; `--full-auto` (low-friction
   sandboxed auto) and `--dangerously-bypass-approvals-and-sandbox` remove approval friction;
   `-a`/`--ask-for-approval` sets the policy. `--skip-git-repo-check` allows running outside a git
   repo; `--ignore-user-config`, `--ignore-rules` isolate from ambient config.
2. **Inject an opening prompt / wire a role.** Prompt as arg, or stdin (`codex exec -`); when both
   are present the arg is the instruction and stdin is context. **No dedicated system-prompt flag** —
   role/persona is carried by `AGENTS.md` / `.rules` files (which `--ignore-rules` suppresses) or by
   the prompt body itself. This is the weakest role-wiring surface of the four.
3. **Load a context bundle.** Prompt arg, stdin, `config.toml`, `AGENTS.md`/`.rules`, MCP servers.
   `--output-schema <path>` constrains the *response* shape.
4. **Be observed.** `--json` emits JSONL events: `thread.started`, `turn.started`,
   `turn.completed`, `item.*`, `error`. `turn.completed` carries a `usage` object —
   `{input_tokens, cached_input_tokens, output_tokens}`. `--output-last-message`/`-o <path>`
   writes the final message to a file. Exit codes on failure.

- **Model selection:** `--model`/`-m`; config keys `model`, `model_provider`,
  `model_reasoning_effort`; custom providers via `[model_providers.NAME]`; named `profiles`
  selected with `--profile`/`-p`.
- **Resume:** `codex exec resume --last "next"` or `codex exec resume <SESSION_ID> "next"`.
  `--ephemeral` skips persisting session files.
- **Stop:** signal the process; sandbox/approval controls bound what it can do.
- **Cost:** **token counts only** — the `usage` object gives tokens (incl. cached), no dollar figure.
  Dollars must be derived (tokens × a price table). No native budget cap flag documented.

## opencode

Sources: [CLI docs](https://opencode.ai/docs/cli/) (opencode.ai; project sst/opencode).

1. **Launch.** `opencode run [message..]` runs non-interactively. Also `opencode serve`
   (headless server for API access) and `opencode web`. Working dir is the project cwd.
2. **Inject an opening prompt / wire a role.** Prompt(s) as args to `run`. Role/persona is carried
   by opencode's configured *agents* (config + `AGENTS.md`), not a per-invocation system-prompt flag.
3. **Load a context bundle.** Message args; config (`opencode.json`); `AGENTS.md`; MCP; and, in
   server mode, the HTTP API.
4. **Be observed.** `--format default|json` on `run` (json = raw JSON events);
   `session list --format table|json`. `opencode serve` exposes the run over an API.

- **Model selection:** `--model`/`-m` in `provider/model` form (e.g. `anthropic/claude-3-5-sonnet`).
- **Resume:** `--continue`/`-c` (last session), `--session`/`-s <id>`, `--fork`.
- **Stop:** signal the process; server endpoints in serve mode.
- **Cost:** native, **out of band.** `opencode stats` reports token usage *and cost*, filterable by
  `--days`, `--tools`, `--models`, `--project`. In-stream `--format json` events carry usage; the
  dollar rollup comes from `stats`, not the live event.

## pi — earendil-works/pi

Sources: [repo](https://github.com/earendil-works/pi),
[coding-agent package README](https://github.com/earendil-works/pi/tree/main/packages/coding-agent),
[docs](https://pi.dev/docs/latest). A lean agent (four built-in tools: read/write/edit/bash;
~300-word system prompt), shipped as an npm package or a standalone Bun binary.

1. **Launch.** `pi -p`/`--print` executes a prompt and exits. `--mode json` streams events as JSON
   lines; `--mode rpc` runs an RPC server over stdin/stdout with strict LF-delimited JSONL framing;
   an SDK (`createAgentSession`) embeds it. Working dir is cwd; a containerization guide exists.
2. **Inject an opening prompt / wire a role.** Positional prompt with `-p`; piped stdin is merged
   with the message. Role/behaviour is shaped by TypeScript **extensions/skills** and the (small)
   system prompt rather than a replace-the-prompt flag.
3. **Load a context bundle.** Prompt arg, stdin, extensions/skills, config; RPC/SDK for richer,
   programmatic handover (multiple messages, multi-session runtime).
4. **Be observed.** `--mode json` = all events as JSON lines; `--mode rpc` = strict JSONL for
   non-Node hosts. The interactive footer shows `↑`/`↓` tokens, cache `R`/`W`, total cost and
   context %, so cost/usage is computed and exposed through the same event data.

- **Model selection:** `--provider <name>`, `--model <pattern|provider/id>`,
  `--thinking off|minimal|low|medium|high|xhigh|max`.
- **Resume:** `--continue`/`-c`, `--resume`/`-r`, `--session <path|id>`, `--fork <path|id>`,
  `--no-session` (ephemeral).
- **Stop:** Escape cancels the current operation; Ctrl-C twice exits; programmatic abort via RPC/SDK.
- **Cost:** tokens **and** cost are tracked (shown in the footer / available in json/rpc events).

---

## The narrowest contract every agent can satisfy

This intersection **is** the adapter interface (ADR 0002). Each capability below is one every one of
the four can do today; the incantation differs per agent, and hiding that difference is the
adapter's whole job.

| Capability | claude | codex | opencode | pi | In the contract? |
|---|---|---|---|---|---|
| Headless launch in a given cwd | `-p` | `codex exec` | `run`/`serve` | `-p`/`--mode` | **yes** |
| Silence first-run/approval with no human | `--dangerously-skip-permissions` etc. | `--full-auto` / `--sandbox` / `--skip-git-repo-check` | non-interactive by default | non-interactive by default | **yes — via a per-adapter incantation** |
| Prompt as arg **and** stdin | ✅ | ✅ | ✅ (args) | ✅ | **yes** |
| Dedicated system-prompt flag | ✅ | ✗ | ✗ (agents) | ✗ (extensions) | **no — wire the role in the prompt body** |
| Structured JSON event stream | `stream-json` | `--json` | `--format json` | `--mode json/rpc` | **yes** |
| Exit code on failure | ✅ | ✅ | ✅ | ✅ | **yes** |
| Token counts in the stream | ✅ | ✅ | ✅ | ✅ | **yes** |
| Native dollar cost | ✅ (`total_cost_usd`) | ✗ (tokens only) | ✅ (`stats`, OOB) | ✅ | **no — derive dollars from tokens** |
| `--model` selection | ✅ | ✅ | ✅ | ✅ | **yes** |
| Resume a prior session | ✅ | ✅ | ✅ | ✅ | present, but **deliberately excluded** — see below |
| Native budget/turn cap | ✅ (`--max-budget-usd`, `--max-turns`) | ✗ | ✗ | ✗ | **no — harness-enforced** |

**The contract, stated plainly.** An adapter must:

1. **`spawn(cwd, model, promptText) → PTY`** — launch its agent headless in `cwd`, on the requested
   `--model`, with the **entire context bundle delivered as the opening prompt text** and the role
   wired *inside that prompt body* (not via a system-prompt flag), having pre-silenced its own
   first-run/approval prompt. The one guaranteed context channel is prompt-arg + stdin + a file on
   disk the prompt points the agent at.
2. **`observe(PTY) → {alive, exited(code), tokens}`** — read liveness and failure from the process
   (exit code) and token usage from the agent's JSON event stream. It does **not** owe a semantic
   "the work is done" signal: per ADR 0004 the harness derives *finished* from the ticket's
   `## Answer` + commit, not from the agent. So the adapter only reports **alive / died / tokens**.
3. **`stop(PTY)`** — terminate the process by signal. Graceful stop is a bonus, not part of the floor.

**Optional per-adapter capabilities, and what degrades without them** (the spec must state each degradation):

- **Dedicated system-prompt injection** (claude, and pi via extensions). *Degrade:* prepend the role
  to the prompt body — the portable path, so the harness should use it *uniformly* rather than
  branching, keeping one prompt-assembly path (ADR 0002: "the harness owns prompt and context assembly").
- **Native dollar cost** (claude in-stream; opencode via `stats`). *Degrade:* multiply the universally
  available token counts by a per-model price table the harness maintains. Tokens are the floor;
  dollars are always computable. (This is what clears the cost-visibility fog — see ticket 14.)
- **Native budget/turn caps** (`--max-budget-usd`, `--max-turns`, claude only). *Degrade:* the harness
  enforces a cap itself by watching token usage in the stream and calling `stop()` at a threshold.
- **MCP / config-file context channels.** *Degrade:* a file on disk the prompt tells the agent to read;
  every agent has a read/bash tool, so a pointed-at file is the lowest common denominator.
- **Resume** (all four support it). **Excluded by design, not merely optional:** ADR 0005 assembles a
  fresh context bundle per spawn and forbids accumulated agent memory. Resuming an agent's own session
  state would reintroduce exactly that memory, so the harness **re-spawns fresh** every time and never
  calls resume. Recorded here so a future adapter author doesn't "helpfully" wire it up.
- **Structured/schema output** (`--json-schema`, `--output-schema`). Not needed for orchestration; ignore.

**One tension worth flagging (not re-decided here).** codex has no clean system-prompt flag *and*
weaker in-stream cost. Neither breaks the contract — role goes in the prompt body, dollars are
derived — but codex is the agent that most tests the "narrowest contract" premise. If a future
capability the harness *needs* turns out to be codex's blind spot, that is the moment to ask whether
codex stays a supported adapter, and it is an ADR 0002 question, not a quiet workaround.
