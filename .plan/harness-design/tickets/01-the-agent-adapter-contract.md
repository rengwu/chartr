---
type: research
blocked_by: []
claimed_by: claude-opus-4-8/wayfinder-session
claimed_at: 2026-07-17T01:36:36Z
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
