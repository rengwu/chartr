---
type: research
blocked_by: []
assets: [.plan/harness-design/assets/01-agent-adapter-contract-research.md]
---

# The agent adapter contract

## Question

The harness is agent-agnostic (ADR 0002), so every session runs through a per-agent **adapter**. What must an adapter be able to do — and can the agents we intend to support actually do it?

Investigate the current CLIs — **claude, codex, opencode, pi**, plus any other worth including — against the four things the harness needs from each:

1. **Launch** into a PTY, in a given working directory, without a human present to answer a first-run prompt.
2. **Inject an opening prompt** that wires the session to a role.
3. **Load a context bundle** — how much can be handed over at spawn, by what mechanism (argument, stdin, a file the agent is told to read, a config file, an MCP server), and what are the size limits?
4. **Be observed** — what, if anything, does the CLI expose about liveness, completion, token spend, or failure? Exit codes, structured or streaming output, log files.

Record per agent, because later tickets need it: how a **model is selected** (ticket 05 needs this for review heterogeneity), whether a session can be **resumed**, how the agent is told to **stop**, and what it reports about **cost** (a fog patch hangs on this).

Produce a cited comparison as a linked asset — one section per agent — ending with the finding that matters most: **the narrowest contract every supported agent can satisfy.** That intersection is the adapter interface. Anything outside it is a per-adapter capability the harness must treat as optional, and the spec has to say what degrades when it is missing.

## Answer

All four agents — **claude, codex, opencode, pi** — can be driven headless, and the intersection of
what they share is a clean adapter interface. Full cited survey:
[agent-adapter-contract-research](../assets/01-agent-adapter-contract-research.md).

**The contract (the adapter interface, ADR 0002) is three operations:**

1. **`spawn(cwd, model, promptText)`** — launch the agent non-interactively in `cwd`, on the chosen
   `--model`, with the **whole context bundle delivered as the opening prompt text** and the role
   wired *in the prompt body*. Every agent has a headless mode (`claude -p`, `codex exec`,
   `opencode run`, `pi -p`/`--mode json`) and a `--model` flag; each silences its own
   first-run/approval prompt by its own incantation, which the adapter hides.
2. **`observe(PTY) → {alive, exited(code), tokens}`** — liveness/failure from the process exit code,
   token usage from the agent's JSON event stream (all four emit one). The adapter owes **no**
   semantic "finished" signal: per ADR 0004 the harness derives *finished* from the ticket's
   `## Answer` + commit.
3. **`stop(PTY)`** — terminate by signal.

**Why the floor sits there — three things are *not* universal**, so they stay optional with a stated degradation:

- **No dedicated system-prompt flag** on codex/opencode/pi. So role wiring goes in the prompt body,
  uniformly, for every agent — keeping one prompt-assembly path (ADR 0002).
- **Dollar cost is native only on claude (in-stream) and opencode (`stats`, out of band).** codex and
  pi report **token counts** only. Floor = tokens (universal); dollars are **derived** from tokens ×
  a per-model price table the harness maintains. *(This clears the cost-visibility fog — now
  [ticket 14](./14-cost-and-token-visibility.md).)*
- **Budget/turn caps are native only on claude** (`--max-budget-usd`, `--max-turns`). For the rest the
  harness enforces a cap itself by watching stream tokens and calling `stop()`.

**One capability is excluded by design, not merely optional:** every agent *can* resume a prior
session, but ADR 0005 (assembled context, no agent memory) means the harness **re-spawns fresh** each
time and never resumes — resuming would smuggle back the accumulated agent memory that ADR rejects.

**Flagged, not re-decided:** codex is the agent that most strains the "narrowest contract" premise
(no system-prompt flag *and* weakest cost reporting). Nothing here breaks — but if a future
*required* capability turns out to be codex's blind spot, that is an ADR 0002 question about whether
codex stays supported, not a quiet workaround.
